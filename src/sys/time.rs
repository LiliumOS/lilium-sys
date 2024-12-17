use core::ffi::c_ulong;

use crate::uuid::{parse_uuid, Uuid};

use super::{device::DeviceHandle, handle::WideHandle, result::SysResult};

/// A `Duration` of time, measuered in a number of `seconds` and then `nanos_of_second` for subsecond values
///
/// `Duration`s are signed, and can represent durations less than 0.
///
/// The `seconds` are measured with a signed `i64`, so can measure durations in excess of +/-2.92e+11 years.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Duration {
    /// The number of seconds the duration represents, between [-1<<63,(1<<63)-1)
    pub seconds: i64,
    /// The subsecond nanos between [0,1,000,000,000)
    pub nanos_of_second: u32,
}

/// A `Clock` or an `Offset` compressed into 16 bytes. Used for [`GetClockOffsets`].
#[repr(C)]
pub union ClockOffset {
    /// A handle to the `Clock` device to read from.
    /// This field may be set instead of a `clockid` to specify a device by handle instead of by id
    pub clockdev: WideHandle<DeviceHandle>,
    /// The id of the `Clock` to read from
    pub clockid: Uuid,
    /// The offset of the clock after reading
    pub offset: Duration,
}

/// A Clock that tracks the realtime offset since the unix Epoch, 1970-01-01T00:00:00.00000000Z
///
/// This clock may be modified to adjust the global system time. This operation requires the WRITE_REALTIME_CLOCK kernel permission.
///
/// Any process can read from this clock, provided they have the READ_CLOCK_OFFSET kernel permision.
///
/// The [precision][GetClockGranularity] of this clock is unspecified, but is at least 0.001 seconds.
pub const CLOCK_EPOCH: Uuid = parse_uuid("c8baabaf-b534-3fa1-929e-6177713e93f4");

/// A Clock that tracks monotonically increasing time from an unspecified point.
/// This clock satisfies three properties:
/// 1. Within a process, given two successive reads from the monotonic clock device (where a *happens-before* relationship is established), the later read from the clock will yield a greater or equal
///  offset than the earlier read.
/// 2. Within a process, the clock advances at a stable granularity relative to wall clock time (that is, successive reads from [`CLOCK_MONOTONIC`] will advance at least at the same rate as successive reads from [`CLOCK_EPOCH`] provided the latter is never [Reset][ResetClockOffset])
/// 3. The epoch is temporally before (but not strictly before) any read of the current offset within a process (That is, all calls to `GetClockOffset` will return a positive value or `0`)
///
/// These guarantees only hold with respect to calls to [`GetClockOffset`] within a given process, and are not guaranteed to hold interprocess.
/// In particular, the order of offsets returned from calls in two different processes are unrelated.
///
/// This clock may not be modified by a thread. Attempting to do so via [`ResetClockOffset`] returns INVALID_OPERATION.
///
/// Any thread can read from this clock, provided they have the READ_CLOCK_OFFSET kernel permision.
///
/// The precision of this clock is unspecified, but shall be at least as precise as [`CLOCK_EPOCH`].
pub const CLOCK_MONOTONIC: Uuid = parse_uuid("df95f5b1-bbb7-3562-8c7a-6c3ce0a5dd95");

unsafe extern "C" {
    ///
    /// Reads the current offset from the epoch, as a Duration, of the specified Clock.
    /// The epoch used for this function is the epoch of the clock.
    ///
    /// ## Errors
    /// Returns UNKNOWN_DEVICE if `clock` is not a valid device id.
    ///
    // Returns UNSUPPORTED_OPERATION if `clock` specifies a device that is not a valid clock device.
    ///
    /// Returns PERMISSION if read access to the clock device is denied,
    ///   or the current thread does not have the READ_CLOCK_OFFSET kernel permission.
    pub fn GetClockOffset(dur: *mut Duration, clock: Uuid) -> SysResult;

    ///
    /// Reads the current offset from the epoch, on multiple clocks, specified either by device id or by handle.
    ///
    /// `output_array` is initialized by the process and kernel as follows:
    /// * For each element of the `output_array`, the process must set either the `clockid` or `clockdev` fields of the `ClockOffset` struct
    /// * For each element of the `output_array`, if the function returns succesfully, the kernel sets the duration field of the `ClockOffset` struct accordingly, and the trailing 4 bytes of the struct are
    ///  set to an indeterminate value.
    ///
    /// This function may be used to synchronize two clocks at a point in time.
    ///
    /// The precision of the values returned is rounded to the maximum common precision of all clocks, S,
    ///   and the deviation of the returned offset from the actual offset of the clock is at most 0.5S.
    ///
    /// ## Errors
    /// Returns UNKNOWN_DEVICE if any `clockid` specified is not a valid Device id in the current device namespace.
    ///
    /// Returns `INVALID_HANDLE` if any `clockdev`specified is not a valid `DeviceHandle`
    ///
    /// Returns `UNSUPPORTED_OPERATION` if any device specified is not a clock device.
    ///
    ///  Returns PERMISSION if read access to any clock device is denied,
    ///  or the current thread dos not have the READ_CLOCK_OFFSET kernel permision.
    ///
    /// In any case other than the last (READ_CLOCK_OFFSET permission is denied), the value of any duration in the array is indeterminate
    pub fn GetClockOffsets(output_array: *mut ClockOffset, len: c_ulong) -> SysResult;

    ///
    /// Modifies the specified clock to start from the given offset. This is not effective on all clock values
    ///
    /// ## Errors
    /// Returns UNKNOWN_DEVICE if `clock` is not a valid device id.
    ///
    /// Returns UNSUPPORTED_OPERATION if `clock` specifies a device that is not a valid clock device.
    ///
    /// Returns INVALID_OPERATION if the clock is not modifable (e.g. The Monotonic Clock).
    ///
    /// Returns PERMISSION if write access to the clock device is denied, or the current thread does not have the WRITE_CLOCK_OFFSET kernel premission.
    ///
    pub fn ResetClockOffset(dur: Duration, clock: Uuid) -> SysResult;

    ///
    /// Obtains the precision of the clock, that is, the smallest time step that the clock can step by - the difference between two calls to `GetClockOffset` without an intervening `ResetClockOffset` call on this clock is guaranteed to be a multiple of this value
    /// ## Errors
    ///
    /// Returns UNKNOWN_DEVICE if `clock` is not a valid Clock device id.
    ///
    /// Returns PERMISSION of read access to the clock is denied, or the current thread does not have the `READ_CLOCK_GRANULARITY` permission.
    pub fn GetClockGranularity(offset: *mut Duration, clock: Uuid) -> SysResult;
}
