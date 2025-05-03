#![cfg_attr(not(kani), no_std)]

#[cfg(not(kani))]
use verifier;
#[cfg(not(kani))]
use sea;
#[cfg(not(kani))]
use seamock::seamock;

#[cfg(not(kani))]
#[seamock]
pub trait Test {
    fn a(&self, x: i32, y: bool) -> i32;
    fn b(&self) -> u8;
    fn c(&self) -> i32;
}

#[cfg(not(kani))]
fn test<T: Test>(mock_test: &T, x: i32, y: bool) -> i32 {
    let ans = mock_test.a(x + 5, y);
    mock_test.b();
    mock_test.c();
    return ans;
}

#[cfg(not(kani))]
#[no_mangle]
pub extern "C" fn entrypt() {
    let mut mock: MockTest = MockTest::new();
    let x: i32 = verifier::any!();
    let y: bool = verifier::any!();

    verifier::assume!(x < 10);
    verifier::assume!(y == true);

    mock
        .times_a(2)
        .times_b(2)
        .times_c(1)
        .with_a((WithVal::Lt(15), WithVal::Eq(true)))
        .returning_a(|x, _y| x + 5)
        .returning_b(|| 4);
    
    verifier::vassert!(mock.a(x, y) < 15);
    verifier::vassert!(mock.b() == 4);
    verifier::vassert!(test(&mock, x, y) < 20);
    verifier::vassert!(mock.expect_times_a(2));
    verifier::vassert!(mock.expect_times_b(2));
    verifier::vassert!(mock.expect_times_c(1));
}
