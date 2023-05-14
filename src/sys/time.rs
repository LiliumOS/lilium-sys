use core::ffi::c_ulong;

use crate::uuid::{parse_uuid, Uuid};

use super::result::SysResult;

#[repr(C)]
pub struct Duration {
    pub seconds: i64,
    pub nanos_of_second: u32,
}

#[repr(C)]
pub struct ClockOffset {
    pub clockid: Uuid,
    pub offset: Duration,
}

/// A Clock that tracks the realtime offset since the unix Epoch, 1970-01-01T00:00:00.00000000Z
///
/// This clock may be modified to adjust the global system time. This operation requires the WRITE_REALTIME_CLOCK kernel permission.
///
/// Any process can read from this clock, provided they have the READ_CLOCK_OFFSET kernel permision.
pub const CLOCK_EPOCH: Uuid = parse_uuid("c8baabaf-b534-3fa1-929e-6177713e93f4");

/// A Clock that tracks monotonically increasing time from an unspecified point.
/// This clock satisfies three properties:
/// 1. Within a process, given two successive reads from the monotonic clock device (where a *happens-before* relationship is established), the later read from the clock will yield a greater or equal
///  offset than the earlier read.
/// 2. Within a process, the clock advances at a stable granularity relative to wall clock time
/// 3. The epoch is temporally before (but not strictly before) any read of the current offset within a process (That is, all calls to `GetClockOffset` will return a positive value or `0)
///
/// These guarantees only hold with respect to calls to `GetClockOffset` within a given process, and are not guaranteed to hold interprocess.
/// In particular, the order of offsets returned from calls in two different processes are unrelated.
///
/// This clock may not be modified by a thread. Attempting to do so via `ResetClockOffset` returns INVALID_OPERATION.
///
/// Any process can read from this clock, provided they have the READ_CLOCK_OFFSET kernel permision.
pub const CLOCK_MONOTONIC: Uuid = parse_uuid("df95f5b1-bbb7-3562-8c7a-6c3ce0a5dd95");

extern "C" {
    ///
    /// Reads the current offset from the epoch, as a Duration, of the specified Clock.
    /// The epoch used for this function is the epoch of the clock.
    ///
    /// ## Errors
    /// Returns UNKNOWN_DEVICE if `clock` is not a valid Clock device id. Returns PERMISSION if read access to the clock device is denied,
    ///   or the current thread does not have the READ_CLOCK_OFFSET kernel permission.
    pub fn GetClockOffset(dur: *mut Duration, clock: Uuid) -> SysResult;

    ///
    /// Reads the current offset from the epoch, as a Duration, of multiple specified Clocks.
    /// This may be used to synchronize two clocks at a point
    ///
    /// ## Errors
    /// Returns UNKNOWN_DEVICE if any `clock_id` specified is not a valid Clock Device id. Returns PERMISSION if read access to the clock device is denied,
    ///  or the current thread dos not have the READ_CLOCK_OFFSET kernel permision.
    ///
    /// In any case other than the last (READ_CLOCK_OFFSET permission is denied), the value of any duration in the array is undefined
    pub fn GetClockOffsets(output_array: *mut ClockOffset, len: c_ulong) -> SysResult;

    ///
    /// Modifies the specified clock to start from the given offset. This is not effective on all clock values
    ///
    /// ## Errors
    /// Returns UNKNOWN_DEVICE if `clock` is not a valid Clock device id. Returns INVALID_OPERATION if the clock is not modifable (e.g. The Monotonic Clock).
    ///
    /// Returns PERMISSION if write access to the clock device is denied, or the current thread does not have the WRITE_CLOCK_OFFSET kernel premission.
    ///
    pub fn ResetClockOffset(dur: Duration, clock: Uuid) -> SysResult;
}
