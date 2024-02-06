use std::cell::UnsafeCell;
use verifier;

// example taken from page 14 of
// https://plv.mpi-sws.org/rustbelt/stacked-borrows/
// this test hangs on kani and seahorn also aborts

#[no_mangle]
fn example2(x: &UnsafeCell<i32>, f: impl FnOnce(&UnsafeCell<i32>), d: i32) -> i32 {
    let _val = unsafe { *x.get() } / d;
    f(x);
    return unsafe { *x.get() } / d; // We want to optimize this to return val .
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let val: i32 = verifier::any!();
    let denom: i32 = verifier::any!();
    let ans: i32 = verifier::any!();
    verifier::assume!(denom != 0);
    verifier::assume!(ans % denom == 0);

    let local = UnsafeCell::new(val);
    let x = &local;
    let result = example2(
        x,
        |inner_x| {
            let raw_pointer: *mut i32 = inner_x.get();
            unsafe {
                *raw_pointer = ans;
            }
        },
        denom,
    );
    verifier::vassert!(ans / denom == result);
}
