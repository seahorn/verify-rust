// ********************************
// This job is for testing SeaHorns limitations when unrolling loops
// To run the job go into the build directory
// to build run: ninja
// to verify run: ./verify src/rust-jobs/custom-vec-loop-unroll
// to add an unroll bound run: ./verify src/rust-jobs/custom-vec-loop-unroll --bound=10
// You can have any bound. Note that increasing the bound will increase the runtime
//
// To test weather SeaHorn analyzes the whole program place a false assertion at the end
// If you get unsat, SeaHorn isn't analyzing the whole program
// To find where SeaHorn stop analyzing teh program, keep adding and removing false assertions
// until you find the fine line where SeaHorn stop analyzing to see where the issue is
// ********************************



#![cfg_attr(not(kani), no_std)]

use verifier;

extern crate alloc;
use alloc::alloc::{Layout, alloc, dealloc, realloc, handle_alloc_error};

use core::mem;
use core::ptr::NonNull;
use core::ptr;



#[no_mangle]
pub extern "C" fn entrypt() {
    test_push();

    // When running properly, the line below should give sat when used as the assertion is always false
    // if not, keep adding more verifier::vassert!(false) throughout the code until you find where SeaHorn stop analyzing
    // You can also try different versions of the Drop methods for CustomVec and RawVec to fix the issue
    // verifier::vassert!(false);
}


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
// #[cfg_attr(kani, kani::unwind(11))]
fn test_push() {
    // Note that this code is for demonstration purposes only

    let original: usize = verifier::any!();
    verifier::assume!(original > 0 && original <= 10);

    let mut v: CustomVec<usize> = CustomVec::new();

    v.len = original;
    v.buf.cap = original;

    v.buf.grow();
    v.push(0);   
    verifier::vassert!(v.len == original + 1);
    verifier::vassert!(v.cap() == original * 2);
}


impl<T> Drop for CustomVec<T> {
    fn drop(&mut self) {
        // Version 1: Remember to update Drop for RawVec
        // This should fail (always give unsat), add unroll bound to fix
        while let Some(_) = self.pop() { }


        // Version 2: Remember to update Drop for RawVec
        // This should work without the unroll bound, up to a limit
        // verifier::printf!("Before Loop");
        // while let Some(_) = self.pop() {
            // verifier::printf!("During Loop");
        // }


        // Version 3: Remember to update Drop for RawVec
        // This should work without the unroll bound, up to a limit
        // Box::from_raw also handles the deallocation of the vector itself
        // unsafe {
        //     let slice: &mut [T] = core::slice::from_raw_parts_mut(self.ptr(), self.len);
        //     _ = Box::from_raw(copy_slice as *mut [T]);
        // }


        // Try Your Own Version
        // Here
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        // Version 1 & 2
        if self.cap != 0 && mem::size_of::<T>() != 0 {
            unsafe {
                dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }


        // Version 3
        // Empty


        // Try You Own Version
        // Here
    }
}


struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

impl<T> RawVec<T> {
    fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 { usize::MAX } else { 0 };

        RawVec {
            ptr: NonNull::dangling(),
            cap: cap,
        }
    }

    fn grow(&mut self) {
        assert!(mem::size_of::<T>() != 0, "capacity overflow");

        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { realloc(old_ptr, old_layout, new_layout.size()) }
        };
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }
}


pub struct CustomVec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> CustomVec<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new() -> Self {
        CustomVec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap() { self.buf.grow(); }

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr().add(self.len)))
            }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.cap() == self.len { self.buf.grow(); }
    
        unsafe {
            ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr().add(index), elem);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr().add(index));
            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );
            result
        }
    }
}
