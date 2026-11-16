#![cfg_attr(not(kani), no_std)]

// Raw-pointer properties that need the verifier to reason about *which*
// memory a pointer refers to, not just the value behind it:
//   1. swap through two pointers that may or may not alias
//   2. pointer arithmetic into a stack buffer at a symbolic index
//   3. a linked list whose nodes are linked with raw pointers
// See raw-pointers-bug for a variant with a wrong assertion (expected sat).

use verifier;

// Plain C-style swap; correct even when a == b.
#[no_mangle]
#[inline(never)]
unsafe fn swap_raw(a: *mut i32, b: *mut i32) {
    let tmp = *a;
    *a = *b;
    *b = tmp;
}

struct Node {
    val: i32,
    next: *mut Node,
}

// Walks at most MAX_NODES nodes. The constant trip count lets LLVM fully
// unroll the loop; an unbounded `while !p.is_null()` is cut to one
// iteration by SeaHorn, which makes the assertion after it unreachable.
const MAX_NODES: usize = 4;

#[inline(never)]
unsafe fn sum_list(mut p: *const Node) -> i32 {
    let mut s: i32 = 0;
    for _ in 0..MAX_NODES {
        if p.is_null() {
            break;
        }
        s = s.wrapping_add((*p).val);
        p = (*p).next;
    }
    s
}

#[no_mangle]
#[cfg_attr(kani, kani::proof)]
pub extern "C" fn entrypt() {
    // 1. swap with possible aliasing
    let x0: i32 = verifier::any!();
    let y0: i32 = verifier::any!();
    let mut x = x0;
    let mut y = y0;
    let alias: bool = verifier::any!();
    let px: *mut i32 = &mut x;
    let py: *mut i32 = if alias { px } else { &mut y };
    unsafe { swap_raw(px, py) };
    if alias {
        verifier::vassert!(x == x0);
    } else {
        verifier::vassert!(x == y0 && y == x0);
    }

    // 2. write through base.add(i); every other slot must be untouched
    let mut buf: [i32; 4] = [verifier::any!(), verifier::any!(), verifier::any!(), verifier::any!()];
    let before = buf;
    // Two distinct in-bounds indices, computed rather than assumed:
    // i ranges over 0..4 and j over the three other slots.
    let n: usize = verifier::any!();
    let m: usize = verifier::any!();
    let i: usize = n % 4;
    let j: usize = (i + 1 + m % 3) % 4;
    let v: i32 = verifier::any!();
    unsafe { *buf.as_mut_ptr().add(i) = v };
    verifier::vassert!(buf[i] == v);
    verifier::vassert!(buf[j] == before[j]);

    // 3. three-node list linked with raw pointers, then mutated through a link
    let a: i32 = verifier::any!();
    let b: i32 = verifier::any!();
    let c: i32 = verifier::any!();
    let mut n3 = Node { val: c, next: core::ptr::null_mut() };
    let mut n2 = Node { val: b, next: &mut n3 };
    let n1 = Node { val: a, next: &mut n2 };
    unsafe { (*n2.next).val = 0 }; // n2.next is n3
    let s = unsafe { sum_list(&n1) };
    verifier::vassert!(s == a.wrapping_add(b));
}
