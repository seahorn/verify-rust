#![no_std]

// Validity of generated values (property 2: no invalid values) and absence of
// generator-induced panics (property 3). Each `test_*` function is verified
// on its own and must be unsat; run with `verify --vac` to check every
// assertion is reached. Each `testfail_*_mut` is the mutation of the test
// above it: the same assertion made off-by-one at the boundary, so it must be
// sat. That shows the test's assertion is live and the boundary is reached.
// Panics are verifier errors here (CUSTOM_PANIC_NO_STD).

use verifier::{any_slice, any_str, any_with, nondet, nondet_option, nondet_with, Arbitrary};

#[derive(Arbitrary)]
#[repr(u8)]
enum Tag {
    A = 10,
    B = 20,
    C = 30,
}

#[derive(Arbitrary)]
enum Shape {
    Dot,
    Square(u8),
    Rect { w: u8, h: u8 },
}

#[derive(Arbitrary)]
struct Pair<T> {
    a: T,
    b: T,
}

#[no_mangle]
pub extern "C" fn entrypt() {}

// Pass a value through memory with a volatile load so LLVM cannot fold the
// assertion using the generator's own assume. Without it, assertions such as
// `len <= 4` are discharged before SeaHorn runs: `unsat` with no assertion
// actually checked (`--vac` lists none).
#[inline(always)]
fn opaque<T: Copy>(x: T) -> T {
    unsafe { core::ptr::read_volatile(&x) }
}

fn scalar_ok(cu: u32) -> bool {
    cu < 0xD800 || (cu > 0xDFFF && cu <= 0x10FFFF)
}

// char is a Unicode scalar value
#[no_mangle]
pub extern "C" fn test_char_valid() {
    let c: char = verifier::any!();
    let cu = c as u32;
    verifier::vassert!(scalar_ok(cu));
    verifier::vassert!(char::from_u32(cu) == Some(c));
}

#[no_mangle]
pub extern "C" fn testfail_char_valid_mut() {
    let c: char = verifier::any!();
    verifier::vassert!((c as u32) < 0x10FFFF);
}

// chars inside Option (niche layout) are valid too
#[no_mangle]
pub extern "C" fn test_option_char_valid() {
    let o: Option<char> = verifier::any!();
    if let Some(c) = o {
        verifier::vassert!(scalar_ok(c as u32));
    }
}

#[no_mangle]
pub extern "C" fn testfail_option_char_valid_mut() {
    let o: Option<char> = verifier::any!();
    if let Some(c) = o {
        verifier::vassert!((c as u32) > 0);
    }
}

// any_str is valid UTF-8, checked byte by byte without from_utf8:
// no C0/C1/F5..FF bytes, no surrogate (ED A0..), no overlong 3/4-byte
// (E0 <A0, F0 <90), nothing above U+10FFFF (F4 >=90); and the decoded chars
// account for every byte.
#[no_mangle]
pub extern "C" fn test_str_valid() {
    let mut bytes = [0u8; 4];
    let st = any_str(&mut bytes);
    let b = st.as_bytes();
    verifier::vassert!(b.len() <= 4);
    for i in 0..b.len() {
        verifier::vassert!(b[i] != 0xC0 && b[i] != 0xC1 && b[i] < 0xF5);
        if i + 1 < b.len() {
            let (x, y) = (b[i], b[i + 1]);
            verifier::vassert!(!(x == 0xED && y >= 0xA0));
            verifier::vassert!(!(x == 0xE0 && y < 0xA0));
            verifier::vassert!(!(x == 0xF0 && y < 0x90));
            verifier::vassert!(!(x == 0xF4 && y >= 0x90));
        }
    }
    let mut n = 0;
    for c in st.chars() {
        verifier::vassert!(scalar_ok(c as u32));
        n += c.len_utf8();
    }
    verifier::vassert!(n == st.len());
}

#[no_mangle]
pub extern "C" fn testfail_str_valid_mut() {
    let mut bytes = [0u8; 4];
    let st = any_str(&mut bytes);
    for &x in st.as_bytes() {
        verifier::vassert!(x < 0xF4);
    }
}

