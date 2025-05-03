extern crate alloc;
use alloc::alloc::{Layout, alloc, dealloc, handle_alloc_error};

// use core::alloc::{self, Layout};
// use std::marker::PhantomData;
use core::mem;
use core::ops::{Deref, DerefMut, Index, IndexMut, Range};
use core::ptr::{self, NonNull};

trait IsZST { const IS_ZST: bool; }
impl<T> IsZST for T { const IS_ZST: bool = mem::size_of::<T>() == 0; }

#[macro_export]
macro_rules! sea_vec {
    // sea_vec![1, 3, 2, 7]; => [1, 3, 2, 7], cap = 4*2 = 8
    ($($values:expr),* $(,)?) => {{
        const ARG_COUNT: usize = 0 $(+ { _ = $values; 1 })*;
        let mut v = SeaVec::new(ARG_COUNT*2);
        $(v.push($values);)*
        v
    }};
    // sea_vec!([1, 3, 4], 12); => [1, 3, 4], cap = 12
    ([$($values:expr),* $(,)?]; $cap:expr) => {{
        const ARG_COUNT: usize = 0 $(+ { _ = $values; 1 })*;
        assert!(ARG_COUNT <= $cap, "Arg Count Exceeds Length");

        let mut v = SeaVec::new($cap);
        $(v.push($values);)*
        v
    }};
    // sea_vec![0, 3, 8] => [0, 0, 0], cap = 8
    ($value:expr; $len:expr; $cap:expr) => {{
        assert!($len <= $cap);

        // have macro expansion instead of for loop
        let mut v = SeaVec::new($cap);
        for _ in 0..$len { v.push($value); }
        v
    }};
    // sea_vec!(0, 4); => [0, 0, 0, 0], cap = 8
    ($value:expr; $len:expr) => {{
        // have macro expansion instead of for loop
        let mut v = SeaVec::new($len * 2);
        for _ in 0..$len { v.push($value); }
        v
    }};
}


struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    fn new(capacity: usize) -> Self {
        if T::IS_ZST || capacity == 0 {
            return RawVec {
                ptr: NonNull::dangling(),
                cap: capacity,
            };
        }

        let layout: Layout = Layout::array::<T>(capacity).unwrap();
        assert!(layout.size() <= isize::MAX as usize, "Allocation too large");

        let pointer: *mut u8 = unsafe { alloc(layout) };
        let pointer: NonNull<T> = match NonNull::new(pointer as *mut T) {
            Some(p) => p,
            None => handle_alloc_error(layout),
        };

        RawVec {
            ptr: pointer,
            cap: capacity,
        }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 && !T::IS_ZST {
            unsafe {
                dealloc(
                    self.ptr.as_ptr() as *mut u8,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}

pub struct SeaVec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> SeaVec<T> {
    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    pub fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new(capacity: usize) -> Self {
        SeaVec {
            buf: RawVec::new(capacity),
            len: 0,
        }
    }
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap() {
            panic!("***Capacity Overflow***");
            // add other methods of handling this
        }

        unsafe { ptr::write(self.ptr().add(self.len), elem); }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "***Index Out of Bounds***");

        if self.cap() == self.len {
            panic!("***Capacity Overflow***");
            // add other methods of handling this
        }

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

    pub fn drain(&mut self, range: Range<usize>) -> Drain<T> {
        unsafe {
            let slice: &[T] = &*self;
            let slice: &[T] = &slice[range.clone()];
            let iter: RawValIter<T> = RawValIter::new(slice);

            Drain {
                const_range: range.clone(),
                iter: iter,
                // vec: PhantomData,
                vec: self,
            }
        }
    }
    pub fn drop(&mut self) {
        // for i in 0..10 {
        //     if i > self.cap { break; }
        //     self.pop();
        // }
    }
}

// #[macro_export]
// macro_rules! drop_sea_vec {
//     ($v:expr, $len:expr) => {{
//         for _ in 0..$v.cap {
//             $v.pop();
//         }
//     }};
// }


impl<T> Drop for SeaVec<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() { }
    }
}

impl<T> Deref for SeaVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for SeaVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

impl<T> Index<usize> for SeaVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len(), "Index Out of Bounds");
        unsafe {
            if T::IS_ZST { NonNull::<T>::dangling().as_ref() }
            else { & *self.ptr().add(index) }
        }
    }
}

impl<T> IndexMut<usize> for SeaVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len(), "Index Out of Bounds");
        unsafe {
            if T::IS_ZST { NonNull::<T>::dangling().as_mut() }
            else { &mut *self.ptr().add(index) }
        }
    }
}

impl<T> IntoIterator for SeaVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            let iter: RawValIter<T> = RawValIter::new(&self);
            let buf: RawVec<T> = ptr::read(&self.buf);
            mem::forget(self);

            IntoIter {
                iter: iter,
                _buf: buf,
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
            end: if T::IS_ZST {
                ((slice.as_ptr() as usize) + slice.len()*mem::align_of::<T>()) as *const _
            } else if slice.len() == 0 {
                slice.as_ptr()
            } else {
                slice.as_ptr().add(slice.len())
            },
        }
    }
}

impl<T> Iterator for RawValIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if T::IS_ZST {
                    self.start = (self.start as usize + 1*mem::align_of::<T>()) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    let old_ptr = self.start;
                    self.start = self.start.offset(1);
                    Some(ptr::read(old_ptr))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // let elem_size: usize = mem::size_of::<T>();
        let len: usize = (self.end as usize - self.start as usize)
                  / if T::IS_ZST { 1*mem::align_of::<T>() } else { mem::size_of::<T>() };
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawValIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                if T::IS_ZST {
                    self.end = (self.end as usize - 1*mem::align_of::<T>()) as *const _;
                    Some(ptr::read(NonNull::<T>::dangling().as_ptr()))
                } else {
                    self.end = self.end.offset(-1);
                    Some(ptr::read(self.end))
                }
            }
        }
    }
}

pub struct IntoIter<T> {
    _buf: RawVec<T>,
    iter: RawValIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

pub struct Drain<'a, T: 'a> {
    // vec: PhantomData<&'a mut SeaVec<T>>,
    const_range: Range<usize>,
    vec: &'a mut SeaVec<T>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut *self { }

        if T::IS_ZST || self.const_range.len() == 0 {
            self.vec.len -= self.const_range.len();
        } else {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.vec.ptr().add(self.const_range.end),
                    self.vec.ptr().add(self.const_range.start),
                    self.vec.len() - self.const_range.end,
                );
            }
            self.vec.len -= self.const_range.len();
        }
    }
}