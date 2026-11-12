#![no_std]

// Deliberately false assertion: it fails only if every generator below can
// produce the specific value named in it. Expected result: sat. An `unsat`
// here would mean some generator is stuck on a subset of its values.

use verifier::{any_slice, any_str, Arbitrary};

#[derive(Arbitrary)]
enum Shape {
    Dot,
    Square(u8),
    Rect { w: u8, h: u8 },
}

#[no_mangle]
pub extern "C" fn entrypt() {
    let s: Shape = verifier::any!();
    let o: Option<u8> = verifier::any!();
    let x: u128 = verifier::any!();
    let c: char = verifier::any!();
    let mut buf = [0u16; 4];
    let sl = any_slice(&mut buf);
    let mut bytes = [0u8; 4];
    let st = any_str(&mut bytes);

    let hit = matches!(s, Shape::Rect { w: 3, h: 4 })
        && o == Some(5)
        && x == u128::MAX
        && c == 'é'
        && sl.len() == 3
        && st == "hé";
    verifier::vassert!(!hit);
}
