#![no_std]

// Properties over nondet values of non-integer types: derived structs and
// enums, Option, arrays, tuples, u128, char, bounded slices and strings.
// Panics are errors here (CUSTOM_PANIC_NO_STD), so the helpers must not panic.

use verifier::{any_slice, any_str, Arbitrary};

#[derive(Arbitrary)]
struct Point {
    x: u8,
    y: i16,
}

#[derive(Arbitrary)]
enum Shape {
    Dot,
    Square(u8),
    Rect { w: u8, h: u8 },
}

fn area(s: &Shape) -> u32 {
    match s {
        Shape::Dot => 0,
        Shape::Square(a) => *a as u32 * *a as u32,
        Shape::Rect { w, h } => *w as u32 * *h as u32,
    }
}

#[no_mangle]
pub extern "C" fn entrypt() {
    let p: Point = verifier::any!();
    verifier::assume!(p.x > 100 && p.y < -5);
    verifier::vassert!((p.x as i32) - (p.y as i32) > 105);

    let s: Shape = verifier::any!();
    verifier::assume!(area(&s) > 0);
    verifier::vassert!(!matches!(s, Shape::Dot));

    let o: Option<u8> = verifier::any!();
    verifier::assume!(o.unwrap_or(7) == 200);
    verifier::vassert!(o.is_some());

    let a: [u8; 4] = verifier::any!();
    verifier::assume!(a[0] < a[1] && a[1] < a[2]);
    verifier::vassert!(a[2] >= 2);

    let (x, y): (u128, i8) = verifier::any!();
    verifier::assume!(x > u64::MAX as u128 && y < 0);
    verifier::vassert!((x >> 64) as u64 != 0 && y != 0);

    let c: char = verifier::any!();
    let cu = c as u32;
    verifier::vassert!(cu <= 0x10FFFF && (cu < 0xD800 || cu > 0xDFFF));

    let mut buf = [0u16; 4];
    let sl = any_slice(&mut buf);
    verifier::vassert!(sl.len() <= 4);

    let mut bytes = [0u8; 4];
    let st = any_str(&mut bytes);
    verifier::vassert!(st.chars().count() <= st.len());
}
