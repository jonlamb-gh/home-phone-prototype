use crate::keypad::{Keypad, StdinKeypad};
use crate::keypad_event::{KeypadBuffer, KeypadEvent, KeypadMode};
use crate::linphone::{Call, CallState, CoreContext, Error, Reason};
use phonenumber::{country, Mode};
use std::time::{Duration, Instant};

const NO_ANSWER_DURATION: Duration = Duration::from_secs(10);

pub struct Phone {
    keypad: StdinKeypad,
    keybuf: KeypadBuffer,
    state: State,
}

pub enum State {
    WaitingForEvents,
    HandlePendingCall(Call, Instant),
    OnGoingCall(Call),
}

impl Phone {
    pub fn new() -> Self {
        Phone {
            keypad: StdinKeypad::new(),
            keybuf: KeypadBuffer::new(),
            state: State::WaitingForEvents,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn keypad_buffer(&self) -> &KeypadBuffer {
        &self.keybuf
    }

    pub fn recover_from_error(&mut self) {
        match &mut self.state {
            State::HandlePendingCall(pending_call, _registration_instant) => {
                pending_call.terminate().ok();
            }
            State::OnGoingCall(call) => {
                call.terminate().ok();
            }
            _ => (),
        }

        self.state = State::WaitingForEvents;
        self.keybuf.clear();
    }

    // TODO - clean this up, move to session type indexed by state?
    pub fn handle_events(&mut self, core: &mut CoreContext) -> Result<(), Error> {
        match &mut self.state {
            State::WaitingForEvents => {
                if self.keypad.next_event().map_or(false, |event| {
                    self.keybuf.push(KeypadMode::WaitForUserDial, event)
                }) {
                    if let Ok(number) = phonenumber::parse(Some(country::US), self.keybuf.data()) {
                        self.keybuf.clear();
                        if phonenumber::is_valid(&number) {
                            println!("Calling {}", number.format().mode(Mode::National));
                            let call = core.invite(&number)?;
                            self.state = State::OnGoingCall(call);
                        } else {
                            println!("Ignoring invalid phone number");
                        }
                    }
                }
            }
            State::HandlePendingCall(pending_call, registration_instant) => {
                // Check for no-response timeout first
                let now = Instant::now();
                if now.duration_since(*registration_instant) > NO_ANSWER_DURATION {
                    println!("Auto-declining call");
                    self.keybuf.clear();
                    pending_call.decline(Reason::NotAnswered)?;
                    self.state = State::WaitingForEvents;
                } else if let Some(event) = self.keypad.next_event() {
                    // Check for accept/decline keys
                    if event == KeypadEvent::KeyPress('#') {
                        println!("Accepting call");
                        self.keybuf.clear();
                        pending_call.accept()?;
                        self.state = State::OnGoingCall(pending_call.clone());
                    } else if event == KeypadEvent::KeyPress('*') {
                        println!("Declining call");
                        self.keybuf.clear();
                        pending_call.decline(Reason::NotAnswered)?;
                        self.state = State::WaitingForEvents;
                    }
                }
            }
            State::OnGoingCall(call) => {
                if let Some(event) = self.keypad.next_event() {
                    if event == KeypadEvent::KeyPress('*') {
                        println!("Terminating active call");
                        self.keybuf.clear();
                        call.terminate()?;
                        self.state = State::WaitingForEvents;
                    } else if let KeypadEvent::KeyPress(c) = event {
                        // Buffer the DTMF for display purposes
                        let _ = self.keybuf.push(KeypadMode::Dtmf, event);

                        println!("Sending '{}' as DTMF", c);
                        call.send_dtmf(c)?;
                    }
                }
            }
        }

        Ok(())
    }

    // TODO - make this better
    pub fn handle_incoming_call(&mut self, mut call: Call) -> Result<(), Error> {
        if call.state() == CallState::CallIncomingReceived {
            match &self.state {
                State::WaitingForEvents => {
                    // Consume the pending call
                    self.state = State::HandlePendingCall(call, Instant::now());
                    Ok(())
                }
                _ => {
                    println!("Declining pending call");
                    call.decline(Reason::NotAnswered)?;
                    Err(Error::CallInProgress)
                }
            }
        } else {
            // Do nothing, drop the object
            Ok(())
        }
    }
}
