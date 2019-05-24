mod linphone;

use crate::linphone::{CallState, CoreCallbacks, CoreContext};
use phonenumber::{country, Mode};
use std::env;
use std::{thread, time};

// TODO - db file override or path management needed
// /home/USER/.local/share/linphone/linphone.db
// doesn't create path

// console uses configs in ~/.linphonec
//
// https://github.com/BelledonneCommunications/linphone/blob/81688335c6415d21a58cd85f775823c8646b2297/console/example/linphonec

// Add some form of config path
// config file = ~/.linphonerc
// zrtpsecrets = ~/.linphone-zidcache

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();

    if args.len() != 1 {
        panic!("Invalid argument");
    }

    let number_arg = args.pop().unwrap();
    let number = phonenumber::parse(Some(country::US), number_arg).unwrap();
    let valid = phonenumber::is_valid(&number);

    if valid == false {
        panic!("Invalid phone number provided\n{:#?}", number);
    }

    let mut core_ctx = CoreContext::new().expect("Core CTX");

    // TODO - what things are needed?
    // https://github.com/BelledonneCommunications/linphone/blob/81688335c6415d21a58cd85f775823c8646b2297/console/linphonec.c#L690

    println!("Core context established");

    println!("Calling {}", number.format().mode(Mode::National));

    // TODO - check if in-call first

    let call = core_ctx.invite(&number).expect("Failed to call");

    let mut call_state = call.state();

    println!("State: {:?}", call_state);

    // linphone_core_is_incoming_invite_pending
    // linphone_core_accept_call
    // linphone_core_in_call
    // linphone_core_get_duration
    // linphone_core_get_remote_address
    // linphone_core_terminate_all_calls

    // linphone_core_cbs_set_call_state_changed
    // fn ptr LinphoneCoreCbsCallStateChangedCb

    // linphone_factory_create_core_cbs(factory)
    let callbacks = CoreCallbacks::new();

    //    callbacks.set_call_state_changed(|core, call, state, msg| {
    //        unimplemented!();
    //    });

    // typedef void(* LinphoneCoreCbsCallStateChangedCb)
    //   (LinphoneCore *lc, LinphoneCall *call, LinphoneCallState cstate, const
    // char *message)

    // linphone_core_add_callbacks(core, cbs)
    // move or ref?
    // Ctx::new(callbacks)?

    // https://aatch.github.io/blog/2015/01/17/unboxed-closures-and-ffi-callbacks/

    loop {
        core_ctx.iterate();

        let new_state = call.state();

        if new_state != call_state {
            println!("State: {:?}", new_state);
        }
        call_state = new_state;

        if call_state == CallState::End {
            println!("Call ended normally");
            break;
        }

        thread::sleep(time::Duration::from_millis(50));
    }

    core_ctx.terminate_call(call).ok();

    println!("All done");
}
