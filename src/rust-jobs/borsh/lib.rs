use borsh::{BorshSerialize, BorshDeserialize};

use verifier;

#[no_mangle]
pub extern "C" fn entrypt() {
    test_string();
    test_fixed_sized_arrays();
}

// Issue: does not work, issues with bmcp() when string has more than 1 character
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_string() {
    let x: String = "abc".to_string();
    
    let encoded: Vec<u8> = x.try_to_vec().unwrap();

    let decoded: String = String::try_from_slice(&encoded).unwrap();

    if decoded != x {
        verifier::vassert!(false);
    }

    verifier::vassert!(decoded == x);
}

// Issue: does not work (probably in int bcmp() )
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(4))]
fn test_fixed_sized_arrays() {
    let y: i32 = verifier::any!();
    verifier::assume!(y > 0);
    let x: [i32; 3] = [0, 0, y];

    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: [i32; 3] = <[i32; 3]>::try_from_slice(&encoded).unwrap();

    verifier::vassert!(x.len() == decoded.len());
    verifier::vassert!(x[0] == decoded[0]);
}

// Issue: does not work, TBD
//#[no_mangle]
//#[cfg_attr(kani, kani::proof)]
//fn test_hashmap() {
    // use borsh::maybestd::collections::HashMap;
    // let mut map: HashMap<u32, u32> = HashMap::default();
//}

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_combinations() {

// }
