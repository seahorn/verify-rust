#![cfg_attr(not(kani), no_std)]

use verifier;

use tinyvec::ArrayVec;


// Testing the error fixed here: https://github.com/Lokathor/tinyvec/pull/178
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let vec: ArrayVec<[u32; u16::MAX as usize + 1]> = ArrayVec::new();

    // When using previous versions of the library where the error is still present,
    // this assertion should fail. For newer versions, it should pass.
    verifier::vassert!(vec.capacity() == u16::MAX as usize);
}
