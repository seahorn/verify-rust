#![cfg_attr(not(kani), no_std)]

// use serde::{Serialize, Deserialize, Serializer, Deserializer};
// use heapless::Vec;

use verifier;
// use serde::{Serialize, Deserialize};
// use core::fmt::Error;

// use serde_cbor::{to_vec, from_slice};
// use serde::__private::Vec;

// extern crate alloc;
// use alloc::vec::Vec;
// use serde_json_core::to_string;
// use serde_json_core::to_vec;


#[no_mangle]
pub extern "C" fn entrypt() {
    test();


    // test_primitive_struct_enum();
    // panic!();
}
// use heapless::String;
// use alloc::string::String;
// use crate::alloc::string::ToString;
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test() {
    let v: i32 = verifier::any!();

    let result: Option<i32> = if (v & 1) == 1 { 
        None 
    } else { 
        Some(v)
    };

    let result = result.unwrap_or(0);

    if (v & 1) == 0 {
        verifier::vassert!(result == v);
    } else {
        verifier::vassert!(result == 0);
    }

    // let x: bool = verifier::any!();
    // if x { panic!(); }
    // verifier::vassert!(false);
    // let data: u32 = 10;

    // let serialized: serde_json_core::heapless::String<1> = serde_json_core::to_string(&data).unwrap();
    // let serialized: String = serde_json_core::to_string::<u32, 1>(&data).unwrap().to_string();
    // panic!();
    // // verifier::vassert!(false);

    // let deserialized: u32 = serde_json_core::from_str(&serialized).unwrap().0;
    

    // verifier::vassert!(data == deserialized);


    // verifier::vassert!(data == deserialized.1);
    // verifier::vassert!(false);


    // let serialized: String = serde_json_core::to_string::<Point, 21>(&data).unwrap().to_string();
    // let deserialized: Point = serde_json_core::from_str(&serialized).unwrap().0;
}

// #[no_mangle]
// fn test_primitive_struct_enum() -> Result<(), Error> {
    // #[derive(Serialize, Deserialize, PartialEq)]
    // struct Point {
    //     i: NumType,
    //     j: NumType,
    //     k: NumType,
    // }
    // #[derive(Serialize, Deserialize, PartialEq)]
    // enum NumType {
    //     U8(u8),
    //     U32(u32),
    //     I8(i8),
    //     I32(i32),
    //     USIZE(usize),
    //     ISIZE(isize),
    // }
    // #[derive(Serialize, Deserialize, PartialEq)]
    // struct Quaternion {
    //     r: NumType,
    //     c: Point,
    // }
    // fn nd_num_type() -> NumType {
    //     let num_type: u8 = verifier::any!();
    //     verifier::assume!(num_type < 6);
    //     match num_type {
    //         0 => NumType::U8(verifier::any!()),
    //         1 => NumType::U32(verifier::any!()),
    //         2 => NumType::I8(sea::nd_i8()),
    //         3 => NumType::I32(verifier::any!()),
    //         4 => NumType::USIZE(verifier::any!()),
    //         5 => NumType::ISIZE(sea::nd_isize()),
    //         _ => panic!(),
    //     }
    // }
    // let q: Quaternion = Quaternion {
    //     // r: nd_num_type(),
    //     // c: Point {
    //     //     i: nd_num_type(),
    //     //     j: nd_num_type(),
    //     //     k: nd_num_type(),
    //     // }
    //     r: NumType::I32(0),
    //     c: Point {
    //         i: NumType::I32(0),
    //         j: NumType::I32(0),
    //         k: NumType::I32(0),
    //     }
    // };

    // let serialized = serde_json_core::to_string(&q).unwrap();
    // let deserialized: Quaternion = serde_json_core::from_str(&serialized).unwrap();
    
    // let serialized = serde_json::to_string(&q).unwrap();
    // let deserialized: Quaternion = serde_json::from_str(&serialized).unwrap();
    

    // verifier::vassert!(q == deserialized);
    // panic!();

    // serialized: Quaternion = serde_json::from_str(&serialized).unwrap();

    // #[derive(Serialize, Deserialize, Debug)]
    // struct MyData {
    //     value: u32,
    // }

    // let data: MyData = MyData { value: 42 };

    // let encoded: Vec<u8> = serde_cbor::to_vec(&data).unwrap();
    // let decoded: MyData = serde_cbor::from_slice(&encoded).unwrap();

    // Ok(())
    // Print the deserialized value
    // println!("Deserialized value: {}", decoded.value);
    // verifier::vassert!(false);
// }