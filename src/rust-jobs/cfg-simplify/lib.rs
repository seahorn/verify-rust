// Seahorn: This test will fail because all assertions will be discharged by frontend before
// hitting BMC.
#![cfg_attr(not(kani), no_std)]
pub use verifier;

// sea::define_sea_nd!(sea_nd_u8, u8, 42);

#[no_mangle]
pub extern "C" fn entrypt() {
    test_simplify_cfg();
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
#[cfg_attr(kani, kani::should_panic)]
fn test_simplify_cfg() {
    let v: u8 = verifier::any!();
    verifier::assume!(v < 3);   

    for i in 0..v as usize {
      panic!();
      verifier::error!();
    }

    let mut sentinel: u32 = verifier::any!();
    // INV: v == 0 => sentinel >= 0 
    verifier::vassert!(sentinel >= (v as u32)*(v as u32));
}
