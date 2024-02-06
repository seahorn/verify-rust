#![cfg_attr(not(kani), no_std)]
extern crate alloc;
use alloc::string::String;

use verifier;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();
    verifier::assume!(v < i32::MAX/2);

    let x: Result<Option<i32>, String> = Ok(Some(v));
    let y: Option<Result<i32, String>> = Some(Ok(v));
    let result: i32 = x.transpose().unwrap().unwrap() +  y.unwrap().unwrap();

    verifier::vassert!(result == v*2);
}
