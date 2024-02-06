#![cfg_attr(not(kani), no_std)]
extern crate core;
use core::result::Result;

use verifier;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let mut x: i32 = verifier::any!();
    verifier::assume!(x > 0);
    verifier::assume!(x < i32::MAX/2);

    let val: Result<&mut i32, i32> = Ok(&mut x);
    let cloned: Result<i32, i32> = val.cloned();
    let result: i32 = cloned.unwrap()*2;

    verifier::vassert!(result > x);
}
