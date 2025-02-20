use core::ffi::{c_ulong, c_void};

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::AnyBitPattern))]
pub struct AuxvEnt {
    pub a_type: c_ulong,
    pub a_value: *mut c_void,
}

pub const AT_NULL: c_ulong = 0;
pub const AT_IGNORE: c_ulong = 1;
pub const AT_PAGESZ: c_ulong = 6;
pub const AT_BASE: c_ulong = 7;
pub const AT_PLATFORM: c_ulong = 8;

pub const AT_SECURE: c_ulong = 23;
pub const AT_BASE_PLATFORM: c_ulong = 24;
pub const AT_RANDOM: c_ulong = 26;

pub const AT_LILIUM_START: c_ulong = 64;
pub const AT_LILIUM_INIT_HANDLES: c_ulong = 64;
pub const AT_LILIUM_INIT_HANDLES_LEN: c_ulong = 65;
pub const AT_LILIUM_EXECHDL: c_ulong = 66;
