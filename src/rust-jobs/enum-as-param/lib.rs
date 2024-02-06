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
    let v: i32 = verifier::any!();
    verifier::assume!(v == 102);

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