use core::{
    any::{Any, TypeId},
    mem::MaybeUninit,
};

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use bytemuck::Zeroable;

use sptr::Strict;

use crate::{
    sys::{
        info as sys,
        kstr::{KSlice, KStrPtr},
        option::{ExtendedOptionHead, OPTION_FLAG_IGNORE},
    },
    uuid::Uuid,
};

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
struct StringIndex(usize);

impl StringIndex {
    fn into_parts(self) -> (usize, usize) {
        (self.0 >> 6, self.0 & 63)
    }
}

pub trait FromRequest: Any {
    const REQ_ID: Uuid;

    /// Finds each `KStrPtr` in `x` and returns an array to them.
    /// The trait implementation may use `init_array` to store the references to the pointer.
    ///
    /// # Safety
    ///
    /// `x` must be a valid [`SysInfoRequest`][sys::SysInfoRequest] corresponding to [`Self::REQ_ID`][FromRequest::REQ_ID].
    unsafe fn find_strings<'a, 'b>(
        x: &'a mut sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a mut KStrPtr>],
    ) -> &'b mut [&'a mut KStrPtr];

    /// # Safety
    /// `x` must be a valid [`SysInfoRequest`][sys::SysInfoRequest] corresponding to [`Self::REQ_ID`][FromRequest::REQ_ID] that was fulfilled,
    /// and all strings indicated by `find_strings` are fully valid (point to fully populated memory that is valid UTF-8).
    ///
    /// Note: Any [`KStrPtr`] referred to by `x` is not guaranteed to outlive the function call
    unsafe fn from_request(x: &sys::SysInfoRequest) -> Self;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OsVersion {
    pub vendor: String,
    pub major_version: u32,
    pub minor_version: u32,
}

impl FromRequest for OsVersion {
    const REQ_ID: Uuid = sys::SYSINFO_REQUEST_OSVER;

