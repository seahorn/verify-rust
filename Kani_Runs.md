
Individual tests are run by going into the job directory and running 
`cargo kani --harness <test_name>`

Interesting Notes:
1. Kani will try and unroll as many time as possible even if there is an assumption that bounds the loop. An example of this is the below code:
```Rust
let x : usize = verifier::any!();
verifier::assume!(x < 8);
for i in 0..x {
	// do something
}
```
2. Kani uses the default `assert!` whereas seahorn does not. This means that for classes that utilized the default `assert!`, seahorn expects the result to be `is_error`, whereas kani will catch the specific error from the `assert!(...)`

`add`: 
- Success

`borsh`:
- This job currently fails seahorn due to the `bcmp()` function not working when testing string compares. When testing with Kani, it gets hung up trying to unwind loops.

`borsh-enums`:
- `test_enums()`: Kani returns success

`borsh-options`:
- `test_option()`: 
	- Seahorn is able to return `unsat` for this job without bounding loops. However, Kani gets stuck in some loop unrolling in the container code for `unwrap()` function. When a bound of 10 is set for loop unrolling, Kani produces success for some tests. However, before exiting the job, it tries to unroll recursion and gets stuck there since the number of iterations cannot be bounded:
```
Unwinding recursion std::ptr::drop_in_place::<std::io::Error> iteration 104
Unwinding recursion std::ptr::drop_in_place::<std::io::error::repr_bitpacked::Repr> iteration 104
Unwinding recursion <std::io::error::repr_bitpacked::Repr as std::ops::Drop>::drop iteration 104
aborting path on assume(false) at file /github/home/.rustup/toolchains/nightly-2023-09-06-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/hint.rs line 105 column 9 function std::hint::unreachable_unchecked thread 0
```

`borsh-primitives`:
- `test_primitives()`:
	- Kani returns success

`borsh-structs`:
- `test_structs()`:
	- Kani returns success
- `test_fields()`:
	- Kani returns success

`copy_stack_buffer`:
- Kani returns success

`custom-print`:
- Kani returns success

`custom-print-no-std`:
- Seahorn returns unsat for this job. `cargo build` also builds the static library successful, but I could not port the code successfully into its own `main.rs` environment so that the code can run. Kani ran into compiler errors where macro names are ambiguous:
```
error[E0659]: `print` is ambiguous
 --> src/rust-jobs/custom-print-no-std/lib.rs:9:5
  |
9 |     print!("test");
  |     ^^^^^ ambiguous name
  |
  = note: ambiguous because of a conflict between a macro-expanded name and a less macro-expanded name from outer scope during import or macro resolution
  = note: `print` could refer to a macro from prelude
note: `print` could also refer to the macro defined here
```

`custom-print-writer`:
- Kani returns success

`custom-vec`:
- `test_new()`:
	- Kani returns success
- `test_realloc()`:
	- Kani returns success
- `test_grow()`:
	- Seahorn returns unsat. However, Kani returns with some errors (below). These errors are valid errors.
	- First error: since there is no message, this particular error is hard to debug. I am unsure what the error is.
	- Second error: the custom vector provided does not have a way to bulk initialize memory. Hence, some work around was used (namely manually setting the size of the memory block even though that is not the size of memory allocated). This causes the error in `rust_dealloc` since memory size does not match its layout.
	- Third error: I presume this is again called by the mismatch between initialized memory and the size that it tries to dealloc.
```
SUMMARY:
 ** 3 of 169 failed (1 unreachable)
Failed Checks: This is a placeholder message; Kani doesn't support message formatted at runtime
 File: "/github/home/.rustup/toolchains/nightly-2023-09-06-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs", line 1652, in std::result::unwrap_failed
Failed Checks: rust_dealloc must be called on an object whose allocated size matches its layout
 File: "/home/liangubuntu2/.kani/kani-0.36.0/library/kani/kani_lib.c", line 86, in __rust_dealloc
Failed Checks: memcpy source region readable
 File: "/home/liangubuntu2/.kani/kani-0.36.0/library/kani/kani_lib.c", line 115, in __rust_realloc
```
- `test_pop()`:
	- Kani returns a similar error as `test_grow()`
