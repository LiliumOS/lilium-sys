use core::ffi::{c_char, c_ulong};

#[repr(C)]
pub struct KStrCPtr {
    pub str_ptr: *const c_char,
    pub len: c_ulong,
}

impl KStrCPtr {
    pub const fn from_str(st: &str) -> Self {
        KStrCPtr {
            str_ptr: st.as_ptr() as *const c_char,
            len: st.len() as c_ulong,
        }
    }

    pub const fn empty() -> Self {
        KStrCPtr {
            str_ptr: core::ptr::NonNull::dangling().as_ptr(),
            len: 0,
        }
    }
}

#[repr(C)]
pub struct KStrPtr {
    pub str_ptr: *mut c_char,
    pub len: c_ulong,
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
}

#[repr(C)]
pub struct KCSlice<T> {
    pub arr_ptr: *const T,
    pub len: c_ulong,
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
            len: sl.len() as c_ulong,
        }
    }
}
