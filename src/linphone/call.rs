// TODO
// linphone_call_get_authentication_token_verified
// other indicators

use crate::linphone::reason::Reason;
use crate::linphone::Error;
use liblinphone_sys::{
    _LinphoneCallState_LinphoneCallStateConnected,
    _LinphoneCallState_LinphoneCallStateEarlyUpdatedByRemote,
    _LinphoneCallState_LinphoneCallStateEarlyUpdating, _LinphoneCallState_LinphoneCallStateEnd,
    _LinphoneCallState_LinphoneCallStateError, _LinphoneCallState_LinphoneCallStateIdle,
    _LinphoneCallState_LinphoneCallStateIncomingEarlyMedia,
    _LinphoneCallState_LinphoneCallStateIncomingReceived,
    _LinphoneCallState_LinphoneCallStateOutgoingEarlyMedia,
    _LinphoneCallState_LinphoneCallStateOutgoingInit,
    _LinphoneCallState_LinphoneCallStateOutgoingProgress,
    _LinphoneCallState_LinphoneCallStateOutgoingRinging,
    _LinphoneCallState_LinphoneCallStatePaused, _LinphoneCallState_LinphoneCallStatePausedByRemote,
    _LinphoneCallState_LinphoneCallStatePausing, _LinphoneCallState_LinphoneCallStateReferred,
    _LinphoneCallState_LinphoneCallStateReleased, _LinphoneCallState_LinphoneCallStateResuming,
    _LinphoneCallState_LinphoneCallStateStreamsRunning,
    _LinphoneCallState_LinphoneCallStateUpdatedByRemote,
    _LinphoneCallState_LinphoneCallStateUpdating, linphone_address_get_username,
    linphone_call_accept, linphone_call_decline, linphone_call_get_duration,
    linphone_call_get_remote_address, linphone_call_get_state, linphone_call_ref,
    linphone_call_send_dtmf, linphone_call_terminate, linphone_call_unref, LinphoneCall,
};
use std::ffi::CStr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Unknown,
    // From linphone
    CallIdle,
    CallIncomingReceived,
    OutgoingInit,
    OutgoingProgress,
    OutgoingRinging,
    OutgoingEarlyMedia,
    Connected,
    StreamsRunning,
    Pausing,
    Paused,
    Resuming,
    Referred,
    Error,
    End,
    PausedByRemote,
    UpdatedByRemote,
    IncomingEarlyMedia,
    Updating,
    Released,
    EarlyUpdatedByRemote,
    EarlyUpdating,
}

pub struct Call {
    pub(super) inner: *mut LinphoneCall,
}

impl Call {
    pub fn state(&self) -> State {
        let istate = unsafe { linphone_call_get_state(self.inner) };

        #[allow(non_snake_case, non_upper_case_globals)]
        match istate {
            _LinphoneCallState_LinphoneCallStateIdle => State::CallIdle,
            _LinphoneCallState_LinphoneCallStateIncomingReceived => State::CallIncomingReceived,
            _LinphoneCallState_LinphoneCallStateOutgoingInit => State::OutgoingInit,
            _LinphoneCallState_LinphoneCallStateOutgoingProgress => State::OutgoingProgress,
            _LinphoneCallState_LinphoneCallStateOutgoingRinging => State::OutgoingRinging,
            _LinphoneCallState_LinphoneCallStateOutgoingEarlyMedia => State::OutgoingEarlyMedia,
            _LinphoneCallState_LinphoneCallStateConnected => State::Connected,
            _LinphoneCallState_LinphoneCallStateStreamsRunning => State::StreamsRunning,
            _LinphoneCallState_LinphoneCallStatePausing => State::Pausing,
            _LinphoneCallState_LinphoneCallStatePaused => State::Paused,
            _LinphoneCallState_LinphoneCallStateResuming => State::Resuming,
            _LinphoneCallState_LinphoneCallStateReferred => State::Referred,
            _LinphoneCallState_LinphoneCallStateError => State::Error,
            _LinphoneCallState_LinphoneCallStateEnd => State::End,
            _LinphoneCallState_LinphoneCallStatePausedByRemote => State::PausedByRemote,
            _LinphoneCallState_LinphoneCallStateUpdatedByRemote => State::UpdatedByRemote,
            _LinphoneCallState_LinphoneCallStateIncomingEarlyMedia => State::IncomingEarlyMedia,
            _LinphoneCallState_LinphoneCallStateUpdating => State::Updating,
            _LinphoneCallState_LinphoneCallStateReleased => State::Released,
            _LinphoneCallState_LinphoneCallStateEarlyUpdatedByRemote => State::EarlyUpdatedByRemote,
            _LinphoneCallState_LinphoneCallStateEarlyUpdating => State::EarlyUpdating,
            _ => State::Unknown,
        }
    }

    pub fn duration(&self) -> Duration {
        let dur_sec = unsafe { linphone_call_get_duration(self.inner) };
        Duration::from_secs(dur_sec as _)
    }

    pub fn remote_address(&self) -> String {
        unsafe {
            let address = linphone_call_get_remote_address(self.inner);
            let username = linphone_address_get_username(address);
            CStr::from_ptr(username).to_string_lossy().into_owned()
        }
    }

    pub fn accept(&mut self) -> Result<(), Error> {
        if self.state() == State::CallIncomingReceived {
            let ret = unsafe { linphone_call_accept(self.inner) };

            if ret == 0 {
                Ok(())
            } else {
                Err(Error::Linphone)
            }
        } else {
            Err(Error::CallNotIncoming)
        }
    }

    pub fn terminate(&mut self) -> Result<(), Error> {
        let ret = unsafe { linphone_call_terminate(self.inner) };

        if ret == 0 {
            Ok(())
        } else {
            Err(Error::Linphone)
        }
    }

    pub fn decline(&mut self, reason: Reason) -> Result<(), Error> {
        if self.state() == State::CallIncomingReceived {
            let ret = unsafe { linphone_call_decline(self.inner, reason.into()) };

            if ret == 0 {
                Ok(())
            } else {
                Err(Error::Linphone)
            }
        } else {
            Err(Error::CallNotIncoming)
        }
    }

    pub fn send_dtmf(&mut self, dtmf: char) -> Result<(), Error> {
        let ret = unsafe { linphone_call_send_dtmf(self.inner, dtmf as _) };

        if ret == 0 {
            Ok(())
        } else {
            Err(Error::Linphone)
        }
    }
}

impl Drop for Call {
    fn drop(&mut self) {
        unsafe {
            linphone_call_unref(self.inner);
        }
    }
}

impl Clone for Call {
    fn clone(&self) -> Call {
        Call {
            inner: unsafe { linphone_call_ref(self.inner) },
        }
    }
}
