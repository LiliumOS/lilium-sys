use crate::sys::option::ExtendedOptionHead;

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union ArchConfigArchOption {
    unknown: super::super::ArchConfigUnknownOption,
    pub require_extensions: ArchConfigRequireThreadExtensions,
}

#[repr(C, align(32))]
#[derive(Copy, Clone, Zeroable)]
pub struct ArchConfigThreadExtensions {
    pub head: ExtendedOptionHead,
    /// The value of the cpuex2 register enabled - see the current register map.
    ///
    /// The active config will be visible via `cpuex2` when read by the process.
    pub cpuex2: u64,
    #[doc(hidden)]
    pub __reserved: [u64; 5],
}
