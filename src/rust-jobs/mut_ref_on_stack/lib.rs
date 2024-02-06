#![cfg_attr(not(kani), no_std)]

// example taken from page 7 of 
// https://plv.mpi-sws.org/rustbelt/stacked-borrows/

use verifier;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {

    let x_value:i32 = verifier::any!();
    let y_value:i32 = verifier::any!();

    let mut local:i32 = verifier::any!();

    let raw_pointer1 = & mut local as * mut i32;
    let raw_pointer2 = & mut local as * mut i32;

    let b:bool = verifier::any!();
    verifier::assume!(b == true);

    let mut result = 0;

    unsafe {
        *raw_pointer1 = x_value;
        *raw_pointer2 = y_value;
        result = *raw_pointer1;
    }

    verifier::vassert!((result == y_value) && b);

}
