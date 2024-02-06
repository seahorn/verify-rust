#![cfg_attr(not(kani), no_std)]
pub use verifier;

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let x: i32 = verifier::any!();
    let y: i32 = verifier::any!();
    let z: i32 = verifier::any!();

    verifier::assume!((x <= y) && (x <= z));

    let mut values: Vec<i32> = vec![x, y, z];

    values.sort();
    values.reverse();

    let result: i32 = values[0];

    verifier::vassert!(result >= x);
}
