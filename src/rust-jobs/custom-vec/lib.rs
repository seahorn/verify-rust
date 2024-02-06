#![cfg_attr(not(kani), no_std)]
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
pub extern "C" fn entrypt() {
    test_new();
    test_grow();
    test_pop();
    test_push();
    test_drop();
    test_deref();
    test_deref_mut();
    test_insert();
    test_remove();

    test_into_iter();
    test_into_iter_size();
    test_into_iter_drop();

    // verifier::vassert!(false);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_new() {
    let v: CustomVec<i32> = CustomVec::new();
    verifier::vassert!(v.len == 0);
    verifier::vassert!(v.cap == 0);
    verifier::vassert!(!v.ptr.as_ptr().is_null());
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_realloc() {
    let v: CustomVec<i32> = CustomVec::new();
    verifier::vassert!(v.len == 0);
    verifier::vassert!(v.cap == 0);
    verifier::vassert!(!v.ptr.as_ptr().is_null());
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_grow() {
    let original = verifier::any!();
    verifier::assume!(original < usize::MAX/2);

    let mut v: CustomVec<i32> = CustomVec::new();

    // initialize memory
    v.grow();
    verifier::vassert!(v.ptr != NonNull::dangling());

    v.len = original;
    v.cap = original;

    v.grow();

    if original == 0 {
        verifier::vassert!(v.cap == 1)
    } else {
        verifier::vassert!(v.cap == 2 * original);
    }
    verifier::vassert!(v.len == original);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_pop() {
    let original = verifier::any!();
    verifier::assume!(original > 0);
    verifier::assume!(original < usize::MAX/2);

    let mut v: CustomVec<i32> = CustomVec::new();
    // initialize memory
    v.grow();
    verifier::vassert!(v.ptr != NonNull::dangling());
    v.len = original;
    v.cap = original;

    v.grow();
    v.pop();

    verifier::vassert!(v.len == original - 1);
    verifier::vassert!(v.cap == original * 2);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_push() {
    let original = verifier::any!();
    verifier::assume!(original > 0);
    verifier::assume!(original < usize::MAX/2);

    let mut v: CustomVec<i32> = CustomVec::new();
    // initialize memory
    v.grow();
    verifier::vassert!(v.ptr != NonNull::dangling());
    v.len = original;
    v.cap = original;

    v.grow();
    v.push(0);   

    verifier::vassert!(v.len == original + 1);
    verifier::vassert!(v.cap == original * 2);
}

#[no_mangle]
#[inline(never)]
#[cfg_attr(kani, kani::proof)]
fn test_drop() {
    pub struct DropTest { _value: i32, }
    impl Drop for DropTest {
        fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
    }
    static mut DROP_COUNT: usize = 0;

    let original: usize = 5;

    let mut v: CustomVec<DropTest> = CustomVec::new();
    for i in 0..original { v.push(DropTest { _value: i.try_into().unwrap() }); }
    _ = v.pop();
    _ = v.pop();
    _ = v.pop();

    drop(v);
    verifier::vassert!(unsafe { DROP_COUNT == original });
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(11))]
fn test_deref() {
    let original: usize = verifier::any!();
    let num_pops: usize = verifier::any!();
    verifier::assume!(num_pops <= original);
    // loop unwind bound
    // verifier::assume!(num_pops < 10);

    let mut v: CustomVec<i32> = CustomVec::new();
    for i in 0..original { v.push(i.try_into().unwrap()); }
    for _i in 0..num_pops { _ = v.pop(); }
    v.push(1);
    let slice: &[i32] = &*v;
    verifier::vassert!(slice.len() == original - num_pops + 1);
    verifier::vassert!(slice[slice.len()-1] == 1);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_deref_mut() {
    let mut v: CustomVec<i32> = CustomVec::new();
    v.push(0);
    v.push(3);
    v.push(5);

    let slice: &mut [i32] = &mut *v;
    let length: usize = slice.len();
    slice[0] = 10;
    slice[1] = 40;
    slice.sort();

    verifier::vassert!(length == 3);
    verifier::vassert!(v.pop() == Some(40));
    verifier::vassert!(v.pop() == Some(10));
    verifier::vassert!(v.pop() == Some(5));
    verifier::vassert!(v.len == 0);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(11))]
fn test_insert() {
    let mut v: CustomVec<i32> = CustomVec::new();
    let n: usize = verifier::any!();
    let index: usize = verifier::any!();
    verifier::assume!(index <= n);
    verifier::assume!(n < 10);

    for _i in 0..n { v.push(1); }
    
    v.insert(index, -1);
    let slice: &mut [i32] = &mut *v;
    verifier::vassert!(slice[index] == -1);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(10))]
fn test_remove() {
    let mut v: CustomVec<i32> = CustomVec::new();
    let n: usize = verifier::any!();
    verifier::assume!(n < 10);
    verifier::assume!(n > 0);
    let index: usize = verifier::any!();
    verifier::assume!(index <= n);

    for i in 0..n { v.push(i.try_into().unwrap()); }
    
    let res: i32 = v.remove(index);
    verifier::vassert!(res == index.try_into().unwrap());
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_into_iter() {
    let n: u32 = 5;
    // iterate forwards
    let mut v: CustomVec<u32> = CustomVec::new();
    for i in 0..n { v.push(i); }
    let mut iter: IntoIter<u32> = v.into_iter();

    for i in 0..n {
        verifier::vassert!(iter.next().unwrap() == i);
    }

    let n: u32 = 3;
    // iterate backwards
    let mut v: CustomVec<u32> = CustomVec::new();
    for i in 0..n { v.push(i); }
    let mut iter: IntoIter<u32> = v.into_iter();

    for i in 0..n {
        verifier::vassert!(iter.next_back().unwrap() == n-i-1);
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_into_iter_size() {
    let n = 10;

    let mut v: CustomVec<u32> = CustomVec::new();
    for i in 0..n { v.push(i); }
    
    let mut iter: IntoIter<u32> = v.into_iter();

    for i in 0..n {
        let front: bool = verifier::any!();
        if front {
            _ = iter.next();
        } else {
            _ = iter.next_back();
        }
        let size: usize = (n-i-1).try_into().unwrap();
        verifier::vassert!(iter.size_hint().0 == size);
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_into_iter_drop() {
    static mut DROP_COUNT: u32 = 0;
    pub struct DropTest { _value: u32, }
    impl Drop for DropTest {
        fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
    }

    let n: u32 = 24;
    let mut v: CustomVec<DropTest> = CustomVec::new();
    for i in 0..n {
        v.push(DropTest { _value: i.try_into().unwrap() });
    }

    let mut iter: IntoIter<DropTest> = v.into_iter();
    let x: u32 = 0;
    for _i in 5..x {
        let front: bool = verifier::any!();
        if front { _ = iter.next(); }
        else { _ = iter.next_back(); }
    }

    verifier::vassert!(unsafe { DROP_COUNT == x });

    drop(iter);
    verifier::vassert!(unsafe { DROP_COUNT == n });
}





pub struct CustomVec<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}

pub struct IntoIter<T> {
    buf: NonNull<T>,
    cap: usize,
    start: *const T,
    end: *const T,
    len: usize,
}

impl<T> CustomVec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        CustomVec {
            ptr: NonNull::dangling(),
            len: 0,
            cap: 0,
        }
    }

    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            // This can't overflow since self.cap <= isize::MAX.
            let new_cap = 2 * self.cap;

            // `Layout::array` checks that the number of bytes is <= usize::MAX,
            // but this is redundant since old_layout.size() <= isize::MAX,
            // so the `unwrap` should never fail.
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
        assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { realloc(old_ptr, old_layout, new_layout.size()) }
        };

        // If allocation fails, `new_ptr` will be null, in which case we abort.
        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap { self.grow(); }
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }
        // Can't fail, we'll OOM first.
        self.len += 1;
    }
    
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr.as_ptr().add(self.len)))
            }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.cap == self.len { self.grow(); }
    
        unsafe {
            ptr::copy(
                self.ptr.as_ptr().add(index),
                self.ptr.as_ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr.as_ptr().add(index), elem);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        
        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr.as_ptr().add(index));
            ptr::copy(
                self.ptr.as_ptr().add(index + 1),
                self.ptr.as_ptr().add(index),
                self.len - index,
            );
            result
        }
    }
}

