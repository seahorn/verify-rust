use borsh::{BorshSerialize, BorshDeserialize};

use verifier;


#[no_mangle]
pub extern "C" fn entrypt() {
    test_enums();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_enums() {
    #[derive(BorshSerialize, BorshDeserialize, PartialEq)]
    enum NdType {
        U32(u32),
        I32(i32),
    }
    let x: NdType = if verifier::any!() {
        NdType::U32(verifier::any!())
    } else {
        NdType::I32(verifier::any!())
    };
    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: NdType = NdType::try_from_slice(&encoded).unwrap();
    verifier::vassert!(x == decoded);
}
