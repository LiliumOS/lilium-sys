
pub type Result<T> = core::result::Result<T,Error>;

macro_rules! error_def{
    {$(#define $name:ident $val:pat)* } => {
        pub enum Error{
            Unknown(core::ffi::c_long),
            $($name),*
        }

        impl Error{
            pub fn from_code(code: core::ffi::c_long) -> Result<()>{
                match code{
                    0..=<core::ffi::c_long>::MAX => Ok(()),
                    $($val => Err(Self::$name),)*
                    x => Err(Self::Unknown(x))
                }
            }
        }
    }
}
with_builtin_macros::with_builtin!{
    let $file = include_from_root!("include/errors.h") in {
        error_def!{$file}
    }
}