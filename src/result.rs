use crate::sys::result::SysResult;

pub type Result<T> = core::result::Result<T, Error>;

macro_rules! error_def{
    {$(#![$outer_meta:meta])* $($(#[$meta:meta])* #define $name:ident $val:pat)* } => {
        paste::paste!{
            #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
            $(#[$outer_meta])*
            pub enum Error{
                Unknown(SysResult),
                $($(#[$meta])* [<$name:camel>]),*
            }

            impl Error{
                pub const fn from_code(code: SysResult) -> Result<()>{
                    match code{
                        0..=<SysResult>::MAX => Ok(()),
                        $($val => Err(Self::[<$name:camel>]),)*
                        x => Err(Self::Unknown(x))
                    }
                }
            }
        }

    }
}
with_builtin_macros::with_builtin! {
    let $file = include_from_root!("include/errors.h") in {
        error_def!{$file}
    }
}
