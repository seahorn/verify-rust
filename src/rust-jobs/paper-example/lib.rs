#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use verifier;

#[no_mangle]
pub extern "C" fn entrypt() {
  test();
}

fn test() {
  let x: u8 = verifier::any!();
  verifier::assume!(x > 0 && x < 10);

  let p: Box<[u8]> = if verifier::any!() {
      Box::new([0u8; 2]) // allocate 2 bytes initialized to 0
  } else {
      Box::new([0u8; 1]) // allocate 1 byte initialized to 0
  };

  let mut p = Box::into_raw(p) as *mut u8; // get raw pointer to the data

  unsafe {
      *p = x;
      *p.add(1) = 0; // this is UB if only 1 byte was allocated!
      verifier::vassert!(0 < *p && *p < 10);
      verifier::vassert!(*p.add(1) == 0); // this is UB if only 1 byte was allocated!
  }
}
