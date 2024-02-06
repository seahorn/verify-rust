#![cfg_attr(not(kani), no_std)]

use verifier;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let mut x: i32 = verifier::any!();
    verifier::assume!(x < 10);
    x += 4;

    verifier::vassert!(x < 14);
}
