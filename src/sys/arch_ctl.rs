use core::mem::MaybeUninit;

use super::{
    kstr::{KCSlice, KSlice},
    option::ExtendedOptionHead,
    result::SysResult,
};

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct ArchConfigUnknownOption {
    pub header: ExtendedOptionHead,
    pub payload: [MaybeUninit<u8>; 32],
}

mod arch {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub mod x86;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub use x86::ArchConfigArchOption;

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    #[repr(C, align(32))]
    #[derive(Copy, Clone)]
    pub union ArchConfigArchOption {
        unknown: super::ArchConfigUnknownOption,
    }
}

pub use arch::*;

#[repr(C, align(32))]
pub union ArchConfigOption {
    /// The Header of the [`ArchConfigOption`].
    ///
    /// The following additional flags bits are defined:
    /// * Bit 16: If set and bit `0` is clear, do not error for a recognized `ty` if unsupported options are set.
    pub head: ExtendedOptionHead,
    pub unknown: ArchConfigUnknownOption,
    pub arch: ArchConfigArchOption,
}

extern "system" {
    pub fn SetArchConfig(config_options: *const KCSlice<ArchConfigOption>) -> SysResult;
    pub fn GetProvidedArchConfig(config_options: *mut KSlice<ArchConfigOption>) -> SysResult;
    pub fn GetActiveArchConfig(config_options: *mut KSlice<ArchConfigOption>) -> SysResult;
}
