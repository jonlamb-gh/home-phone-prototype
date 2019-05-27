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

//use phonenumber::{country, Mode};
//let number_arg = args.pop().unwrap();
//let number = phonenumber::parse(Some(country::US), number_arg).unwrap();
//let valid = phonenumber::is_valid(&number);
//if valid == false {
//    panic!("Invalid phone number provided\n{:#?}", number);
//}
//println!("Calling {}", number.format().mode(Mode::National));
//let mut call = core_ctx.invite(&number).expect("Failed to call");
//
// call ends normally on CallState::End
//let duration = call.duration();
//let address = call.remote_address();
//call.terminate().ok();

fn main() {
    // SIGINT will do a graceful shutdown
    let should_be_running = Arc::new(AtomicBool::new(true));
    let r = should_be_running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut keybuf = KeypadBuffer::new();
    let mut keypad = StdinKeypad::new();
    let mut phone = Phone::new();

    let mut callbacks = CoreCallbacks::new().expect("Callbacks");
    callbacks.set_call_state_changed(|call, msg| {
        println!("Call state changed - State: {:?}\n  {}", call.state(), msg);

        //let call_c = call.clone();
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
        // TODO - buffer the keys when not in a call,
        // fill a number until '#'
        //
        // when on a call, each key is treated as a dtmf key
        // to be sent on the call
        // '*' to hangup?
        if let Some(event) = keypad.next_event() {
            println!("{:?}", event);

            if let Some(string) = keybuf.push(KeypadMode::WaitForUserDial, event) {
                println!("  -> {:?}", string);
            }
        }

        core_ctx.iterate();

        thread::sleep(time::Duration::from_millis(50));
    }
}
