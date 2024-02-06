use borsh::{BorshSerialize, BorshDeserialize};

use verifier;

#[no_mangle]
pub extern "C" fn entrypt() {
    // this takes around 90 seconds to run
    test_primitives();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_primitives() {
    let x: () = ();
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: () = <()>::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: bool = verifier::any!();
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: bool = bool::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: u8 = verifier::any!();
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: u8 = u8::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: i8 = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = i8::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: u16 = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = u16::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: i16 = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = i16::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: u32 = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = u32::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: i32 = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = i32::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: u64 = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = u64::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: i64 = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = i64::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: usize = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = usize::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: isize = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = isize::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: usize = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = usize::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
    let x: isize = verifier::any!();
    let encoded = x.try_to_vec().unwrap();
    let decoded = isize::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
}
