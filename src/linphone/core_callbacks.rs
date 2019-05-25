use crate::linphone::{Call, Error};
use liblinphone_sys::{
    linphone_call_ref, linphone_core_cbs_get_user_data, linphone_core_cbs_set_call_state_changed,
    linphone_core_cbs_set_user_data, linphone_core_get_current_callbacks,
    linphone_factory_create_core_cbs, linphone_factory_get, LinphoneCall, LinphoneCallState,
    LinphoneCore, LinphoneCoreCbs, LinphoneFactory,
};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr;

pub struct CoreCallbacks {
    pub(super) inner: *mut LinphoneCoreCbs,
}

impl CoreCallbacks {
    pub fn new() -> Result<Self, Error> {
        let inner = unsafe {
            let factory: *mut LinphoneFactory = linphone_factory_get();
            let callbacks: *mut LinphoneCoreCbs = linphone_factory_create_core_cbs(factory);
            callbacks
        };

        if inner == ptr::null_mut() {
            Err(Error::Linphone)
        } else {
            Ok(CoreCallbacks { inner })
        }
    }

    pub fn set_call_state_changed<F>(&mut self, f: F)
    where
        F: Fn(Call, String),
    {
        // TODO - do we loose this closure pointer on the
        // stack when this returns?
        let user_data = &f as *const _ as *mut c_void;
        unsafe {
            linphone_core_cbs_set_user_data(self.inner, user_data);
            linphone_core_cbs_set_call_state_changed(
                self.inner,
                Some(set_call_state_changed_wrapper::<F>),
            );
        }

        // Internal shim interface function
        extern "C" fn set_call_state_changed_wrapper<F>(
            lc: *mut LinphoneCore,
            call: *mut LinphoneCall,
            _cstate: LinphoneCallState,
            message: *const c_char,
        ) where
            F: FnMut(Call, String),
        {
            let closure: *mut c_void = unsafe {
                let cbs = linphone_core_get_current_callbacks(lc);
                assert_ne!(cbs, ptr::null_mut());
                linphone_core_cbs_get_user_data(cbs)
            };

            let call = Call {
                inner: unsafe { linphone_call_ref(call) },
            };

            let msg = unsafe { CStr::from_ptr(message).to_string_lossy().into_owned() };

            let closure: &mut F = unsafe { &mut *(closure as *mut F) };
            (*closure)(call, msg);

            //let opt_closure = closure as *mut Option<F>;
            //unsafe {
            //    (*opt_closure).take().unwrap()(call, msg);
            //}
        }
    }
}