- `test_push()`:
	- Kani returns a similar error as `test_grow()`
- `test_drop()`:
	- Kani only returns an error on mismatch in size when trying to dealloc.
- `test_deref()`:
	- Kani tries to loop unwind during `pop()`.  A manual upper bound of the number of pops (10) is set. However, Kani complains about an unwinding assertion (which can be disabled using the `--unwinding-assertions` flag). Again, it also errors on the mismatch in size.
- `test_deref_mut()`:
	- Kani returns success
- `test_insert()`:
	- Kani again gets stuck trying to unwind the for loop that pushes elements. By limiting the number of elements pushed (effectively setting a bound on the for loop unwind), Kani returns only an error on mismatch in size in dealloc.
- `test_remove()`:
	- Kani again gets stuck trying to unwind loops. Once loop is bounded, the only error Kani returns is the size mismatch in dealloc. Another thing to note is Kani caught an index out of bounds error where as Seahorn did not (code below). This occurs at `index == n`. However, this case is handled by `remove()`, with the second code below, so it should have gone into an `panic` state. Kani gave an error instead.
```Rust
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

pub fn remove(&mut self, index: usize) -> T {
	assert!(index < self.len, "index out of bounds");
	...
}
```
- `test_into_iter()`:
	- Kani returns success
- `test_into_iter_size()`:
	- Kani returns success
- `test_into_iter_drop()`:
	- Kani returns success

`custom-vec-drain`:
- `test_into_iter_front()`:
	- Kani returns success
- `test_into_iter_back()`:
	- Kani returns success
- `test_into_iter_size()`:
	- Kani returns success
- `test_into_iter_drop()`:
	- Kani returns success
- `test_drain_front()`:
	- Kani returns success
- `test_drain_back()`:
	- Kani returns success
- `test_drain_size()`:
	- Kani requires loop bounds for this test. Also, Kani returns an error saying alloc must be on size greater than 0; however, I am unsure what causes this error.
```
SUMMARY:
 ** 1 of 380 failed (5 unreachable)
Failed Checks: __rust_alloc must be called with a size greater than 0
 File: "/home/liangubuntu2/.kani/kani-0.36.0/library/kani/kani_lib.c", line 44, in __rust_alloc
```
- `test_drain_drop()`:
	- Kani returns success

`custom-vec-final`:
- `test_zst()`:
	- Kani returns success
- `test_alignment_bug()`:
	- Kani returns success

`custom-vec-loop-unroll`:
- `test_push()`:
	- Kani requires an upper bound for the loop unwind. Without it, it will unwind for a long time even though the assumption will set a limit on the loop. When the attribute `kani::unwind(11)` is attached, it throws an unwinding failure.

`custom-vec-raw_vec`:
- Again, this test job has issues with manually setting the size of a supposed memory region that it does not have access to, similar to `custom-vec` job.

`custom-vec-zst-alignment-issue`:
- `test_alignment_bug()`:
	- Kani returns success

`empty-print`:
- Kani returns success

`enum-as-param`:
- Kani returns success

`mut_ref_on_stack`:
- Kani returns success

`option`:
- Kani returns success

`option-and-then`:
- Kani returns success

`option-is-some-and`:
- Kani has compile error. It seems that Kani does not support "Executing code compiled with platform features that the current platform does not support"
```
   Compiling option-is-some-and-lib v0.1.0 (/home/liangubuntu2/kani/src/rust-jobs/option-is-some-and)
error[E0635]: unknown feature `is_some_with`
 --> src/rust-jobs/option-is-some-and/lib.rs:1:12
  |
1 | #![feature(is_some_with)]
  |            ^^^^^^^^^^^^

error[E0631]: type mismatch in closure arguments
  --> src/rust-jobs/option-is-some-and/lib.rs:18:30
   |
18 |     let result: bool = value.is_some_and(|num: &i32| *num > 0);
   |                              ^^^^^^^^^^^ ----------- found signature defined here
   |                              |
   |                              expected due to this
   |
   = note: expected closure signature `fn(i32) -> _`
              found closure signature `for<'a> fn(&'a i32) -> _`
note: required by a bound in `std::option::Option::<T>::is_some_and`
  --> /github/home/.rustup/toolchains/nightly-2023-09-06-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:619:5
help: consider adjusting the signature so it does not borrow its argument
   |
18 -     let result: bool = value.is_some_and(|num: &i32| *num > 0);
18 +     let result: bool = value.is_some_and(|num: i32| *num > 0);
   |

error: aborting due to 2 previous errors

Some errors have detailed explanations: E0631, E0635.
For more information about an error, try `rustc --explain E0631`.
error: could not compile `option-is-some-and-lib` (lib) due to 3 previous errors
error: Failed to execute cargo (exit status: 101). Found 3 compilation errors.
```

