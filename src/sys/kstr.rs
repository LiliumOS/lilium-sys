use core::ffi::{c_char, c_ulong};

#[repr(C)]
pub struct KStrCPtr {
    pub str_ptr: *const c_char,
    pub len: c_ulong,
}

impl KStrCPtr {
    pub const fn from_str(st: &str) -> KStrCPtr {
        KStrCPtr {
            str_ptr: st.as_ptr() as *const c_char,
            len: st.len() as c_ulong,
        }
    }
}

#[repr(C)]
pub struct KStrPtr {
    pub str_ptr: *mut c_char,
    pub len: c_ulong,
}
