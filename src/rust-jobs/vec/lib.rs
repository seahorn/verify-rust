#![cfg_attr(not(kani), no_std)]
pub use verifier;

extern crate alloc;
use alloc::vec;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let mut v: vec::Vec<i32> = vec![1, 2, 3]; // L = 3

    v.push(4); // L = 4
    v.push(5); // L = 5
    v.push(6); // L = 6
    v.pop(); // L = 5

    let result: usize = v.len();

    verifier::vassert!(result == 5);
}
