use super::result::SysResult;

macro_rules! sysno_def{
    {[#[cfg(feature = $feat_name:literal)] subsys $subsys_name:ident] $(#![$outer_meta:meta])* $($(#[$meta:meta])* #define $name:ident $val:expr_2021)* } => {
        $(#[$outer_meta])*
        #[cfg(any(feature = $feat_name, doc))]
        #[allow(non_upper_case_globals)]
        pub mod $subsys_name{
            $($(#[$meta])* pub const $name: usize = $val;)*
        }

    }
}
with_builtin_macros::with_builtin! {
    let $file = include_from_root!("include/syscalls/base.h") in {
        sysno_def!{[#[cfg(feature = "base")] subsys base] $file}
    }
}

unsafe extern "C" {
    unsafe fn syscall(sysno: usize, ...) -> SysResult;
}
