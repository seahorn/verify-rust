#![cfg_attr(not(kani), no_std)]
use verifier;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();
    let w: i32 = verifier::any!();

    let val1: Option<i32> = if (v & 1) == 1 { 
        None 
    } else { 
        Some(v)
    };

    let val2: Option<i32> = Some(w);

    let result: i32 = val1.or(val2).unwrap();

    if (v & 1) == 0 {
        verifier::vassert!(result == v);
    } else {
        verifier::vassert!(result == w);
    }
    // panic!();
}
