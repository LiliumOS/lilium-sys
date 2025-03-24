use core::ffi::c_void;

use crate::uuid::Uuid;

use crate::uuid::parse_uuid;

use super::handle::Handle;
use super::handle::HandlePtr;
use super::io::IOHandle;
use super::kstr::KCSlice;
use super::kstr::KSlice;
use super::result::SysResult;

pub mod arch {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub mod x86;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub use x86::*;
}

#[repr(C)]
pub union FaultInfo {
    pub arch: arch::ArchFaultInfo,
    pub generic: [*mut c_void; 32],
}

/// The UUID of the vti subsystem. This can be passed to [`SysInfoRequestSupportedSubsystem`][crate::sys::info::SysInfoRequestSupportedSubsystem].
pub const SUBSYSTEM_VTI: Uuid = parse_uuid("c3579d97-6467-5865-a42c-01bf1833c415");

#[repr(transparent)]
pub struct VmHandle(Handle);

def_option! {
    pub union CreateVmOptions(64){

    }
}

def_option_type! {
    pub struct VmConfigPhysResources("882b9b65-b35a-59a2-8680-ebfba58c9ca7") {
        pub max_pmem: usize,
        pub max_avail_cpus: usize,
        #[doc(hidden)]
        pub __reserved: [usize; 6]

    }
}

def_option! {
    pub union VmConfigOption(64) {
        pub phys_resources: VmConfigPhysResources,
    }
}

pub const EXIT_REASON_VMCALL: u64 = 0x100000001;
pub const EXIT_REASON_IOMTRAP: u64 = 0x100000002;
pub const EXIT_REASON_IOPORT: u64 = 0x100000003;

#[expect(
    improper_ctypes,
    reason = "`HandlePtr<T>` is a hecking thin raw pointer"
)]
unsafe extern "system" {
    pub unsafe fn CreateVm(
        hdl: *mut HandlePtr<VmHandle>,
        options: KCSlice<CreateVmOptions>,
    ) -> SysResult;
    pub unsafe fn DisposeVm(hdl: HandlePtr<VmHandle>) -> SysResult;
    pub unsafe fn ConfigureVm(
        hdl: HandlePtr<VmHandle>,
        cfg_options: KCSlice<VmConfigOption>,
    ) -> SysResult;
    pub unsafe fn GetCurrentVmConfig(
        hdl: HandlePtr<VmHandle>,
        options: KSlice<VmConfigOption>,
    ) -> SysResult;
    /// Obtains a handle to the memory region used by the VM
    ///
    /// The handle is [`CHAR_READABLE`][super::io::CHAR_READABLE], [`CHAR_WRITABLE`][super::io::CHAR_WRITABLE], [`CHAR_SEEKABLE`][super::io::CHAR_SEEKABLE], and [`CHAR_RANDOMACCESS`][super::io::CHAR_RANDOMACCESS]
    pub unsafe fn GetVmMemoryHandle(
        hdl: HandlePtr<VmHandle>,
        ioh: *mut HandlePtr<IOHandle>,
    ) -> SysResult;

    /// Maps an [`IOHandle`] into the virtual memory region.
    pub unsafe fn MapIoMem(
        vm: HandlePtr<VmHandle>,
        stream: HandlePtr<IOHandle>,
        paddr: usize,
        size: usize,
    ) -> SysResult;

    /// Traps accesses to the `IO` region
    pub unsafe fn TrapIoMem(vm: HandlePtr<VmHandle>, paddr: usize, size: usize) -> SysResult;
}
