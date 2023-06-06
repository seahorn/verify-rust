#![no_std]
pub use sea_rs_common::CAllocator;

extern crate alloc;
use alloc::string::String;

extern crate core;
use core::result::Result;

use sea;

sea::define_sea_nd!(sea_nd_int, i32, 42);

#[no_mangle]
pub extern "C" fn entrypt() {
    let v: i32 = sea_nd_int();

    let x: Result<String, i32> = Err(v);
    let y: Result<&str, &i32> = Err(&v);

    let x_error: i32 = match x {
        Err(err) => err,
        _ => 0,
    };

    let y_error: i32 = match y {
        Err(err) => *err,
        _ => 0,
    };

    let result: i32 = x_error + y_error;

    sea::sassert!(result == v*2);
}
