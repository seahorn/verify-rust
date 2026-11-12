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

  let sentinel: u32 = verifier::any!();
  // INV: v == 0 => sentinel >= 0
  // On this path -O3 knows v == 0 and folds `sentinel >= v * v` to `true`,
  // so SeaHorn would see no assertion. Reading v * v back through a volatile
  // load hides the value from LLVM without constraining it, so the
  // comparison survives for SeaHorn to check.
  verifier::vassert!(sentinel >= opaque((v as u32) * (v as u32)));
}

#[inline(always)]
fn opaque<T: Copy>(x: T) -> T {
  unsafe { core::ptr::read_volatile(&x) }
}
