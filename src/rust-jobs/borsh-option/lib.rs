use borsh::{BorshSerialize, BorshDeserialize};

use verifier;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    test_option();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(10))]
fn test_option() {
    let x: Option<u32> = Some(verifier::any!());
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: Option<u32> = Option::<u32>::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);

    #[derive(BorshSerialize, BorshDeserialize, PartialEq)]
    struct TestStruct { a: i32, b: u8, }

    let x: Option<TestStruct> = Some(TestStruct { a: verifier::any!(), b: verifier::any!() });
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: Option<TestStruct> = Option::<TestStruct>::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);

    let x: Option<u8> = None;
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: Option<u8> = Option::<u8>::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
}