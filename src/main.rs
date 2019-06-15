mod display;
mod display_data;
mod keypad;
mod keypad_event;
mod linphone;
mod phone;

#[macro_use]
extern crate keypad as keypad_builder;

use crate::display::{Display, Row};
use crate::display_data::DisplayData;
use crate::linphone::{CoreCallbacks, CoreContext, Error};
use crate::phone::Phone;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// TODO - db file override or path management needed
// /home/USER/.local/share/linphone/linphone.db
// doesn't create path

// TLS and SRTP
// https://stackoverflow.com/questions/41462750/ssl-client-certificate-verification-on-linphone
//
// make a deb
// https://lib.rs/crates/cargo-deb

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

// some errors
//2019-06-13 22:42:05:833 liblinphone-error-Could not resolv
// /home/pi/.linphone.ecstate: No such file or directory

fn main() {
    // SIGINT will do a graceful shutdown
    let should_be_running = Arc::new(AtomicBool::new(true));
    let r = should_be_running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut display_data = DisplayData::new();
    let mut display = Display::new().unwrap();

    // TODO - startup display, clear
    display.display(&display_data).unwrap();
    display.set_row(Row::R0, "Loading phonebook").unwrap();

    let mut phone = Phone::new();
    display.display(&display_data).unwrap();

    // TODO - add core context ref
    let mut callbacks = CoreCallbacks::new().expect("Callbacks");
    callbacks.set_call_state_changed(|call, msg| {
        println!("Call state changed - State: {:?}\n  {}", call.state(), msg);

        // TODO - handle errors
        // terminate all calls and reset phone?
        if let Err(e) = phone.handle_call_state_changed(call) {
            if e != Error::CallInProgress {
                phone.recover_from_error();
            }
        }

        // TODO - need to handle phone state

        display_data.update(None, &phone);

        display.display(&display_data).unwrap();
    });

    let mut core_ctx = CoreContext::new(Some(&callbacks)).expect("Core CTX");

    // Drop any pending/existing calls on startup
    if core_ctx.in_call() || core_ctx.is_incoming_invite_pending() {
        println!("Terminating pending calls before initializing");
        core_ctx.terminate_all_calls().unwrap();
    }

    display_data.update(None, &phone);
    display.display(&display_data).unwrap();

    let mut last_update = Instant::now();
    let mut last_redraw = Instant::now();

    while should_be_running.load(Ordering::SeqCst) {
        // TODO - handle errors
        // terminate all calls and reset phone?

        let mut should_redraw: bool = false;

        match phone.handle_events(&mut core_ctx) {
            Err(_e) => {
                phone.recover_from_error();
                core_ctx.terminate_all_calls().unwrap();
                should_redraw = true;
            }
            Ok(state_changed) => should_redraw |= state_changed,
        }

        // NOTE: example polling was 50 ms
        if Instant::now().duration_since(last_update) >= Duration::from_millis(50) {
            core_ctx.iterate();
            last_update = Instant::now();
        }

        // TODO - only redraw when needed?
        // on phone state change
        // or once a second to update the clock/date display
        // use keypad gpio interrupts?
        //
        if Instant::now().duration_since(last_redraw) >= Duration::from_secs(1)
            || (should_redraw == true)
        {
            //println!("redraw");
            let missed_calls = core_ctx.missed_calls_count(false).unwrap_or(0);
            display_data.update(Some(missed_calls), &phone);
            display.display(&display_data).unwrap();
            last_redraw = Instant::now();
        }

        // TODO - wake/sleep
        if phone.is_idle() == true {
            thread::sleep(Duration::from_millis(20));
        } else {
            thread::sleep(Duration::from_millis(5));
        }
    }

    println!("{}", display_data);
}
