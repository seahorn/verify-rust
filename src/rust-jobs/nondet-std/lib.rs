// any_vec / any_string (std feature): reachability (`testfail_*`, expected
// sat, one edge value each) and validity (`test_*`, expected unsat; each has
// an off-by-one `testfail_*_mut` twin that must be sat).
//
// Capacity is fixed (any_vec: N, any_string: len). A nondet capacity was
// tried and made SeaHorn lose the stored elements/bytes (test_vec_contents and
// test_string_valid went sat), so capacity states such as len 1 / cap 1 are a
// documented limitation of the generators, not tested here.
//
// sea.yaml sets horn-bv2-word-size: 1: at the default word size SeaHorn loses
// bytes copied with a symbolic length, so String contents are unreachable.

use verifier::{any_string, any_vec, any_with, Arbitrary};

#[no_mangle]
pub extern "C" fn entrypt() {}

#[inline(always)]
fn opaque<T: Copy>(x: T) -> T {
    unsafe { core::ptr::read_volatile(&x) }
}

// --- any_vec ---

#[no_mangle]
pub extern "C" fn testfail_vec_empty() {
    let v: Vec<u8> = any_vec::<u8, 4>();
    verifier::vassert!(!v.is_empty());
}

#[no_mangle]
pub extern "C" fn testfail_vec_full_last() {
    let v: Vec<u8> = any_vec::<u8, 4>();
    verifier::vassert!(!(v.len() == 4 && v[3] == 0xAB));
}

#[no_mangle]
pub extern "C" fn test_vec_len() {
    let v: Vec<u8> = any_vec::<u8, 4>();
    verifier::vassert!(opaque(v.len()) <= 4);
}

#[no_mangle]
pub extern "C" fn testfail_vec_len_mut() {
    let v: Vec<u8> = any_vec::<u8, 4>();
    verifier::vassert!(opaque(v.len()) < 4);
}

#[no_mangle]
pub extern "C" fn test_vec_inv() {
    let v: Vec<u8> = any_vec::<u8, 4>();
    let (l, c) = (opaque(v.len()), opaque(v.capacity()));
    verifier::vassert!((l <= c) & (c <= 4));
}

#[no_mangle]
pub extern "C" fn testfail_vec_inv_mut() {
    let v: Vec<u8> = any_vec::<u8, 4>();
    verifier::vassert!(opaque(v.len()) < opaque(v.capacity()));
}

// elements drawn under a constraint keep it once stored in the Vec's heap
// buffer; fails if SeaHorn loses the stored bytes
#[derive(Clone, Copy)]
struct Small(u8);
impl Arbitrary for Small {
    fn any() -> Self { Small(any_with(|x: &u8| *x < 10)) }
}

#[no_mangle]
pub extern "C" fn test_vec_contents() {
    let v: Vec<Small> = any_vec::<Small, 4>();
    for x in v.iter() {
        verifier::vassert!(opaque(x.0) < 10);
    }
}

#[no_mangle]
pub extern "C" fn testfail_vec_contents_mut() {
    let v: Vec<Small> = any_vec::<Small, 4>();
    for x in v.iter() {
        verifier::vassert!(opaque(x.0) < 9);
    }
}

// --- any_string ---

#[no_mangle]
pub extern "C" fn testfail_string_empty() {
    let s = any_string::<4>();
    verifier::vassert!(!s.is_empty());
}

#[no_mangle]
pub extern "C" fn testfail_string_emoji() {
    let s = any_string::<4>();
    verifier::vassert!(s != "😀");
}

#[no_mangle]
pub extern "C" fn test_string_valid() {
    let s = any_string::<4>();
    verifier::vassert!(opaque(s.len()) <= 4);
    let mut n = 0;
    for c in s.chars() {
        let cu = c as u32;
        verifier::vassert!(cu < 0xD800 || (cu > 0xDFFF && cu <= 0x10FFFF));
        n += c.len_utf8();
    }
    verifier::vassert!(opaque(n) == s.len());
}

#[no_mangle]
pub extern "C" fn testfail_string_valid_mut() {
    let s = any_string::<4>();
    verifier::vassert!(opaque(s.len()) < 4);
}
