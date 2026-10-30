#![no_std]

// Reachability of edge values (property 1 of a nondet generator: every valid
// value is reachable). Each `testfail_*` function asserts that one generator
// does NOT produce one specific edge value, and is verified on its own
// (`sea_discover_tests` runs `verify --expect=sat --entry=<fn>`). So each
// `sat` is evidence for exactly one edge value. An `unsat` would mean the
// generator (or the bound) silently excludes that value, and every unsat
// proof that uses it covers less than it claims.
//
// Floats are compared by bit pattern: SeaHorn does not support fcmp.

use verifier::{any_slice, any_str, Arbitrary};

#[derive(Arbitrary)]
enum Shape {
    Dot,
    Square(u8),
    Rect { w: u8, h: u8 },
}

// generic struct
#[derive(Arbitrary)]
struct Pair<T> {
    a: T,
    b: T,
}

// tuple struct
#[derive(Arbitrary)]
struct Wrap(u8, bool);

// generic enum with a nested derived type
#[derive(Arbitrary)]
enum Outer<T> {
    Leaf(T),
    Node(Shape, Option<T>),
}

// single variant: its `_` arm is the only arm
#[derive(Arbitrary)]
enum One {
    Only(u16),
}

// explicit discriminants: the nondet tag is a variant index, not the discriminant
#[derive(Arbitrary)]
#[repr(u8)]
enum Tag {
    A = 10,
    B = 20,
    C = 30,
}

// const generic parameter
#[derive(Arbitrary)]
struct Buf<const N: usize> {
    d: [u8; N],
}

// The C main calls this; the tests are separate entry points.
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

// `reach!(name, Type, |v| edge)`: sat iff `edge` holds for some `v: Type`.
macro_rules! reach {
    ($name:ident, $ty:ty, |$v:ident| $edge:expr) => {
        #[no_mangle]
        pub extern "C" fn $name() {
            let $v: $ty = verifier::any!();
            verifier::vassert!(!($edge));
        }
    };
}

// integers wider than the C stubs
reach!(testfail_u128_max, u128, |x| x == u128::MAX);
reach!(testfail_u128_hi_only, u128, |x| x == 1u128 << 64);
reach!(testfail_i128_min, i128, |x| x == i128::MIN);

// char: both ends of both scalar-value ranges
reach!(testfail_char_nul, char, |c| c == '\0');
reach!(testfail_char_d7ff, char, |c| c == '\u{D7FF}');
reach!(testfail_char_e000, char, |c| c == '\u{E000}');
reach!(testfail_char_max, char, |c| c == char::MAX);

// floats: special bit patterns
reach!(testfail_f32_qnan, f32, |f| f.to_bits() == 0x7FC0_0000);
reach!(testfail_f32_neg_inf, f32, |f| f.to_bits() == 0xFF80_0000);
reach!(testfail_f64_snan, f64, |f| f.to_bits() == 0x7FF0_0000_0000_0001);
reach!(testfail_f64_neg_zero, f64, |f| f.to_bits() == 0x8000_0000_0000_0000);

// Option / Result
reach!(testfail_option_none, Option<u8>, |o| o.is_none());
reach!(testfail_option_some_max, Option<u8>, |o| o == Some(255));
reach!(testfail_result_err_min, Result<u8, i8>, |r| r == Err(-128));
reach!(testfail_result_ok_zero, Result<u8, i8>, |r| r == Ok(0));

// arrays: one inside the job bound (4), one well past it
reach!(testfail_array4, [u8; 4], |a| a == [0xFF, 0, 0xFF, 1]);
reach!(testfail_array16, [u8; 16], |a| a[0] == 0xCD && a[15] == 0xAB);

// widest tuple
reach!(testfail_tuple6, (u8, i8, u16, bool, char, u128), |t| t
    == (1, -1, 2, true, 'z', 3));

// derive: every variant of a data enum
reach!(testfail_shape_dot, Shape, |s| matches!(s, Shape::Dot));
reach!(testfail_shape_square, Shape, |s| matches!(s, Shape::Square(255)));
reach!(testfail_shape_rect, Shape, |s| matches!(s, Shape::Rect { w: 0, h: 255 }));
// derive: generic, tuple, nested, single-variant, explicit-discriminant, const generic
reach!(testfail_pair_char, Pair<char>, |p| p.a == char::MAX && p.b == '\0');
reach!(testfail_wrap, Wrap, |w| w.0 == 7 && w.1);
reach!(testfail_outer_leaf, Outer<i8>, |o| matches!(o, Outer::Leaf(-128)));
reach!(testfail_outer_node, Outer<i8>, |o| matches!(
    o,
    Outer::Node(Shape::Rect { w: 1, h: 2 }, None)
));
reach!(testfail_one_only, One, |o| matches!(o, One::Only(0xFFFF)));
reach!(testfail_tag_a, Tag, |t| matches!(t, Tag::A));
reach!(testfail_tag_c, Tag, |t| matches!(t, Tag::C));
reach!(testfail_buf3, Buf<3>, |b| b.d == [1, 2, 3]);

// any_slice: length 0, full length N, and the last element
#[no_mangle]
pub extern "C" fn testfail_slice_empty() {
    let mut buf = [0u16; 4];
    let sl = any_slice(&mut buf);
    verifier::vassert!(!sl.is_empty());
}

#[no_mangle]
pub extern "C" fn testfail_slice_full_last() {
    let mut buf = [0u16; 4];
    let sl = any_slice(&mut buf);
    verifier::vassert!(!(sl.len() == 4 && sl[3] == 0xFFFF));
}

// any_str: empty, full-length ASCII, and each UTF-8 width at its edges
macro_rules! reach_str {
    ($name:ident, $s:expr) => {
        #[no_mangle]
        pub extern "C" fn $name() {
            let mut bytes = [0u8; 4];
            let st = any_str(&mut bytes);
            verifier::vassert!(st != $s);
        }
    };
}

reach_str!(testfail_str_empty, "");
reach_str!(testfail_str_ascii4, "a\u{7F}z\0");
reach_str!(testfail_str_2byte_x2, "é\u{7FF}");
reach_str!(testfail_str_d7ff, "\u{D7FF}");
reach_str!(testfail_str_e000, "\u{E000}");
reach_str!(testfail_str_emoji, "😀");
reach_str!(testfail_str_max, "\u{10FFFF}");

// Controls: values the generators must NOT reach, through the same harness
// shape (`test_*`, expected unsat). They show that the `sat` results above
// are not an artifact of the per-entry setup.
macro_rules! unreach {
    ($name:ident, $ty:ty, |$v:ident| $edge:expr) => {
        #[no_mangle]
        pub extern "C" fn $name() {
            let $v: $ty = verifier::any!();
            verifier::vassert!(!($edge));
        }
    };
}

unreach!(test_unreach_char_d800, char, |c| opaque(c as u32) == 0xD800);
unreach!(test_unreach_char_dfff, char, |c| opaque(c as u32) == 0xDFFF);
unreach!(test_unreach_char_110000, char, |c| opaque(c as u32) == 0x11_0000);

#[no_mangle]
pub extern "C" fn test_unreach_slice_len5() {
    let mut buf = [0u16; 4];
    let sl = any_slice(&mut buf);
    verifier::vassert!(opaque(sl.len()) != 5);
}

#[no_mangle]
pub extern "C" fn test_unreach_str_lone_continuation() {
    let mut bytes = [0u8; 4];
    let st = any_str(&mut bytes);
    verifier::vassert!(!(st.len() == 1 && st.as_bytes()[0] == 0x80));
}