`option-or`:
- Kani returns success

`option-unwrap-or`:
- Kani returns success

`option-vec-map`:
- Kani tries to loop unroll, and gets stuck. When a bound is set, Kani returns an error regarding index access out of bounds. I am unsure of where the error is caused by.
```
SUMMARY:
 ** 1 of 713 failed (10 unreachable)
Failed Checks: index out of bounds: the length is less than or equal to the given index
 File: "/github/home/.rustup/toolchains/nightly-2023-09-06-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs", line 267, in <usize as std::slice::SliceIndex<[std::option::Option<u32>]>>::index_mut

VERIFICATION:- FAILED
Verification Time: 117.26897s
```

`panic-test`:
- Kani errors due to explicit panic, but that is the intended behavior.
```
SUMMARY:
 ** 1 of 1 failed
Failed Checks: explicit panic
 File: "/home/liangubuntu2/kani/src/rust-jobs/panic-test/lib.rs", line 23, in test
```

`pointer`:
- Kani returns success

`pointer_same_arg`:
- Kani errors since it does not support code that is "Breaking pointer aliasing rules".

`read_across_f`:
- Kani returns success

`realloc-not-initiallized`:
- This job illustrates the problem in `custom-vec` regarding initialization of memory with `NonNull::dangling` pointer.

`result-and_then`:
- Kani returns success

`result-as_deref`:
- Kani returns error due to addition overflow. However, this should never be the case, since values are restricted through assumptions.

`result-cloned`:
- Kani returns success

`result-copied`:
- Kani returns success

`result-iter_mut`:
- Kani returns error due to multiplication overflow. However, this should never be the case, since values are restricted through assumptions.

`result-transpose`:
- Kani returns error due to addition overflow. However, this should never be the case, since values are restricted through assumptions.

`result-unwrap_or_else`:
- Kani cannot be run on this since the test gets inputs from C code and returns a value.

`ro_shared_references`:
- Job not fully completed due to some issues trying to get it to work for both Seahorn and Kani

`sea-vec`:
- Kani cannot currently be run on this job. Sea-vec requires code from the sea-lib library, and Seahorn requries this to use the `panic_error` feature. However, Kani requires `alloc_error_handler` from std, and thus generates this error:
```
error: the `#[alloc_error_handler]` in std conflicts with allocation error handler in: sea

error[E0152]: duplicate lang item in crate `sea` (which `sea_vec_lib` depends on): `panic_impl`.
  |
  = note: the lang item is first defined in crate `std` (which `std` depends on)
  = note: first definition in `std` loaded from /home/liangubuntu2/.kani/kani-0.36.0/lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd-b69fe0ff706bf9b2.rlib
  = note: second definition in `sea` loaded from /home/liangubuntu2/kani/target/kani/x86_64-unknown-linux-gnu/debug/deps/libsea-9e71633808db5ff4.rlib

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0152`.
error: could not compile `sea-vec-lib` (lib) due to 3 previous errors
error: Failed to execute cargo (exit status: 101). Found 3 compilation errors.
```

`serde`:
- `test()`: 
	- Kani returns success

`smallvec`:
- `test_clear()`:
	- Kani returns success
- `test_extend_from_slice()`:
	- Kani hangs and stalls when it tries to loop unroll
	- Kani also produces this error when waited for long enough: `CBMC failed with status 137 VERIFICATION:- FAILED`
- `test_from_buf()`:
	- Kani returns success
- `test_from_buf_and_len()`:
	- Kani returns success
- `test_from_buf_and_len_unchecked()`:
	- Kani returns success
- `test_from_const()`:
	- Kani returns success
