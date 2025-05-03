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


#[inline(always)]
pub fn any<T: Arbitrary>() -> T {
    T::any()
}
