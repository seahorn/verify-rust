#![cfg_attr(not(kani), no_std)]
#![feature(new_uninit)]

use verifier;

extern crate alloc;
use alloc::alloc::{Layout, alloc, dealloc, realloc, handle_alloc_error};

use core::mem;
// use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::ptr;
use core::marker::PhantomData;



#[no_mangle]
pub extern "C" fn entrypt() {
    // Running all the tests causes SeaHorn to crash

    // **********************
    // These tests are the same as those in custom-vec-raw_vec
    // test_new();
    // test_grow();
    // test_pop();
    // test_push();
    // test_drop();
    // test_deref();
    // test_deref_mut();
    // test_insert();
    // test_remove();
    // **********************

    // **********************
    // These tests are the same. Only the format is modified.
    // test_into_iter_front();
    // test_into_iter_back();
    // test_into_iter_size();
    // test_into_iter_drop();

    // test_drain_front();
    // test_drain_back();
    // test_drain_size();
    // test_drain_drop();

    // **********************
    // Tests for Zero Sized Types
    test_zst();
    test_alignment_bug();
    // verifier::vassert!(false);
}


// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_new() {
//     let v: CustomVec<i32> = CustomVec::new();
//     verifier::vassert!(custom_vec_valid_after_init(&v));
//     verifier::vassert!(v.len == 0);
//     verifier::vassert!(v.cap() == 0);
//     verifier::vassert!(!v.ptr().is_null());
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_grow() {
//     let original = verifier::any!();

//     let mut v: CustomVec<i32> = CustomVec::new();
//     verifier::vassert!(custom_vec_valid_after_init(&v));

//     v.len = original;
//     v.buf.cap = original;

//     v.buf.grow();

//     if original == 0 {
//         verifier::vassert!(v.cap() == 1)
//     } else {
//         verifier::vassert!(v.cap() == 2 * original);
//     }
//     verifier::vassert!(v.len == original);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_pop() {
//     let original = verifier::any!();
//     verifier::assume!(original > 0);

//     let mut v: CustomVec<i32> = CustomVec::new();
//     verifier::vassert!(custom_vec_valid_after_init(&v));

//     v.len = original;
//     v.buf.cap = original;

//     v.buf.grow();
//     v.pop();

//     verifier::vassert!(v.len == original - 1);
//     verifier::vassert!(v.cap() == original * 2);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_push() {
//     let original = verifier::any!();
//     verifier::assume!(original > 0 && original < 1000000);

//     let mut v: CustomVec<i32> = CustomVec::new();
//     verifier::vassert!(custom_vec_valid_after_init(&v));

//     v.len = original;
//     v.buf.cap = original;

//     v.buf.grow();
//     v.push(0);   
//     verifier::vassert!(v.len == original + 1);
//     verifier::vassert!(v.cap() == original * 2);
//     // verifier::printf!("End of Test Push");
//     // verifier::vassert!(false);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_drop() {
//     pub struct DropTest { _value: i32, }
//     impl Drop for DropTest {
//         fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
//     }
//     static mut DROP_COUNT: usize = 0;

//     let original: usize = 5;

//     let mut v: CustomVec<DropTest> = CustomVec::new();
//     for i in 0..original { v.push(DropTest { _value: i.try_into().unwrap() }); }
//     _ = v.pop();
//     _ = v.pop();
//     _ = v.pop();

//     drop(v);
//     verifier::vassert!(unsafe { DROP_COUNT == original });
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_deref() {
//     let original: usize = verifier::any!();
//     let num_pops: usize = verifier::any!();
//     verifier::assume!(num_pops <= original);

//     let mut v: CustomVec<i32> = CustomVec::new();
//     for i in 0..original { v.push(i.try_into().unwrap()); }
//     for _i in 0..num_pops { _ = v.pop(); }
//     v.push(1);
//     let slice: &[i32] = &*v;
//     verifier::vassert!(slice.len() == original - num_pops + 1);
//     verifier::vassert!(slice[slice.len()-1] == 1);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_deref_mut() {
//     let mut v: CustomVec<i32> = CustomVec::new();
//     v.push(0);
//     v.push(3);
//     v.push(5);

//     let slice: &mut [i32] = &mut *v;
//     let length: usize = slice.len();
//     slice[0] = 10;
//     slice[1] = 40;
//     slice.sort();

//     verifier::vassert!(length == 3);
//     verifier::vassert!(v.pop() == Some(40));
//     verifier::vassert!(v.pop() == Some(10));
//     verifier::vassert!(v.pop() == Some(5));
//     verifier::vassert!(v.len == 0);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_insert() {
//     let mut v: CustomVec<i32> = CustomVec::new();
//     let n: usize = verifier::any!();
//     let index: usize = verifier::any!();
//     verifier::assume!(index <= n);

//     for _i in 0..n { v.push(1); }
    
//     v.insert(index, -1);
//     let slice: &mut [i32] = &mut *v;
//     verifier::vassert!(slice[index] == -1);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_remove() {
//     let mut v: CustomVec<i32> = CustomVec::new();
//     let n: usize = verifier::any!();
//     verifier::assume!(n < 10);
//     let index: usize = verifier::any!();
//     verifier::assume!(index <= n);

//     for i in 0..n { v.push(i.try_into().unwrap()); }
    
//     let res: i32 = v.remove(index);
//     verifier::vassert!(res == index.try_into().unwrap());
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_into_iter_front() {
//     let n: u32 = 3;

//     let mut v: CustomVec<u32> = CustomVec::new();
//     for i in 0..n {
//         v.push(i);
//     } 

//     let mut iter: IntoIter<u32> = v.into_iter();
//     for i in 0..n {
//         verifier::vassert!(iter.next().unwrap() == i);
//     }
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_into_iter_back() {
//     let n: u32 = 3;

//     let mut v: CustomVec<u32> = CustomVec::new();
//     for i in 0..n {
//         v.push(i);
//     } 

//     let mut iter: IntoIter<u32> = v.into_iter();
//     for i in 0..n {
//         verifier::vassert!(iter.next_back().unwrap() == n-i-1);

//     }
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_into_iter_size() {
//     let n = 10;

//     let mut v: CustomVec<u32> = CustomVec::new();
//     for i in 0..n { v.push(i); }
    
//     let mut iter: IntoIter<u32> = v.into_iter();

//     for i in 0..n {
//         let front: bool = verifier::any!();
//         if front {
//             _ = iter.next();
//         } else {
//             _ = iter.next_back();
//         }
//         let size: usize = (n-i-1).try_into().unwrap();
//         verifier::vassert!(iter.size_hint().0 == size);
//     }
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_into_iter_drop() {
//     let n: u32 = 4; // max elements it works for (6 due to manual pushes)
//     static mut DROP_COUNT: u32 = 0;
//     pub struct DropTest { _value: u32, }
//     impl Drop for DropTest {
//         fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
//     }

//     let mut v: CustomVec<DropTest> = CustomVec::new();
//     v.push(DropTest { _value: 0 });
//     for i in 0..n {
//         v.push(DropTest { _value: i.try_into().unwrap() });
//     }
//     v.push(DropTest { _value: 0 });

//     let mut iter: IntoIter<DropTest> = v.into_iter();
//     iter.next();
//     iter.next();
//     iter.next_back();
//     iter.next_back();

//     verifier::vassert!(unsafe { DROP_COUNT == 4 });

//     drop(iter);
//     verifier::vassert!(unsafe { DROP_COUNT == 6 });
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_drain_front() {
//     let n: u32 = 8; // most elements it will work for

//     let mut v: CustomVec<u32> = CustomVec::new();
//     for i in 0..n {
//         v.push(i);
//     }

//     let mut drained: Drain<'_, u32> = v.drain();
//     for i in 0..n {
//         verifier::vassert!(drained.next().unwrap() == i);
//     }
//     drop(drained);
    
//     verifier::vassert!(v.pop() == None);
//     v.push(0);
//     verifier::vassert!(v.len == 1);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_drain_back() {
//     let n = 4; // most elements it will work for

//     let mut v: CustomVec<u32> = CustomVec::new();
//     for i in 0..n {
//         v.push(i);
//     }

//     let mut drained: Drain<'_, u32> = v.drain();
//     for i in 0..n {
//         verifier::vassert!(drained.next_back().unwrap() == n-i-1);
//     }

//     drop(drained);
//     verifier::vassert!(v.pop() == None);
//     v.push(0);
//     verifier::vassert!(v.len == 1);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_drain_size() {
//     let n: usize = 8;
//     let mut v: CustomVec<u32> = CustomVec::new();
//     for i in 0..n {
//         v.push(i.try_into().unwrap());
//     }

//     let mut drained: Drain<'_, u32> = v.drain();
//     verifier::vassert!(drained.size_hint().0 == n);
//     for i in 0..n {
//         let front: bool = verifier::any!();
//         if front { _ = drained.next(); }
//         else { _ = drained.next_back(); }

//         verifier::vassert!(drained.size_hint().0 == n-i-1);
//     }
//     drop(drained);

//     verifier::vassert!(v.len == 0);
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_drain_drop() {
//     let n: u32 = 10;
//     static mut DROP_COUNT: u32 = 0;
//     pub struct DropTest { _value: u32, }
//     impl Drop for DropTest {
//         fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
//     }

//     let mut v: CustomVec<DropTest> = CustomVec::new();
//     v.push(DropTest { _value: 0 });
//     for i in 0..n {
//         v.push(DropTest { _value: i });
//     }
//     v.push(DropTest { _value: 0 });

//     let mut drained: Drain<'_, DropTest> = v.drain();
//     _ = drained.next();
//     _ = drained.next();
//     _ = drained.next_back();
//     _ = drained.next();
//     _ = drained.next_back();

//     verifier::vassert!(unsafe { DROP_COUNT == 5 });

//     drop(drained);
//     verifier::vassert!(unsafe { DROP_COUNT == n+2 });

//     v.push(DropTest { _value: 0 });
//     verifier::vassert!(v.len == 1);

//     drop(v);
//     verifier::vassert!(unsafe { DROP_COUNT == n+3 });
// }

// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_alignment() {
//     #[repr(align(2))]
//     struct ZST { }

//     let mut v: CustomVec<u32> = CustomVec::new();
//     for _ in 0.. 10 { v.push(0); }
//     let mut iter: IntoIter<u32> = v.into_iter();
//     for i in 0.. 10 {
//         if i%2 == 0 {
//             _ = iter.next();
//         } else {
//             _ = iter.next_back();
//         }
//     }
//     drop(iter);

//     let mut v: CustomVec<u32> = CustomVec::new();
//     for _ in 0.. 10 { v.push(0); }
//     let mut drain: Drain<'_, u32> = v.drain();
//     for i in 0.. 10 {
//         if i%2 == 0 {
//             _ = drain.next();
//         } else {
//             _ = drain.next_back();
//         }
//     }
//     drop(drain);

//     let mut v: CustomVec<ZST> = CustomVec::new();
//     for _ in 0.. 10 { v.push(ZST {}); }
//     let mut iter: IntoIter<ZST> = v.into_iter();
//     for i in 0.. 10 {
//         if i%2 == 0 {
//             _ = iter.next();
//         } else {
//             _ = iter.next_back();
//         }
//     }
//     drop(iter);

//     let mut v: CustomVec<ZST> = CustomVec::new();
//     for _ in 0.. 10 { v.push(ZST {}); }
//     let mut drain: Drain<'_, ZST> = v.drain();
//     for i in 0.. 10 {
//         if i%2 == 0 {
//             _= drain.next();
//         } else {
//             _ = drain.next_back();
//         }
//     }
//     drop(drain);
// }


impl<T> Drop for CustomVec<T> {
    fn drop(&mut self) {
        // verifier::printf!("Before Drop While Loop");
        // let mut i = 0;
        while let Some(_) = self.pop() {
            // verifier::printf!("In While Loop");
            // // let size = mem::size_of::<T>();
            // // if size == usize::MAX { verifier::vassert!(false); }
            
            // if i == 19 {
            //     verifier::vassert!(false);
            // }
            // i += 1;
        }
        // panic!();

        // verifier::vassert!(false);
        // verifier::printf!("After Drop While Loop");

        // unsafe {
        //     if mem::size_of::<T>() == 0 {
        //         for _ in 0.. self.len { _ = self.pop(); }
        //     } else {
        //         let slice: &mut [T] = core::slice::from_raw_parts_mut(self.ptr(), self.len);
        //         let copy_ptr: *mut T = alloc(Layout::array::<T>(self.len).unwrap()) as *mut T;
        //         copy_ptr.copy_from_nonoverlapping(slice.as_ptr(), self.len);
    
        //         let copy_slice: &mut [T] = core::slice::from_raw_parts_mut(copy_ptr, self.len);
    
        //         _ = Box::from_raw(copy_slice as *mut [T]);
        //     }
        // }
        // deallocation is handled by RawVec
    }
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_zst() {
    static mut DROP_COUNT: usize = 0;
    #[derive(PartialEq)]
    struct ZST { }
    impl Drop for ZST { fn drop(&mut self) { unsafe { DROP_COUNT += 1; } } }
    
    // new, push, pop
    let mut v: CustomVec<ZST> = CustomVec::new();
    verifier::vassert!(v.len == 0 && v.buf.cap == usize::MAX);
    verifier::vassert!(v.pop() == None);

    v.push(ZST {});
    v.push(ZST {});
    verifier::vassert!(v.len == 2);

    verifier::vassert!(v.pop() == Some(ZST {})); // causes extra drop from Some(ZST)
    verifier::vassert!(v.len == 1);
    verifier::vassert!(unsafe { DROP_COUNT == 2 });

    for _ in 0.. 5 { v.push(ZST {}); }
    verifier::vassert!(v.pop() == Some(ZST {})); // causes extra drop from Some(ZST)

    drop(v);
    verifier::vassert!(unsafe { DROP_COUNT == 9 }); // 7 push + 2 in sassert

    // deref and deref mut
    let mut v: CustomVec<ZST> = CustomVec::new();
    for _ in 0.. 5 { v.push(ZST {}); }
    _ = v.pop();
    let slice: &[ZST] = &*v;
    verifier::vassert!(slice.len() == 4);
    verifier::vassert!(slice[0] == ZST {});
    drop(slice);

    let slice: &mut [ZST] = &mut *v;
    slice[0] = ZST {};
    verifier::vassert!(slice.len() == 4);
    verifier::vassert!(slice[0] == ZST {});
    slice[slice.len()-1] = ZST {};
    verifier::vassert!(v.pop() == Some(ZST {}));
    drop(v);

    // test_drain_front();
    // test_drain_back();
    // test_drain_size();
    // test_drain_drop();

    // insert and remove
    let mut v: CustomVec<ZST> = CustomVec::new();
    for _ in 0.. 4 { v.push(ZST {}); }
    v.insert(2, ZST {});
    let slice: &[ZST] = &*v;
    verifier::vassert!(v.len == 5);
    verifier::vassert!(slice.len() == 5);
    verifier::vassert!(slice[2] == ZST {});

    verifier::vassert!(v.remove(1) == ZST {});
    verifier::vassert!(v.len == 4);
    let slice: &[ZST] = &*v;
    verifier::vassert!(slice.len() == 4);
    verifier::vassert!(v.remove(3) == ZST {});
    verifier::assume!(v.len == 3);

    // into_iter: front, back, size, drop
    let mut v: CustomVec<ZST> = CustomVec::new();
    for _ in 0..5 { v.push(ZST {}); }
    let mut iter: IntoIter<ZST> = v.into_iter();
    for _ in 0..5 {
        verifier::vassert!(iter.next() == Some(ZST {}));
    }

    let mut v: CustomVec<ZST> = CustomVec::new();
    for _ in 0.. 5 { v.push(ZST {}); }
    let mut iter: IntoIter<ZST> = v.into_iter();
    for _ in 0.. 5 {
        verifier::vassert!(iter.next_back() == Some(ZST {}));
    }

    let mut v: CustomVec<ZST> = CustomVec::new();
    for _ in 0.. 5 { v.push(ZST {}); }
    let mut iter: IntoIter<ZST> = v.into_iter();
    for i in 0.. 5 {
        if i%2 == 0 { _ = iter.next(); }
        else { iter.next_back(); }
        let size: usize = (5-i-1).try_into().unwrap();
        verifier::vassert!(iter.size_hint().0 == size);
    }

    unsafe { DROP_COUNT = 0; }
    let mut v: CustomVec<ZST> = CustomVec::new();
    v.push(ZST {});
    for _ in 0.. 5 { v.push(ZST {}); }
    v.push(ZST {});
    let mut iter: IntoIter<ZST> = v.into_iter();
    _ = iter.next();
    _ = iter.next();
    _ = iter.next_back();
    _ = iter.next_back();
    verifier::vassert!(unsafe { DROP_COUNT == 4 });
    drop(iter);
    verifier::vassert!(unsafe { DROP_COUNT == 7 });
}



// #[no_mangle]
// #[cfg_attr(kani, kani::proof)]
// fn test_into_iter_drop() {
//     let mut iter: IntoIter<DropTest> = v.into_iter();
//     iter.next();
//     iter.next();
//     iter.next_back();
//     iter.next_back();

//     verifier::vassert!(unsafe { DROP_COUNT == 4 });

//     drop(iter);
//     verifier::vassert!(unsafe { DROP_COUNT == 6 });
// }

fn custom_vec_valid_after_init<T>(vec: &CustomVec<T>) -> bool {
    vec.len == 0 &&
    vec.buf.cap == 0 && {
        if mem::size_of::<T>() != 0 { vec.ptr() == NonNull::<T>::dangling().as_ptr() } 
        else { true }
    }
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
        let address: usize;
        if i%2 == 0 {
            _ = iter.next();
            address = unsafe { ptr::addr_of!(*iter.iter.start) as usize };
            if (address % alignment_size) != 0 {
                unaligned_start = true;
            }
        } else {
            _ = iter.next_back();
            address = unsafe { ptr::addr_of!(*iter.iter.end) as usize };
            if (address % alignment_size) != 0 {
                unaligned_end = true;
            }
        }
    }

    verifier::vassert!(unaligned_start);
    verifier::vassert!(unaligned_end);
}


pub struct Drain<'a, T: 'a> {
    vec: PhantomData<&'a mut CustomVec<T>>,
    iter: RawValIter<T>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> { self.iter.next_back() }
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
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let pointer: *const T;
                if mem::size_of::<T>() == 0 {
                    pointer = NonNull::<T>::dangling().as_ptr();
                    self.start = (self.start as usize + 1) as *const _;
                } else {
                    pointer = self.start;
                    self.start = self.start.offset(1);
                }

                let alignment_size: usize = mem::align_of::<T>();
                verifier::vassert!((pointer as usize) % alignment_size == 0);

                Some(ptr::read(pointer))
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
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let pointer: *const T = if mem::size_of::<T>() == 0 {
                    self.end = (self.end as usize - 1) as *const _;
                    NonNull::<T>::dangling().as_ptr()
                } else {
                    self.end = self.end.offset(-1);
                    self.end
                };

                let alignment_size: usize = mem::align_of::<T>();
                verifier::vassert!((pointer as usize) % alignment_size == 0);

                Some(ptr::read(pointer))
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
    type Item = T;
    fn next(&mut self) -> Option<T> { self.iter.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> { self.iter.next_back() }
}

// impl<T> Drop for IntoIter<T> {
//     fn drop(&mut self) {
//         // for _ in &mut *self {}
//         unsafe {
//             for mut _i in 0..self.len {
//                 if self.iter.start >= self.iter.end { 
//                     _i = self.len;
//                     continue;
//                 }
                
//                 ptr::drop_in_place(self.iter.start as *mut T);
//                 self.iter.start = self.iter.start.offset(1);
//             }
//         }
//     }
// }

impl<T> IntoIterator for CustomVec<T> {
    type Item = T;
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

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // for _ in &mut *self {}
        unsafe {
            for mut _i in 0..self.len {
                if self.iter.start >= self.iter.end { 
                    _i = self.len;
                    continue;
                }
                if mem::size_of::<T>() == 0 {
                    ptr::drop_in_place(NonNull::<T>::dangling().as_ptr());
                    self.iter.start = (self.iter.start as usize + 1) as *const _;
                    
                } else {
                    ptr::drop_in_place(self.iter.start as *mut T);
                    self.iter.start = self.iter.start.offset(1);
                }
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

// impl<T> Drop for CustomVec<T> {
//     fn drop(&mut self) {
//         unsafe {
//             let slice: &mut [T] = core::slice::from_raw_parts_mut(self.ptr(), self.len);

//             let copy_ptr: *mut T = alloc(Layout::array::<T>(self.len).unwrap()) as *mut T;
//             copy_ptr.copy_from_nonoverlapping(slice.as_ptr(), self.len);
//             let copy_slice: &mut [T] = core::slice::from_raw_parts_mut(copy_ptr, self.len);
//             _ = Box::from_raw(copy_slice as *mut [T]);
//         }
//         // deallocation is handled by RawVec
//     }
// }

unsafe impl<T: Send> Send for CustomVec<T> {}
unsafe impl<T: Sync> Sync for CustomVec<T> {}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

