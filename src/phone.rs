use crate::keypad::{Keypad, StdinKeypad};
use crate::keypad_event::{KeypadBuffer, KeypadEvent, KeypadMode};
use crate::linphone::{Call, CoreContext, Error};
use phonenumber::{country, Mode, PhoneNumber};

pub struct Phone {
    keypad: StdinKeypad,
    keybuf: KeypadBuffer,
    //call: Option<Call>,
    state: State,
}

pub enum State {
    WaitingForEvents,
    HandlePendingCall(Call),
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

    pub fn handle_events(&mut self, core: &mut CoreContext) -> Result<(), Error> {
        match &mut self.state {
            State::WaitingForEvents => {
                if self.keypad.next_event().map_or(false, |event| {
                    self.keybuf.push(KeypadMode::WaitForUserDial, event)
                }) {
                    if let Ok(number) = phonenumber::parse(Some(country::US), self.keybuf.data()) {
                        if phonenumber::is_valid(&number) {
                            println!("Calling {}", number.format().mode(Mode::National));
                            let call = core.invite(&number)?;
                            self.state = State::OnGoingCall(call);
                        } else {
                            println!("Ignoring invalid phone number");
                        }
                    }
                    self.keybuf.clear();
                }
            }
            State::HandlePendingCall(pending_call) => {
                // TODO
                unimplemented!()
            }
            State::OnGoingCall(call) => {
                if let Some(event) = self.keypad.next_event() {
                    // Buffer the DTMF for display purposes
                    let _ = self.keybuf.push(KeypadMode::Dtmf, event);

                    if event == KeypadEvent::KeyPress('*') {
                        println!("Terminating active call");
                        call.terminate()?;
                        self.state = State::WaitingForEvents;
                    } else if let KeypadEvent::KeyPress(c) = event {
                        println!("Sending '{}' as DTMF", c);
                        call.send_dtmf(c)?;
                    }
                }
            }
        }

        Ok(())
    }

    // Expects state == CallIncomingReceived ?
    // call is decline if failed
    pub fn handle_incoming_call(&mut self, mut call: Call) -> Result<(), Call> {
        unimplemented!()
    }
}
