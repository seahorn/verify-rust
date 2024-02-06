#![cfg_attr(not(kani), no_std)]

use verifier;

use smallvec::SmallVec;

#[no_mangle]
pub extern "C" fn entrypt() {
    let v: u8 = verifier::any!();
    match v {
        0 => test_push(),
        1 => test_retain(),
        2 => test_retain_mut(),
        3 => test_try_grow(),
        _ => (),
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(6))]
fn test_push() {
    const CAP: usize = 4;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for i in 0..len {
        v.push(verifier::any!());
        verifier::vassert!(v.len() == i + 1);
    }

    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == CAP);

    for _i in len..CAP + 1 {
        v.push(verifier::any!());
    }

    verifier::vassert!(v.len() == CAP + 1);
    verifier::vassert!(v.capacity() > CAP);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(5))]
fn test_retain() {
    const CAP: usize = 4;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for i in 1..=len {
        let val: u32 = verifier::any!();
        if (i & 1) == 0 {
            verifier::assume!((val & 1) == 0);
        } else {
            verifier::assume!((val & 1) == 1);
        }
        v.push(val);
    }

    v.retain(|x| (*x & 1) == 0);

    verifier::vassert!(v.len() == len >> 1);
    verifier::vassert!(v.capacity() == CAP);

    v.retain(|x| (*x & 1) == 1);

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == CAP);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(5))]
fn test_retain_mut() {
    const CAP: usize = 4;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for i in 1..=len {
        let val: u32 = verifier::any!();
        if (i & 1) == 0 {
            verifier::assume!((val & 1) == 0);
        } else {
            verifier::assume!((val & 1) == 1);
        }
        v.push(val);
    }

    v.retain_mut(|x| (*x & 1) == 0);

    verifier::vassert!(v.len() == len / 2);
    verifier::vassert!(v.capacity() == CAP);

    v.retain_mut(|x| (*x & 1) == 1);

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == CAP);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_try_grow() {
    const CAP: usize = 4;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let new_cap: usize = verifier::any!();
    verifier::assume!(new_cap > CAP && new_cap <= CAP * 2);

    let result: Result<(), smallvec::CollectionAllocErr> = v.try_grow(new_cap);

    verifier::vassert!(result.is_ok());
    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == new_cap);

    let new_cap2: usize = verifier::any!();
    verifier::assume!(new_cap2 < len);

    let result2: Result<(), smallvec::CollectionAllocErr> = v.try_grow(new_cap2);

    verifier::vassert!(result2.is_err());
}
