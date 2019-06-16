use crate::keypad::Keypad;
use crate::keypad_event::{KeypadBuffer, KeypadEvent, KeypadMode};
use crate::linphone::{Call, CallState, CoreContext, Error, Reason};
use phonenumber::{country, Mode};

// TODO - fix keypad buffer
// - knows how to reset itself, long press
// - knows when to check for num, long press
// or get rid of it and do key event handling in here

pub struct Phone {
    core: Option<CoreContext>,
    keypad: Keypad,
    keybuf: KeypadBuffer,
    state: State,
}

pub enum State {
    WaitingForEvents,
    HandlePendingCall(Call),
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
            core: None,
            keypad: Keypad::new(),
            keybuf: KeypadBuffer::new(),
            state: State::WaitingForEvents,
        }
    }

    // TODO - session state this or something else
    // Unitialized/Initialized
    pub fn set_core(&mut self, mut core: CoreContext) {
        // Drop any pending/existing calls on startup
        if core.in_call() || core.is_incoming_invite_pending() {
            println!("Terminating pending calls before initializing");
            core.terminate_all_calls().unwrap();
        }

        self.core = Some(core);
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
        println!("Recovering from error");

        match &mut self.state {
            State::HandlePendingCall(pending_call) => {
                pending_call.terminate().ok();
            }
            State::OnGoingCall(call) => {
                call.terminate().ok();
            }
            _ => (),
        }

        self.state = State::WaitingForEvents;
        self.keybuf.clear();

        if let Some(core) = self.core.as_mut() {
            core.terminate_all_calls().unwrap();
        }
    }

    pub fn iterate(&mut self) {
        self.core.as_mut().unwrap().iterate();
    }

    // TODO - clean this up, move to session type indexed by state?
    // return bool, true for some state change
    pub fn handle_events(&mut self) -> Result<bool, Error> {
        let mut state_changed: bool = false;

        match &mut self.state {
            State::WaitingForEvents => {
                if self.keypad.next_event().map_or(false, |event| {
                    state_changed = true;
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
                            let call = self.core.as_mut().unwrap().invite(&number)?;
                            self.state = State::OnGoingCall(call);
                        } else {
                            println!("Ignoring invalid phone number");
                        }
                    }
                }
            }
            State::HandlePendingCall(pending_call) => {
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
            CallState::Error => {
                println!("Terminating from error");
                self.recover_from_error();
            }
            CallState::Released => (),
            _ => (),
        }

        Ok(())
    }

    // TODO - make this better
    fn handle_incoming_call(&mut self, mut call: Call) -> Result<(), Error> {
        if call.state() == CallState::CallIncomingReceived {
            match &self.state {
                State::WaitingForEvents => {
                    println!("New Incoming {}", call.remote_address());

                    // Consume the pending call
                    self.state = State::HandlePendingCall(call);
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
