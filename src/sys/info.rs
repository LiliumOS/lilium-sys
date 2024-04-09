use super::{
    kstr::{KSlice, KStrPtr},
    option::ExtendedOptionHead,
};

use core::mem::MaybeUninit;

use crate::uuid::{parse_uuid, Uuid};

/// Fallback type to represent unknown requests
#[repr(C, align(32))]
pub struct SysInfoRequestUnknown {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The body of the request, content depends on the type.
    pub body: [MaybeUninit<u8>; 64],
}

/// Requests OS Version Information
#[repr(C, align(32))]
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

pub mod arch_info {
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
#[non_exhaustive]
pub union SysInfoRequest {
    pub head: ExtendedOptionHead,
    pub os_version: SysInfoRequestOsVersion,
    pub kernel_vendor: SysInfoRequestKernelVendor,
    pub arch_info: SysInfoRequestArchInfo,
    pub unknown: SysInfoRequestUnknown,
}

pub const SYSINFO_REQUEST_OSVER: Uuid = parse_uuid("22c479ab-c119-58d5-9c1e-fa03ddf9426a");
pub const SYSINFO_REQUEST_KVENDOR: Uuid = parse_uuid("01adbfd8-3b43-5115-9abd-5b2974375358");
pub const SYSINFO_REQUEST_ARCH_INFO: Uuid = parse_uuid("416eed18-85ca-53c9-849f-4b54bb0568b7");

/// Fallback type to represent unknown requests
#[repr(C, align(32))]
pub struct ProcInfoRequestUnknown {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The body of the request, content depends on the type.
    pub body: [MaybeUninit<u8>; 64],
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

}
