#![cfg_attr(not(kani), no_std)]
use verifier;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();
    verifier::assume!(v > 0);
    verifier::assume!(v < i32::MAX/2);

    let result: Option<i32> = if (v & 1) == 1 {
        None
    } else {
        Some(v)
    };

    let result = match double(result) {
        Some(val) => val,
        None => 0
    };

    if (v & 1) == 0 {
        verifier::vassert!(result > v);
    } else {
        verifier::vassert!(result == 0);
    }
}

fn double(x: Option<i32>) -> Option<i32> {
    x.and_then(|num: i32| Some(num * 2))
}
