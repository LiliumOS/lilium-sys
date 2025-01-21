use core::{
    ffi::{c_ulong, c_void},
    marker::PhantomData,
    mem::MaybeUninit,
};

pub use crate::sys::io::IOHandle;
use crate::{
    handle::{AsHandle, HandleRef, OwnedHandle},
    sys::{
        fs::FileHandle,
        handle::HandlePtr,
        io::{CloseIOStream, IOAbort, IORead},
    },
};

unsafe impl<'a, H> AsHandle<'a, IOHandle> for H
where
    H: AsHandle<'a, FileHandle>,
{
    fn as_handle(&self) -> HandlePtr<IOHandle> {
        <Self as AsHandle<'a, FileHandle>>::as_handle(self).cast()
    }
}

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
        let code = unsafe { CloseIOStream(self.0) };
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

unsafe impl<'a, 'b> AsHandle<'a, IOHandle> for ReadMemBuf<'b>
where
    'b: 'a,
{
    fn as_handle(&self) -> HandlePtr<IOHandle> {
        self.0
    }
}

#[cfg(feature = "std")]
mod std_impl {
    use super::*;
    use ::std::io as sio;

    /// A type that provides advisory information about whether or not random access can interfere with non-random access operations.
    pub enum RandomAccessBehaviour {
        /// Indicates
        Seek,
    }

    /// A trait for streams that can have their size queried without interfering with their position
    pub trait StreamSize {
        /// Determines the size of the stream, or returns an error.
        /// Unlike [`Seek::stream_len`][sio::Seek::stream_len],  
        fn data_size(&self) -> sio::Result<u128>;
    }

    /// A trait for streams that can be read from an arbitrary position.
    /// Note: depending on the stream configuration, this can have inconsistent effects when combined with [`Read`][sio::Read]
    pub trait ReadRandomAccess: sio::Read + sio::Seek {
        fn read_from(&self, buf: &mut [u8], base: u128) -> sio::Result<usize>;

        fn read_exact_from(&self, buf: &mut [u8], base: u128) -> io::Result<()> {}
    }
}

#[cfg(feature = "std")]
pub use self::std_impl::*;
