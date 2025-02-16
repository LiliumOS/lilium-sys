use core::ffi::c_void;

use crate::uuid::{parse_uuid, Uuid};

use super::{device::DeviceHandle, handle::HandlePtr, result::SysResult};

/// The default random device that reads from a global enthropy pool
///
/// Reseeding require the kernel permission `WRITE_ENTHROPY_POOL`.
/// It will run a suitable message digest on the written bytes, and append the result to the entropy pool.
pub const RANDOM_DEVICE: Uuid = parse_uuid("43c320fa-fe2e-3322-b80c-9a996bd8001c");

#[cfg(any(feature = "io", doc))]
#[expect(improper_ctypes)]
unsafe extern "system" {
    /// Reads the specified random device for `len` bytes and fills `out` with that data.
    ///
    /// ## Errors
    ///
    /// If `source` is not a valid device, returns `UNKNOWN_DEVICE`.
    /// If `source` refers to a device that does not support `RandomBytes`, returns `UNSUPPORTED_OPERATION`.
    ///
    /// If `source` requires an enthropy pool and that pool is exhausted, returns `DEVICE_UNAVAILABLE`
    ///
    /// If permission `GetRandomBytes` is denied to the calling thread by the device, returns `PERMISSION`.
    ///
    pub fn GetRandomBytes(out: *mut c_void, len: usize, source: *const Uuid) -> SysResult;

    /// Reads the random device specified by `hdl` for `len` bytes and fills `out` with that data.
    ///
    /// ## Errors
    ///
    /// If `hdl` is not a valid device handle, returns `INVALID_HANDLE`.
    /// If `source` refers to a device that does not support `RandomBytes`, returns `UNSUPPORTED_OPERATION`.
    ///
    /// If `source` requires an enthropy pool and that pool is exhausted, returns `DEVICE_UNAVAILABLE`
    ///
    /// If `hdl` does not have the right `GetRandomBytes`, returns `PERMISSION`.
    ///
    pub fn GetDeviceRandomBytes(
        hdl: HandlePtr<DeviceHandle>,
        out: *mut c_void,
        len: usize,
    ) -> SysResult;

    /// Seeds the specified random device. The exact result of this operation is device-specific
    ///
    /// ## Errors
    ///
    /// If `source` is not a valid device, returns `UNKNOWN_DEVICE`.
    /// If `source` refers to a device that does not support `RandomBytes`, returns `UNSUPPORTED_OPERATION`.
    /// If `source` refers to a device that supports `RandomBytes`, but does not support re-seeding, returns `INVALID_OPERATION`.
    ///
    /// If `source` requires an enthropy pool and that pool is exhausted, returns `DEVICE_UNAVAILABLE`
    ///
    /// If permission `SeedRandomBytes` is denied to the calling thread by the device, returns `PERMISSION`.
    ///
    pub fn SeedRandomBytes(seed: *const c_void, len: usize, source: *const Uuid) -> SysResult;

    /// Reads the random device specified by `hdl` for `len` bytes and fills `out` with that data.
    ///
    /// ## Errors
    ///
    /// If `hdl` is not a valid device handle, returns `INVALID_HANDLE`.
    /// If `source` refers to a device that does not support `RandomBytes`, returns `UNSUPPORTED_OPERATION`.
    /// If `source` refers to a device that supports `RandomBytes`, but does not support re-seeding, returns `INVALID_OPERATION`.
    ///
    /// If `source` requires an enthropy pool and that pool is exhausted, returns `DEVICE_UNAVAILABLE`
    ///
    /// If `hdl` does not have the right `SeedRandomBytes`, returns `PERMISSION`.
    ///
    pub fn SeedDeviceRandomBytes(
        hdl: HandlePtr<DeviceHandle>,
        seed: *const c_void,
        len: usize,
    ) -> SysResult;
}
