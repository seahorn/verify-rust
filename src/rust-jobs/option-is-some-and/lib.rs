#![feature(is_some_with)]
#![cfg_attr(not(kani), no_std)]
use verifier;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();
    verifier::assume!(v <= 0);
    
    let value: Option<i32> = if (v & 1) == 0 {
        Some(v)
    } else {
        None
    };

    let result: bool = value.is_some_and(|num: &i32| *num > 0);

    verifier::vassert!(result == false);
}
