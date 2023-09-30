/// The Integer Type that is the same size as a machine word.
pub type SysResult = core::ffi::c_long;

mod nonzero {
    use core::num::*;
    pub trait Scalar {
        type NonZeroTy;
    }

    macro_rules! impl_scalar_nonzero{
        ($($base:ty => $nonzero:ty),* $(,)?) => {
            $(
                impl Scalar for $base{
                    type NonZeroTy = $nonzero;
                }
            )*
        };
    }

    impl_scalar_nonzero! {
        i8 => NonZeroI8,
        i16 => NonZeroI16,
        i32 => NonZeroI32,
        i64 => NonZeroI64,
        i128 => NonZeroI128,
        u8 =>   NonZeroU8,
        u16 =>  NonZeroU16,
        u32 =>  NonZeroU32,
        u64 =>  NonZeroU64,
        u128 => NonZeroU128,
        isize => NonZeroIsize,
        usize => NonZeroUsize
    }
    pub type NonZeroSysResult = <super::SysResult as Scalar>::NonZeroTy;
}

/// The NonZeroI* type that corresponds to `SysResult`
pub type NonZeroSysResult = nonzero::NonZeroSysResult;

pub mod errors {
    macro_rules! error_def{
        {$($(#[$meta:meta])* #define $name:ident $val:expr)* } => {
            $($(#[$meta])* pub const $name: super::SysResult = $val;)*
        }
    }
    with_builtin_macros::with_builtin! {
        let $file = include_from_root!("include/errors.h") in {
            error_def!{$file}
        }
    }
}