impl<T> Drop for CustomVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            unsafe {
                let slice: &mut [T] = core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len);
                _ = Box::from_raw(slice as *mut [T]);
            }
            core::mem::forget(self.ptr);
        }
    }
}

impl<T> Deref for CustomVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> DerefMut for CustomVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> IntoIterator for CustomVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        let vec: ManuallyDrop<CustomVec<T>> = ManuallyDrop::new(self);

        let ptr: NonNull<T> = vec.ptr;
        let cap: usize = vec.cap;
        let len: usize = vec.len;

        unsafe {
            IntoIter {
                buf: ptr,
                cap: cap,
                start: ptr.as_ptr(),
                end: if cap == 0 {
                    ptr.as_ptr()
                } else {
                    ptr.as_ptr().add(len)
                },
                len: len,
            }
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result: T = ptr::read(self.start);
                self.start = self.start.offset(1);
                self.len -= 1;
                Some(result)
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize)
                         / mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                self.len -= 1;
                Some(ptr::read(self.end))
            }
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            if core::mem::needs_drop::<T>() {
                unsafe {
                    let start_ptr = self.start as *mut T;
            
                    for i in 0..self.len {
                        let element_ptr = start_ptr.add(i);
                        ptr::drop_in_place(element_ptr);
                    }
                }
            }
            core::mem::forget(self.start);
            core::mem::forget(self.end);

            let layout: Layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.buf.as_ptr() as *mut u8, layout);
            }
        }
    }
}