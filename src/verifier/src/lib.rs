#![cfg_attr(not(feature = "std"), no_std)]

// Nondet helpers for types that need caller-supplied storage or a derive.
// `any!()` covers everything that implements `Arbitrary`.
pub use sea::{any_slice, any_str, Arbitrary};
#[cfg(feature = "std")]
pub use sea::{any_string, any_vec};

#[macro_export]
macro_rules! any {
    () => {{
        use cfg_if::cfg_if;

        cfg_if!{
            if #[cfg(kani)] {
                kani::any()
            } else {
                sea::any()
            }
        }
    }};
}

#[macro_export]
macro_rules! assume {
    ($cond: expr) => {{
        use cfg_if::cfg_if;

        cfg_if!{
            if #[cfg(kani)] {
                kani::assume($cond)
            } else {
                sea::assume($cond);
            }
        }
    }};
}

#[macro_export]
macro_rules! vassert {
    ($cond: expr) => {{
        use cfg_if::cfg_if;

        cfg_if!{
            if #[cfg(kani)] {
                assert!($cond)
            } else {
                sea::sassert!($cond)
            }
        }
    }};
}

#[macro_export]
macro_rules! error {
    () => {{
        use cfg_if::cfg_if;

        cfg_if!{
            if #[cfg(kani)] {
                assert!(false)
            } else {
                sea::error!()
            }
        }
    }};
}


#[macro_export]
macro_rules! printf {
    ($cond: expr) => {{
        use cfg_if::cfg_if;

        cfg_if!{
            if #[cfg(kani)] {
                // println!($cond)
            } else {
                sea::sea_printf!(true)
            }
        }
    }};
}
