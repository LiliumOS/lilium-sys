use super::{
    kstr::{KSlice, KStrPtr},
    option::ExtendedOptionHead,
    result::SysResult,
};

use core::mem::MaybeUninit;

use crate::uuid::{parse_uuid, Uuid};

/// The Number of bytes for the body of a [`SysInfoRequest`] - large enough to store the larger of 8 pointers and 64 bytes.
pub const SYS_INFO_REQUEST_BODY_SIZE: usize = if core::mem::size_of::<usize>() > 8 {
    core::mem::size_of::<usize>() * 8
} else {
    64
};



/// Requests OS Version Information
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct SysInfoRequestOsVersion {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The name of the OS vendor
    pub osvendor_name: KStrPtr,
    /// The major OS version.
    pub os_major: u32,
    /// The minor os version
    pub os_minor: u32,
}

/// Requests Kernel Vendor Name
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct SysInfoRequestKernelVendor {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The name of the kernel vendor
    pub kvendor_name: KStrPtr,
    /// The kernel build id
    pub build_id: Uuid,
    /// The major kernel version
    pub kernel_major: u32,
    /// The minor kernel version
    pub kernel_minor: u32,
}

/// Requests Global Architecture Info
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct SysInfoRequestArchInfo {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The type of the architecture
    pub arch_type: Uuid,
    /// The architecture version.
    ///
    /// This is a generic version, intended to differentiate between similar targets sharing the same arch (IE. i386 vs. i686, or ABI microarchitecture versions of x86_64).
    /// This may be different from vendor-specific processor versions.
    pub arch_version: u32,
}

/// Requests computer name information
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct SysInfoRequestComputerName {
    /// The header of the request
    pub head: ExtendedOptionHead,
    /// The system host name
    pub hostname: KStrPtr,
    /// The system unique identifier
    pub sys_id: Uuid,
    /// The System Display Name
    pub sys_display_name: KStrPtr,
    /// The System Label
    pub sys_label: KStrPtr,
}

