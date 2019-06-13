//! 4x20 character display data

use chrono::prelude::*;
use crate::phone::{Phone, State as PhoneState};
use phonenumber::Mode;
use std::fmt;
//use std::time::{Duration, Instant};
//use crate::linphone::CoreContext;

const ROWS: usize = 4;
const COLS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alignment {
    Left,
    Center,
    Right,
}

pub struct DisplayData {
    rows: Vec<String>,
}

impl DisplayData {
    pub fn new() -> Self {
        DisplayData {
            rows: vec![String::with_capacity(COLS + 1); ROWS],
        }
    }

    pub fn row(&self, row: Row) -> &str {
        &self.rows[row as usize]
    }

    // TODO - clean this up
    // needs CoreContext for missed call logs
    pub fn update(&mut self, phone: &Phone) {
        self.clear();

        self.update_system_time();

        let phone_state = phone.state();

        match phone_state {
            PhoneState::WaitingForEvents => {
                // TODO - show missed call info if idle
                //
                // otherwise show number/keys being pressed?

                if phone.is_idle() == true {
                    self.set_row(
                        Row::Zero,
                        Alignment::Center,
                        "'*' Next | Clear '#'".to_string(),
                    );
                    self.set_row(Row::One, Alignment::Left, "XYZ Missed Calls".to_string());
                } else {
                    self.set_row(
                        Row::Zero,
                        Alignment::Left,
                        phone.keypad_buffer().data().to_string(),
                    );
                }
            }
            PhoneState::HandlePendingCall(pending_call, _registration_instant) => {
                let remote_address = if let Some(number) = phone.remote_address() {
                    number.format().mode(Mode::National).to_string()
                } else {
                    pending_call.remote_address()
                };

                self.set_row(
                    Row::Zero,
                    Alignment::Center,
                    "'*' Ans | Decl '#'".to_string(),
                );
                self.set_row(Row::Two, Alignment::Left, remote_address);
            }
            PhoneState::OnGoingCall(call) => {
                let duration = call.duration();

                let remote_address = if let Some(number) = phone.remote_address() {
                    number.format().mode(Mode::National).to_string()
                } else {
                    call.remote_address()
                };

                self.set_row(Row::Zero, Alignment::Left, remote_address);

                self.set_row(
                    Row::One,
                    Alignment::Left,
                    String::from(format!("Duration: {} sec", duration.as_secs())),
                );
            }
        }
    }

    // Row 3 is always the system time
    fn update_system_time(&mut self) {
        let local: DateTime<Local> = Local::now();
        self.set_row(
            Row::Three,
            Alignment::Center,
            local.format("%a %b %e %H:%M:%S").to_string(),
        );
    }

    fn clear(&mut self) {
        for row in &mut self.rows {
            row.clear();
        }
    }

    fn set_row(&mut self, row: Row, align: Alignment, string: String) {
        //self.rows[row as usize].clear();

        // Truncate to COLS worth of chars
        let truncated_string = match string.char_indices().nth(COLS) {
            None => string,
            Some((idx, _)) => (&string[..idx]).to_string(),
        };

        self.rows[row as usize] = match align {
            Alignment::Left => format!("{: <20}", truncated_string),
            Alignment::Center => format!("{: ^20}", truncated_string),
            Alignment::Right => format!("{: >20}", truncated_string),
        };
    }
}

impl fmt::Display for DisplayData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DisplayData
            |--------------------|
            |{}|
            |{}|
            |{}|
            |{}|
            |--------------------|",
            self.rows[Row::Zero as usize],
            self.rows[Row::One as usize],
            self.rows[Row::Two as usize],
            self.rows[Row::Three as usize],
        )
    }
}
