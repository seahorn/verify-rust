#![cfg_attr(not(kani), no_std)]

use verifier;

use smallvec::SmallVec;

#[no_mangle]
pub extern "C" fn entrypt() {
    let v: u8 = verifier::any!();
    match v {
        0 => test_append(),
        1 => test_drain(),
        2 => test_drain_panic(),
        3 => test_insert_many(),
        4 => test_insert_many_panic(),
        5 => test_resize(),
        6 => test_resize2(),
        7 => test_resize_with(),
        8 => test_resize_with2(),
        9 => test_shrink_to_fit(),
        _ => ()
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_append() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let mut v2: SmallVec<[u32; CAP]> = SmallVec::new();

    let len2: usize = verifier::any!();
    verifier::assume!(len2 <= CAP);

    for _i in 0..len2 {
        v2.push(verifier::any!());
    }

    v.append(&mut v2);

    verifier::vassert!(v.len() == len + len2);
    verifier::vassert!(v.capacity() >= CAP);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_drain() {
    const CAP: usize = 2;
    let mut v1: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v1.push(verifier::any!());
    }

    let drain_point: usize = verifier::any!();
    verifier::assume!(drain_point < len);
    let mut v2: SmallVec<[u32; CAP]> = v1.drain(drain_point..).collect();

    verifier::vassert!(v1.len() == drain_point);
    verifier::vassert!(v2.len() == len - drain_point);

    let v3: SmallVec<[u32; CAP]> = v1.drain(drain_point..).collect();

    verifier::vassert!(v1.len() == drain_point);
    verifier::vassert!(v3.len() == 0);

    let drain_point2: usize = verifier::any!();
    verifier::assume!(drain_point2 < len - drain_point);
    let v4: SmallVec<[u32; CAP]> = v2.drain(drain_point2..len - drain_point).collect();

    verifier::vassert!(v2.len() == drain_point2);
    verifier::vassert!(v4.len() == len - drain_point - drain_point2);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_drain_panic() {
    const CAP: usize = 2;
    let mut v1: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v1.push(verifier::any!());
    }

    if verifier::any!() {
        let drain_point: usize = verifier::any!();
        verifier::assume!(drain_point > len);

        // End is greater than length, so this should panic.
        let _: SmallVec<[u32; CAP]> = v1.drain(drain_point..).collect();
    } else {
        let drain_point: usize = verifier::any!();
        let drain_point2: usize = verifier::any!();
        verifier::assume!(drain_point < len);
        verifier::assume!(drain_point2 < len);
        verifier::assume!(drain_point2 > drain_point);

        // Start is greater than end, so this should panic.
        let _: SmallVec<[u32; CAP]> = v1.drain(drain_point2..drain_point).collect();
    }

    // This assertion should not be reachable since the previous call to drain should panic.
    verifier::vassert!(false);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_insert_many() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();
    let mut v2: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let len2: usize = verifier::any!();
    verifier::assume!(len2 <= CAP);
    verifier::assume!(len + len2 <= CAP);

    for _i in 0..len2 {
        v2.push(verifier::any!());
    }

    let insert_point: usize = verifier::any!();
    verifier::assume!(insert_point < len);

    v.insert_many(insert_point, v2.clone());

    verifier::vassert!(v.len() == len + len2);
    verifier::vassert!(v2.len() == len2);
    verifier::vassert!(v.capacity() == CAP);
    verifier::vassert!(v2.capacity() == CAP);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_insert_many_panic() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();
    let mut v2: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let len2: usize = verifier::any!();
    verifier::assume!(len2 <= CAP);
    verifier::assume!(len + len2 <= CAP);

    for _i in 0..len2 {
        v2.push(verifier::any!());
    }

    let insert_point: usize = verifier::any!();
    verifier::assume!(insert_point > len + len2);
    
    // Index is out of bounds so this should panic.
    v.insert_many(insert_point, v2.clone());

    // This assertion should not be reachable since the previous operation should panic.
    verifier::vassert!(false);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_resize() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let resize_point: usize = verifier::any!();
    verifier::assume!(resize_point <= CAP);
    v.resize(resize_point, verifier::any!());

    verifier::vassert!(v.len() == resize_point);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_resize2() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let resize_point: usize = verifier::any!();
    verifier::assume!(resize_point > CAP && resize_point <= 2*CAP);

    v.resize(resize_point, verifier::any!());

    verifier::vassert!(v.len() == resize_point);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_resize_with() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let resize_point: usize = verifier::any!();
    verifier::assume!(resize_point <= CAP);
    v.resize_with(resize_point, || verifier::any!());

    verifier::vassert!(v.len() == resize_point);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_resize_with2() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let resize_point: usize = verifier::any!();
    verifier::assume!(resize_point > CAP && resize_point <= 2*CAP);

    v.resize_with(resize_point, || verifier::any!());

    verifier::vassert!(v.len() == resize_point);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(4))]
fn test_shrink_to_fit() {
    const CAP: usize = 2;
    let mut v: SmallVec<[u32; CAP]> = SmallVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);
 
    for _i in 0..len {
        v.push(verifier::any!());
    }

    v.shrink_to_fit();

    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == CAP);

    for _i in len..CAP + 1 {
        v.push(verifier::any!());
    }

    verifier::vassert!(v.len() == CAP + 1);
    verifier::vassert!(v.capacity() > CAP);

    v.pop();

    v.shrink_to_fit();

    verifier::vassert!(v.len() == CAP);
    verifier::vassert!(v.capacity() == CAP);
}
