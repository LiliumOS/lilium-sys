use core::ffi::c_void;

use crate::uuid::{parse_uuid, Uuid};

use super::{result::SysResult, handle::HandlePtr, device::DeviceHandle};

/// The default random device that reads from a global enthropy pool
pub const RANDOM_DEVICE: Uuid = parse_uuid("43c320fa-fe2e-3322-b80c-9a996bd8001c");

extern "C" {
    /// Reads the specified random device for `len` bytes and fills `out` with that data.
    ///
    /// ## Errors
    ///
    /// If `source` is not a valid device, returns `UNKNOWN_DEVICE`.
    /// If `source` refers to a device that does not support `GetRandomBytes`, returns `UNSUPPORTED_OPERATION`.
    ///
    /// If `source` requires an enthropy pool and that pool is exhausted, returns `DEVICE_UNAVAILABLE`
    ///
    pub fn GetRandomBytes(out: *mut c_void, len: usize, source: *const Uuid) -> SysResult;


    /// Reads the random device specified by `hdl` for `len` bytes and fills `out` with that data.
    /// 
    /// ## Errors
    /// 
    /// If `hdl` is not a valid device handle, returns `INVALID_HANDLE`. 
    /// If `source` refers to a device that does not support `GetRandomBytes`, returns `UNSUPPORTED_OPERATION`.
    ///
    /// If `source` requires an enthropy pool and that pool is exhausted, returns `DEVICE_UNAVAILABLE`
    ///
    pub fn GetDeviceRandomBytes(hdl: HandlePtr<DeviceHandle>, out: *mut c_void, len: usize) -> SysResult;
}
