use crate::linphone::{Call, Error};
use liblinphone_sys::{
    linphone_core_cbs_get_user_data, linphone_core_cbs_set_call_state_changed,
    linphone_core_cbs_set_user_data, linphone_core_get_current_callbacks,
    linphone_factory_create_core_cbs, linphone_factory_get, LinphoneCall, LinphoneCallState,
    LinphoneCore, LinphoneCoreCbs, LinphoneFactory,
};
use std::ptr;

// TODO
use std::os::raw::{c_char, c_int, c_void};

pub struct CoreCallbacks {
    inner: *mut LinphoneCoreCbs,
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

    // LinphoneCoreCbsCallStateChangedCb
    // linphone_core_cbs_set_call_state_changed(cbs, Some(fn))
    // linphone_core_cbs_get_user_data()
    // linphone_core_cbs_set_user_data()
    pub fn do_thing<F>(&mut self, f: F)
    where
        F: Fn(i32, i32),
    {
        let user_data = &f as *const _ as *mut c_void;
        unsafe {
            linphone_core_cbs_set_user_data(self.inner, user_data);

            linphone_core_cbs_set_call_state_changed(self.inner, Some(do_thing_wrapper::<F>));
        }

        // Shim interface function
        extern "C" fn do_thing_wrapper<F>(
            lc: *mut LinphoneCore,
            call: *mut LinphoneCall,
            cstate: LinphoneCallState,
            message: *const c_char,
        ) where
            F: Fn(i32, i32),
        {
            let closure: *mut c_void = unsafe {
                let cbs = linphone_core_get_current_callbacks(lc);
                assert_ne!(cbs, ptr::null_mut());
                linphone_core_cbs_get_user_data(cbs)
            };

            let opt_closure = closure as *mut Option<F>;
            unsafe {
                (*opt_closure).take().unwrap()(1, 1);
            }
        }
    }
}
