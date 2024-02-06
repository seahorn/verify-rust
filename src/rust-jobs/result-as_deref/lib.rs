#![cfg_attr(not(kani), no_std)]
extern crate alloc;
use alloc::string::String;

extern crate core;
use core::result::Result;

use verifier;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();
    verifier::assume!(v < i32::MAX/2);

    let x: Result<String, i32> = Err(v);
    let y: Result<&str, &i32> = Err(&v);

    let x_error: i32 = match x {
        Err(err) => err,
        _ => 0,
    };

    let y_error: i32 = match y {
        Err(err) => *err,
        _ => 0,
    };

    let result: i32 = x_error + y_error;

    verifier::vassert!(result == v*2);
}
