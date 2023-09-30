use core::ffi::c_void;

use crate::uuid::{parse_uuid, Uuid};

use super::result::SysResult;

pub const RANDOM_DEVICE: Uuid = parse_uuid("43c320fa-fe2e-3322-b80c-9a996bd8001c");

extern "C" {
    pub fn GetRandomBytes(out: *mut c_void, len: usize, source: *const Uuid) -> SysResult;
}
