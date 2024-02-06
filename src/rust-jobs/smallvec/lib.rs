#![cfg_attr(not(kani), no_std)]

use verifier;

use core::mem::{self, MaybeUninit};
use smallvec::SmallVec;

#[no_mangle]
pub extern "C" fn entrypt() {
    let v: u8 = verifier::any!();
    match v {
        0 => test_clear(),
        1 => test_extend_from_slice(),
        2 => test_from_buf(),
        3 => test_from_buf_and_len(),
        4 => test_from_buf_and_len_unchecked(),
        5 => test_from_const(),
        6 => test_from_elem(),
        7 => test_from_raw_parts(),
        8 => test_from_slice(),
        9 => test_grow(),
        10 => test_insert(),
        11 => test_insert_from_slice(),
        12 => test_new(),
        13 => test_new_const(),
        14 => test_pop(),
        15 => test_reserve(),
        16 => test_reserve_exact(),
        17 => test_set_len(),
        18 => test_truncate(),
        19 => test_try_reserve(),
        20 => test_try_reserve_exact(),
        21 => test_with_capacity(),
        _ => (),
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_clear() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    v.clear();

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_extend_from_slice() {
    let mut v1: SmallVec<[u32; 8]> = SmallVec::new();
    let mut v2: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v1.push(verifier::any!());
    }

    let len2: usize = verifier::any!();
    verifier::assume!(len2 <= 8);

    for _i in 0..len2 {
        v2.push(verifier::any!());
    }

    v1.extend_from_slice(&v2);

    verifier::vassert!(v1.len() == len + len2);
    verifier::vassert!(v1.capacity() >= 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_from_buf() {
    let buf: [u32; 8] = [verifier::any!(); 8];

    let v: SmallVec<[u32; 8]> = SmallVec::from_buf(buf);

    verifier::vassert!(v.len() == 8);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_from_buf_and_len() {
    let buf: [u32; 8] = [verifier::any!(); 8];

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    let v: SmallVec<[u32; 8]> = SmallVec::from_buf_and_len(buf, len);

    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_from_buf_and_len_unchecked() {
    let buf: [u32; 8] = [verifier::any!(); 8];

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    let v: SmallVec<[u32; 8]> =
        unsafe { SmallVec::from_buf_and_len_unchecked(MaybeUninit::new(buf), len) };

    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_from_const() {
    let v: SmallVec<[u32; 8]> = SmallVec::from_const([verifier::any!(); 8]);

    verifier::vassert!(v.len() == 8);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_from_elem() {
    let elem: u32 = verifier::any!();

    let v: SmallVec<[u32; 8]> = SmallVec::from_elem(elem, 8);

    verifier::vassert!(v.len() == 8);
    verifier::vassert!(v.capacity() == 8);

    for i in 0..8 {
        verifier::vassert!(v[i] == elem);
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_from_raw_parts() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let ptr: *mut u32 = v.as_mut_ptr();

    unsafe {
        mem::forget(v);

        // Capacity has to be greater than original capacity for this to work.
        let v2: SmallVec<[u32; 8]> = SmallVec::from_raw_parts(ptr, len, 16);

        verifier::vassert!(v2.len() == len);
        verifier::vassert!(v2.capacity() == 16);
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_from_slice() {
    let mut buf: [u32; 8] = [0; 8];

    for i in 0..8 {
        buf[i] = verifier::any!();
    }

    let v: SmallVec<[u32; 8]> = SmallVec::from_slice(&buf);

    verifier::vassert!(v.len() == 8);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_grow() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let new_cap: usize = verifier::any!();
    verifier::assume!(new_cap >= 8 && new_cap <= 16);

    v.grow(new_cap);

    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == new_cap);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_insert() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len > 0 && len < 6);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let insert_point: usize = verifier::any!();
    verifier::assume!(insert_point < len);
    v.insert(insert_point, verifier::any!());

    verifier::vassert!(v.len() == len + 1);
    verifier::vassert!(v.capacity() == 8);

    let insert_point2: usize = verifier::any!();
    verifier::assume!(insert_point2 > len + 1);

    // Index is out of bounds so this should panic.
    v.insert(insert_point2, verifier::any!());

    // Previous insertion should panic so this shouldn't be reachable.
    verifier::vassert!(false);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_insert_from_slice() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();
    let mut v2: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let len2: usize = verifier::any!();
    verifier::assume!(len + len2 <= 8);

    for _i in 0..len2 {
        v2.push(verifier::any!());
    }

    let insert_point: usize = verifier::any!();
    verifier::assume!(insert_point < len);

    v.insert_from_slice(insert_point, &v2);

    verifier::vassert!(v.len() == len + len2);
    verifier::vassert!(v2.len() == len2);
    verifier::vassert!(v.capacity() == 8);
    verifier::vassert!(v2.capacity() == 8);

    let insert_point2: usize = verifier::any!();
    verifier::assume!(insert_point2 > len + len2);

    // Index is out of bounds so this should panic.
    v.insert_from_slice(insert_point2, &v2);

    // This assertion should not be reachable since the previous operation should panic.
    verifier::vassert!(false);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_new() {
    let v: SmallVec<[u32; 8]> = SmallVec::new();

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_new_const() {
    let v: SmallVec<[u32; 8]> = SmallVec::new_const();

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_pop() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len > 0 && len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    for i in 0..len {
        v.pop();
        verifier::vassert!(v.len() == len - i - 1);
    }

    let result: Option<u32> = v.pop();
    verifier::vassert!(result.is_none());
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_reserve() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let new_cap: usize = verifier::any!();
    verifier::assume!(new_cap >= 8);

    v.reserve(new_cap);

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() >= new_cap);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_reserve_exact() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let new_cap: usize = verifier::any!();
    verifier::assume!(new_cap >= 8 && new_cap <= 16);

    v.reserve_exact(new_cap);

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == new_cap);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_set_len() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    unsafe {
        v.set_len(len);
    }

    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_truncate() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 8);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let truncate_point: usize = verifier::any!();
    verifier::assume!(truncate_point <= len);

    v.truncate(truncate_point);

    verifier::vassert!(v.len() == truncate_point);
    verifier::vassert!(v.capacity() == 8);

    let truncate_point2: usize = verifier::any!();
    verifier::assume!(truncate_point2 > truncate_point);

    v.truncate(truncate_point2);

    verifier::vassert!(v.len() == truncate_point);
    verifier::vassert!(v.capacity() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_try_reserve() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let new_cap: usize = verifier::any!();
    verifier::assume!(new_cap >= 8 && new_cap <= u16::MAX as usize);

    let result: Result<(), smallvec::CollectionAllocErr> = v.try_reserve(new_cap);

    verifier::vassert!(result.is_ok());
    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() >= new_cap);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_try_reserve_exact() {
    let mut v: SmallVec<[u32; 8]> = SmallVec::new();

    let new_cap: usize = verifier::any!();
    verifier::assume!(new_cap >= 8 && new_cap <= u16::MAX as usize);

    let result: Result<(), smallvec::CollectionAllocErr> = v.try_reserve_exact(new_cap);

    verifier::vassert!(result.is_ok());
    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == new_cap);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(16))]
fn test_with_capacity() {
    let cap: usize = verifier::any!();
    verifier::assume!(cap >= 8);

    let v: SmallVec<[u32; 8]> = SmallVec::with_capacity(cap);

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == cap);
}
