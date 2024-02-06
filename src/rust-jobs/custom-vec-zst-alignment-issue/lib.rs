#![cfg_attr(not(kani), no_std)]

extern crate alloc;
use alloc::alloc::{Layout, alloc, realloc, dealloc, handle_alloc_error};
use alloc::boxed::Box;

use core::{mem, ptr};
use core::ptr::NonNull;
use core::ops::DerefMut;
use core::ops::Deref;
use core::marker::PhantomData;



#[no_mangle]
pub extern "C" fn entrypt() {
    test_alignment_bug();
}


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_alignment_bug() {
    #[repr(align(4))] // alignmnet size of 4
    struct ZST { }

    let alignment_size: usize = mem::align_of::<ZST>();

    let n: u32 = 10;
    let mut v: CustomVec<ZST> = CustomVec::new();
    for _ in 0..n {
        v.push(ZST {});
    }

    let mut iter: IntoIter<ZST> = v.into_iter();
    let mut unaligned_start: bool = false;
    let mut unaligned_end: bool = false;

    for i in 0..n {
        if i%2 == 0 {
            match iter.next() {
                Some((_value, ptr)) => {
                    if (ptr as usize) % alignment_size != 0 {
                        unaligned_start = true;
                    }
                },
                None => {},
            };
        } else {
            match iter.next_back() {
                Some((_value, ptr)) => {
                    if (ptr as usize) % alignment_size != 0 {
                        unaligned_end = true;
                    }
                },
                None => {},
            };
        }
    }

    verifier::vassert!(unaligned_start);
    verifier::vassert!(unaligned_end);
    // verifier::vassert!(false);
}


pub struct Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut CustomVec<T>>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = (T, *const T);
    fn next(&mut self) -> Option<(T, *const T)> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<(T, *const T)> { self.iter.next_back() }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

impl<T> CustomVec<T> {
    pub fn drain(&mut self) -> Drain<T> {
        unsafe {
            let iter = RawValIter::new(&self);

            // this is a mem::forget safety thing. If Drain is forgotten, we just
            // leak the whole Vec's contents. Also we need to do this *eventually*
            // anyway, so why not do it now?
            self.len = 0;

            Drain {
                iter: iter,
                vec: PhantomData,
            }
        }
    }
}

struct RawValIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawValIter<T> {
    unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                ((slice.as_ptr() as usize) + slice.len()) as *const _
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = (T, *const T);
    fn next(&mut self) -> Option<(T, *const T)> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                let old_ptr = self.start;
                self.start = if mem::size_of::<T>() == 0 {
                    (self.start as usize + 1) as *const _
                } else {
                    self.start.offset(1)
                };
                Some((result, old_ptr))
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        let len = (self.end as usize - self.start as usize)
                  / if elem_size == 0 { 1 } else { elem_size };
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<(T, *const T)> {
        if self.start == self.end {
            None
        }
        else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.end = (self.end as usize - 1) as *const _;
                    // Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    self.end = self.end.offset(-1);
                }
                Some((ptr::read(self.end), self.end))
            }
        } 
    }
}

pub struct IntoIter<T> {
    _buf: RawVec<T>, // we don't actually care about this. Just need it to live.
    iter: RawValIter<T>,
    len: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = (T, *const T);
    fn next(&mut self) -> Option<(T, *const T)> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<(T, *const T)> { self.iter.next_back() }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // for _ in &mut *self {}
        unsafe {
            for mut _i in 0..self.len {
                if self.iter.start >= self.iter.end { 
                    _i = self.len;
                    continue;
                }
                
                ptr::drop_in_place(self.iter.start as *mut T);
                self.iter.start = self.iter.start.offset(1);
            }
        }
    }
}

impl<T> IntoIterator for CustomVec<T> {
    type Item = (T, *const T);
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let iter = RawValIter::new(&self);
            let buf = ptr::read(&self.buf);
            let len = self.len;

            mem::forget(self);

            IntoIter {
                iter: iter,
                _buf: buf,
                len: len,
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

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        let elem_size = mem::size_of::<T>();

        if self.cap != 0 && elem_size != 0 {
            unsafe {
                dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
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

impl<T> Drop for CustomVec<T> {
    fn drop(&mut self) {
        unsafe {
            let slice: &mut [T] = core::slice::from_raw_parts_mut(self.ptr(), self.len);

            let copy_ptr: *mut T = alloc(Layout::array::<T>(self.len).unwrap()) as *mut T;
            copy_ptr.copy_from_nonoverlapping(slice.as_ptr(), self.len);
            let copy_slice: &mut [T] = core::slice::from_raw_parts_mut(copy_ptr, self.len);
            _ = Box::from_raw(copy_slice as *mut [T]);
        }
        // deallocation is handled by RawVec
    }
}

unsafe impl<T: Send> Send for CustomVec<T> {}
unsafe impl<T: Sync> Sync for CustomVec<T> {}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

