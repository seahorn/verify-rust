// #![cfg_attr(not(kani), no_std)]

// use verifier;

// extern crate alloc;
// use alloc::vec::Vec;
// use alloc::vec;


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let mut reader: &[u8] = b"hello";
    let mut writer: Vec<u8> = vec![];
    let n = std::io::copy(&mut reader, &mut writer).unwrap();
    verifier::vassert!(n == 5);
    verifier::vassert!(writer.len() == 5);
}


// #[no_mangle] extern "C" fn __rust_probestack () {}
