use core::mem::MaybeUninit;

use crate::uuid::{parse_uuid, Uuid};

use super::{
    kstr::{KCSlice, KSlice},
    option::ExtendedOptionHead,
    result::SysResult,
};

pub use super::result::errors::*;

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable))]
pub struct ErrorContextEntryUnknown {
    pub head: ExtendedOptionHead,
    pub payload: [MaybeUninit<u8>; 64],
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::AnyBitPattern for ErrorContextEntryUnknown {}

#[repr(C, align(32))]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
pub struct ErrorContextInvalidOption {
    pub head: ExtendedOptionHead,
    /// Contains a "reason" code for the [`INVALID_OPTION`] error.
    /// The bottom 16 bits is a generic reason code, the top 16 bits can communicate additional per-reason information
    pub reason_code: u32,
    /// The index into the provided extended Option array that flagged the error.
    pub index: usize,
    /// The type of the invalid option.
    pub option_id: Uuid,
}

pub const ERROR_CONTEXT_TYPE_INVALID_OPTION: Uuid =
    parse_uuid("ef80b847-30d9-56f7-8349-5a358bc46e7a");

/// The option was invalid because it had a malformed header (invalid flag bits set or reserved bytes were nonzero).
///
pub const ECTX_INVALID_OPTION_REASON_BAD_HEAD: u32 = 1;
/// The option was invalid because the kernel does not understand a non-optional type.
pub const ECTX_INVALID_OPTION_REASON_UNKNOWN: u32 = 2;
/// The option was invalid for a type-dependant reason. The top 16 bits may contain some info about the reason.
pub const ECTX_INVALID_OPTION_REASON_TYPE_DEPENDANT: u32 = 3;

#[repr(C, align(32))]
#[derive(Copy, Clone)]
pub union ErrorContextEntry {
    pub head: ExtendedOptionHead,
    pub unknown: ErrorContextEntryUnknown,
    pub invalid_option: ErrorContextInvalidOption,
}

#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::Zeroable for ErrorContextEntry {}
#[cfg(feature = "bytemuck")]
unsafe impl bytemuck::AnyBitPattern for ErrorContextEntry {}

unsafe extern "system" {
    /// Obtains the last error context stack.
    ///
    pub unsafe fn GetLastErrorContext(ctx: *mut KSlice<ErrorContextEntry>) -> SysResult;
    ///
    pub unsafe fn AddErrorContext(ctx: KCSlice<ErrorContextEntry>) -> SysResult;

    pub unsafe fn ResetErrorContext(errc: SysResult) -> SysResult;
}
