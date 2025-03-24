//! [`sysno`][self] is used to define raw syscalls for Lilium.
//! Submodules are defined for each subsystem.
//! 
//! ## Lilium syscall format
//! 
//! A Lilium syscall number consists of a subsystem and a system function number. 
//! The system function number is defined in the lower 12 bits of the syscall number. The remaining upper bits is the subsystem number.
//! 
//! The `SYS_*` constants in each subsystem module define the **system function number**, not the syscall number.
//! Core subsystems also specify `SUBSYSTEM_BASE` which specifies the syscall region the system functions that belong to the subsystem.
//! The syscall number (for the [`syscall`] function or a for assembly) can be computed by adding `SUBSYSTEM_BASE` to the appropriate `SYS_*` constant.
//! 
//! Non-core subsystems do not define `SUBSYSTEM_BASE` as they do not have fixed subsystem numbers. Instead you need to query the subsystem base with [`crate::sys::info::SysInfoRequestSupportedSubsystem`]
//!  using the ID that corresponds to the subsystem.

use super::result::SysResult;

macro_rules! sysno_def{
    {[#[cfg(feature = $feat_name:literal)] subsys $subsys_name:ident] $(#![$outer_meta:meta])* $($(#[$meta:meta])* #define $name:ident $val:expr_2021)* } => {
        $(#[$outer_meta])*
        #[cfg(any(feature = $feat_name, doc, feature = "raw"))]
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

#[cfg(feature = "libc")]
unsafe extern "C" {
    pub unsafe fn syscall(sysno: usize, ...) -> SysResult;
}
