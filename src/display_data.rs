//! 4x20 character display data

use chrono::prelude::*;
use crate::phone::{Phone, State as PhoneState};
use phonenumber::{country, Mode};
use std::fmt;
use std::time::{Duration, Instant};

const ROWS: usize = 4;
const COLS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
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

    // TODO - clean this up
    pub fn update(&mut self, phone: &Phone) {
        self.clear();

        self.update_system_time();

        let phone_state = phone.state();

        match phone_state {
            PhoneState::WaitingForEvents => {
                // TODO - show missed call info if idle
                //
                // otherwise show number/keys being pressed?

                self.set_row(
                    Row::Zero,
                    Alignment::Left,
                    phone.keypad_buffer().data().to_string(),
                );
            }
            PhoneState::HandlePendingCall(pending_call, registration_instant) => (),
            PhoneState::OnGoingCall(call) => {
                let remote_address = call.remote_address();
                let duration = call.duration();

                // TODO - move this to a fn(&Call)
                // remote_address().as_number() -> Result<PhoneNumber> ?
                if let Ok(number) = phonenumber::parse(Some(country::US), remote_address.clone()) {
                    if phonenumber::is_valid(&number) {
                        self.set_row(
                            Row::Zero,
                            Alignment::Left,
                            number.format().mode(Mode::National).to_string(),
                        );
                    } else {
                        self.set_row(Row::Zero, Alignment::Left, remote_address);
                    }
                } else {
                    self.set_row(Row::Zero, Alignment::Left, remote_address);
                }

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
