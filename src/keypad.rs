#![allow(deprecated)]

use crate::keypad_event::KeypadEvent;
use keypad_builder::embedded_hal::digital::InputPin as HALInputPin;
//use keypad_builder::KeypadInput;
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::thread;
use std::time::{Duration, Instant};

// BCM pin numbers, not physical

const GPIO_KP_R0: u8 = 5;
const GPIO_KP_R1: u8 = 6;
const GPIO_KP_R2: u8 = 13;
const GPIO_KP_R3: u8 = 19;

const GPIO_KP_C0: u8 = 17;
const GPIO_KP_C1: u8 = 27;
const GPIO_KP_C2: u8 = 22;

keypad_struct!{
    struct PhoneKeypad {
        rows: (
            InputPin,
            InputPin,
            InputPin,
            InputPin,
        ),
        columns: (
            OutputPin,
            OutputPin,
            OutputPin,
        ),
    }
}

const CHAR_MAP: [[char; 3]; 4] = [
    ['1', '2', '3'],
    ['4', '5', '6'],
    ['7', '8', '9'],
    ['*', '0', '#'],
];

/// Considered idle if no activity after 5 seconds
const IDLE_TIMEOUT: Duration = Duration::from_secs(5);

const LONGPRESS_DURATION: Duration = Duration::from_secs(2);

const PREKEY_WAIT: Duration = Duration::from_millis(800);

const DEBOUNCE_DURATION: Duration = Duration::from_millis(5);

pub struct Keypad {
    last_activity: Instant,
    last_char: Option<char>,
    kp: PhoneKeypad,
}

impl Keypad {
    pub fn new() -> Self {
        let gpio = Gpio::new().unwrap();

        let pin_kp_r0 = gpio.get(GPIO_KP_R0).unwrap().into_input_pullup();
        let pin_kp_r1 = gpio.get(GPIO_KP_R1).unwrap().into_input_pullup();
        let pin_kp_r2 = gpio.get(GPIO_KP_R2).unwrap().into_input_pullup();
        let pin_kp_r3 = gpio.get(GPIO_KP_R3).unwrap().into_input_pullup();

        let pin_kp_c0 = gpio.get(GPIO_KP_C0).unwrap().into_output();
        let pin_kp_c1 = gpio.get(GPIO_KP_C1).unwrap().into_output();
        let pin_kp_c2 = gpio.get(GPIO_KP_C2).unwrap().into_output();

        let keypad = keypad_new!(PhoneKeypad {
            rows: (pin_kp_r0, pin_kp_r1, pin_kp_r2, pin_kp_r3,),
            columns: (pin_kp_c0, pin_kp_c1, pin_kp_c2,),
        });

        Keypad {
            last_activity: Instant::now(),
            last_char: None,
            kp: keypad,
        }
    }

    pub fn is_idle(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.last_activity) > IDLE_TIMEOUT
    }

    // TODO - make this better
    pub fn next_event(&mut self) -> Option<KeypadEvent> {
        if let Some(event) = self.event() {
            if let Some(prev_char) = self.last_char {
                if prev_char == event.inner() {
                    // Wait N millis before registering
                    // an event of the same character
                    if Instant::now().duration_since(self.last_activity) < PREKEY_WAIT {
                        return None;
                    }
                }
                self.last_char = None;
            }

            self.last_char = Some(event.inner());
            self.last_activity = Instant::now();
            Some(event)
        } else {
            None
        }
    }

    fn event(&mut self) -> Option<KeypadEvent> {
        let keys = self.kp.decompose();

        for (row_index, row) in keys.iter().enumerate() {
            for (col_index, key) in row.iter().enumerate() {
                if key.is_low() {
                    thread::sleep(DEBOUNCE_DURATION);
                } else {
                    continue;
                }

                if key.is_low() {
                    let c = CHAR_MAP[row_index][col_index];

                    // Wait for release or long-press duration
                    let event = loop {
                        if Instant::now().duration_since(self.last_activity) >= LONGPRESS_DURATION {
                            break KeypadEvent::LongPress(c);
                        } else if key.is_low() == false {
                            break KeypadEvent::KeyPress(c);
                        }
                    };

                    thread::sleep(DEBOUNCE_DURATION);
                    return Some(event);
                }
            }
        }

        None
    }
}
