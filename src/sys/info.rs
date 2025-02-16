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

impl SysInfoRequestOsVersion {
    pub const INIT: Self = Self {
        head: ExtendedOptionHead {
            ty: SYSINFO_REQUEST_OSVER,
            ..ExtendedOptionHead::ZERO
        },
        osvendor_name: KStrPtr::empty(),
        os_major: 0,
        os_minor: 0,
    };
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

impl SysInfoRequestKernelVendor {
    pub const INIT: Self = Self {
        head: ExtendedOptionHead {
            ty: SYSINFO_REQUEST_KVENDOR,
            ..ExtendedOptionHead::ZERO
        },
        kvendor_name: KStrPtr::empty(),
        build_id: Uuid::NIL,
        kernel_major: 0,
        kernel_minor: 0,
    };
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

impl SysInfoRequestArchInfo {
    pub const INIT: Self = Self {
        head: ExtendedOptionHead {
            ty: SYSINFO_REQUEST_ARCH_INFO,
            ..ExtendedOptionHead::ZERO
        },
        arch_type: Uuid::NIL,
        arch_version: 0,
    };
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

impl SysInfoRequestComputerName {
    pub const INIT: Self = Self {
        head: ExtendedOptionHead {
            ty: SYSINFO_REQUEST_COMPUTER_NAME,
            ..ExtendedOptionHead::ZERO
        },
        hostname: KStrPtr::empty(),
        sys_id: Uuid::NIL,
        sys_display_name: KStrPtr::empty(),
        sys_label: KStrPtr::empty(),
    };
}

pub mod arch_info {
    use super::SysInfoRequestArchInfo;
    use crate::uuid::{parse_uuid, Uuid};
    /// [`SysInfoRequestArchInfo::arch_type`] value indicating x86-64 (64-bit x86 processors).
    ///
    /// [`SysInfoRequestArchInfo::arch_version`] is set to the microarch version known to be supported, or `0` if the microarch version cannot be determined
    pub const ARCH_TYPE_X86_64: Uuid = parse_uuid("52aa8be1-822d-502c-8309-cf4d785ad524");

    /// [`SysInfoRequestArchInfo::arch_type`] value indicating 32-bit x86 (IA-32).
    ///
    /// [`SysInfoRequestArchInfo::arch_version`] is set as follows:
    /// * 3: Processor equivalent to the Intel i386 Processor or within 3rd Generation.
    /// * 4: Processor equivalent to the Intel i486 Processor or within 4th Generation.
    /// * 5: Processor equivalent to the Intel Pentium Processor or within the 5th Generation.
    /// * 6: Processor equivalent to the Intel Pentium Pro Processor or within the 6th Generation.
    /// * 7: Processor equivalent to the Intel Pentium 4 Processor or newer (including 32-bit Kernels running on 64-bit x86 processors)
    ///
    /// ## Notes
    /// The `arch_version` value will never be set below 3, as that indicates a 16-bit processor, rather than a 32-bit processor.
    /// Additionally, the standard Lilium kernel will not set this below 6, as the minimum supported processor for 32-bit x86 is the Pentium Pro
    ///  (however, a 3rd party kernel or emulator that complies with the SCI may return a smaller value).
    ///
    /// The value of `arch_version` is such that the Compiler Target according to the [LCCC Project](https://github.com/lccc-project/lccc) given by `i{arch_version}86-pc-lilium-standard`
    ///  will generate code that is correct for the current target.
    pub const ARCH_TYPE_X86_IA_32: Uuid = parse_uuid("84d2de8d-00e5-55bd-a65c-e28a842c2778");
    pub const ARCH_VERSION_X86_IA_32_386: u32 = 3;
    pub const ARCH_VERSION_X86_IA_32_486: u32 = 4;
    pub const ARCH_VERSION_X86_IA_32_586: u32 = 5;
    pub const ARCH_VERSION_X86_IA_32_686: u32 = 6;
    pub const ARCH_VERSION_X86_IA_32_P4: u32 = 7;
    /// [`SysInfoRequestArchInfo::arch_type`] value indicating Clever-ISA.
    ///
    /// [`SysInfoRequestArchInfo::arch_version`] is set to the major version of the Clever-ISA Specification known to be implemented.
    pub const ARCH_TYPE_CLEVER_ISA: Uuid = parse_uuid("311dbdf0-32e5-5e7f-a2df-3d822c137b68");
    /// [`SysInfoRequestArchInfo::arch_type`] value indicating 32-bit ARM
    ///
    /// [`SysInfoRequestArchInfo::arch_version`] is set to the version of the ARM Specification known to be implemented.
    pub const ARCH_TYPE_ARM32: Uuid = parse_uuid("691cb76d-a4d5-5639-92b6-8e890ff6d09e");
    /// [`SysInfoRequestArchInfo::arch_type`] value indicating Aarch64
    ///
    /// [`SysInfoRequestArchInfo::arch_version`] is set to the version of the ARM Specification known to be implemented. This value will never be less than 8 (since ARMv8 is the first version to include the Aarch64 instruction set)
    pub const ARCH_TYPE_AARCH64: Uuid = parse_uuid("5c8fc578-f44d-5c7d-91cf-4a9446466f1a");
    /// [`SysInfoRequestArchInfo::arch_type`] value indicating 32-bit RISC-V.
    ///
    /// [`SysInfoRequestArchInfo::arch_version`] is set to the version of the RISC-V specification implemented
    pub const ARCH_TYPE_RISCV32: Uuid = parse_uuid("394463df-b66a-5f10-a970-a4bdda21c80e");
    /// [`SysInfoRequestArchInfo::arch_type`] value indicating 64-bit RISC-V.
    ///
    /// [`SysInfoRequestArchInfo::arch_version`] is set to the version of the RISC-V specification implemented
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

pub const SYSINFO_REQUEST_PHYSICAL_INFO: Uuid = parse_uuid("f5601b29-da4a-5db7-b37f-b2518bbce903");

impl SysInfoRequestPhysicalInfo {
    pub const INIT: Self = Self {
        head: ExtendedOptionHead {
            ty: SYSINFO_REQUEST_PHYSICAL_INFO,
            ..ExtendedOptionHead::ZERO
        },
        physical_core_count: 0,
        logical_core_count: 0,
        discrete_processor_count: 0,
    };
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
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

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
pub struct SysInfoRequestSupportedSubsystem {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The version of the subsystem.
    /// The exact value of this depends on the subsystem queried.
    pub subsys_version: u64,

