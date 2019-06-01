mod display_data;
mod keypad;
mod keypad_event;
mod linphone;
mod phone;

use crate::display_data::DisplayData;
use crate::linphone::{CallState, CoreCallbacks, CoreContext, Error};
use crate::phone::Phone;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

// TODO - db file override or path management needed
// /home/USER/.local/share/linphone/linphone.db
// doesn't create path

// check out keypad
// https://github.com/e-matteson/keypad
//
// with
// https://github.com/golemparts/rppal

// TODO
// - a top level Error
//
// - db/path management
// linphone_core_set_call_logs_database_path
//
// - mut mic when not on a call
// linphone_core_enable_mic
// linphone_core_mic_enabled
//
// - missed call log, logic to clear/ack
// linphone_core_get_missed_calls_count
// linphone_core_reset_missed_calls_count
//
// linphone_core_get_call_logs
// linphone_core_get_last_outgoing_call_log
// linphone_core_clear_call_logs
//
// - do something with call log?
// linphone_call_log_get_call_id
// linphone_call_log_get_duration
// linphone_call_log_get_remote_address
//
// - dtmf sounds to the user?
// linphone_core_play_dtmf

fn main() {
    // SIGINT will do a graceful shutdown
    let should_be_running = Arc::new(AtomicBool::new(true));
    let r = should_be_running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut phone = Phone::new();

    let mut callbacks = CoreCallbacks::new().expect("Callbacks");
    callbacks.set_call_state_changed(|call, msg| {
        println!("Call state changed - State: {:?}\n  {}", call.state(), msg);

        // TODO - handle errors
        // terminate all calls and reset phone?
        if call.state() == CallState::CallIncomingReceived {
            if let Err(e) = phone.handle_incoming_call(call.clone()) {
                if e != Error::CallInProgress {
                    phone.recover_from_error();
                }
            }
        }
    });

    let mut core_ctx = CoreContext::new(false, Some(&callbacks)).expect("Core CTX");

    // Drop any pending/existing calls on startup
    if core_ctx.in_call() || core_ctx.is_incoming_invite_pending() {
        println!("Terminating pending calls before initializing");
        core_ctx.terminate_all_calls().unwrap();
    }

    // TODO
    // Call.remote_address().as_number() -> Result<PhoneNumber>
    // ok if valid, default to display address

    let mut display_data = DisplayData::new();

    while should_be_running.load(Ordering::SeqCst) {
        // TODO - handle errors
        // terminate all calls and reset phone?
        if let Err(_e) = phone.handle_events(&mut core_ctx) {
            phone.recover_from_error();
            core_ctx.terminate_all_calls().unwrap();
        }

        core_ctx.iterate();

        display_data.update(&phone);

        //println!("{}", display_data);

        // TODO - wake/sleep
        thread::sleep(time::Duration::from_millis(50));
    }

    println!("{}", display_data);
}
