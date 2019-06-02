// TODO - long-press?

use crate::keypad_event::KeypadEvent;
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

/// Considered idle if no activity after 5 seconds
const IDLE_TIMEOUT: Duration = Duration::from_secs(5);

pub trait Keypad {
    fn next_event(&mut self) -> Option<KeypadEvent>;
}

pub struct StdinKeypad {
    last_activity: Instant,
    rx: Receiver<KeypadEvent>,
}

impl StdinKeypad {
    pub fn new() -> Self {
        StdinKeypad {
            last_activity: Instant::now(),
            rx: spawn_stdin_channel(),
        }
    }

    pub fn is_idle(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.last_activity) > IDLE_TIMEOUT
    }
}

impl Keypad for StdinKeypad {
    fn next_event(&mut self) -> Option<KeypadEvent> {
        if let Ok(event) = self.rx.try_recv() {
            self.last_activity = Instant::now();
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
        let _stdin = io::stdin().read_line(&mut buffer).unwrap();

        buffer
            .chars()
            .filter(|c| sanitize_key(*c).is_some())
            .for_each(|c| tx.send(KeypadEvent::KeyPress(c)).unwrap());
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
