#![cfg_attr(not(kani), no_std)]

use verifier;

macro_rules! print { ($($args:tt)*) => { } }
macro_rules! println { ($($args:tt)*) => { } }
macro_rules! eprint { ($($args:tt)*) => { } }
macro_rules! eprintln { ($($args:tt)*) => { } }

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let v: i32 = verifier::any!();
    verifier::assume!(v >= 1);
    verifier::assume!(v < i32::MAX/2);
    let result: i32 = v * 2;

    print!("test");
    println!("test");
    print!("test {}", 42);
    println!("test {}", 42);
    eprint!("test");
    eprintln!("test");
    eprint!("test {}", 42);
    eprintln!("test {}", 42);

    verifier::vassert!(result > v);
}
