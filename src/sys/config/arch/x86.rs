use crate::sys::option::ExtendedOptionHead;

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union SysConfigArchOption {
    unknown: super::super::SysConfigUnknownOption,
    pub require_extensions: SysConfigRequireThreadExtensions,
}

/// Allows enabling
#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub struct SysConfigRequireThreadExtensions {
    pub head: ExtendedOptionHead,
    /// Sets base extensions required by the thread.
    ///
    /// Note that these switches aren't required to make use of these features,
    ///
    /// The following bits are defined, all other bits must be `0` or the option is unsupported:
    /// * Bit 8: rdpmc can be executed by the thread
    /// * Bit 9: SSE (and related extensions) are enabled. FXSAVE/FXRSTOR can be executed by the thread. No effect in 64-bit processes
    /// * Bit 18: xsave_extensions field is available. XSAVE/XRSTOR can be executed by the thread. Requires CPUID.01H:ECX.XSAVE=1
    pub base_extensions: u32,
    /// Sets supplemental extensions required by the thread that are enabled by the xsave feature state (`xcr0` register).
    ///
    /// Bit 18 of `base_extensions` must be set to `1` or this field must be set to `0`.
    ///
    /// The following bits are defined if `base_extensions[bit 18]` is set to `1`:
    /// * Bit 0: x87 FPU - unused
    /// * Bit 1: SSE (unused if `base_extensions[bit 9]` is set, must be `0` if `base_extensions[bit 9]` is clear)
    /// * Bit 2: AVX instructions are enabled
    /// * Bit 5: AVX-512 kregs are enabled*
    /// * Bit 6: AVX-512 64-byte zmm registers are enabled*
    /// * Bit 7: AVX-512 Upper 16 mm registers are enabled*
    /// * Bit 11: Control Enforcement Technology (User Mode)
    /// * Bit 17/18: AMX is enabled (either bit may be set)
    /// * Bit 19: APX is enabled
    ///
    ///
    /// The value saved here is in the same format as the `xsave` instruction
    ///
    /// *On processors without the AVX10 feature set, the option is invalid if any of Bits 5, 6, or 7 are set to `1` without all 3 bits being set to `1`
    pub xsave_extensions: u64,
}
