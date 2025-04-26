#![cfg_attr(not(kani), no_std)]

pub use verifier;

#[no_mangle]
pub extern "C" fn entrypt() {
  test_simplify_cfg();
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_simplify_cfg() {
  let v: u8 = verifier::any!();
  verifier::assume!(v < 3);   

  for i in 0..v as usize {
    return;
  }

  let mut sentinel: u32 = verifier::any!();
  // INV: v == 0 => sentinel >= 0 
  verifier::vassert!(sentinel >= (v as u32)*(v as u32));
}
