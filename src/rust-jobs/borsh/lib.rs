use borsh::{BorshSerialize, BorshDeserialize};

use verifier;
// use std::collections::HashMap;
// // use std::io::{self};

#[derive(BorshSerialize, BorshDeserialize, PartialEq)]
struct A {
    x: u64,
    y: i8,
}

#[no_mangle]
pub extern "C" fn entrypt() {

    test_string();
    // add to_sting job
    // verifier::vassert!(false);

    // test_fixed_sized_arrays();
    // test_hashmap();    
    // test_combinations();

    // llvm script
    // cont borsh
    // example of non-overalaping
    // to_string
    // 

}



// Issue: does not work, issues with bmcp() when string has more than 1 character
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_string() {
    let x: String = "abcd".to_string();
    
    let encoded: Vec<u8> = x.try_to_vec().unwrap();

    let decoded: String = String::try_from_slice(&encoded).unwrap();

    if decoded != x {
        verifier::vassert!(false);
    }

    verifier::vassert!(decoded == x);

    // verifier::vassert!(x == decoded);
    // verifier::vassert!(false);

    // verifier::vassert!(x.len() == decoded.len());
    // verifier::vassert!(x == "asdf".to_string());
    // verifier::vassert!(false);
    // verifier::vassert!(false);

    // verifier::vassert!(++)
    // verifier::vassert!(
    //     match x.cmp(&decoded) {
    //         std::cmp::Ordering::Equal => true,
    //         _ => false,
    //     }
    // );

    // verifier::vassert!(x == decoded);

    // let mut v: Vec<i32> = vec![1, 2, 3, 4];
    // let encoded: Vec<u8> = v.try_to_vec().unwrap();
    // let decoded: Vec<i32> = Vec::<i32>::try_from_slice(&encoded).unwrap();
    // verifier::vassert!(v == decoded);
    // verifier::vassert!(false);

}

// #[no_mangle] extern "C" fn __rust_probestack () {}


// Issue: does not work (probably in int bcmp() )
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_fixed_sized_arrays() {
    let y: i32 = verifier::any!();
    verifier::assume!(y > 0);
    let x: [i32; 3] = [0, 0, y];

    let encoded: Vec<u8> = x.try_to_vec().unwrap();
    let decoded: [i32; 3] = <[i32; 3]>::try_from_slice(&encoded).unwrap();

    verifier::vassert!(x.len() == decoded.len());
    verifier::vassert!(x[0] == decoded[0]);
    // verifier::vassert!(decoded[2] >= 0);
    // verifier::vassert!(x[2] == decoded[2]);


    // for i in 0..5 {
    //     verifier::vassert!(x[i] == 0);
    //     verifier::vassert!(decoded[i] == 0);
    // }
    
    // verifier::vassert!(x == decoded);
}

// Issue: does not work, TBD
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_hashmap() {
    // use borsh::maybestd::collections::HashMap;
    // let mut map: HashMap<u32, u32> = HashMap::default();
}

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_combinations() {

// }
