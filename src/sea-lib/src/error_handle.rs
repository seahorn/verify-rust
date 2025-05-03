/*
    To use specify CUSTOM_PANIC_NO_STD as a variable arg in
    the crate's cmake. For example:
    c_rust_llvm(panic-test panic-test.c ... CUSTOM_PANIC_NO_STD ...)
    The CUSTOM_PANIC_NO_STD variable flag builds std with the custom panic handler.
    To specify behavior pass in features and add functionality

    When adding your own feature, make sure to update lib.rs and make
    the feature mutually exclusive with "std"
*/


use core::panic::PanicInfo;
use core::intrinsics;


#[cfg(feature = "panic_error")]
use crate::seahorn::verifier_error;

#[no_mangle]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    #[cfg(feature = "panic_error")]
    verifier_error();

    intrinsics::abort();
}

#[no_mangle]
#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    #[cfg(feature = "panic_error")]
    verifier_error();

    intrinsics::abort();
}

