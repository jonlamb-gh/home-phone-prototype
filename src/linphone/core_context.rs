use crate::linphone::{Call, CoreCallbacks, Error};
use liblinphone_sys::{
    linphone_call_ref, linphone_core_clear_call_logs, linphone_core_destroy,
    linphone_core_get_missed_calls_count, linphone_core_in_call, linphone_core_invite,
    linphone_core_is_incoming_invite_pending, linphone_core_iterate,
    linphone_core_reset_missed_calls_count, linphone_core_set_user_certificates_path,
    linphone_core_set_zrtp_secrets_file, linphone_core_terminate_all_calls,
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
    pub(super) inner: *mut LinphoneCore,
}

impl CoreContext {
    pub fn new(callbacks: Option<&CoreCallbacks>) -> Result<Self, Error> {
        // TODO - error handling
        let home = match env::var(ENV_HOME) {
            Ok(val) => val,
            Err(e) => panic!("could not find {}: {}", ENV_HOME, e),
        };

        let home_path = Path::new(&home);

        let inner = unsafe {
            let factory: *mut LinphoneFactory = linphone_factory_get();

            let callback_ptr: *mut LinphoneCoreCbs = if let Some(cb) = callbacks {
                cb.inner
            } else {
                linphone_factory_create_core_cbs(factory)
            };

            let core: *mut LinphoneCore = linphone_factory_create_core(
                factory,
                callback_ptr,
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

    pub fn terminate_all_calls(&mut self) -> Result<(), Error> {
        let ret = unsafe { linphone_core_terminate_all_calls(self.inner) };

        if ret == 0 {
            Ok(())
        } else {
            Err(Error::Linphone)
        }
    }

    pub fn in_call(&self) -> bool {
        let ret = unsafe { linphone_core_in_call(self.inner) };
        ret != 0
    }

    pub fn is_incoming_invite_pending(&mut self) -> bool {
        let ret = unsafe { linphone_core_is_incoming_invite_pending(self.inner) };
        ret != 0
    }

    pub fn missed_calls_count(&mut self, reset_count: bool) -> Result<usize, Error> {
        let ret = unsafe { linphone_core_get_missed_calls_count(self.inner) };

        if ret >= 0 {
            if reset_count == true {
                unsafe {
                    linphone_core_reset_missed_calls_count(self.inner);
                }
            }

            Ok(ret as usize)
        } else {
            Err(Error::Linphone)
        }
    }

    pub fn clear_call_logs(&mut self) -> Result<(), Error> {
        unsafe {
            linphone_core_clear_call_logs(self.inner);
        }
        Ok(())
    }
}

impl Drop for CoreContext {
    fn drop(&mut self) {
        self.terminate_all_calls()
            .map_err(|_e| println!("Failed to terminate calls"))
            .ok();

        unsafe {
            linphone_core_destroy(self.inner);
        }
    }
}
