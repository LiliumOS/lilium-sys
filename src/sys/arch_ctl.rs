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

    #[cfg(target_arch = "clever")]
    pub mod clever;
    #[cfg(target_arch = "clever")]
    pub use clever::ArchConfigArchOption;

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "clever")))]
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
    /// Sets the current arch config. The ability set any particular config depends on the Arch, kernel support, and the particular CPU.
    ///
    pub fn SetArchConfig(config_options: *const KCSlice<ArchConfigOption>) -> SysResult;
    /// Retrieves the current configuration the CPU is presently providing to the thread.
    /// Note that, unless you configure the thread via [`SetArchConfig`], it is not guaranteed throughout the lifetime of the thread that any feature will remain available,
    ///  however it will always be valid to call [`SetArchConfig`] with the same array as is returned by
    ///
    /// Each [`ArchConfigOption`] uses bit `0` to configure whether it is mandatory or optional - if it is set, unsupported options are ignored and do not cause an error.
    /// If bit `0` is set for any type that is recognized, the kernel will clear it when writing to the option.
    /// Bit 16 does not have any effect for [`GetProvidedArchConfig`].
    pub fn GetProvidedArchConfig(config_options: *mut KSlice<ArchConfigOption>) -> SysResult;
    /// This returns the active configuration for the thread. If set by [`SetArchConfig`], the value returned matches the value last set there.
    /// The default config depends on the Kernel Version and CPU. However, it is guaranteed that the thread can rely on the default configuration throughout its entire lifetime,
    ///  unless it changes to a different feature set.
    ///
    /// Each [`ArchConfigOption`] uses bit `0` to configure whether it is mandatory or optional - if it is set, unsupported options are ignored and do not cause an error.
    /// If bit `0` is set for any type that is recognized, the kernel will clear it when writing to the option.
    /// Bit 16 does not have any effect for [`GetActiveArchConfig`].
    pub fn GetActiveArchConfig(config_options: *mut KSlice<ArchConfigOption>) -> SysResult;
}