pub mod arch_info {
    use crate::uuid::{parse_uuid, Uuid};
    pub const ARCH_TYPE_X86_64: Uuid = parse_uuid("52aa8be1-822d-502c-8309-cf4d785ad524");
    pub const ARCH_TYPE_X86_IA_32: Uuid = parse_uuid("84d2de8d-00e5-55bd-a65c-e28a842c2778");
    pub const ARCH_VERSION_X86_IA_32_386: u32 = 3;
    pub const ARCH_VERSION_X86_IA_32_486: u32 = 4;
    pub const ARCH_VERSION_X86_IA_32_586: u32 = 5;
    pub const ARCH_VERSION_X86_IA_32_686: u32 = 6;
    pub const ARCH_VERSION_X86_IA_32_P4: u32 = 7;
    pub const ARCH_TYPE_CLEVER_ISA: Uuid = parse_uuid("311dbdf0-32e5-5e7f-a2df-3d822c137b68");
    pub const ARCH_TYPE_ARM32: Uuid = parse_uuid("691cb76d-a4d5-5639-92b6-8e890ff6d09e");
    pub const ARCH_TYPE_AARCH64: Uuid = parse_uuid("5c8fc578-f44d-5c7d-91cf-4a9446466f1a");
    pub const ARCH_TYPE_RISCV32: Uuid = parse_uuid("394463df-b66a-5f10-a970-a4bdda21c80e");
    pub const ARCH_TYPE_RISCV64: Uuid = parse_uuid("d6129403-1104-5d03-8b4c-1176fc9f17fd");
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct SysInfoRequestPhysicalInfo {
    /// The header of the request
    pub head: ExtendedOptionHead,
    /// The number of physical cores accross all active processors
    pub physical_core_count: u32,
    /// The number of logical cores (threads)
    /// May be different if the processor supports hyperthreading or shared-state parallelism
    pub logical_core_count: u32,
    /// The number of physically installed discrete Processors
    pub discrete_processor_count: u32,
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct SysInfoRequestAddressSpace {
    /// The header of the request
    pub head: ExtendedOptionHead,
    /// The Minimum Virtual address that a userspace program can allocate,
    pub min_mapping_addr: usize,
    /// The Maximum Virtual Address that a userspace program can allocate
    pub max_mapping_addr: usize,
    /// The Page Granularity
    pub page_size: usize,
}


/// Fallback type to represent unknown requests
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct SysInfoRequestUnknown {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The body of the request, content depends on the type.
    pub body: [MaybeUninit<u8>; SYS_INFO_REQUEST_BODY_SIZE],
}
/// Option struct for obtaining information about the kernel
///
/// Additional extended option flags:
/// * Bit 16: `SYSINFO_REQUEST_FLAG_SKIP` - used by USI impls to indicate that the kernel should treat the request as unrecognized. Must be set together with [`OPTION_FLAG_IGNORE`][super::option::OPTION_FLAG_IGNORE].
///   This bit should not be set by users, and does not have an associated constant. USI impls are not required to request this flag for requests it fulfills, and may clear it when set by the user.
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union SysInfoRequest {
    pub head: ExtendedOptionHead,
    pub os_version: SysInfoRequestOsVersion,
    pub kernel_vendor: SysInfoRequestKernelVendor,
    pub arch_info: SysInfoRequestArchInfo,
    pub computer_name: SysInfoRequestComputerName,
    pub processor_info: SysInfoRequestPhysicalInfo,
    pub addr_space: SysInfoRequestAddressSpace,
    /// Allows querying information about processors common to all CPUs.
    pub common_processor_info: ProcInfoRequest,
    pub unknown: SysInfoRequestUnknown,
}

pub const SYSINFO_REQUEST_OSVER: Uuid = parse_uuid("22c479ab-c119-58d5-9c1e-fa03ddf9426a");
pub const SYSINFO_REQUEST_KVENDOR: Uuid = parse_uuid("01adbfd8-3b43-5115-9abd-5b2974375358");
pub const SYSINFO_REQUEST_ARCH_INFO: Uuid = parse_uuid("416eed18-85ca-53c9-849f-4b54bb0568b7");
pub const SYSINFO_REQUEST_COMPUTER_NAME: Uuid = parse_uuid("82b314fe-0476-51ca-99de-bbd9711403cf");

/// Fallback type to represent unknown requests
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct ProcInfoRequestUnknown {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The body of the request, content depends on the type.
    pub body: [MaybeUninit<u8>; SYS_INFO_REQUEST_BODY_SIZE],
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod x86;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86::ProcInfoArchRequest;

#[cfg(any(target_arch = "clever"))]
mod clever;

#[cfg(any(target_arch = "clever"))]
pub use clever::ProcInfoArchRequest;

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "clever")))]
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union ProcInfoArchRequest {
    unknown: ProcInfoRequestUnknown,
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union ProcInfoRequest {
    pub head: ExtendedOptionHead,
    pub unknown: ProcInfoRequestUnknown,
    pub arch: ProcInfoArchRequest,
}

extern "system" {
    /// Obtains information about the OS.
    /// Each information request is provided as a [`SysInfoRequest`]. The caller is responsibile for initializing the header of the request,
    ///  and if the request is recongized, the fields of the request are set by the kernel.
    ///
    /// If any request is not recognized, it is ignored if the [`OPTION_FLAG_IGNORE`][super::option::OPTION_FLAG_IGNORE] flag is set in the header, otherwise, an error is returned.
    /// If an [`OPTION_FLAG_IGNORE`][super::option::OPTION_FLAG_IGNORE] request is fulfilled, that flag is cleared in the header by the kernel.
    ///
    /// The following info requests are guaranteed to be supported:
    /// * [`SYSINFO_REQUEST_OSVER`] - requests Operating System Version Information
    /// * [`SYSINFO_REQUEST_KVENDOR`] - requests Vendor information about the Kernel build
    /// * [`SYSINFO_REQUEST_ARCH_INFO`] - requests general processor architecture information
    ///
    pub fn GetSystemInfo(reqs: KSlice<SysInfoRequest>) -> SysResult;

    pub fn GetProcessorInfo(proc_id: u32, reqs: KSlice<ProcInfoRequest>) -> SysResult;
}
