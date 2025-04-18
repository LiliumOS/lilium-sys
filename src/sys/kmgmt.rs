//! `kmgmt` is an incredibly low level subsystem.
//!
//! Many of the syscalls within it are high-privilege (typically requiring kernel permissions only available to `SYSTEM`) and can directly access the running kernel.
//! Misuse can result in system instability.
use core::mem::MaybeUninit;

use super::{
    fs::FileHandle,
    handle::HandlePtr,
    kstr::{KCSlice, KStrCPtr},
    option::ExtendedOptionHead,
    result::SysResult,
};

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct KernelCfgUnknownOption {
    pub header: ExtendedOptionHead,
    pub payload: [MaybeUninit<u8>; 32],
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::AnyBitPattern for KernelCfgUnknownOption {}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
pub union KernelCfgOption {
    pub head: ExtendedOptionHead,
    pub unknown: KernelCfgUnknownOption,
}

pub const SUBMODULE_LOADING_OPTION_FLAG_GLOBAL: u32 = 0x00000001;

def_option_type! {
    pub struct SubmoduleLoadingOptionFlags ("876ab526-e3e1-5691-9dd9-ef9ff636dee1") {
        pub flags: u32,
    }
}

def_option! {
    pub union SubmoduleLoadingOption (64){
        pub flags: SubmoduleLoadingOptionFlags,
    }
}

#[cfg(any(feature = "kmgmt", doc))]
unsafe extern "system" {
    pub unsafe fn ConfigureKernel(cfg: KCSlice<KernelCfgOption>) -> SysResult;
    /// Opens a kernel module specified by `path`. If `path` is relative, it is resolved by `res_base` or the
    ///  standard search directory (kernel dependant)
    ///
    /// The calling thread must have the kernel permission `LOAD_MODULE`
    /// ## About Kernel Modules
    ///
    /// A Kernel Module is a shared object file that is loaded in the context of the kernel.
    /// It can perform high-privilege actions, have direct access to attached devices, and interact with kernel data structures.
    ///
    /// Kernel Modules are generally not portable between kernels (but might be compatible between different versions of the same kernel).
    /// Whether or not this is detected by the [`OpenKModule`] function is not specified.
    /// If detected, this is typically reported by an [`INTERP_ERROR`][crate::sys::result::errors::INTERP_ERROR] error.
    ///
    /// >[!WARNING]
    /// > There are few checks on kernel modules, and they run with the same capabilities as the kernel.
    /// > Improper use of the function can lead to system instability or system crashes.
    /// > Only load modules you want loaded.
    pub unsafe fn OpenKModule(res_base: HandlePtr<FileHandle>, path: KStrCPtr) -> SysResult;

    /// Closes the kernel module specified by `path` at `res_base`.
    /// Modules are identified by virtual object identifier (the device id of the filesystem it resides on, combined with the object number within that filesystem)
    ///
    /// Requires kernel permission `UNLOAD_MODULE`.
    pub unsafe fn CloseKModule(res_base: HandlePtr<FileHandle>, path: KStrCPtr) -> SysResult;

    /// Loads a new "subsystem" which is accessible to the current process and its children.
    ///
    /// `name` is a subsystem name. This is a slightly more portable way to open kernel modules that are subsystems (that provide system calls).
    ///
    /// Subsystems are looked up in a kernel dependant way.
    /// This includes matching well-known names, and searching the "standard search directory" for module.
    ///
    /// ## Unprivileged Threads
    ///
    /// A thread typically needs kernel permission `LOAD_MODULE` to call this function.
    /// However certain subsystems can be regarded as "privileged". `name` can be recognized as a privileged subsystem for one of the following reasons:
    /// * It's a privileged well-known subsystem,
    /// * It is a subsystem name which resolves to subsystem object in the standard search directory, and the file opened has the following properties:
    ///   * It is owned by `SYSTEM` (NIL UUID),
    ///   * It has an `InstallSecurityContext` stream, and
    ///   * The `InstallSecurityContext` stream has a [`GrantPermission`][crate::sys::permission::encoded::SecurityContextInstruction::GrantPermission] instruction that grants kernel permission "PRIVILEGED_SUBSYSTEM".
    ///
    /// A call only can be made from an unprivileged thread if it does not set any [`SubmoduleLoadingOptionFlags`] (IE. an unprivileged thread may only load a subsystem in the context of the current process)
    pub unsafe fn OpenSubsystem(
        name: KStrCPtr,
        options: KCSlice<SubmoduleLoadingOption>,
    ) -> SysResult;
}
