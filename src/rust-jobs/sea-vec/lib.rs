#![cfg_attr(not(kani), no_std)]
use verifier;
use sea::{ SeaVec, sea_vec };


#[no_mangle]
pub extern "C" fn entrypt() {
    test_macros();
    test_push_pop();
    test_indexing();
    test_deref_deref_mut();
    test_insert_remove();

    test_drop();
    test_iter();
    test_drain();
    test_zst();

    // verifier::vassert!(false);
}


#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_drop() {
    static mut DROP_COUNT: usize = 0;
    struct DropTest { _x: usize }
    impl Drop for DropTest {
        fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
    }

    let mut v: SeaVec<DropTest> = SeaVec::new(8);
    for i in 0..5 { v.push( DropTest { _x: i }); }
    _ = v.pop();
    _ = v.pop();
    verifier::vassert!(unsafe { DROP_COUNT == 2 });
    drop(v);
    verifier::vassert!(unsafe{ DROP_COUNT == 5 });
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_macros() {
    let mut v: SeaVec<i32> = sea_vec![1, 2, 3, 4];
    verifier::vassert!(v.len() == 4);
    verifier::vassert!(v.cap() == 8);
    for i in 0..4 { verifier::vassert!(v.pop() == Some(4-i)); }
    verifier::vassert!(v.pop() == None && v.len() == 0);

    let mut v: SeaVec<i32> = sea_vec!([1, 2, 3]; 12);
    verifier::vassert!(v.len() == 3);
    verifier::vassert!(v.cap() == 12);
    for i in 0..3 { verifier::vassert!(v.pop() == Some(3-i)); }
    verifier::vassert!(v.pop() == None && v.len() == 0);

    let mut v: SeaVec<i32> = sea_vec!(0; 3; 8);
    verifier::vassert!(v.len() == 3);
    verifier::vassert!(v.cap() == 8);
    for _ in 0..3 { verifier::vassert!(v.pop() == Some(0)); }
    verifier::vassert!(v.pop() == None && v.len() == 0);

    let mut v = sea_vec!(0; 4);
    verifier::vassert!(v.len() == 4);
    verifier::vassert!(v.cap() == 8);
    for _ in 0..4 { verifier::vassert!(v.pop() == Some(0)); }
    verifier::vassert!(v.pop() == None && v.len() == 0);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_push_pop() {
    let mut v: SeaVec<usize> = SeaVec::new(10);
    for i in 0..10 {
        verifier::vassert!(v.len() == i);
        v.push(i);
    }
    verifier::vassert!(v.cap() == 10);

    for i in 0..10 {
        verifier::vassert!(v.len() == 10-i);
        verifier::vassert!(v.pop() == Some(9-i));
    }
    verifier::vassert!(v.pop() == None);
    verifier::vassert!(v.cap() == 10 && v.len() == 0);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_indexing() {
    let mut v: SeaVec<usize> = sea_vec![0, 1, 2, 3, 4, 5];
    for i in 0..6 {
        verifier::vassert!(v[i] == i);
        v[i] = i*10;
        verifier::vassert!(v[i] == i*10);
    }
    verifier::vassert!(v.len() == 6 && v.cap() == 12);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_deref_deref_mut() {
    let mut v: SeaVec<usize> = sea_vec![0, 1, 2, 3];

    let mut i: usize = 0;
    for elem in &*v {
        verifier::vassert!(*elem == i);
        i += 1;
    }

    let slice: &mut [usize] = &mut *v;
    for i in 0..4 { slice[i] *= 10; }
    
    let mut i: usize = 0;
    for elem in &*v {
        verifier::vassert!(*elem == 10*i);
        i += 1;
    }

    verifier::vassert!(v.len() == 4 && v.cap() == 8);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_insert_remove() {
    let mut v: SeaVec<usize> = sea_vec![0, 1, 2, 3, 4];
    v.insert(5, 5);
    verifier::vassert!(v.len() == 6);
    verifier::vassert!(v.pop() == Some(5));
    
    verifier::vassert!(v.remove(2) == 2);
    verifier::vassert!(v.len() == 4);

    v.insert(2, 20);
    verifier::vassert!(v.remove(2) == 20);
    v.insert(0, 0);
    verifier::vassert!(v.remove(4) == 4);
    verifier::vassert!(v.len() == 4);
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_iter() {
    let v: SeaVec<usize> = sea_vec![0, 1, 2, 3, 4, 5];

    let mut iter: sea::IntoIter<usize> = v.into_iter();
    for i in 0..6 {
        if i%2 == 0 {
            verifier::vassert!(iter.next() == Some(i/2));
        } else {
            verifier::vassert!(iter.next_back() == Some(5-i/2));
        }
        verifier::vassert!(iter.size_hint() == (5-i, Some(5-i)));
    }


    static mut DROP_COUNT: usize = 0;
    struct DropTest { _x: usize }
    impl Drop for DropTest {
        fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
    }

    let mut v: SeaVec<DropTest> = SeaVec::new(6);
    for i in 0..6 { v.push(DropTest { _x: i }); }
    let mut iter = v.into_iter();
    _ = iter.next();
    _ = iter.next();
    _ = iter.next();
    verifier::vassert!(unsafe { DROP_COUNT == 3 });

    drop(iter);
    verifier::vassert!(unsafe { DROP_COUNT == 6 });
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_drain() {
    let mut v: SeaVec<usize> = sea_vec![0, 1, 2, 3, 4, 5];
    let mut drain: sea::Drain<'_, usize> = v.drain(0..6);
    for i in 0..6 {
        if i%2 == 0 { verifier::vassert!(drain.next() == Some(i/2)); }
        else { verifier::vassert!(drain.next_back() == Some(5-i/2)); }
        verifier::vassert!(drain.size_hint() == (5-i, Some(5-i)));
    }
    drop(drain);
    verifier::vassert!(v.len() == 0 && v.cap() == 12 && v.pop() == None);

    static mut DROP_COUNT: usize = 0;
    struct DropTest { x: usize }
    impl Drop for DropTest {
        fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
    }
    let cap = 8;
    let mut v: SeaVec<DropTest> = SeaVec::new(cap);
    for i in 0..4 { v.push(DropTest { x: i }); }

    let mut drain: sea::Drain<'_, DropTest> = v.drain(1..3);
    verifier::vassert!(drain.next().unwrap().x == 1);
    verifier::vassert!(drain.size_hint() == (1, Some(1)));
    verifier::vassert!(unsafe { DROP_COUNT == 1 });
    drop(drain);
    verifier::vassert!(unsafe { DROP_COUNT == 2 });

    verifier::vassert!(v[0].x == 0 && v[1].x == 3 && v.len() == 2);
    
    // elements must be popped before drop, or else SeaHorn will always give unsat
    for _ in 0..4 { v.pop(); }
    drop(v);

    verifier::vassert!(unsafe { DROP_COUNT == 4 });
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
fn test_zst() {
    static mut DROP_COUNT: usize = 0;
    #[derive(PartialEq)]
    struct ZST {}
    impl Drop for ZST {
        fn drop(&mut self) { unsafe { DROP_COUNT += 1; } }
    }

    let mut v: SeaVec<ZST> = SeaVec::new(5);
    v.push(ZST {});
    v.push(ZST {});
    v.insert(2, ZST {});
    v[0] = ZST {};

    verifier::vassert!(v[0] == v[1] && v[1] == v[2]);
    verifier::vassert!(v.remove(1) == ZST {}); // checking for equivalence increases drop count by one

    verifier::vassert!(v.len() == 2);
    v.insert(0, ZST {});
    verifier::vassert!(unsafe { DROP_COUNT == 3 });

    let mut iter: sea::IntoIter<ZST> = v.into_iter();
    verifier::vassert!(iter.next() == Some(ZST {})); // checking for equivalence increases drop count by one
    verifier::vassert!(iter.size_hint().0 == 2);
    verifier::vassert!(iter.next_back() == Some(ZST {})); // checking for equivalence increases drop count by one
    verifier::vassert!(iter.next() == Some(ZST {})); // checking for equivalence increases drop count by one
    verifier::vassert!(iter.next() == None);
    verifier::vassert!(iter.size_hint().0 == 0);
    verifier::vassert!(unsafe { DROP_COUNT == 9 });

    let mut v: SeaVec<ZST> = SeaVec::new(5);
    for _ in 0..5 { v.push(ZST {}); }
    let mut drain = v.drain(1..4);
    verifier::vassert!(drain.next() == Some(ZST {})); // checking for equivalence increases drop count by one
    verifier::vassert!(drain.size_hint().0 == 2);
    verifier::vassert!(drain.next_back() == Some(ZST {})); // checking for equivalence increases drop count by one
    verifier::vassert!(drain.next() == Some(ZST {})); // checking for equivalence increases drop count by one
    verifier::vassert!(drain.next() == None);
    verifier::vassert!(drain.size_hint().0 == 0);
    verifier::vassert!(unsafe { DROP_COUNT == 15 });
    drop(drain);
    v.pop();
    v.pop();
    verifier::vassert!(v.pop() == None && v.len() == 0);
    drop(v);
}

