use crate::keypad::Keypad;
use crate::keypad_event::{KeypadBuffer, KeypadEvent, KeypadMode};
use crate::linphone::{Call, CallState, CoreContext, Error, Reason};
use phonenumber::{country, Mode, PhoneNumber};
use std::time::{Duration, Instant};

pub struct Phone {
    keypad: Keypad,
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
        let test_number = "123-456-1234";
        println!("Loading phonenumber cache");
        if let Ok(number) = phonenumber::parse(Some(country::US), test_number) {
            let _valid = phonenumber::is_valid(&number);
        }

        Phone {
            keypad: Keypad::new(),
            keybuf: KeypadBuffer::new(),
            state: State::WaitingForEvents,
        }
    }

    /// True if in state WaitingForEvents and Keypad::is_idle() == true
    pub fn is_idle(&self) -> bool {
        match &self.state {
            // TODO - not idle if keybuf has data?
            State::WaitingForEvents => {
                if self.keybuf.len() == 0 {
                    self.keypad.is_idle()
                } else {
                    false
                }
            }
            _ => false,
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

    // TODO - make this better
    pub fn remote_address(&self) -> Option<PhoneNumber> {
        match &self.state {
            State::HandlePendingCall(call, _) | State::OnGoingCall(call) => {
                if let Ok(number) =
                    phonenumber::parse(Some(country::US), call.remote_address().clone())
                {
                    if phonenumber::is_valid(&number) {
                        Some(number)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // TODO - clean this up, move to session type indexed by state?
    // return bool, true for some state change
    pub fn handle_events(&mut self, core: &mut CoreContext) -> Result<bool, Error> {
        let mut state_changed: bool = false;

        match &mut self.state {
            State::WaitingForEvents => {
                if self.keypad.next_event().map_or(false, |event| {
                    state_changed = true;
                    //println!("push key {:?}", event);
                    if event == KeypadEvent::LongPress('0') {
                        self.keybuf.clear();
                        false
                    } else {
                        self.keybuf.push(KeypadMode::WaitForUserDial, event)
                    }
                }) {
                    println!("Checking '{}'", self.keybuf.data());

                    if let Ok(number) = phonenumber::parse(Some(country::US), self.keybuf.data()) {
                        state_changed = true;
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
            State::HandlePendingCall(pending_call, _registration_instant) => {
                if let Some(event) = self.keypad.next_event() {
                    state_changed = true;

                    // Check for accept/decline keys
                    if event == KeypadEvent::LongPress('#') {
                        println!("Accepting call");
                        self.keybuf.clear();
                        pending_call.accept()?;
                        self.state = State::OnGoingCall(pending_call.clone());
                    } else if event == KeypadEvent::LongPress('*') {
                        println!("Declining call");
                        self.keybuf.clear();
                        pending_call.decline(Reason::NotAnswered)?;
                        self.state = State::WaitingForEvents;
                    }
                }
            }
            State::OnGoingCall(call) => {
                // TODO - detect remote hangup in state changed handler

                if let Some(event) = self.keypad.next_event() {
                    state_changed = true;

                    if event == KeypadEvent::LongPress('*') {
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

        Ok(state_changed)
    }

    // TODO - cmp for Call, this just assumes its ours
    pub fn handle_call_state_changed(&mut self, call: &Call) -> Result<(), Error> {
        match call.state() {
            CallState::CallIncomingReceived => self.handle_incoming_call(call.clone())?,
            CallState::Error | CallState::Released => {
                println!("Terminating");
                self.recover_from_error();
            }
            _ => (),
        }

        Ok(())
    }

    // TODO - make this better
    fn handle_incoming_call(&mut self, mut call: Call) -> Result<(), Error> {
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
