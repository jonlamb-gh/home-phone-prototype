use crate::linphone::{Call, Error};
use liblinphone_sys::{
    linphone_call_ref, linphone_core_destroy, linphone_core_enable_logs, linphone_core_invite,
    linphone_core_iterate, linphone_core_set_user_certificates_path,
    linphone_core_set_zrtp_secrets_file, linphone_core_terminate_call,
    linphone_factory_create_core, linphone_factory_create_core_cbs, linphone_factory_get,
    LinphoneCall, LinphoneCore, LinphoneCoreCbs, LinphoneFactory,
};
use phonenumber::{Mode, PhoneNumber};
use std::env;
use std::path::Path;
use std::ptr;

const ENV_HOME: &'static str = "HOME";

const CONFIG_FILE: &'static str = ".linphonerc\0";
const USR_CERTS_PATH: &'static str = ".linphone-usr-crt\0";
const ZRTP_SEC_FILE: &'static str = ".linphone-zidcache\0";

pub struct CoreContext {
    inner: *mut LinphoneCore,
}

impl CoreContext {
    pub fn new() -> Result<Self, Error> {
        // TODO - error handling
        let home = match env::var(ENV_HOME) {
            Ok(val) => val,
            Err(e) => panic!("could not find {}: {}", ENV_HOME, e),
        };

        let home_path = Path::new(&home);

        let inner = unsafe {
            //linphone_core_enable_logs(ptr::null_mut());

            let factory: *mut LinphoneFactory = linphone_factory_get();

            let callbacks: *mut LinphoneCoreCbs = linphone_factory_create_core_cbs(factory);

            let core: *mut LinphoneCore = linphone_factory_create_core(
                factory,
                callbacks,
                home_path
                    .join(CONFIG_FILE)
                    .to_str()
                    .expect("Bad file path")
                    .as_ptr() as _,
                ptr::null(),
            );

            if core != ptr::null_mut() {
                linphone_core_set_zrtp_secrets_file(
                    core,
                    home_path
                        .join(ZRTP_SEC_FILE)
                        .to_str()
                        .expect("Bad file path")
                        .as_ptr() as _,
                );

                linphone_core_set_user_certificates_path(
                    core,
                    home_path
                        .join(USR_CERTS_PATH)
                        .to_str()
                        .expect("Bad file path")
                        .as_ptr() as _,
                );
            }

            core
        };

        if inner == ptr::null_mut() {
            Err(Error::Linphone)
        } else {
            Ok(CoreContext { inner })
        }
    }

    pub fn iterate(&mut self) {
        unsafe {
            linphone_core_iterate(self.inner);
        }
    }

    pub fn invite(&mut self, dest: &PhoneNumber) -> Result<Call, Error> {
        let url_string = dest.format().mode(Mode::National).to_string();

        let call_ptr: *mut LinphoneCall =
            unsafe { linphone_core_invite(self.inner, url_string.as_ptr() as _) };

        if call_ptr == ptr::null_mut() {
            Err(Error::Linphone)
        } else {
            Ok(Call {
                inner: unsafe { linphone_call_ref(call_ptr) },
            })
        }
    }

    pub fn terminate_call(&mut self, call: Call) -> Result<(), Error> {
        let ret = unsafe { linphone_core_terminate_call(self.inner, call.inner) };

        // Call::drop() is invoked to unref
        if ret == 0 {
            Ok(())
        } else {
            Err(Error::Linphone)
        }
    }
}

impl Drop for CoreContext {
    fn drop(&mut self) {
        unsafe {
            linphone_core_destroy(self.inner);
        }
    }
}
