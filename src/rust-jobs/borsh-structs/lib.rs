use borsh::{BorshSerialize, BorshDeserialize};

use verifier;

#[no_mangle]
pub extern "C" fn entrypt() {
    test_structs();
    test_fields();
}


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_structs() {
    #[derive(BorshSerialize, BorshDeserialize, PartialEq)]
    struct TestPair { a: bool, b: () }
    #[derive(BorshSerialize, BorshDeserialize, PartialEq)]
    struct TestStruct { x: u32, y: i64, z: TestPair }

    let t: TestStruct = TestStruct {
        x: verifier::any!(),
        y: verifier::any!(),
        z: TestPair { 
            a: verifier::any!(),
            b: (),
        },
    };
    let encoded: Vec<u8> = t.try_to_vec().unwrap();
    let decoded: TestStruct = TestStruct::try_from_slice(&encoded).unwrap();
    verifier::vassert!(t == decoded);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_fields() {
    let x: (i64, u8, bool, ()) = {
        (verifier::any!(), verifier::any!(), verifier::any!(), ())
    };
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: (i64, u8, bool, ()) = <(i64, u8, bool, ())>::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);

    #[derive(BorshSerialize, BorshDeserialize, PartialEq)]
    struct TestField(u32, bool, i64);
    let x: TestField = TestField(verifier::any!(), verifier::any!(), verifier::any!());
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: TestField = TestField::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
}
