#![cfg_attr(not(kani), no_std)]
pub use verifier;

extern crate alloc;
use alloc::string::String;
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn entrypt() {
    let x: i32   = verifier::any!();
    let y: i32 = verifier::any!();
    verifier::assume!(y == 0);
    let res = unwrap_or_else(x, y);
    verifier::vassert!(res == -1);
}

fn unwrap_or_else(x: i32, y: i32) -> i32 {
    let result: Result<i32, String> = divide_result(x, y);

    let value: i32 = result.unwrap_or_else(|_| {
        -1
    });
    value
}

fn divide_result(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Cannot divide by zero"))
    } else if a < 0 || b < 0 {
        Err(String::from("Cannot have negative values"))
    } else {
        Ok(a / b)
    }
}