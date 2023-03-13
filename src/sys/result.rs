
pub type SysResult = core::ffi::c_long;

pub mod errors{
    macro_rules! error_def{
        {$(#define $name:ident $val:expr)* } => {
            $(pub const $name: super::SysResult = $val;)*
        }
    }
    with_builtin_macros::with_builtin!{
        let $file = include_from_root!("include/errors.h") in {
            error_def!{$file}
        }
    }
}