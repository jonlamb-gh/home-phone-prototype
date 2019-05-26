use crate::linphone::{Call, CallState, CoreCallbacks, CoreContext, Error};
use phonenumber::PhoneNumber;

pub struct Phone {
    active_call: Option<Call>,
}

impl Phone {
    pub fn new() -> Self {
        Phone { active_call: None }
    }

    pub fn on_call(&self) -> bool {
        self.active_call.is_some()
    }

    pub fn end_call(&mut self) -> Result<(), Error> {
        if let Some(mut call) = self.active_call.take() {
            call.terminate()?;
        }
        Ok(())
    }

    // TODO - call info/duration/etc
    //
    // inspect CallState

    // Returns Call if already on a call or if incoming CallState is not ...
    pub fn take_incoming_call(&mut self, mut call: Call) -> Result<(), Call> {
        if self.active_call.is_none() {
            let res = call.accept();
            if res.is_ok() {
                self.active_call = Some(call);
                Ok(())
            } else {
                Err(call)
            }
        } else {
            Err(call)
        }
    }

    pub fn place_outgoing_call(
        &mut self,
        number: PhoneNumber,
        core: &mut CoreContext,
    ) -> Result<(), Error> {
        if self.active_call.is_none() {
            self.active_call = Some(core.invite(&number)?);
            Ok(())
        } else {
            Err(Error::CallInProgress)
        }
    }
}
