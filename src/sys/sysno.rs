use super::result::SysResult;

macro_rules! sysno_def{
    {[subsys $subsys_name:ident] $(#![$outer_meta:meta])* $($(#[$meta:meta])* #define $name:ident $val:expr_2021)* } => {
        $(#[$outer_meta])*
        #[allow(non_upper_case_globals)]
        pub mod $subsys_name{
            $($(#[$meta])* pub const $name: usize = $val;)*
        }

    }
}
with_builtin_macros::with_builtin! {
    let $file = include_from_root!("include/syscalls/base.h") in {
        sysno_def!{[subsys base] $file}
    }
}

unsafe extern "C" {
    unsafe fn syscall(sysno: usize, ...) -> SysResult;
}