    unsafe fn find_strings<'a, 'b>(
        x: &'a mut sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a mut KStrPtr>],
    ) -> &'b mut [&'a mut KStrPtr] {
        init_arr[0] = Some(&mut x.os_version.osvendor_name);

        unsafe { core::slice::from_raw_parts_mut(init_arr.as_mut_ptr() as *mut &'a mut KStrPtr, 1) }
    }

    unsafe fn from_request(x: &sys::SysInfoRequest) -> Self {
        let sys::SysInfoRequestOsVersion {
            osvendor_name,
            os_major,
            os_minor,
            ..
        } = x.os_version;

        let vendor = osvendor_name.as_str().to_string();

        Self {
            vendor,
            major_version: os_major,
            minor_version: os_minor,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct KernelVendor {
    pub vendor: String,
    pub build_id: Uuid,
    pub major_version: u32,
    pub minor_version: u32,
}

impl FromRequest for KernelVendor {
    const REQ_ID: Uuid = sys::SYSINFO_REQUEST_KVENDOR;

    unsafe fn find_strings<'a, 'b>(
        x: &'a mut sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a mut KStrPtr>],
    ) -> &'b mut [&'a mut KStrPtr] {
        init_arr[0] = Some(&mut x.kernel_vendor.kvendor_name);

        unsafe { core::slice::from_raw_parts_mut(init_arr.as_mut_ptr() as *mut &'a mut KStrPtr, 1) }
    }

    unsafe fn from_request(x: &sys::SysInfoRequest) -> Self {
        let sys::SysInfoRequestKernelVendor {
            kvendor_name,
            kernel_major,
            kernel_minor,
            build_id,
            ..
        } = x.kernel_vendor;

        let vendor = kvendor_name.as_str().to_string();

        Self {
            vendor,
            build_id,
            major_version: kernel_major,
            minor_version: kernel_minor,
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ArchInfo {
    pub arch_id: Uuid,
    pub version: u32,
}

impl FromRequest for ArchInfo {
    const REQ_ID: Uuid = sys::SYSINFO_REQUEST_ARCH_INFO;

    unsafe fn find_strings<'a, 'b>(
        _: &'a mut sys::SysInfoRequest,
        _: &'b mut [Option<&'a mut KStrPtr>],
    ) -> &'b mut [&'a mut KStrPtr] {
        &mut []
    }

    unsafe fn from_request(x: &sys::SysInfoRequest) -> Self {
        Self {
            arch_id: x.arch_info.arch_type,
            version: x.arch_info.arch_version,
        }
    }
}

impl core::fmt::Debug for ArchInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        struct StrNoEscape<'a>(&'a str);

        impl<'a> core::fmt::Debug for StrNoEscape<'a> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(self.0)
            }
        }

        let mut st = f.debug_struct("ArchInfo");
        match self.arch_id {
            sys::arch_info::ARCH_TYPE_X86_64 => {
                st.field("arch_id", &StrNoEscape("x86_64"));
                if self.version == 0 {
                    st.field("version", &StrNoEscape("x86_64"));
                } else {
                    st.field("version", &format_args!("x86_64v{}", self.version));
                }
            }
            sys::arch_info::ARCH_TYPE_X86_IA_32 => {
                st.field("arch_id", &StrNoEscape("ia32"));
                st.field("version", &format_args!("i{}86", self.version));
            }
            sys::arch_info::ARCH_TYPE_CLEVER_ISA => {
                st.field("arch_id", &StrNoEscape("clever"));
                st.field("version", &format_args!("Clever 1.{}", self.version));
            }
            sys::arch_info::ARCH_TYPE_AARCH64 => {
                st.field("arch_id", &StrNoEscape("aarch64"));
                st.field("version", &self.version);
            }
            sys::arch_info::ARCH_TYPE_ARM32 => {
                st.field("arch_id", &StrNoEscape("arm"));
                st.field("version", &self.version);
            }
            sys::arch_info::ARCH_TYPE_RISCV32 => {
                st.field("arch_id", &StrNoEscape("riscv32"));
                st.field("version", &self.version);
            }
            sys::arch_info::ARCH_TYPE_RISCV64 => {
                st.field("arch_id", &StrNoEscape("riscv64"));
                st.field("version", &self.version);
            }
            _ => {
                st.field("arch_id", &self.arch_id);
                st.field("version", &self.version);
            }
        }

        st.finish()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ComputerName {
    pub hostname: String,
    pub display_name: String,
    pub label: String,
    pub computer_id: Uuid,
}

impl FromRequest for ComputerName {
    const REQ_ID: Uuid = sys::SYSINFO_REQUEST_COMPUTER_NAME;
    unsafe fn find_strings<'a, 'b>(
        x: &'a mut sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a mut KStrPtr>],
    ) -> &'b mut [&'a mut KStrPtr] {
        let req = &mut x.computer_name;

        init_arr[0] = Some(&mut req.hostname);
        init_arr[1] = Some(&mut req.sys_display_name);
        init_arr[2] = Some(&mut req.sys_label);

        unsafe { core::slice::from_raw_parts_mut(init_arr.as_mut_ptr() as *mut &'a mut KStrPtr, 3) }
    }

    unsafe fn from_request(x: &sys::SysInfoRequest) -> Self {
        let sys::SysInfoRequestComputerName {
            hostname,
            sys_display_name,
            sys_label,
            sys_id,
            ..
        } = x.computer_name;

        let hostname = hostname.as_str().to_string();
        let display_name = sys_display_name.as_str().to_string();
        let label = sys_label.as_str().to_string();

        Self {
            hostname,
            display_name,
            label,
            computer_id: sys_id,
        }
    }
}

pub struct RequestBuilder {
    requests: Vec<sys::SysInfoRequest>,
    strings: Vec<(StringIndex, Vec<u8>)>,
    impls: BTreeMap<TypeId, (usize, fn(*mut (), &sys::SysInfoRequest))>,
}

impl RequestBuilder {
    pub const fn new() -> Self {
        Self {
            requests: Vec::new(),
            strings: Vec::new(),
            impls: BTreeMap::new(),
        }
    }

    pub fn request<T: FromRequest>(mut self) -> Self {
        let id = TypeId::of::<T>();
        if !self.impls.contains_key(&id) {
            let idx = self.requests.len();
            let mut req = sys::SysInfoRequest {
                head: ExtendedOptionHead {
                    ty: T::REQ_ID,
                    flags: 0,
                    ..Zeroable::zeroed()
                },
            };

            let mut storage_buffer = [None, None, None, None];

            let addr = core::ptr::addr_of!(req).addr();

            let strings = unsafe { T::find_strings(&mut req, &mut storage_buffer) };

            for str in strings {
                let offset = core::ptr::addr_of!(*str).addr().wrapping_sub(addr);

                if offset > 96 {
                    panic!("Wrong index of string. {} attempted to designate string at address {:p} ({} bytes away from the base of the request)", core::any::type_name::<T>(), str as *mut _, offset as isize)
                }

                let index = StringIndex((idx << 6) | (offset - 32));
                // Most SysRequests will return up to 32 bytes, so this is a reasonable base address
                let mut vec = Vec::with_capacity(32);
                str.len = 32;
                str.str_ptr = vec.as_mut_ptr();
                self.strings.push((index, vec));
            }

            self.requests.push(req);
            let ctor_fn: fn(*mut (), &sys::SysInfoRequest) =
                |ptr, req| unsafe { ptr.cast::<T>().write(T::from_request(req)) };

            self.impls.insert(id, (idx, ctor_fn));
        }
        self
    }

    pub fn opt_request<T: FromRequest>(mut self) -> Self {
        let id = TypeId::of::<T>();
        if !self.impls.contains_key(&id) {
            let idx = self.requests.len() | (1 << (usize::BITS - 1));
            let mut req = sys::SysInfoRequest {
                head: ExtendedOptionHead {
                    ty: T::REQ_ID,
                    flags: OPTION_FLAG_IGNORE,
                    ..Zeroable::zeroed()
                },
            };

            let mut storage_buffer = [None, None, None, None];

            let addr = core::ptr::addr_of!(req).addr();

            let strings = unsafe { T::find_strings(&mut req, &mut storage_buffer) };

            for str in strings {
                let offset = core::ptr::addr_of!(*str).addr().wrapping_sub(addr);

                if offset > 96 {
                    panic!("Wrong index of string. {} attempted to designate string at address {:p} ({} bytes away from the base of the request)", core::any::type_name::<T>(), str as *mut _, offset as isize)
                }

                let index = StringIndex((idx << 6) | (offset - 32));
                // Most SysRequests will return up to 32 bytes, so this is a reasonable base address
                let mut vec = Vec::with_capacity(32);
                str.len = 32;
                str.str_ptr = vec.as_mut_ptr();
                self.strings.push((index, vec));
            }

            let ctor_fn: fn(*mut (), &sys::SysInfoRequest) = |ptr, req| unsafe {
                // Check if the kernel/USI impl has unset the ignore flag, indicating that the request has been fulfilled
                if (req.head.flags & OPTION_FLAG_IGNORE) == 0 {
                    ptr.cast::<Option<T>>().write(Some(T::from_request(req)));
                } else {
                    ptr.cast::<Option<T>>().write(None)
                }
            };

            self.requests.push(req);

            self.impls.insert(id, (idx, ctor_fn));
        }
        self
    }

    pub fn resolve(self) -> crate::result::Result<RequestResults> {
        let Self {
            mut requests,
            mut strings,
            impls,
        } = self;

        while let Err(e) = crate::result::Error::from_code(unsafe {
            sys::GetSystemInfo(KSlice::from_slice_mut(&mut requests))
        }) {
            match e {
                crate::result::Error::InsufficientLength => {
                    let mut work_done = false;

                    for (offset, string) in &mut strings {
                        let (index, offset) = offset.into_parts();

                        let st = unsafe {
                            &mut *core::ptr::addr_of_mut!(requests[index])
                                .cast::<u8>()
                                .add(offset)
                                .cast::<KStrPtr>()
                        };

                        if st.len > string.capacity() {
                            string.reserve(st.len as usize);
                            st.str_ptr = string.as_mut_ptr();
                            work_done = true
                        }
                    }

                    if !work_done {
                        // We don't know what else is causing an `INVALID_LENGTH` error, so just forward it back to the caller
                        return Err(crate::result::Error::InsufficientLength);
                    }
                }
                e => return Err(e),
            }
        }

        Ok(RequestResults {
            requests,
            strings,
            impls,
        })
    }
}

#[derive(Clone)]
pub struct RequestResults {
    requests: Vec<sys::SysInfoRequest>,
    strings: Vec<(StringIndex, Vec<u8>)>,
    impls: BTreeMap<TypeId, (usize, fn(*mut (), &sys::SysInfoRequest))>,
}

impl RequestResults {
    pub fn get<T: FromRequest>(&self) -> T {
        let ty = TypeId::of::<T>();

        let (idx, ctor_fn) = match self.impls.get(&ty) {
            Some(&data) => data,
            None => panic!(
                "Attempt to obtain results from request `{}`, which was not made",
                core::any::type_name::<T>()
            ),
        };

        if idx & (1 << (usize::BITS - 1)) != 0 {
            panic!(
                "Attempted to obtain results from request `{}`, but that request was optional",
                core::any::type_name::<T>()
            )
        }

        let mut buf = MaybeUninit::<T>::uninit();

        ctor_fn(buf.as_mut_ptr().cast(), &self.requests[idx]);

        unsafe { buf.assume_init() }
    }

    pub fn get_opt<T: FromRequest>(&self) -> Option<T> {
        let ty = TypeId::of::<T>();

        let (idx, ctor_fn) = match self.impls.get(&ty) {
            Some(&data) => data,
            None => panic!(
                "Attempt to obtain results from request `{}`, which was not made",
                core::any::type_name::<T>()
            ),
        };

        if idx & (1 << (usize::BITS - 1)) == 0 {
            panic!("Attempted to obtain results from optional request `{}`, but that request was not marked optional", core::any::type_name::<T>())
        }

        let mut buf = MaybeUninit::<Option<T>>::uninit();

        ctor_fn(buf.as_mut_ptr().cast(), &self.requests[idx]);

        unsafe { buf.assume_init() }
    }
}
