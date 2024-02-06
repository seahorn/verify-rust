#![cfg_attr(not(kani), no_std)]

use verifier;

use smallvec::SmallVec;

#[no_mangle]
pub extern "C" fn entrypt() {
    let v: u8 = verifier::any!();
    match v {
        1 => test_dedup(),
        2 => test_dedup_by(),
        3 => test_dedup_by_key(),
        4 => test_remove(),
        5 => test_swap_remove(),
        _ => ()
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_dedup() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    v.dedup();

    verifier::vassert!(v.len() <= len);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_dedup_by() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    v.dedup_by(|a: &mut u32, b: &mut u32| a == b);

    verifier::vassert!(v.len() <= len);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_dedup_by_key() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    v.dedup_by_key(|a: &mut u32| *a);

    verifier::vassert!(v.len() <= len);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_remove() {
    const CAP: usize = 8;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();
    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let remove_point1: usize = verifier::any!();
    verifier::assume!(remove_point1 < len);

    v.remove(remove_point1);

    verifier::vassert!(v.len() == len - 1);
    verifier::vassert!(v.capacity() == CAP);

    let remove_point2: usize = verifier::any!();
    verifier::assume!(remove_point2 < len - 1);

    v.remove(remove_point2);

    verifier::vassert!(v.len() == len - 2);
    verifier::vassert!(v.capacity() == CAP);

    for i in 0..len - 2 {
        v.remove(0);
        verifier::vassert!(v.len() == len - 3 - i);
    }

    // v is empty, so this should panic
    v.remove(0);

    // This assertion should not be reachable since the call to remove panics.
    verifier::vassert!(false);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_swap_remove() {
    const CAP: usize = 8;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();
    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let remove_point1: usize = verifier::any!();
    verifier::assume!(remove_point1 < len);

    v.swap_remove(remove_point1);

    verifier::vassert!(v.len() == len - 1);
    verifier::vassert!(v.capacity() == CAP);

    let remove_point2: usize = verifier::any!();
    verifier::assume!(remove_point2 < len - 1);

    v.swap_remove(remove_point2);

    verifier::vassert!(v.len() == len - 2);
    verifier::vassert!(v.capacity() == CAP);

    for i in 0..len - 2 {
        v.swap_remove(0);
        verifier::vassert!(v.len() == len - 3 - i);
    }

    // v is empty, so this should panic
    v.swap_remove(0);

    // This assertion should not be reachable since the call to remove panics.
    verifier::vassert!(false);
}
