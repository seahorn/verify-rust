#![cfg_attr(not(kani), no_std)]

use verifier;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let x = verifier::any!();
    verifier::assume!(x < i32::MAX/2);
    verifier::assume!(x > i32::MIN/2);
    let res = iter_mut(x);

    // `expected` is opaque to rustc: it is tied to `x` only through
    // verifier::assume!, which is an extern "C" call the front end cannot see
    // through. Asserting against `x` directly lets LLVM 18 inline `iter_mut`,
    // correlate the two uses of `x >= 0`, and fold the assertion to `true`
    // before SeaHorn runs.
    let expected: i32 = verifier::any!();
    if x >= 0 {
        verifier::assume!(expected == x);
    } else {
        verifier::assume!(expected == x*2);
    }

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