- `test_from_elem()`:
	- Kani returns success
- `test_from_raw_parts()`:
	- Kani says there is an error involving dealloc: `Failed Checks: rust_dealloc must be called on an object whose allocated size matches its layout`
- `test_from_slice()`:
	- Kani returns success
- `test_grow()`:
	- Kani returns success
- `test_insert()`:
	- This job expects panic to occur, not sure what the behavior of Kani is meant to be
	- Kani produces errors such as addition overflow
- `test_insert_from_slice()`:
	- This job expects panic to occur, not sure what the behavior of Kani is meant to be
	- Kani produces the following error: `CBMC failed with status 137 VERIFICATION:- FAILED`
- `test_new()`:
	- Kani returns success
- `test_new_const()`:
	- Kani returns success
- `test_pop()`:
	- Kani returns success
- `test_reserve()`:
	- Kani errors with capacity overflow
- `test_reserve_exact()`:
	- Kani returns success
- `test_set_len()`:
	- Kani returns success
- `test_truncate()`:
	- Kani returns success
- `test_try_reserve()`:
	- Kani returns success
- `test_try_reserve_exact()`:
	- Kani returns success
- `test_with_capacity()`:
	- Kani errors with capacity overflow

`smallvec-allocation`:
- `test_clear()`:
	- Kani returns success
- `test_extend_from_slice()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_from_buf()`:
	- Kani returns success
- `test_from_buf_and_len()`:
	- Kani returns success
- `test_from_buf_and_len_unchecked()`:
	- Kani returns success
- `test_from_const()`:
	- Kani returns success
- `test_from_elem()`:
	- Kani returns success
- `test_from_raw_parts()`:
	- Rust dealloc requires memory size to match
```
SUMMARY:
 ** 1 of 340 failed (10 unreachable)
Failed Checks: rust_dealloc must be called on an object whose allocated size matches its layout
 File: "/home/liangubuntu2/.kani/kani-0.36.0/library/kani/kani_lib.c", line 86, in __rust_dealloc
```
- `test_from_slice()`:
	- Kani returns success
- `test_grow()`:
	- Kani returns success
- `test_insert()`:
	- If expected panic is removed, Kani returns success
- `test_insert_from_slice()`:
	- This somehow crashes my WSL2 connection
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_new()`:
	- Kani returns success
- `test_new_const()`:
	- Kani returns success
- `test_pop()`:
	- Kani returns success
- `test_reserve()`:
	- Kani returns success
- `test_reserve_exact()`:
	- Kani returns success
- `test_set_len()`:
	- Kani returns success
- `test_truncate()`:
	- Kani returns success
- `test_try_reserve()`:
	- Kani returns success
- `test_try_reserve_exact()`:
	- Kani returns success
- `test_with_capacity()`:
	- Kani returns error: capacity overflow (from panic)

`smallvec-allocation-bound2`:
- `test_append()`:
	- Kani returns success
- `test_drain()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_drain_panic()`:
	- If panic check is removed, kani returns error from assert made in class
	- If panic check is kept in, the following error is seen:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_insert_many():
	- Kani returns success
- `test_insert_many_panc()`:
	- Kani catches assert before it panics:
```
SUMMARY:
 ** 3 of 573 failed (22 unreachable)
Failed Checks: attempt to add with overflow
 File: "/home/liangubuntu2/kani/src/rust-jobs/smallvec-allocation-bound2/lib.rs", line 2304, in test_insert_many_panic
Failed Checks: attempt to add with overflow
 File: "/home/liangubuntu2/kani/src/rust-jobs/smallvec-allocation-bound2/lib.rs", line 1108, in SmallVec::<[u32; 2]>::insert_many::<SmallVec<[u32; 2]>>
Failed Checks: assertion failed: index <= old_len
 File: "/home/liangubuntu2/kani/src/rust-jobs/smallvec-allocation-bound2/lib.rs", line 1112, in SmallVec::<[u32; 2]>::insert_many::<SmallVec<[u32; 2]>>	  
