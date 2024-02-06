#![cfg_attr(not(kani), no_std)]
extern crate core;
use core::result::Result;

use verifier;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();
    verifier::assume!(v > 0);
    verifier::assume!(v < i32::MAX/2);

    let x: Result<&i32, i32> = Ok(&v);
    let copied: Result<i32, i32> = x.copied();
    let result: i32 = copied.unwrap()*2;

    verifier::vassert!(result > v);
}
