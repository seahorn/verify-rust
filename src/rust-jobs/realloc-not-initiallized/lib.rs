#![feature(new_uninit)]

use verifier;

extern crate alloc;
use alloc::alloc::{Layout, alloc, realloc, dealloc, handle_alloc_error};
use alloc::boxed::Box;

use core::mem;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::ptr;

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    let new_layout = Layout::array::<i32>(10).unwrap();
    let old_layout = Layout::array::<i32>(1).unwrap();
    let old_ptr = NonNull::dangling().as_ptr() as *mut u8;

    unsafe { realloc(old_ptr, old_layout, new_layout.size()) };

    let flag : bool = verifier::any!();
    verifier::assume!(flag == true);

    verifier::vassert!(flag);
}
