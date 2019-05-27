mod keypad;
mod keypad_event;
mod linphone;
mod phone;

use crate::keypad::{Keypad, StdinKeypad};
use crate::keypad_event::{KeypadBuffer, KeypadMode};
use crate::linphone::{Call, CallState, CoreCallbacks, CoreContext};
use crate::phone::Phone;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

// TODO - db file override or path management needed
// /home/USER/.local/share/linphone/linphone.db
// doesn't create path

// TODO - a top level Error
//
// - mut mic when not on a call

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

        if call.state() == CallState::CallIncomingReceived {
            if let Err(declined_call) = phone.handle_incoming_call(call.clone()) {
                println!("Declined call");
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

    while should_be_running.load(Ordering::SeqCst) {
        // TODO - handle errors
        phone.handle_events(&mut core_ctx).ok();

        core_ctx.iterate();

        thread::sleep(time::Duration::from_millis(50));
    }
}
