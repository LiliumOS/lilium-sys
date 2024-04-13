#[repr(C)]
#[derive(Copy, Clone)]
pub struct KStrCPtr {
    pub str_ptr: *const u8,
    pub len: usize,
}

impl KStrCPtr {
    pub const fn from_str(st: &str) -> Self {
        KStrCPtr {
            str_ptr: st.as_ptr() as *const u8,
            len: st.len() as usize,
        }
    }

    pub const fn empty() -> Self {
        KStrCPtr {
            str_ptr: core::ptr::NonNull::dangling().as_ptr(),
            len: 0,
        }
    }

    /// # Safety
    ///
    /// As `str_ptr` and `len` fields are public,
    /// it is your responsibility to ensure that they refer to a correct `str` slice that outlives the return value.
    pub unsafe fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                self.str_ptr.cast(),
                self.len as usize,
            ))
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KStrPtr {
    pub str_ptr: *mut u8,
    pub len: usize,
}

impl KStrPtr {
    pub const fn empty() -> Self {
        KStrPtr {
            str_ptr: core::ptr::NonNull::dangling().as_ptr(),
            len: 0,
        }
    }

    pub const fn as_const(&self) -> KStrCPtr {
        KStrCPtr {
            str_ptr: self.str_ptr,
            len: self.len,
        }
    }

    /// # Safety
    ///
    /// As `str_ptr` and `len` fields are public,
    /// it is your responsibility to ensure that they refer to a correct `str` slice that outlives the return value.
    ///
    /// Note that, after any syscall that returns successfully, a [`KStrPtr`] passed to the syscall will be initialized to valid UTF-8, and set the length field to the length of the valid string
    /// (at most the length of the buffer indicated by the kernel).
    pub unsafe fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                self.str_ptr.cast(),
                self.len as usize,
            ))
        }
    }
}

#[repr(C)]
pub struct KCSlice<T> {
    pub arr_ptr: *const T,
    pub len: usize,
}

impl<T> KCSlice<T> {
    pub const fn empty() -> Self {
        Self {
            arr_ptr: core::ptr::NonNull::dangling().as_ptr(),
            len: 0,
        }
    }

    pub const fn from_slice(sl: &[T]) -> KCSlice<T> {
        Self {
            arr_ptr: sl.as_ptr(),
            len: sl.len() as usize,
        }
    }
}

#[repr(C)]
pub struct KSlice<T> {
    pub arr_ptr: *mut T,
    pub len: usize,
}

impl<T> KSlice<T> {
    pub const fn empty() -> Self {
        Self {
            arr_ptr: core::ptr::NonNull::dangling().as_ptr(),
            len: 0,
        }
    }

    pub fn from_slice_mut(sl: &mut [T]) -> KSlice<T> {
        Self {
            arr_ptr: sl.as_mut_ptr(),
            len: sl.len() as usize,
        }
    }
}
