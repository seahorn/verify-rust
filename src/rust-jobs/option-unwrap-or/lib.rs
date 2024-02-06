#![cfg_attr(not(kani), no_std)]
use verifier;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();

    let result: Option<i32> = if (v & 1) == 1 { 
        None 
    } else { 
        Some(v)
    };

    let result = result.unwrap_or(0);

    if (v & 1) == 0 {
        verifier::vassert!(result == v);
    } else {
        verifier::vassert!(result == 0);
    }
}
