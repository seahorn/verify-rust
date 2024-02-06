#![cfg_attr(not(kani), no_std)]
#![feature(core_panic)]

use verifier;


// ************************************
//      Reminder:
//          When using custom panic, make sure to update CMakeLists.txt
//          Add CUSTOM_PANIC_NO_STD as a variable argument
//          Also make sure to pass features = ["panic_error"] under sea
// ************************************

#[no_mangle]
pub extern "C" fn entrypt() {
    test();
    verifier::vassert!(true);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test() {
    panic!();

    // Another way to panic would be:
    // core::panicking::panic("explicit panic");
}
