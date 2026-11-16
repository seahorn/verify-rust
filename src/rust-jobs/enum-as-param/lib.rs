#![cfg_attr(not(kani), no_std)]
pub use verifier;

#[repr(C)]
pub enum CEnum {
    KValOne,
    KValTwo,
    KValThree
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    // `opaque` (a volatile read) keeps the expected value from being
    // constant-folded against the inlined match, without adding an assumption.
    let v: i32 = opaque(102);

    let result: i32 = enum_param_test(CEnum::KValTwo);

    verifier::vassert!(result == v);
}

#[no_mangle]
fn enum_param_test(param: CEnum) -> i32 {
    match param {
        CEnum::KValOne => 101,
        CEnum::KValTwo => 102,
        CEnum::KValThree => 103
    }
}

// Volatile read: LLVM cannot see through it, so an assertion against the
// result is not folded away, and no assumption is added.
#[inline(always)]
fn opaque<T: Copy>(x: T) -> T {
    unsafe { core::ptr::read_volatile(&x) }
}
