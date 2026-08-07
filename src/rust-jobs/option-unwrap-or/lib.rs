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

    // `expected` is opaque to rustc: it is tied to `v` only through
    // verifier::assume!, which is an extern "C" call the front end cannot see
    // through. Asserting `result == v` directly lets LLVM 18 correlate the two
    // uses of `(v & 1)` and fold the assertion to `true` before SeaHorn runs.
    let expected: i32 = verifier::any!();
    if (v & 1) == 0 {
        verifier::assume!(expected == v);
    } else {
        verifier::assume!(expected == 0);
    }

    verifier::vassert!(result == expected);
}
