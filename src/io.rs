use core::{
    ffi::{c_ulong, c_void},
    marker::PhantomData,
    mem::MaybeUninit,
};

pub use crate::sys::io::IOHandle;
use crate::{
    handle::{HandleRef, OwnedHandle},
    sys::{
        handle::HandlePtr,
        io::{CloseMemoryBuffer, IOAbort, IORead},
    },
};

impl HandleRef<IOHandle> {
    pub fn read(&self, buf: &mut [u8]) -> crate::result::Result<usize> {
        let len = buf.len() as c_ulong;
        let code = unsafe {
            IORead(
                self.as_raw(),
                buf as *mut [u8] as *mut u8 as *mut c_void,
                len,
            )
        };

        if code == crate::sys::result::errors::PENDING {
            unsafe {
                IOAbort(self.as_raw());
            }
        }

        crate::result::Error::from_code(code).map(|()| code as usize)
    }
}

pub struct ReadMemBuf<'a>(HandlePtr<IOHandle>, PhantomData<&'a [u8]>);

impl<'a> Drop for ReadMemBuf<'a> {
    fn drop(&mut self) {
        let code = unsafe { CloseMemoryBuffer(self.0) };
        debug_assert_eq!(
            code,
            0,
            "Failed to close memory buffer {:?}",
            crate::result::Error::from_code(code)
        );
    }
}

impl<'a> ReadMemBuf<'a> {
    pub fn open(buf: &'a [u8]) -> crate::result::Result<Self> {
        let len = buf.len() as c_ulong;
        let buf = buf.as_ptr();

        let mut hdl = MaybeUninit::uninit();

        crate::result::Error::from_code(unsafe {
            crate::sys::io::CreateMemoryBuffer(
                hdl.as_mut_ptr(),
                crate::sys::io::MODE_BLOCKING,
                buf.cast::<c_void>().cast_mut(),
                len,
                crate::sys::io::CHAR_READABLE
                    | crate::sys::io::CHAR_RANDOMACCESS
                    | crate::sys::io::CHAR_SEEKABLE,
            )
        })?;

        Ok(Self(unsafe { hdl.assume_init() }, PhantomData))
    }
}