    /// The number of the subsystem for this supported subsystem.
    ///
    /// The base number for syscalls is this number `<<12`, and the base number for error numbers is this number `<<8`, and negated.
    ///
    /// This number is guaranteed to never exceed 2^20.
    pub subsystem_no: u16,
    /// The maximum syscall number supported (within the syscall region allocated to the subsystem)
    ///
    /// Note that it is not guaranteed that every syscall in the allocated region will be supported by the subsystem
    pub max_sysno: u16,
}

/// Fallback type to represent unknown requests
#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct SysInfoRequestUnknown {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The body of the request, content depends on the type.
    pub body: [MaybeUninit<u8>; SYS_INFO_REQUEST_BODY_SIZE],
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::AnyBitPattern for SysInfoRequestUnknown {}

/// Option struct for obtaining information about the kernel
///
/// Additional extended option flags:
/// * Bit 16: `SYSINFO_REQUEST_FLAG_SKIP` - used by USI impls to indicate that the kernel should treat the request as unrecognized. Must be set together with [`OPTION_FLAG_IGNORE`][super::option::OPTION_FLAG_IGNORE].
///   This bit should not be set by users, and does not have an associated constant. USI impls are not required to request this flag for requests it fulfills, and may clear it when set by the user.
#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
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
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct ProcInfoRequestUnknown {
    /// The Header of the request
    pub head: ExtendedOptionHead,
    /// The body of the request, content depends on the type.
    pub body: [MaybeUninit<u8>; SYS_INFO_REQUEST_BODY_SIZE],
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::AnyBitPattern for ProcInfoRequestUnknown {}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod x86;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use x86::ProcInfoArchRequest;

#[cfg(any(target_arch = "clever"))]
pub mod clever;

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
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
pub union ProcInfoRequest {
    pub head: ExtendedOptionHead,
    pub unknown: ProcInfoRequestUnknown,
    pub arch: ProcInfoArchRequest,
}

#[cfg(feature = "base")]
unsafe extern "system" {
    /// Obtains information about the System (OS, Kernel, or CPU).
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
    /// ## [`GetProcessorInfo`]
    /// [`GetSystemInfo`] can be used to make information requests supported by [`GetProcessorInfo`].
    /// Only some [`ProcInfoRequest`]s are supported in this manner and on a multicore system
    ///
    /// ## `INVALID_LENGTH`` strings
    ///
    /// Special behaviour is guaranteed for [`GetSystemInfo`] and [`GetProcessorInfo`] with [`KStrPtr`]s embedded in a request.
    /// Before returning `INVALID_LENGTH` as a result of a `KStrPtr` field having an insufficient length, the following is guaranteed to hold:
    /// * All other `KStrPtr`s in the same request will be fulfilled (written up to capacity with the total length put in the `len` field),
    /// * All fields that don't contain either a `KStrPtr` or a `KSlice` are guaranteed to be filled,
    ///
    /// This allows for requests that may return multiple strings (such as [`SYSINFO_REQUEST_COMPUTER_NAME`]) to be used to only fill one string, or to read all direct values.
    ///
    pub unsafe fn GetSystemInfo(reqs: KSlice<SysInfoRequest>) -> SysResult;

    /// Obtains information about a specific processor on the system
    pub unsafe fn GetProcessorInfo(proc_id: u32, reqs: KSlice<ProcInfoRequest>) -> SysResult;
}
