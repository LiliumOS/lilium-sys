//! Interfaces for specifying device commands in userspace

use core::ffi::c_ulong;

#[repr(C)]
pub struct DeviceCommandParameter {
    pub direction: u32,
    pub ty: u32,
    pub related: c_ulong,
}

/// Input parameter:
/// For int, long, handle, and buffer-size, preserves as-is
/// For UUID, adds `*const`
/// For KSTR, yields a `KStrCPtr`
/// For Buffer, yields a `*const c_void`
pub const DIR_IN: u32 = 0x01;
/// Output-only parameter
/// For int, long, handle, and UUID, adds `*mut`
/// For KStr, yields a `*mut KStrPtr` with an appropriately sized writable buffer
/// For Buffer, yields a `*mut c_void`.
///
/// Using this direction with the buffer-size parameter has undefined results.
///
/// The value of the pointees are undefined, and cannot be relied upon. In the case of KStr, the buffer pointed to by the KStr has undefined contents.
pub const DIR_OUT: u32 = 0x02;
/// Bidirectional (read-write) parameter
///
/// Same as `DIR_OUT`, except that the contents of the values passed to the `IssueDeviceCommand` syscall are preserved by the kernel.
pub const DIR_INOUT: u32 = 0x03;

/// A plain integer, `u32` or `i32`.
pub const PARAM_TY_INT: u32 = 0x01;
/// A long integer, `c_long`, or `c_ulong`
pub const PARAM_TY_LONG: u32 = 0x02;
/// A Kernel String or buffer suitable for writing a kernel string
pub const PARAM_TY_KSTR: u32 = 0x03;
/// A `Uuid`
pub const PARAM_TY_UUID: u32 = 0x04;
/// A `HandlePtr` of some type
pub const PARAM_TY_HANDLE: u32 = 0x05;
/// A raw buffer
/// The size is given by the `related` field
pub const PARAM_BUFFER: u32 = 0x06;
/// Same as `PARAM_TY_LONG`, but specifically the size of a buffer.
pub const PARAM_BUFFER_SIZE: u32 = 0x07;
