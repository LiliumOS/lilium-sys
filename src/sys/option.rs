use crate::uuid::Uuid;

/// The Header of a extended option type.
/// This is common to all extended option types
#[repr(C, align(32))]
#[derive(Copy, Clone, bytemuck::Zeroable)]
pub struct ExtendedOptionHead {
    /// The type of the option
    pub ty: Uuid,
    /// Flags for the option.
    /// The following bits are defined:
    /// * Bit 0 ([`OPTION_FLAG_IGNORE`]): If set, the kernel may ignore the option if the type is not recognized. Otherwise must error with [`INVALID_OPTION`][`lilium_sys::sys::result::error::INVALID_OPTION`]
    /// * Bits 16-32: Reserved for per-type flags
    pub flags: u32,
    #[doc(hidden)]
    pub __reserved: [u32; 3],
}

/// Indicates that the option may be safely ignored by the kernel if it does not implement the type of the option.
pub const OPTION_FLAG_IGNORE: u32 = 0x00000001;
