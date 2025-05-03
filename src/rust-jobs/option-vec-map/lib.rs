#![cfg_attr(not(kani), no_std)]
pub use verifier;
extern crate alloc;
use alloc::vec::Vec;

#[no_mangle]
pub extern "C" fn entrypt() {
    test();
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
#[cfg_attr(kani, kani::should_panic)]
fn test() {
    let v: u8 = verifier::any!();
    verifier::assume!(v < 3);   
    let mut nums: Vec<Option<u32>> = Vec::with_capacity(v as usize);

    for i in 0..v as usize {
        let value: u32 = 1 + i as u32;
        nums[i] = Some(value);
    }

    let result: Vec<Option<u32>> = nums.into_iter().map(square).collect();

    let mut sum: u32 = 0;
    for val in result {
        if let Some(x) = val {
            sum += x;
        }
    }
 
    verifier::vassert!(sum >= (v as u32)*(v as u32));

}

fn square(val: Option<u32>) -> Option<u32> {
    val.map(|x: u32| x * x)
}
