
use core::ffi::{c_ulong, c_void};

use crate::{handle::HandleRef, sys::io::{IORead, IOAbort}};
pub use crate::sys::io::IOHandle;

impl HandleRef<IOHandle>{
    pub fn read(&self, buf: &mut [u8]) -> crate::result::Result<usize>{
        let len = buf.len() as c_ulong;
        let code = unsafe{IORead(self.as_raw(), buf as *mut [u8] as *mut u8 as *mut c_void, len)};

        if code==crate::sys::result::errors::PENDING{
            unsafe{IOAbort(self.as_raw());}
        }

        crate::result::Error::from_code(code)
            .map(|()| code as usize)
    }
}