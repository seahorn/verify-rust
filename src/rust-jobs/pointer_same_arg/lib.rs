#![cfg_attr(not(kani), no_std)]

// example taken from page 2 of 
// https://plv.mpi-sws.org/rustbelt/stacked-borrows/

// This example demonstrates a discrepency between Seahorn and Kani.
//
// Llvm will optimze the return value in example1 function. What this means
// is that it will return x_value instead (in llvm_ir). Hence, when validating
// with seahorn, that is the behavior seen.
//
// I am unsure how Kani analyzes this. Never the less, kani will return y_value instead
// since *x does not get optimized to x_value.

use verifier;

#[no_mangle]
fn example1 (x: & mut i32 , y: & mut i32, x_value: i32, y_value: i32 ) -> i32 {
    *x = x_value;
    *y = y_value;
    return *x;
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let x_value:i32 = verifier::any!();
    let y_value:i32 = verifier::any!();

    // verifier::assume!(x_value != y_value);

    let b:bool = verifier::any!();
    verifier::assume!(b == true);

    let mut local = verifier::any!();

    let raw_pointer = & mut local as * mut i32;
    let result:i32 = unsafe { example1 (& mut * raw_pointer , & mut * raw_pointer , x_value, y_value) };

    // this will pass seahorn but not pass kani
    verifier::vassert!((result == x_value) && b);

    // this will pass kani but not pass seahorn
    verifier::vassert!((result == y_value) && b);
}
