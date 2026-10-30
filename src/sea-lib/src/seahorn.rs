use crate::bindings::*;
use sea_nd_func::generate_impl;

#[no_mangle]
pub fn verifier_error() { unsafe { __VERIFIER_error(); } }

#[no_mangle]
pub fn assume(v: bool) { unsafe { __VERIFIER_assume(v.into()); } }

#[macro_export]
macro_rules! sea_printf {
    ($message:expr $(, $args:expr)*) => {{
        use crate::sea::bindings::sea_printf;
        use core::ffi::c_char;
        unsafe { sea_printf($message.as_ptr() as *const c_char, $($args),*); }
    }}
}

#[macro_export]
macro_rules! sassert {
    ($cond:expr) => {{
        // We need a verifier_assert statement to enable vacuity checking.
        // This is a nop if vacuity is not enabled in bmc run.
        // Note: cond may have side-effects, so we have to evaluate it only once.
        let val = $cond.into();
        unsafe { sea::bindings::__VERIFIER_assert(val); }
        if !val {
            unsafe { sea::bindings::__VERIFIER_error(); }
        }
    }};
}

#[macro_export]
macro_rules! error {
    () => {{
      unsafe { sea::bindings::__VERIFIER_error(); }
    }};
}


pub trait Arbitrary
where
    Self: Sized,
{
    fn any() -> Self;
}

generate_impl!(i8);
generate_impl!(u8);
generate_impl!(i16);
generate_impl!(u16);
generate_impl!(i32);
generate_impl!(u32);
generate_impl!(i64);
generate_impl!(u64);
generate_impl!(bool);
generate_impl!(usize);
generate_impl!(isize);

// The impls below are built from the primitive nondet functions above, so they
// need no extra C stubs. Each one only produces values that are valid for the
// type (e.g. no surrogate chars), because an invalid value would be UB that
// LLVM can use to delete code before SeaHorn sees it.

impl Arbitrary for () {
    #[inline(always)]
    fn any() -> Self {}
}

impl Arbitrary for u128 {
    #[inline(always)]
    fn any() -> Self {
        ((u64::any() as u128) << 64) | (u64::any() as u128)
    }
}

impl Arbitrary for i128 {
    #[inline(always)]
    fn any() -> Self {
        u128::any() as i128
    }
}

impl Arbitrary for char {
    #[inline(always)]
    fn any() -> Self {
        let v = u32::any();
        // valid chars are Unicode scalar values: [0, 0xD7FF] and [0xE000, 0x10FFFF]
        assume(v < 0xD800 || (v > 0xDFFF && v <= 0x10FFFF));
        unsafe { char::from_u32_unchecked(v) }
    }
}

// SeaHorn does not model floating point: float values are havoc'd and float
// comparisons are not supported by the BMC engine. These impls let float
// fields appear in derived types, but don't assert on float values.
impl Arbitrary for f32 {
    #[inline(always)]
    fn any() -> Self {
        f32::from_bits(u32::any())
    }
}

impl Arbitrary for f64 {
    #[inline(always)]
    fn any() -> Self {
        f64::from_bits(u64::any())
    }
}

impl<T: Arbitrary> Arbitrary for Option<T> {
    #[inline(always)]
    fn any() -> Self {
        if bool::any() { Some(T::any()) } else { None }
    }
}

impl<T: Arbitrary, E: Arbitrary> Arbitrary for Result<T, E> {
    #[inline(always)]
    fn any() -> Self {
        if bool::any() { Ok(T::any()) } else { Err(E::any()) }
    }
}

impl<T: Arbitrary, const N: usize> Arbitrary for [T; N] {
    #[inline(always)]
    fn any() -> Self {
        core::array::from_fn(|_| T::any())
    }
}

macro_rules! tuple_impl {
    ($($name:ident)+) => {
        impl<$($name: Arbitrary),+> Arbitrary for ($($name,)+) {
            #[inline(always)]
            fn any() -> Self {
                ($($name::any(),)+)
            }
        }
    };
}

tuple_impl!(A);
tuple_impl!(A B);
tuple_impl!(A B C);
tuple_impl!(A B C D);
tuple_impl!(A B C D E);
tuple_impl!(A B C D E F);

#[inline(always)]
pub fn any<T: Arbitrary>() -> T {
    T::any()
}

/// A nondet value for which `f` holds: `any_with(|x: &u32| *x < 10)`.
/// Same as `any()` followed by `assume(f(&v))`.
#[inline(always)]
pub fn any_with<T: Arbitrary, F: FnOnce(&T) -> bool>(f: F) -> T {
    let v = T::any();
    assume(f(&v));
    v
}

/// `None`, or `Some(f())`, chosen nondeterministically.
#[inline(always)]
pub fn nondet_option<T, F: FnOnce() -> T>(f: F) -> Option<T> {
    if bool::any() { Some(f()) } else { None }
}

// Certora CVLR's names for the same operations, so harnesses written against
// `cvlr::nondet` port without edits.
#[inline(always)]
pub fn nondet<T: Arbitrary>() -> T {
    T::any()
}

#[inline(always)]
pub fn nondet_with<T: Arbitrary, F: FnOnce(&T) -> bool>(f: F) -> T {
    any_with(f)
}

/// Fill `buf` with nondet values and return a prefix of nondet length
/// `0..=N`. Slices are unsized, so the caller supplies the backing storage;
/// `N` is the bound on the slice length.
#[inline(always)]
pub fn any_slice<T: Arbitrary, const N: usize>(buf: &mut [T; N]) -> &mut [T] {
    *buf = <[T; N]>::any();
    let len = usize::any();
    assume(len <= N);
    &mut buf[..len]
}

/// A nondet `&str` of at most `N` bytes, backed by `buf`. Every valid UTF-8
/// string up to that length is possible (not just ASCII).
#[inline(always)]
pub fn any_str<const N: usize>(buf: &mut [u8; N]) -> &str {
    let bytes = any_slice(buf);
    assume(core::str::from_utf8(bytes).is_ok());
    unsafe { core::str::from_utf8_unchecked(bytes) }
}

#[cfg(feature = "std")]
/// A nondet `Vec<T>` of length `0..=N`. Capacity is always `N`.
///
/// Capacity is deliberately not nondet: with a capacity drawn independently of
/// the length, SeaHorn loses the stored elements (they read back as arbitrary
/// values), so states such as len 1 / capacity 1 are not generated.
pub fn any_vec<T: Arbitrary, const N: usize>() -> Vec<T> {
    let len = usize::any();
    assume(len <= N);
    let mut v = Vec::with_capacity(N);
    for _ in 0..N {
        v.push(T::any());
    }
    v.truncate(len);
    v
}

#[cfg(feature = "std")]
/// A nondet `String` of at most `N` bytes. Capacity always equals length.
///
/// Spare capacity is deliberately not modelled: with a nondet capacity,
/// SeaHorn loses the copied bytes (decoded chars no longer add up to `len`),
/// even at `horn-bv2-word-size: 1`. Jobs that reason about contents need word
/// size 1: at the default (8) the symbolic-length copy loses the bytes too.
pub fn any_string<const N: usize>() -> String {
    let mut buf = [0u8; N];
    String::from(any_str(&mut buf))
}

pub use sea_nd_func::Arbitrary;