```
- `test_resize()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_resize2()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_resize_with()`:
	- Kani returns success
- `test_resize_with2()`:
	- Kani returns success
- `test_shrink_to_fit()`:
	- Kani returns success

`smallvec-allocation-bound4`:
- `test_push()`:
	- Kani returns success
- `test_retain()`:
	- Kani returns success
- `test_retain_mut()`:
	- Kani returns success
- `test_try_grow()`:
	- Error caused by having `assert!` in class code

`smallvec-allocation-bound8`:
- `test_dedup()`:
	- Kani returned success
- `test_dedup_by()`:
	- Kani returned success
- `test_dedup_by_key()`:
	- Kani returned success
- `test_remove()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_swap_remove()`:
	- If panic check is removed, kani returns success
	- If panic check is kept in, the following error is seen:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
`smallvec-bound2`:
- `test_append()`:
	- Kani returned success
- `test_drain()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_drain_panic()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_insert_many()`:
	- Kani returned success
- `test_insert_many_panic()`:
	- Expects to panic, Kani detects index out of bounds first
- `test_resize()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_resize2()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_resize_with()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_resize_with2()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_shrink_to_fit()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```

`smallvec-bound4`:
- `test_push()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_retain()`:
	- Kani returned success
- `test_retain_mut()`:
	- Kani returned success
- `test_try_grow()`:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```

`smallvec-bound8`:
- `test_dedup():`
	- Kani returned success
- `test_dedup_by()`:
	- Kani returned success
- `test_dedup_by_key()`:
	- Kani returned success
- `test_remove()`:
	- Kani returned with the following error:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_remove_swap()`: also returned the same error above
  
`smallvec-drain-error`:
```
SUMMARY:
 ** 1 of 495 failed (17 unreachable)
Failed Checks: attempt to add with overflow
 File: "/home/liangubuntu2/kani/src/rust-jobs/smallvec-drain-error/lib.rs", line 729, in SmallVec::<[u8; 8]>::drain::<std::ops::RangeToInclusive<usize>>

VERIFICATION:- FAILED
Verification Time: 2.2963533s
```

`smallvec-grow-error`:
```
SUMMARY:
 ** 1 of 298 failed (4 unreachable)
Failed Checks: assertion failed: !v.spilled()
 File: "/home/liangubuntu2/kani/src/rust-jobs/smallvec-grow-error/lib.rs", line 1633, in entrypt

VERIFICATION:- FAILED
Verification Time: 4.229166s
```

`smallvec-insert_many-error`:
- Kani gets stuck after converting to SSA

`smallvec-insert_many-fix`:
- Kani gets stuck after converting to SSA

`smallvec-insert-optimization`:
- Kani returned success

`smallvec-memory-leak`:
```
SUMMARY:
 ** 1 of 3537 failed (3536 undetermined)
Failed Checks: try is not currently supported by Kani. Please post your example at https://github.com/model-checking/kani/issues/267
 File: "/github/home/.rustup/toolchains/nightly-2023-09-06-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/panicking.rs", line 490, in std::panicking::r#try::<(), [closure@src/rust-jobs/smallvec-memory-leak/lib.rs:1784:45: 1784:52]>

VERIFICATION:- FAILED
** WARNING: A Rust construct that is not currently supported by Kani was found to be reachable. Check the results for more details.
Verification Time: 2.0432367s
```

`string`:
- Kani returns success

`string-parse`:
- Kani returns success

`string-to_string`:
- Kani returns success

`tinyvec-arrayvec`:
- `test_append()`:
	- Kani returned a runtime error, presumably from intended panic
```
SUMMARY:
 ** 1 of 37 failed (1 unreachable)
Failed Checks: This is a placeholder message; Kani doesn't support message formatted at runtime
 File: "/home/liangubuntu2/.cargo/registry/src/index.crates.io-6f17d22bba15001f/tinyvec-1.6.0/src/arrayvec.rs", line 822, in tinyvec::ArrayVec::<[u32; 8]>::set_len

