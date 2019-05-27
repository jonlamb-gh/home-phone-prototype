// TODO - long-press?

use crate::keypad_event::KeypadEvent;
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;

pub trait Keypad {
    fn next_event(&mut self) -> Option<KeypadEvent>;
}

pub struct StdinKeypad {
    rx: Receiver<KeypadEvent>,
}

impl StdinKeypad {
    pub fn new() -> Self {
        StdinKeypad {
            rx: spawn_stdin_channel(),
        }
    }
}

impl Keypad for StdinKeypad {
    fn next_event(&mut self) -> Option<KeypadEvent> {
        if let Ok(event) = self.rx.try_recv() {
            Some(event)
        } else {
            None
        }
    }
}

fn spawn_stdin_channel() -> Receiver<KeypadEvent> {
    let (tx, rx) = mpsc::channel::<KeypadEvent>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        let stdin = io::stdin().read_line(&mut buffer).unwrap();
        if let Some(c) = sanitize_key(buffer.chars().next().unwrap()) {
            tx.send(KeypadEvent::Key(c)).unwrap();
        }
    });
    rx
}

fn sanitize_key(input: char) -> Option<char> {
    match input {
        '1' => Some(input),
        '2' => Some(input),
        '3' => Some(input),
        '4' => Some(input),
        '5' => Some(input),
        '6' => Some(input),
        '7' => Some(input),
        '8' => Some(input),
        '9' => Some(input),
        '0' => Some(input),
        '*' => Some(input),
        '#' => Some(input),
        _ => None,
    }
}