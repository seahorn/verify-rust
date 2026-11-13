#![cfg_attr(not(kani), no_std)]

use verifier;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let x = verifier::any!();
    verifier::assume!(x < i32::MAX/2);
    verifier::assume!(x > i32::MIN/2);
    let res = iter_mut(x);

    // `expected` is hidden from rustc by `opaque` (a volatile read), so LLVM 18
    // cannot inline `iter_mut`, correlate the two uses of `x >= 0`, and fold
    // the assertion to `true` before SeaHorn runs. No assumption is added.
    let expected: i32 = if x >= 0 { opaque(x) } else { opaque(x*2) };

    verifier::vassert!(res == expected);
}

#[no_mangle]
pub extern "C" fn iter_mut(input: i32) -> i32 {
    if input >= 0 {
        let mut x: Result<i32, &str> = Ok(input);
        match x.iter_mut().next() {
            Some(v) => *v = input,
            None => {},
        };
        let y: Result<i32, i32> = Err(x.unwrap());
        match y {
            Ok(value) => value,
            Err(err) => err,
        }
    } else {
        let y: Result<i32, i32> = Err(input);
        let mut x: Result<i32, &str> = Ok(match y {
            Ok(value) => value,
            Err(err) => err,
        });
        
        match x.iter_mut().next() {
            Some(v) => *v = input,
            None => {},
        };
        x.unwrap()*2
    }

}

// #[no_mangle]
// #[inline(never)]
// pub extern "C" fn sea_nd_foo() -> i32 {
//     0
// }

// Volatile read: LLVM cannot see through it, so an assertion against the
// result is not folded away, and no assumption is added.
#[inline(always)]
fn opaque<T: Copy>(x: T) -> T {
    unsafe { core::ptr::read_volatile(&x) }
}
