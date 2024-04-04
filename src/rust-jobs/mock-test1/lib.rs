#![cfg_attr(not(kani), no_std)]

use verifier;
use seamock::seamock;

// TODO: move this enum into the library
enum WithVal<T> {
    Gt(T),
    Gte(T),
    Lt(T),
    Lte(T),
    Eq(T),
}

#[seamock]
pub trait Test {
    fn a(&self, z: bool, a: i32) -> i32;
    fn b(&self) -> u8;
    fn c(&self) -> i32;
}

#[no_mangle]
pub extern "C" fn entrypt() {
    let mut x: MockTest = MockTest::new();
	let mut y: i32 = verifier::any!();
    let mut z: bool = verifier::any!();

    verifier::assume!(y < 10);

    x
        .times_a(2)
        .returning_a(|z, a| a + 6)
        .returning_b(|| 4)
        .times_b(2);
    
    // verifier::vassert!(y == x.c());
    verifier::vassert!(x.a(z, y) < 15);
}