// derived enum: the discriminant is one of the declared ones
#[no_mangle]
pub extern "C" fn test_tag_discriminant() {
    let t: Tag = verifier::any!();
    // `t as u8` is a select of constants, so without `opaque` LLVM folds the
    // check away. `|` not `||`: with `||` LLVM splits the assert per branch
    // and `--vac` then flags the dead `assert(false)` copy as vacuous.
    let d = opaque(t as u8);
    verifier::vassert!((d == 10) | (d == 20) | (d == 30));
}

#[no_mangle]
pub extern "C" fn testfail_tag_discriminant_mut() {
    let t: Tag = verifier::any!();
    let d = opaque(t as u8);
    verifier::vassert!((d == 10) | (d == 20));
}

// any_slice length is within the buffer
#[no_mangle]
pub extern "C" fn test_slice_len() {
    let mut buf = [0u32; 4];
    let sl = any_slice(&mut buf);
    verifier::vassert!(opaque(sl.len()) <= 4);
}

#[no_mangle]
pub extern "C" fn testfail_slice_len_mut() {
    let mut buf = [0u32; 4];
    let sl = any_slice(&mut buf);
    verifier::vassert!(opaque(sl.len()) < 4);
}

// every generator together: no panic path; the final assertion is reached
#[no_mangle]
pub extern "C" fn test_no_panic() {
    let _: () = verifier::any!();
    let _: (u128, i128, char, f32, f64) = verifier::any!();
    let _: (Option<u8>, Result<u16, i16>, [u8; 4]) = verifier::any!();
    let _: (u8, i8, u16, bool, char, u128) = verifier::any!();
    let s: Shape = verifier::any!();
    let p: Pair<Tag> = verifier::any!();
    let mut buf = [0u16; 4];
    let sl = any_slice(&mut buf);
    let mut bytes = [0u8; 4];
    let st = any_str(&mut bytes);
    let k = st.len() + sl.len() + matches!(s, Shape::Dot) as usize + p.a as usize;
    verifier::vassert!(opaque(k) <= 4 + 4 + 1 + 30);
}

#[no_mangle]
pub extern "C" fn testfail_no_panic_mut() {
    let mut buf = [0u16; 4];
    let sl = any_slice(&mut buf);
    let mut bytes = [0u8; 4];
    let st = any_str(&mut bytes);
    // a panic is an error here: indexing one past the end must be caught
    let i = st.len();
    if i == 4 {
        verifier::vassert!(sl[4] == 0);
    }
}

// any_with / nondet_with: the predicate always holds, and its boundary is
// reachable (the mutation asserts the boundary value never occurs).
#[no_mangle]
pub extern "C" fn test_any_with() {
    let x: u32 = any_with(|x: &u32| *x > 0 && *x <= 100);
    let y: u32 = nondet_with(|y: &u32| *y >= 7);
    verifier::vassert!((opaque(x) >= 1) & (opaque(x) <= 100) & (opaque(y) >= 7));
}

#[no_mangle]
pub extern "C" fn testfail_any_with_mut() {
    let x: u32 = any_with(|x: &u32| *x > 0 && *x <= 100);
    verifier::vassert!(opaque(x) < 100);
}

// nondet(): same as any!(); both ends of the range are reachable
#[no_mangle]
pub extern "C" fn testfail_nondet_max() {
    let x: u8 = nondet();
    verifier::vassert!(opaque(x) != u8::MAX);
}

// nondet_option: both None and Some are reachable, and Some carries f()'s value
#[no_mangle]
pub extern "C" fn testfail_nondet_option_none() {
    let o = nondet_option(|| 5u8);
    verifier::vassert!(opaque(o.is_some()));
}

#[no_mangle]
pub extern "C" fn testfail_nondet_option_some() {
    let o = nondet_option(|| 5u8);
    verifier::vassert!(opaque(o.is_none()));
}

#[no_mangle]
pub extern "C" fn test_nondet_option_value() {
    let o = nondet_option(|| 5u8);
    if let Some(v) = o {
        verifier::vassert!(opaque(v) == 5);
    }
}

#[no_mangle]
pub extern "C" fn testfail_nondet_option_value_mut() {
    let o = nondet_option(|| 5u8);
    if let Some(v) = o {
        verifier::vassert!(opaque(v) == 6);
    }
}
