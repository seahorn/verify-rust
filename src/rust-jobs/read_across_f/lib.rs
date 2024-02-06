#![cfg_attr(not(kani), no_std)]

// example taken from page 16 of 
// https://plv.mpi-sws.org/rustbelt/stacked-borrows/
// need to figure out how to port std


use verifier;

fn example2_down (x: &i32 , f: impl FnOnce (& i32 )) -> i32 {
    let val = *x;
    f(x);
    return val ; // Can return *x instead .
}


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let value:i32 = verifier::any!();
    let mut local = value; // Stored at location ℓ, with tag 0.
    let raw_pointer = & mut local as * mut i32;
    
    let val = example2_down (
    unsafe { &* raw_pointer }, // = Pointer(ℓ, 2)
        | x_inner | unsafe { * raw_pointer = verifier::any!(); }, // Changes *x.
    );
        
    verifier::vassert!(val == value);
}
