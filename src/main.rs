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

    let mut callbacks = CoreCallbacks::new().expect("Callbacks");
    callbacks.set_call_state_changed(|call, msg| {
        println!("Call state changed - State: {:?}\n  {}", call.state(), msg);

        // deprecated?
        // linphone_core_is_incoming_invite_pending
        //
        // linphone_call_accept
        // linphone_call_decline
        // linphone_call_terminate

        // copy for
        //phone.take_incoming_call(call.copy(), core)
    });

    let mut core_ctx = CoreContext::new(Some(&callbacks)).expect("Core CTX");

    println!("Core context established");

    if core_ctx.in_call() {
        core_ctx.terminate_all_calls().unwrap();
    }

    println!("Calling {}", number.format().mode(Mode::National));

    let mut call = core_ctx.invite(&number).expect("Failed to call");

    // linphone_core_is_incoming_invite_pending
    // linphone_core_accept_call

    while running.load(Ordering::SeqCst) {
        core_ctx.iterate();

        if call.state() == CallState::End {
            println!("Call ended normally");
            break;
        }

        thread::sleep(time::Duration::from_millis(50));
    }

    let duration = call.duration();

    let address = call.remote_address();

    call.terminate().ok();

    println!("duration: {:?}", duration);
    println!("address: {}", address);

    println!("All done");
}
