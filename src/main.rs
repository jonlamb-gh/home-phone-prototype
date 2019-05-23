use liblinphone_sys::*;
use std::ptr::null;

// TODO - db file override or path management needed
// /home/USER/.local/share/linphone/linphone.db
// doesn't create path

fn main() {
    println!("Hello, world!");

    unsafe {
        linphone_core_enable_logs(null::<FILE>() as *mut _);

        println!("\nNew\n");

        let factory: *mut LinphoneFactory = linphone_factory_get();

        let callbacks: *mut LinphoneCoreCbs = linphone_factory_create_core_cbs(factory);

        let ctx: *mut LinphoneCore = linphone_factory_create_core(
            factory,
            callbacks,
            null::<::std::os::raw::c_char>(),
            null::<::std::os::raw::c_char>(),
        );
        assert_ne!(ctx, null::<LinphoneCore>() as _);

        println!("\nGot Ctx\n");

        linphone_core_destroy(ctx);
    }

    println!("Done");
}