VERIFICATION:- FAILED
Verification Time: 0.6672357s
```
- `test_clear()`:
	- Kani returned success
- `test_drain()`:
	- Kani again will try to unroll the loop, but will not respect the bounds given through assume. So, an explicit bound is given, and it still takes a very long time to run this test. 
- `test_extend_from_slice()`:
	- Kani returned a runtime error, presumably from intended panic
- `test_fill()`:
	- Kani again will try to unroll the loop, but will not respect the bounds given through assume. So, an explicit bound is given, and Kani returned success
- `test_from_array_empty()`:
	- Kani returned success
- `test_from_array_len()`:
	- Kani returned a runtime error, presumably from intended panic
- `test_insert()`:
	- Kani again will try to unroll the loop, but will not respect the bounds given through assume. So, an explicit bound is given. Kani still returned capacity overflow error and runtime error.
```
SUMMARY:
 ** 2 of 208 failed (15 unreachable)
Failed Checks: ArrayVec::insert> capacity overflow!
 File: "/home/liangubuntu2/.cargo/registry/src/index.crates.io-6f17d22bba15001f/tinyvec-1.6.0/src/arrayvec.rs", line 518, in tinyvec::ArrayVec::<[u32; 8]>::insert
Failed Checks: This is a placeholder message; Kani doesn't support message formatted at runtime
 File: "/home/liangubuntu2/.cargo/registry/src/index.crates.io-6f17d22bba15001f/tinyvec-1.6.0/src/arrayvec.rs", line 541, in tinyvec::ArrayVec::<[u32; 8]>::try_insert

VERIFICATION:- FAILED
Verification Time: 34.26493s
```
- `test_new()`:
	- Kani returned success
- `test_pop()`:
	- Kani returned success
- `test_push()`:
	- Kani returns capacity overflow error, as expected.
- `test_remove()`:
	- Kani again will try to unroll the loop, but will not respect the bounds given through assume. So, an explicit bound is given. Still, loop unwinding takes a very long time, and produced the following error:
```
CBMC failed with status 137
VERIFICATION:- FAILED
```
- `test_resize()`:
	- Kani produced error for capacity overflow
- `test_resize_with()`:
	- Kani produced capacity overflow error
- `test_retain()`:
	- Kani again will try to unroll the loop, but will not respect the bounds given through assume. So, an explicit bound is given. Still, loop unwinding takes a very long time.
- `test_set_len()`:
	- Kani returned a runtime error, presumably from intended panic
- `test_splice()`:
	- Kani again will try to unroll the loop, but will not respect the bounds given through assume. So, an explicit bound is given. Still, loop unwinding takes a very long time.
- `test_splice_panic()`:
	- Kani again will try to unroll the loop, but will not respect the bounds given through assume. So, an explicit bound is given. Still, loop unwinding takes a very long time.
- `test_split_off()`:
	- Kani returned a runtime error, presumably from intended panic
- `test_swap_remove()`:
	- Kani returned a runtime error, presumably from intended panic
- `test_truncate()`: 
	- Kani returned success
- `test_try_append()`:
	- Kani returned success
- `test_try_from_array_len()`:
	- Kani returned success
- `test_try_insert()`:
	- Kani returned a runtime error, presumably from intended panic
- `test_try_push()`:
	- Kani returned success

`tinyvec-capacity-error`:
- Kani enters infinite unrolling of the loop (to i16::MAX)

`tinyvec-remove-error`:
- Kani errors with the following:
```
SUMMARY:
 ** 3 of 189 failed (1 unreachable)
Failed Checks: assertion failed: false
 File: "/home/liangubuntu2/kani/src/rust-jobs/tinyvec-remove-error/lib.rs", line 32, in entrypt
Failed Checks: This is a placeholder message; Kani doesn't support message formatted at runtime
 File: "/github/home/.rustup/toolchains/nightly-2023-09-06-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs", line 52, in core::slice::index::slice_start_index_len_fail_rt
Failed Checks: attempt to subtract with overflow
 File: "/home/liangubuntu2/.cargo/registry/src/index.crates.io-6f17d22bba15001f/tinyvec-0.2.0/src/arrayvec.rs", line 375, in tinyvec::ArrayVec::<[u32; 8]>::remove

VERIFICATION:- FAILED
Verification Time: 12.107946s
```

`vec`:
- Kani returns success

`vec-filter`:
- Kani fails due to addition overflow. Tried adding `verifier::assume!(x+y+z < i32::MAX);` but it produced more errors.

`vec-sort-reverse`:
- Kani returns success

