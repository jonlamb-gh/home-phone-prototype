mod linphone;
mod phone;

use crate::linphone::{Call, CallState, CoreCallbacks, CoreContext};
use phonenumber::{country, Mode};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{thread, time};

// TODO - db file override or path management needed
// /home/USER/.local/share/linphone/linphone.db
// doesn't create path

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    if args.len() != 1 {
        panic!("Invalid argument");
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let number_arg = args.pop().unwrap();
    let number = phonenumber::parse(Some(country::US), number_arg).unwrap();
    let valid = phonenumber::is_valid(&number);

    if valid == false {
        panic!("Invalid phone number provided\n{:#?}", number);
    }

    let mut active_call: Option<Call> = None;

    let mut callbacks = CoreCallbacks::new().expect("Callbacks");
    callbacks.set_call_state_changed(|call, msg| {
        println!("Call state changed - State: {:?}\n  {}", call.state(), msg);

        // take a ref/move on the Call?
        // after linphone_core_accept_call()

        active_call = Some(call);
    });

    let mut core_ctx = CoreContext::new(Some(&callbacks)).expect("Core CTX");

    println!("Core context established");

    if core_ctx.in_call() {
        core_ctx.terminate_all_calls().unwrap();
    }

    println!("Calling {}", number.format().mode(Mode::National));

    let call = core_ctx.invite(&number).expect("Failed to call");

    // linphone_core_is_incoming_invite_pending
    // linphone_core_accept_call
    // linphone_core_get_duration
    // linphone_core_get_remote_address
    // linphone_call_get_remote_address_as_string
    // linphone_address_clean
    // linphone_address_as_string_uri_only

    while running.load(Ordering::SeqCst) {
        core_ctx.iterate();

        if call.state() == CallState::End {
            println!("Call ended normally");
            break;
        }

        thread::sleep(time::Duration::from_millis(50));
    }

    core_ctx.terminate_call(call).ok();

    println!("All done");
}
