#![cfg_attr(not(kani), no_std)]

use verifier;
use sea;
use tinyvec::ArrayVec;

#[no_mangle]
pub extern "C" fn entrypt() {
    let v: u8 = verifier::any!();
    match v {
        0 => test_append(),
        1 => test_clear(),
        2 => test_drain(),
        3 => test_extend_from_slice(),
        4 => test_fill(),
        5 => test_from_array_empty(),
        6 => test_from_array_len(),
        // 7  => test_grab_spare_slice(),
        8 => test_insert(),
        9 => test_new(),
        10 => test_pop(),
        11 => test_push(),
        12 => test_remove(),
        13 => test_resize(),
        14 => test_resize_with(),
        15 => test_retain(),
        16 => test_set_len(),
        17 => test_splice(),
        18 => test_splice_panic(),
        19 => test_split_off(),
        20 => test_swap_remove(),
        21 => test_truncate(),
        22 => test_try_append(),
        23 => test_try_from_array_len(),
        24 => test_try_insert(),
        25 => test_try_push(),
        26 => test_drain_panic(),
        _ => (),
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]

fn test_append() {
    const CAP: usize = 4;
    let mut v1: ArrayVec<[u32; CAP]> = ArrayVec::new();
    let mut v2: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len1: usize = verifier::any!();
    verifier::assume!(len1 <= CAP);

    let len2: usize = verifier::any!();
    verifier::assume!(len2 <= CAP);
    
    for _i in 0..len1 {
        v1.push(verifier::any!());
    }
    
    verifier::vassert!(v1.len() == len1);

    for _i in 0..len2 {
        v2.push(verifier::any!());
    }

    verifier::vassert!(v2.len() == len2);

    // Panics if the capacity is exceeded.
    v1.append(&mut v2);

    if len1 + len2 <= CAP {
        verifier::vassert!(v1.len() == len1 + len2);
        verifier::vassert!(v2.len() == 0);
    } else {
        // This assertion should not be reachable since the previous operation should panic.
        verifier::error!();
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_clear() {
    let mut v: ArrayVec<[u32; 8]> = ArrayVec::new();

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
#[cfg_attr(kani, kani::unwind(4))]
fn test_drain() {
    const CAP : usize = 4;
    let mut v1: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len >= 2 && len <= CAP);

    for _i in 0..len {
        v1.push(verifier::any!());
    }

    let drain_point: usize = verifier::any!();
    verifier::assume!(drain_point >= 1 && drain_point < len);
    let v2: ArrayVec<[u32; CAP]> = v1.drain(drain_point..).collect();

    verifier::vassert!(v1.len() == drain_point);
    verifier::vassert!(v2.len() == len - drain_point);

    let v3: ArrayVec<[u32; CAP]> = v1.drain(drain_point..).collect();

    verifier::vassert!(v1.len() == drain_point);
    verifier::vassert!(v3.len() == 0);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(4))]
fn test_drain_panic() {
    const CAP : usize = 4;
    let mut v1: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len >= 2 && len <= CAP);

    for _i in 0..len {
        v1.push(verifier::any!());
    }

    if verifier::any!() {
        let drain_point3: usize = verifier::any!();
        verifier::assume!(drain_point3 > len);

        // End is greater than length, so this should panic.
        let _: ArrayVec<[u32; CAP]> = v1.drain(drain_point3..).collect();
    } else {
        let drain_point3: usize = verifier::any!();
        let drain_point4: usize = verifier::any!();
        verifier::assume!(drain_point3 < len);
        verifier::assume!(drain_point4 < len);
        verifier::assume!(drain_point4 > drain_point3);

        // Start is greater than end, so this should panic.
        let _: ArrayVec<[u32; 4]> = v1.drain(drain_point4..drain_point3).collect();
    }

    // This assertion should not be reachable since the previous call to drain should panic.
    verifier::error!();
}



#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_extend_from_slice() {
    const CAP: usize = 4;
    let mut v1: ArrayVec<[u32; CAP]> = ArrayVec::new();
    let mut v2: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len1: usize = verifier::any!();
    let len2: usize = verifier::any!();

    verifier::assume!(len1 <= CAP);
    verifier::assume!(len2 <= CAP);

    for _i in 0..len1 {
        v1.push(verifier::any!());
    }

    for _i in 0..len2 {
        v2.push(verifier::any!());
    }

    // This should panic since len1 + len2 > CAP.
    v1.extend_from_slice(v2.as_slice());

    if len1 + len2 <= CAP {
        verifier::vassert!(v1.len() == len1 + len2);
        verifier::vassert!(v2.len() == len2);
    } else {
        // This assertion should not be reachable since the previous operation should panic.
        verifier::error!();
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_fill() {
    const CAP : usize = 4;
    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();

    v.fill(0..len as u32);

    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == CAP);

    for n in 0..len {
        verifier::vassert!(n as u32 == v[n]);
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_from_array_empty() {
    const CAP : usize = 8;
    let v: ArrayVec<[u32; CAP]> = ArrayVec::from_array_empty([0; CAP]);

    verifier::vassert!(v.len() == 0);

    // Necessary to make seahorn work.
    let x: u32 = verifier::any!();
    verifier::assume!(x < u32::MAX/2);
    let result: u32 = x * 2;
    verifier::vassert!(result >= x);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_from_array_len() {
    const CAP : usize = 8;
    let len: usize = verifier::any!();

    if len <= CAP {
        let v: ArrayVec<[u32; CAP]> = ArrayVec::from_array_len([0; CAP], len);

        verifier::vassert!(v.len() == len);
    } else {
        // Specified length is larger than capacity of array, so this should panic.
        let _: ArrayVec<[u32; CAP]> = ArrayVec::from_array_len([0; CAP], len);

        // This assertion should be unreachable since the previous operation panics.
        verifier::error!();
    }

    // Necessary to make seahorn work.
    let x: u32 = verifier::any!();
    verifier::assume!(x < u32::MAX/2);
    let result: u32 = x * 2;
    verifier::vassert!(result >= x);
}

// Documentation lists this as a function, but the compiler says it doesn't exist.
// https://docs.rs/tinyvec/latest/tinyvec/struct.ArrayVec.html#method.grab_spare_slice
/*
#[no_mangle]
fn test_grab_spare_slice() {
    let mut v: ArrayVec<[u32; 4]> = ArrayVec::new();

    let slice = v.grab_spare_slice();

    verifier::vassert!(slice.len() == 4);

    v.push(1);
    v.push(2);

    let slice = v.grab_spare_slice();

    verifier::vassert!(slice.len() == 2);

    v.push(3);
    v.push(4);

    let slice = v.grab_spare_slice();

    verifier::vassert!(slice.len() == 0);
}
*/

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_insert() {
    const CAP : usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len > 0 && len <= CAP - 1);
    
    for _i in 0..len {
        v.push(verifier::any!());
    }

    let insert_point: usize = verifier::any!();
    verifier::assume!(insert_point < len);
    v.insert(insert_point, verifier::any!());

    verifier::vassert!(v.len() == len + 1);
    verifier::vassert!(v.capacity() == CAP);

    if len < CAP - 2 {
        let insert_point2: usize = verifier::any!();
        verifier::assume!(insert_point2 > len + 1);

        // Index is greater than length, so insertion should panic.
        v.insert(insert_point2, verifier::any!());
    } else {
        if len == CAP - 2 { v.push(verifier::any!()); }
        let insert_point2: usize = verifier::any!();
        verifier::assume!(insert_point2 <= CAP - 1);

        // Vector is at capacity, so insertion should panic.
        v.insert(insert_point2, verifier::any!());
    }

    // This assertion should not be reachable as the previous insertion should panic.
    verifier::error!();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_new() {
    const CAP: usize = 4;
    let v: ArrayVec<[u32; CAP]> = ArrayVec::new();

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == CAP);

    // Necessary to make seahorn work.
    let x: u32 = verifier::any!();
    verifier::assume!(x < u32::MAX/2);
    let result: u32 = x * 2;
    verifier::vassert!(result >= x);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_pop() {
    const CAP : usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    for i in 0..len {
        let result: Option<u32> = v.pop();
        verifier::vassert!(result.is_some());
        verifier::vassert!(v.len() == len - i - 1);
    }
    sea::sea_printf!("len: %d\n", v.len());
    // v is empty, so this should return None.
    let result: Option<u32> = v.pop();

    verifier::vassert!(result.is_none());
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_push() {
    const CAP : usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();
    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for i in 0..len {
        v.push(verifier::any!());
        verifier::vassert!(v.len() == i + 1);
    }

    verifier::vassert!(v.len() == len);
    verifier::vassert!(v.capacity() == CAP);

    if len == CAP {
        // Vector is at capacity, so push should panic.
        v.push(verifier::any!());

        // This assertion should not be reachable since the previous push panics.
        verifier::error!();
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_remove() {
    const CAP : usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();
    let len: usize = verifier::any!();
    verifier::assume!(2 <= len && len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let remove_point1: usize = verifier::any!();
    verifier::assume!(remove_point1 < len);

    v.remove(remove_point1);

    verifier::vassert!(v.len() == len - 1);
    verifier::vassert!(v.capacity() == CAP);

    let remove_point2: usize = verifier::any!();
    verifier::assume!(remove_point2 < len);

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
    verifier::error!();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_resize() {
    const CAP : usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let resize_point: usize = verifier::any!();
    verifier::assume!(resize_point <= CAP);
    v.resize(resize_point, verifier::any!());

    verifier::vassert!(v.len() == resize_point);

    let resize_point2: usize = verifier::any!();
    v.resize(resize_point2, verifier::any!());

    verifier::vassert!(v.len() == resize_point2);

    let resize_point3: usize = verifier::any!();
    verifier::assume!(resize_point3 > CAP);
    // This is larger than the capacity of the vector and should panic.
    v.resize(resize_point3, verifier::any!());

    // This assertion should not be reachable since the previous operation should panic.
    verifier::error!();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_resize_with() {
    const CAP : usize = 4;
    let mut v: ArrayVec<[u32; 4]> = ArrayVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= 4);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let resize_point: usize = verifier::any!();
    verifier::assume!(resize_point <= CAP);
    v.resize_with(resize_point, || verifier::any!());

    verifier::vassert!(v.len() == resize_point);

    let resize_point2: usize = verifier::any!();
    v.resize_with(resize_point2, || verifier::any!());

    verifier::vassert!(v.len() == resize_point2);

    let resize_point3: usize = verifier::any!();
    verifier::assume!(resize_point3 > CAP);
    // This is larger than the capacity of the vector and should panic.
    v.resize_with(resize_point3, || verifier::any!());

    // This assertion should not be reachable since the previous operation should panic.
    verifier::error!();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(3))]
fn test_retain() {
    const CAP : usize = 2;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();

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

    v.retain(|&x| (x & 1) == 0);

    verifier::vassert!(v.len() == len / 2);
    verifier::vassert!(v.capacity() == CAP);

    v.retain(|&x| (x & 1) == 1);

    verifier::vassert!(v.len() == 0);
    verifier::vassert!(v.capacity() == CAP);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_set_len() {
    const CAP: usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len: usize = verifier::any!();

    v.set_len(len);

    if len > CAP {
        // This assertion should not be reachable since the previous operation panics.
        verifier::error!();
    } else {
        verifier::vassert!(v.len() == len);
        verifier::vassert!(v.capacity() == CAP);
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(5))]
fn test_splice() {
    const CAP: usize = 4;
    let mut v1: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v1.push(verifier::any!());
    }

    let splice_point: usize = verifier::any!();
    verifier::assume!(splice_point < len);

    let val: u32 = verifier::any!();

    let v2: ArrayVec<[u32; CAP]> = v1.splice(splice_point.., val..val + len as u32 - splice_point as u32).collect();

    verifier::vassert!(v1.len() == len);
    verifier::vassert!(v2.len() == len - splice_point);

    let splice_point2: usize = verifier::any!();
    verifier::assume!(splice_point2 < len);

    let val: u32 = verifier::any!();

    let v3: ArrayVec<[u32; CAP]> = v1.splice(splice_point2..splice_point2, val..val).collect();

    verifier::vassert!(v1.len() == len);
    verifier::vassert!(v3.len() == 0);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(5))]
fn test_splice_panic() {
    const CAP: usize = 4;
    let mut v1: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v1.push(verifier::any!());
    }

    if verifier::any!() {
        let splice_point3: usize = verifier::any!();
        verifier::assume!(splice_point3 < len);

        let splice_point4: usize = verifier::any!();
        verifier::assume!(splice_point4 < splice_point3);

        // Start is greater than end, so panic should occur.
        let _: ArrayVec<[u32; CAP]> = v1.splice(splice_point3..splice_point4, verifier::any!()..).collect();
    } else if verifier::any!() {
        let splice_point5: usize = verifier::any!();
        verifier::assume!(splice_point5 > len);

        let val: u32 = verifier::any!();

        // End is past end of vector, so panic should occur.
        let _: ArrayVec<[u32; CAP]> = v1.splice(..splice_point5, val..val + splice_point5 as u32).collect();
    } else {
        let splice_point6: usize = verifier::any!();
        verifier::assume!(splice_point6 < len);

        for _i in 0..CAP - len {
            v1.push(verifier::any!());
        }

        let val: u32 = verifier::any!();

        // New length would overflow the vector, so panic should occur.
        let _: ArrayVec<[u32; CAP]> = v1.splice(splice_point6..splice_point6 + 1 as usize, val..val + 2).collect();
    }

    // This assertion should not be reachable since the previous assertion should panic.
    verifier::error!();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_split_off() {
    const CAP: usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();
    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let split_point: usize = verifier::any!();

    // Panics if split_point > len.
    let v2: ArrayVec<[u32; CAP]> = v.split_off(split_point);

    if split_point > len {
        // This assertion should not be reachable since the previous operation panics.
        verifier::error!();
    } else {
        verifier::vassert!(v.len() == split_point);
        verifier::vassert!(v2.len() == len - split_point);
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(6))]
fn test_swap_remove() {
    const CAP: usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();
    let len: usize = verifier::any!();
    verifier::assume!(2 <= len && len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let remove_point1: usize = verifier::any!();
    verifier::assume!(remove_point1 < len);

    v.swap_remove(remove_point1);

    verifier::vassert!(v.len() == len - 1);
    verifier::vassert!(v.capacity() == CAP);

    let remove_point2: usize = verifier::any!();
    verifier::assume!(remove_point2 < len);

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
    verifier::error!();
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_truncate() {
    let mut v: ArrayVec<[u32; 8]> = ArrayVec::new();

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
#[cfg_attr(kani, kani::unwind(9))]
fn test_try_append() {
    const CAP: usize = 4;
    let mut v1: ArrayVec<[u32; CAP]> = ArrayVec::new();
    let mut v2: ArrayVec<[u32; CAP]> = ArrayVec::new();

    let len1: usize = verifier::any!();
    verifier::assume!(len1 <= CAP);

    let len2: usize = verifier::any!();
    verifier::assume!(len2 <= CAP);
    
    for _i in 0..len1 {
        v1.push(verifier::any!());
    }
    
    verifier::vassert!(v1.len() == len1);

    for _i in 0..len2 {
        v2.push(verifier::any!());
    }

    verifier::vassert!(v2.len() == len2);

    let result: Option<&mut ArrayVec<[u32; CAP]>> = v1.try_append(&mut v2);

    if len1 + len2 <= CAP {
        verifier::vassert!(result.is_none());
        verifier::vassert!(v1.len() == len1 + len2);
        verifier::vassert!(v2.len() == 0);
    } else {
        verifier::vassert!(result.is_some());
        verifier::vassert!(v1.len() == len1);
        verifier::vassert!(v2.len() == len2);
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_try_from_array_len() {
    let len: usize = verifier::any!();

    let v: Result<ArrayVec<[u32; 8]>, _>  = ArrayVec::try_from_array_len([0; 8], len);

    if len <= 8 {
        verifier::vassert!(v.is_ok());
        verifier::vassert!(v.unwrap().len() == len);
    } else {
        verifier::vassert!(v.is_err());
    }

    // Necessary to make seahorn work.
    let x: u32 = verifier::any!();
    verifier::assume!(x < u32::MAX/2);
    let result: u32 = x * 2;
    verifier::vassert!(result >= x);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
fn test_try_insert() {
    const CAP: usize = 4;
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();
    
    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    for _i in 0..len {
        v.push(verifier::any!());
    }

    let insert_point: usize = verifier::any!();

    let result: Option<u32> = v.try_insert(insert_point, verifier::any!());

    if insert_point > len {
        // This assertion should not be reachable since the previous operation panics.
        verifier::error!();
    } else if len == CAP {
        verifier::vassert!(result.is_some());
        verifier::vassert!(v.len() == CAP);
        verifier::vassert!(v.capacity() == CAP);
    } else {
        verifier::vassert!(result.is_none());
        verifier::vassert!(v.len() == len + 1);
        verifier::vassert!(v.capacity() == CAP);
    }    
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(5))]
fn test_try_push() {
    const CAP: usize = 4;
    // NOTE: Create a vector of fixed size capacity
    let mut v: ArrayVec<[u32; CAP]> = ArrayVec::new();

    // NOTE: Create a ND number of elements to push
    let len: usize = verifier::any!();
    verifier::assume!(len <= CAP);

    // NOTE: INVARIANT: We should always be able to push len elements since
    // len is <= capacity
    for i in 0..len {
        let result: Option<u32> = v.try_push(verifier::any!());
        verifier::vassert!(result.is_none());
        verifier::vassert!(v.len() == i + 1); // len is 1-based, iterator is 0-based
    }

    // NOTE: INVARIANT: When len == capacity then another push fails
    if v.len() == v.capacity() {
        let result: Option<u32> = v.try_push(verifier::any!());
        verifier::vassert!(result.is_some());
    }
}
