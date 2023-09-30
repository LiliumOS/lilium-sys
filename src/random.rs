use core::ffi::c_void;

use crate::uuid::Uuid;

#[derive(Copy, Clone, Debug)]
pub struct RandomDevice(Uuid);

impl RandomDevice {
    pub const fn from_device_id(id: Uuid) -> Self {
        Self(id)
    }

    pub const SYSRANDOM: Self = Self(crate::sys::random::RANDOM_DEVICE);

    pub fn read_bytes(&self, bytes: &mut [u8]) -> crate::result::Result<()> {
        let len = bytes.len();
        crate::result::Error::from_code(unsafe {
            crate::sys::random::GetRandomBytes(bytes as *mut [u8] as *mut c_void, len, &self.0)
        })
    }
}
