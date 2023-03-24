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
