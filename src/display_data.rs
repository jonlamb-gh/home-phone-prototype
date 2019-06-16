//! 4x20 character display data

use chrono::prelude::*;
use crate::linphone::CallState;
use crate::phone::{Phone, State as PhoneState};
use phonenumber::{country, Mode};
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
    pub fn update(&mut self, missed_calls: Option<usize>, phone: &Phone) {
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

                    self.set_row(
                        Row::One,
                        Alignment::Center,
                        format!("{} Missed Calls", missed_calls.unwrap_or(0)),
                    );
                } else {
                    self.set_row(
                        Row::Zero,
                        Alignment::Left,
                        phone.keypad_buffer().data().to_string(),
                    );
                }
            }
            PhoneState::HandlePendingCall(pending_call) => {
                // TODO - remote address is showing ours?
                // linphone_call_get_dir()
                // this is a from_address
                //let remote_address = pending_call.to_address();
                let remote_address = Self::number_string(pending_call.remote_address());

                self.set_row(
                    Row::Zero,
                    Alignment::Center,
                    "'*' Decl | Ans '#'".to_string(),
                );
                self.set_row(Row::One, Alignment::Center, "Incoming Call".to_string());
                self.set_row(Row::Two, Alignment::Center, remote_address);
            }
            PhoneState::OnGoingCall(call) => {
                let duration = call.duration();
                let remote_address = Self::number_string(call.remote_address());

                self.set_row(Row::Zero, Alignment::Center, remote_address);

                match call.state() {
                    CallState::OutgoingInit
                    | CallState::OutgoingProgress
                    | CallState::OutgoingRinging
                    | CallState::OutgoingEarlyMedia => {
                        self.set_row(Row::One, Alignment::Center, String::from("Calling..."));
                    }
                    CallState::Connected | CallState::StreamsRunning => {
                        self.set_row(
                            Row::One,
                            Alignment::Center,
                            String::from(format!("Duration: {} sec", duration.as_secs())),
                        );
                    }
                    CallState::End => {
                        self.set_row(Row::One, Alignment::Center, String::from("Call Ended"));
                    }
                    state => {
                        self.set_row(
                            Row::One,
                            Alignment::Left,
                            String::from(format!("{:?}", state)),
                        );
                    }
                }
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

    fn number_string(number: String) -> String {
        if let Ok(num) = phonenumber::parse(Some(country::US), &number) {
            if phonenumber::is_valid(&num) {
                return num.format().mode(Mode::National).to_string();
            }
        }

        number
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
