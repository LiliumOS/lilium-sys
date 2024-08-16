use crate::sys::{kstr::KStrPtr, option::ExtendedOptionHead};

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union ProcInfoArchRequest {
    unknown: super::ProcInfoRequestUnknown,
    pub cpuid_features: ProcInfoX86CpuidFeatures,
    pub xsave_features: ProcInfoX86XsaveFeatures,
    pub avx10_features: ProcInfoX86Avx10Features,
}

/// Allows determining cpu features.
///
/// This is generally equivalent to executing a `cpuid` instruction on the appropriate processor (except in some edge cases, described on the `cpu_feature_info` field),
///  however, the result may be cached by the kernel, or by the USI implementation.
/// Thus, making a `ProcInfoX86CpuidFeatures` request on the appropriate processor is not guaranteed to produce full serialization.
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct ProcInfoX86CpuidFeatures {
    /// The Header of the Option
    pub head: ExtendedOptionHead,
    /// Contains the feature array.
    /// The layout of the array is as follows:
    /// * cpuid[eax=1].ecx
    /// * cpuid[eax=1].edx
    /// * cpuid[eax=7,ecx=0].ecx
    /// * cpuid[eax=7,ecx=0].edx
    /// * cpuid[eax=7,ecx=0].ebx
    /// * cpuid[eax=7,ecx=1].eax
    /// * cpuid[eax=7,ecx=1].ecx
    /// * cpuid[eax=7,ecx=1].edx
    /// * cpuid[eax=7,ecx=1].ebx
    /// * cpuid[eax=7,ecx=2].eax
    /// * cpuid[eax=7,ecx=2].ecx
    /// * cpuid[eax=7,ecx=2].edx
    /// * Reserved
    /// * Reserved
    /// * cpuid[eax=0x80000001].ecx*
    /// * cpuid[eax=0x80000001].edx
    ///
    /// Reserved fields are set to `0` in the described version of the Kernel. The value may be changed in future versions and must not be relied upon by the Software.
    ///
    /// ## Notes about Extended Processor Info (cpuid[eax=0x80000001])
    /// The value set in `cpu_feature_info[14]` does not exactly match the content of the `ecx` register after a `cpuid` instruction for that leaf,
    ///  specifically the following differences are observed:
    /// * Bits 0-9, 12-17, 23, and 24, which are mirrors of the same bits in `cpuid[eax=1].ecx` (`cpu_feature_info[0]`) on AMD Processors only, are set to `0` regardless of the processor,
    /// * Bit 10, which indicates `syscall` support on the AMD k6 processor only, is clear,
    /// * Bit 11, which indicates `syscall` support, is set to `1` on an AMD k6 processor that indicates support via `cpuid[eax=0x80000001].ecx[10]`, and
    /// * Bit 11 may be set to `0` if executed from a 32-bit process running on a 64-bit OS, even if `cpuid` would report it's support.
    pub cpu_feature_info: [u32; 16],
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct ProcInfoX86XsaveFeatures {
    /// The Header of the Option
    pub head: ExtendedOptionHead,
    /// All components supported by xcr0 on the current CPUID
    /// (Returned by `cpuid[eax=0x0D, ecx=0].edx:eax`)
    pub xsave_supported_components: u64,
    /// The maximum size (in bytes) of the `xsave` save area if all components indicated by `xsave_supported_components` were enabled simultaneously
    /// (Returned by `cpuid[eax=0x0D,ecx=0].ecx`)`
    pub xsave_area_max_size: u32,
    /// The xsave feature flags
    /// (Returned by `cpuid[eax=0x0D,ecx=1].eax`)
    pub xsave_features: u32,
}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct ProcInfoX86Avx10Features {
    /// The Header of the Option
    pub head: ExtendedOptionHead,
    /// The CPU Feature Info for AVX10
    ///
    /// The layout of the array is as follows:
    /// * `cpuid[eax=0x24,ecx=0].ebx`
    /// * Remaining elements are reserved
    pub avx10_feature_info: [u32; 16],
}

/// Returns string Names about the Manufacturer and Processor Brand
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct ProcInfoX86ProcessorBrand {
    /// The Header of the Option
    pub head: ExtendedOptionHead,
    /// Manufacturer ID
    /// This corresponds to concatenation of `cpuid[eax=0].{ebx,edx,ecx}`.
    /// Due to the definition, this string is always exactly 12 bytes long
    pub manufacturer_id: KStrPtr,
    /// The Processor Brand String
    /// This corresponds to the concatenation of `cpuid[eax=0x800000002].{eax,ebx,ecx,edx}`, `cpuid[eax=0x800000003].{eax,ebx,ecx,edx}`, and `cpuid[eax=0x800000004].{eax,ebx,ecx,edx}`
    ///  stopping at the first `0` byte.
    ///
    /// Due to the definition, this string is at most 48 bytes long (but may be truncated).
    ///
    /// This is an empty string if the CPUID functions 0x800000002 through 0x80000004 are unimplemented on the current CPU (`cpuid[eax=0x80000000].eax<0x80000002`).
    pub processor_brand_string: KStrPtr,
}

/// Returns the Processor Version information present in `cpuid[eax=1].eax`
/// Note that these are only useful for determining
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct ProcInfoX86ProcessorVersion {
    /// The Header of the Option
    pub head: ExtendedOptionHead,

    /// The Processor type field (bits 12-13)
    pub processor_type: u8,

    /// Model ID
    /// This is obtained from bits 4-7, and if the family ID is 6 or >=15, bits 16-19 in the upper 4 bits
    pub model_id: u8,

    /// Processor Family ID obtained by bits 8-11. If the resulting value is 15, then the value in bits 20-27 are added (without being shifted)
    pub family_id: u16,
}
