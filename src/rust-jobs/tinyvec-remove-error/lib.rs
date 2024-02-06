#![cfg_attr(not(kani), no_std)]

use verifier;

use tinyvec::ArrayVec;

// Testing the error fixed here: https://github.com/Lokathor/tinyvec/pull/29
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
pub extern "C" fn entrypt() {
    let mut v: ArrayVec<[u32; 8]> = ArrayVec::new();
    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let remove_point: usize = verifier::any!();
    verifier::assume!(remove_point <= 8);

    v.remove(remove_point);

    // When using previous versions of the library where the error is still present,
    // the removal doesn't panic if the index is out of bounds and the "verifier::vassert!f(false)" 
    // will be reached. For newer versions, it will panic and the assertion won't be reached.
    if remove_point < len {
        verifier::vassert!(v.len() == len - 1);
    } else {
        // If remove_point is out of bounds, then the call to remove should panic and this assertion should not be reachable.
        verifier::vassert!(false);
    }
}
