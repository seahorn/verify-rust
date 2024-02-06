#![cfg_attr(not(kani), no_std)]

extern crate sea;

custom_print::define_macros!({ print, println, eprint, eprintln }, fmt, |_value: &str| { });

#[no_mangle]
pub extern "C" fn entrypt() {
    print!("test");
    println!("test");
    print!("test {}", 42);
    println!("test {}", 42);
    eprint!("test");
    eprintln!("test");
    eprint!("test {}", 42);
    eprintln!("test {}", 42);
}
