#![cfg_attr(not(kani), no_std)]
pub use verifier;
extern crate alloc;
use alloc::vec::Vec;

// sea::define_sea_nd!(sea_nd_u8, u8, 42);

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(10))]
pub extern "C" fn entrypt() {
    // let v: u8 = sea_nd_u8();
    let v: u8 = verifier::any!();

    let capacity: usize = v as usize;
    let mut nums: Vec<Option<u32>> = Vec::with_capacity(capacity);

    for i in 1..=capacity {
        let value: u32 = i as u32 - 1;
        nums[i - 1] = Some(value);
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
