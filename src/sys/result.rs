/// The signed Integer Type that is the same size as a machine word.
pub type SysResult = isize;
/// The NonZeroI* type that corresponds to `SysResult`
pub type NonZeroSysResult = core::num::NonZeroIsize;

macro_rules! error_def{
    {$(#![$outer_meta:meta])* $($(#[$meta:meta])* #define $name:ident $val:expr)* } => {
        $(#[$outer_meta])*
        pub mod errors{
            $($(#[$meta])* pub const $name: super::SysResult = $val;)*
        }

    }
}
with_builtin_macros::with_builtin! {
    let $file = include_from_root!("include/errors.h") in {
        error_def!{$file}
    }
}

#[macro_export]
macro_rules! sys_try {
    ($e:expr) => {{
        let val: $crate::sys::result::SysResult = $e;

        if val < 0 {
            return val;
        }
        val
    }};
}

#[macro_export]
macro_rules! sys_try_nonzero {
    ($e:expr) => {{
        let val: $crate::sys::result::NonZeroSysResult = $e;

        if val.get() < 0 {
            return val.into();
        }
        val
    }};
}
