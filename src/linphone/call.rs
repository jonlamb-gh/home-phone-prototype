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
    _LinphoneCallState_LinphoneCallStateUpdating, linphone_call_get_state, linphone_call_unref,
    LinphoneCall,
};

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
}

impl Drop for Call {
    fn drop(&mut self) {
        unsafe {
            linphone_call_unref(self.inner);
        }
    }
}
