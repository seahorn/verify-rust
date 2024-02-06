#![cfg_attr(not(kani), no_std)]
use verifier;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();

    verifier::assume!(v > 0);
    verifier::assume!(v < i32::MAX/2);

    let result: i32 = match double_if_even(v) {
        Some(v) => v,
        None => 0
    };

    if (v & 1) == 0 {
        verifier::vassert!(result > v);
    } else {
        verifier::vassert!(result == 0);
    }
}

fn double_if_even(num: i32) -> Option<i32> {
    if (num & 1) == 0 {
        Some(2 * num)
    } else {
        None
    }
}
