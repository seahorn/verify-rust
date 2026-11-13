#![cfg_attr(not(kani), no_std)]
use verifier;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();
    let w: i32 = verifier::any!();

    let val1: Option<i32> = if (v & 1) == 1 { 
        None 
    } else { 
        Some(v)
    };

    let val2: Option<i32> = Some(w);

    let result: i32 = val1.or(val2).unwrap();

    // `expected` is hidden from rustc by `opaque` (a volatile read), so LLVM 18
    // cannot correlate it with the `(v & 1)` branch that built `result` and
    // fold the assertion to `true` before SeaHorn runs. No assumption is added.
    let expected: i32 = if (v & 1) == 0 { opaque(v) } else { opaque(w) };

    verifier::vassert!(result == expected);
    // panic!();
}

// Volatile read: LLVM cannot see through it, so an assertion against the
// result is not folded away, and no assumption is added.
#[inline(always)]
fn opaque<T: Copy>(x: T) -> T {
    unsafe { core::ptr::read_volatile(&x) }
}
