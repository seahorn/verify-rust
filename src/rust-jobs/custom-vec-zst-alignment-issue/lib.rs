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
    let v: u8 = verifier::any!();
    match v {
        0 => testfail_alignment_bug(),
        1 => test_alignment_fixed(),
        _ => (),
    }
}

// A ZST whose dangling pointer must be 4-aligned.
#[repr(align(4))]
struct ZST {}

const N: u32 = 10;

// Push N ZSTs, then take N + 2 items alternately from the front and the back.
// Every pointer the iterator hands out must be aligned for ZST, and exactly N
// items must come out: the two extra calls (one per end) must return None.
// Exhaustion is checked through the counter rather than `vassert!(x.is_none())`,
// which LLVM lowers to `assume(start != end); assert(false)`. --vac reports that
// form as unreachable even when it is proven.
fn drain_both_ends<const ALIGNED: bool>() {
    let alignment_size: usize = mem::align_of::<ZST>();

    let mut v: CustomVec<ZST> = CustomVec::new();
    for _ in 0..N {
        v.push(ZST {});
    }

    let mut iter: IntoIter<ZST, ALIGNED> = v.into_raw_iter();
    let mut yielded: u32 = 0;

    for i in 0..N + 2 {
        let item = if i % 2 == 0 { iter.next() } else { iter.next_back() };
        match item {
            Some((_value, ptr)) => {
                verifier::vassert!((ptr as usize) % alignment_size == 0);
                yielded += 1;
            }
            None => {}
        }
    }

    verifier::vassert!(yielded == N);
}

// The bug this job documents. For a ZST, RawValIter uses `start`/`end` as byte
// counters starting at the dangling pointer (address 4 here) and reads through
// them, so the second read is at address 13 (the first `next_back`), which is
// misaligned. Reading through a misaligned pointer is UB even for a ZST.
//
// Expected sat: core's debug check in `ptr::read` ("ptr::read requires that the
// pointer argument is aligned and non-null") panics, and the panic is a
// verification error (CUSTOM_PANIC_NO_STD + verifier/panic_error). Without that
// check, the alignment vassert in drain_both_ends fails on the same pointer.
//
// The original harness asserted that the pointers *were* misaligned and was an
// unsat test. Under abort-on-panic, the `ptr::read` check cut every path before
// those asserts, so that proof was vacuous.
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::should_panic)]
fn testfail_alignment_bug() {
    drain_both_ends::<false>();
}

// The same harness with the fix std uses: keep the byte counters for
// bookkeeping, but read and hand out `NonNull::dangling()`, which is aligned.
#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_alignment_fixed() {
    drain_both_ends::<true>();
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

// ALIGNED = false is the original (buggy) ZST handling. ALIGNED = true reads
// ZSTs through an aligned dangling pointer instead of the byte counter.
struct RawValIter<T, const ALIGNED: bool = false> {
    start: *const T,
    end: *const T,
}

impl<T, const ALIGNED: bool> RawValIter<T, ALIGNED> {
    unsafe fn new(slice: &[T]) -> Self {
        RawValIter {
            start: slice.as_ptr(),
            end: if mem::size_of::<T>() == 0 {
                slice.as_ptr().wrapping_byte_add(slice.len())
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }

    // The pointer to read the current element through. For a ZST, `counter` is
    // a byte counter and need not be aligned.
    fn elem_ptr(counter: *const T) -> *const T {
        if ALIGNED && mem::size_of::<T>() == 0 {
            NonNull::<T>::dangling().as_ptr()
        } else {
            counter
        }
    }
}

impl<T, const ALIGNED: bool> Iterator for RawValIter<T, ALIGNED> {
    type Item = (T, *const T);
    fn next(&mut self) -> Option<(T, *const T)> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let old_ptr = Self::elem_ptr(self.start);
                let result = ptr::read(old_ptr);
                self.start = if mem::size_of::<T>() == 0 {
                    self.start.wrapping_byte_add(1)
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

impl<T, const ALIGNED: bool> DoubleEndedIterator for RawValIter<T, ALIGNED> {
    fn next_back(&mut self) -> Option<(T, *const T)> {
        if self.start == self.end {
            None
        }
        else {
            unsafe {
                if mem::size_of::<T>() == 0 {
                    self.end = self.end.wrapping_byte_sub(1);
                } else {
                    self.end = self.end.offset(-1);
                }
                let elem = Self::elem_ptr(self.end);
                Some((ptr::read(elem), elem))
            }
        } 
    }
}

pub struct IntoIter<T, const ALIGNED: bool = false> {
    _buf: RawVec<T>, // we don't actually care about this. Just need it to live.
    iter: RawValIter<T, ALIGNED>,
    len: usize,
}

impl<T, const ALIGNED: bool> Iterator for IntoIter<T, ALIGNED> {
    type Item = (T, *const T);
    fn next(&mut self) -> Option<(T, *const T)> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<T, const ALIGNED: bool> DoubleEndedIterator for IntoIter<T, ALIGNED> {
    fn next_back(&mut self) -> Option<(T, *const T)> { self.iter.next_back() }
}

impl<T, const ALIGNED: bool> Drop for IntoIter<T, ALIGNED> {
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
        self.into_raw_iter()
    }
}

impl<T> CustomVec<T> {
    fn into_raw_iter<const ALIGNED: bool>(self) -> IntoIter<T, ALIGNED> {
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

