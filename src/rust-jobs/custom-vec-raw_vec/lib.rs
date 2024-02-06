#![cfg_attr(not(kani), no_std)]
#![feature(new_uninit)]

use verifier;

extern crate alloc;
use alloc::alloc::{Layout, alloc, dealloc, realloc, handle_alloc_error};
use alloc::boxed::Box;

use core::mem;
// use core::mem::ManuallyDrop;
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

    test_into_iter_front();
    test_into_iter_back();
    test_into_iter_size();
    test_into_iter_drop();

    // verifier::vassert!(false);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_new() {
    let v: CustomVec<i32> = CustomVec::new();
    verifier::vassert!(custom_vec_valid_after_init(&v));
    verifier::vassert!(v.len == 0);
    verifier::vassert!(v.cap() == 0);
    verifier::vassert!(!v.ptr().is_null());
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_grow() {
    let original = verifier::any!();

    let mut v: CustomVec<i32> = CustomVec::new();
    verifier::vassert!(custom_vec_valid_after_init(&v));

    v.len = original;
    v.buf.cap = original;

    v.buf.grow();

    if original == 0 {
        verifier::vassert!(v.cap() == 1)
    } else {
        verifier::vassert!(v.cap() == 2 * original);
    }
    verifier::vassert!(v.len == original);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_pop() {
    let original = verifier::any!();
    verifier::assume!(original > 0);

    let mut v: CustomVec<i32> = CustomVec::new();
    verifier::vassert!(custom_vec_valid_after_init(&v));

    v.len = original;
    v.buf.cap = original;

    v.buf.grow();
    v.pop();

    verifier::vassert!(v.len == original - 1);
    verifier::vassert!(v.cap() == original * 2);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_push() {
    let original = verifier::any!();
    verifier::assume!(original > 0);

    let mut v: CustomVec<i32> = CustomVec::new();
    verifier::vassert!(custom_vec_valid_after_init(&v));

    v.len = original;
    v.buf.cap = original;

    v.buf.grow();
    v.push(0);   
    verifier::vassert!(v.len == original + 1);
    verifier::vassert!(v.cap() == original * 2);
    // verifier::vassert!(false);
}

#[no_mangle]
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
fn test_deref() {
    let original: usize = verifier::any!();
    let num_pops: usize = verifier::any!();
    verifier::assume!(num_pops <= original);

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
fn test_insert() {
    let mut v: CustomVec<i32> = CustomVec::new();
    let n: usize = verifier::any!();
    let index: usize = verifier::any!();
    verifier::assume!(index <= n);

    for _i in 0..n { v.push(1); }
    
    v.insert(index, -1);
    let slice: &mut [i32] = &mut *v;
    verifier::vassert!(slice[index] == -1);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_remove() {
    let mut v: CustomVec<i32> = CustomVec::new();
    let n: usize = verifier::any!();
    verifier::assume!(n < 10);
    let index: usize = verifier::any!();
    verifier::assume!(index <= n);

    for i in 0..n { v.push(i.try_into().unwrap()); }
    
    let res: i32 = v.remove(index);
    verifier::vassert!(res == index.try_into().unwrap());
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[inline(never)]
fn test_into_iter_front() {
    let n: u32 = 5;

    let mut v: CustomVec<u32> = CustomVec::new();
    for i in 0..n {
        v.push(i);
    } 

    let mut iter: IntoIter<u32> = v.into_iter();
    for i in 0..n {
        verifier::vassert!(iter.next() == Some(i));
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_into_iter_back() {
    let n: u32 = 5;

    let mut v: CustomVec<u32> = CustomVec::new();
    for i in 0..n {
        v.push(i);
    } 

    let mut iter: IntoIter<u32> = v.into_iter();
    for i in 0..n {
        verifier::vassert!(iter.next_back() == Some(n-i-1));
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

    let mut v: CustomVec<DropTest> = CustomVec::new();
    v.push(DropTest { _value: 0 });
    for i in 0..4{
        v.push(DropTest { _value: i.try_into().unwrap() });
    }
    v.push(DropTest { _value: 0 });

    let mut iter: IntoIter<DropTest> = v.into_iter();
    iter.next();
    iter.next();
    iter.next_back();
    iter.next_back();

    verifier::vassert!(unsafe { DROP_COUNT == 4 });

    drop(iter);
    verifier::vassert!(unsafe { DROP_COUNT == 6 });
}


fn custom_vec_valid_after_init<T>(vec: &CustomVec<T>) -> bool {
    vec.len == 0 &&
    vec.buf.cap == 0 && {
        if mem::size_of::<T>() != 0 { vec.ptr() == NonNull::dangling().as_ptr() } 
        else { true }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        unsafe {
            for mut _i in 0..self.len {
                if self.start >= self.end { 
                    _i = self.len;
                    continue;
                }
                
                ptr::drop_in_place(self.start as *mut T);
                self.start = self.start.offset(1);
            }
        }
    }
}

impl<T> Drop for CustomVec<T> {
    fn drop(&mut self) {
        if self.len != 0 {
            unsafe {
                let slice: &mut [T] = core::slice::from_raw_parts_mut(self.ptr(), self.len);
                
                let copy_ptr: *mut T = alloc(Layout::array::<T>(self.len).unwrap()) as *mut T;
                copy_ptr.copy_from_nonoverlapping(slice.as_ptr(), self.len);
                let copy_slice: &mut [T] = core::slice::from_raw_parts_mut(copy_ptr, self.len);
                _ = Box::from_raw(copy_slice as *mut [T]);
            }
        }
    
        // deallocation is handled by RawVec
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

impl<T> RawVec<T> {
    fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "TODO: implement ZST support");
        RawVec {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };

        let new_layout = Layout::array::<T>(new_cap).unwrap();

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

impl<T> Deref for CustomVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(self.ptr(), self.len)
        }
    }
}

impl<T> DerefMut for CustomVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr(), self.len)
        }
    }
}

pub struct IntoIter<T> {
    _buf: RawVec<T>,
    start: *const T,
    end: *const T,
    len: usize,
}


impl<T> IntoIterator for CustomVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let buf = ptr::read(&self.buf);
            let len = self.len;
            mem::forget(self);

            IntoIter {
                start: buf.ptr.as_ptr(),
                end: if buf.cap == 0 {
                    buf.ptr.as_ptr()
                } else {
                    buf.ptr.as_ptr().add(len)
                },
                _buf: buf,
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
            // self.len -= 1;

            unsafe {
                let result: T = ptr::read(self.start);
                self.start = self.start.offset(1);
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
            // self.len -= 1;
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}

unsafe impl<T: Send> Send for CustomVec<T> {}
unsafe impl<T: Sync> Sync for CustomVec<T> {}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}