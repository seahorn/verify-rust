#![cfg_attr(not(feature = "std"), no_std)]

#![feature(core_intrinsics)]
#![feature(alloc_error_handler)]
#![feature(slice_ptr_get)] 

#[cfg(all(feature = "std", feature = "panic_error"))]
compile_error!("Features \"std\" and \"panic_error\" are mutually exclusive and cannot be enabled at the same time.
\"panic_error\" is to be used when using #![no_std]");

#[cfg(not(feature = "std"))]
pub mod allocator;
#[cfg(not(feature = "std"))]
pub mod error_handle;


pub mod bindings;
pub mod print_macros;
pub mod seahorn;
pub use seahorn::*;
