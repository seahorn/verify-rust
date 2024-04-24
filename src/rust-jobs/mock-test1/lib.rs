#![cfg_attr(not(kani), no_std)]

use verifier;
use sea;
use seamock::seamock;

#[seamock]
pub trait Test {
    fn a(&self, x: i32, y: bool) -> i32;
    fn b(&self) -> u8;
    fn c(&self) -> i32;
}

fn test<T: Test>(mock_test: &T, x: i32, y: bool) -> i32 {
    let ans = mock_test.a(x, y);
    mock_test.b();
    mock_test.c();
    return ans;
}

#[no_mangle]
pub extern "C" fn entrypt() {
    let mut mock: MockTest = MockTest::new();
	let x: i32 = verifier::any!();
    let y: bool = verifier::any!();

    verifier::assume!(x < 10);

    mock
        .times_a(2)
        .times_b(2)
        .times_c(1)
        .returning_a(|x, _y| x + 5)
        .returning_b(|| 4);
    
    verifier::vassert!(mock.a(x, y) < 15);
    verifier::vassert!(mock.b() == 4);
    verifier::vassert!(test(&mock, x, y) < 15);
    verifier::vassert!(mock.expect_times_a(2));
    verifier::vassert!(mock.expect_times_b(2));
    verifier::vassert!(mock.expect_times_c(1));
}
