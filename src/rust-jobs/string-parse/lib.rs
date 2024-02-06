#![cfg_attr(not(kani), no_std)]
pub use verifier;

extern crate alloc;
use alloc::string::String;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let value: String = String::from("42");
    let result: i32 = value.parse().unwrap();

    verifier::vassert!(result == 42);
}
