#![cfg_attr(not(kani), no_std)]

// Same swap as raw-pointers, but the assertion forgets that the two
// pointers may alias. When a == b the swap is a no-op, so x keeps x0 and the
// assertion fails whenever x0 != y0. Expected result: sat (bug found).

use verifier;

#[no_mangle]
#[inline(never)]
unsafe fn swap_raw(a: *mut i32, b: *mut i32) {
    let tmp = *a;
    *a = *b;
    *b = tmp;
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let x0: i32 = verifier::any!();
    let y0: i32 = verifier::any!();
    let mut x = x0;
    let mut y = y0;
    let alias: bool = verifier::any!();
    let px: *mut i32 = &mut x;
    let py: *mut i32 = if alias { px } else { &mut y };
    unsafe { swap_raw(px, py) };
    verifier::vassert!(x == y0);
}
