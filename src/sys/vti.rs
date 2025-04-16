use core::ffi::c_void;

use crate::uuid::Uuid;

use crate::uuid::parse_uuid;

use super::handle::Handle;
use super::handle::HandlePtr;
use super::io::IOHandle;
use super::kstr::KCSlice;
use super::kstr::KSlice;
use super::result::SysResult;
use super::thread::ThreadHandle;

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

def_option_type! {
    pub struct VmConfigInMemory("7a1e1349-c0cd-5909-9647-8b9a8f6973b8") {
        /// The start of the region used by the vti host
        pub vmm_region_start: *mut c_void,
        /// THe length (in pages) of the region used by the vti host
        pub vmm_region_len: usize,
        /// The access flags (like [`CreateMapping`][crate::sys::process::CreateMapping]) of the vmm memory in the guest memory space.
        pub vmm_region_guest_access: u32,
    }
}

pub const VMM_MODE_HYPERVISOR: i32 = -1;
pub const VMM_MODE_SUPERVISOR: i32 = 0;

def_option_type! {
    pub struct VmConfigVirtualizeMode("acdeb37b-4ed6-52fd-b9bf-b6a17bc786ff") {
        pub vmm_mode: i32,
    }
}

def_option! {
    pub union VmConfigOption(64) {
        pub phys_resources: VmConfigPhysResources,
        pub in_memory: VmConfigInMemory,
        pub mode: VmConfigVirtualizeMode,
    }
}

pub const EXIT_REASON_VMCALL: u64 = 0x100000001;
pub const EXIT_REASON_IOMTRAP: u64 = 0x100000002;
pub const EXIT_REASON_IOPORT: u64 = 0x100000003;
pub const EXIT_REASON_HWTRAP: u64 = 0x100000004;

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
