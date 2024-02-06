#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(kani), feature = "std"))]
pub use sea_std as sea;

#[cfg(all(not(kani), feature = "panic_error"))]
pub use sea_panic_error as sea;

#[cfg(all(not(kani), not(feature = "std"), not(feature = "panic_error")))]
pub use sea_no_std as sea;

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
