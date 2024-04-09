use core::{any::TypeId, mem::MaybeUninit};

use alloc::collections::BTreeMap;
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
        x: &'a sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a KStrPtr>],
    ) -> &'b [&'a KStrPtr];

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
        x: &'a sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a KStrPtr>],
    ) -> &'b [&'a KStrPtr] {
        init_arr[0] = Some(&x.os_version.osvendor_name);

        unsafe {
            core::slice::from_raw_parts(init_arr as *mut Option<&'a KStrPtr> as *mut &'a KStrPtr, 1)
        }
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
        x: &'a sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a KStrPtr>],
    ) -> &'b [&'a KStrPtr] {
        init_arr[0] = Some(&x.kernel_vendor.kvendor_name);

        unsafe {
            core::slice::from_raw_parts(init_arr as *mut Option<&'a KStrPtr> as *mut &'a KStrPtr, 1)
        }
    }

    unsafe fn from_request(x: &sys::SysInfoRequest) -> Self {
        let sys::SysInfoRequestKernelVendor {
            kvendor_name,
            kernel_major,
            kernel_minor,
            build_id,
            ..
        } = x.kernel_vendor;

        let vendor = kvendor_name.as_const().to_string();

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
        x: &'a sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a KStrPtr>],
    ) -> &'b [&'a KStrPtr] {
        &[]
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
        let mut st = f.debug_struct("ArchInfo");
        match self.arch_id {
            sys::arch_info::ARCH_TYPE_X86_64 => {
                st.field_with("arch_id", |f| f.write_str("x86_64"))?;
                if self.version == 0 {
                    st.field_with("version", |f| f.write_str("x86_64"))?;
                } else {
                    st.field_with("version", |f| {
                        f.write_fmt(format_args!("x86_64v{}", self.version))
                    })?;
                }
            }
            sys::arch_info::ARCH_TYPE_X86_IA_32 => {
                st.field_with("arch_id", |f| f.write_str("ia32"))?;
                st.field_with("version", |f| {
                    f.write_fmt(format_args!("i{}86", self.version))
                })?;
            }
            sys::arch_info::ARCH_TYPE_CLEVER_ISA => {
                st.field_with("arch_id", |f| f.write_str("clever"))?;
                st.field_with("version", |f| {
                    f.write_fmt(format_args!("Clever 1.{}", self.version))
                })?;
            }
            sys::arch_info::ARCH_TYPE_AARCH64 => {
                st.field_with("arch_id", |f| f.write_str("aarch64"))?;
                st.field("version", &self.version)?;
            }
            sys::arch_info::ARCH_TYPE_ARM32 => {
                st.field_with("arch_id", |f: &mut core::fmt::Formatter<'static>| {
                    f.write_str("arm32")
                })?;
                st.field("version", &self.version)?;
            }
            sys::arch_info::ARCH_TYPE_RISCV32 => {
                st.field_with("arch_id", |f: &mut core::fmt::Formatter<'static>| {
                    f.write_str("riscv32")
                })?;
                st.field("version", &self.version)?;
            }
            sys::arch_info::ARCH_TYPE_RISCV64 => {
                st.field_with("arch_id", |f: &mut core::fmt::Formatter<'static>| {
                    f.write_str("riscv64")
                })?;
                st.field("version", &self.version)?;
            }
            _ => {
                st.field("arch_id", &self.arch_id)?;
                st.field("version", &self.version)?;
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
        x: &'a sys::SysInfoRequest,
        init_arr: &'b mut [Option<&'a KStrPtr>],
    ) -> &'b [&'a KStrPtr] {
        let req = &x.computer_name;

        init_arr[0] = Some(&req.hostname);
        init_arr[1] = Some(&req.sys_display_name);
        init_arr[2] = Some(&req.sys_label);

        unsafe {
            core::slice::from_raw_parts(init_arr as *mut Option<&'a KStrPtr> as *mut &'a KStrPtr, 3)
        }
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
    strings: Vec<(StringIndex, Vec<i8>)>,
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

            let mut storage_buffer = [None; 4];

            let strings = unsafe { T::find_strings(&req, &mut storage_buffer) };

            for &str in strings {
                let offset = core::ptr::addr_of!(str)
                    .addr()
                    .wrapping_sub(core::ptr::addr_of!(req).addr());

                if offset > 64 {
                    panic!("Wrong index of string. {} attempted to designate string at address {:p} ({} bytes away from ")
                }

                let index = StringIndex((idx << 6) | offset);
                // Most SysRequests will return up to 32 bytes, so this is a reasonable base address
                let mut vec = Vec::with_capacity(32);

                let st = unsafe {
                    &mut *core::ptr::addr_of_mut!(req)
                        .cast::<u8>()
                        .add(offset)
                        .cast::<KStrPtr>()
                };
                st.len = 32;
                st.str_ptr = vec.as_mut_ptr();
                self.strings.push((index, vec));
            }

            self.requests.push(req);
            let ctor_fn = |ptr, req| unsafe { ptr.cast::<T>().write(T::from_request(req)) };

            self.impls.insert(id, (idx, ctor_fn));
        }
        self
    }

    pub fn opt_request<T: FromRequest>(mut self) -> Self {
        let id = TypeId::of::<T>();
        if !self.impls.contains_key(&id) {
            let idx = self.requests.len() | (1 << (usize::BITS - 1));
            self.requests.push(sys::SysInfoRequest {
                head: ExtendedOptionHead {
                    ty: T::REQ_ID,
                    flags: OPTION_FLAG_IGNORE,
                    ..Zeroable::zeroed()
                },
            });

            let mut storage_buffer = [None; 4];

            let strings = unsafe { T::find_strings(&req, &mut storage_buffer) };

            for &str in strings {
                let offset = core::ptr::addr_of!(str)
                    .addr()
                    .wrapping_sub(core::ptr::addr_of!(req).addr());

                if offset > 64 {
                    panic!("Wrong index of string. {} attempted to designate string at address {:p} ({} bytes away from ")
                }

                let index = StringIndex((idx << 6) | offset);
                // Most SysRequests will return up to 32 bytes, so this is a reasonable base address
                let mut vec = Vec::with_capacity(32);

                let st = unsafe {
                    &mut *core::ptr::addr_of_mut!(req)
                        .cast::<u8>()
                        .add(offset)
                        .cast::<KStrPtr>()
                };
                st.len = 32;
                st.str_ptr = vec.as_mut_ptr();
                self.strings.push((index, vec));
            }

            let ctor_fn = |ptr, req| unsafe {
                // Check if the kernel/USI impl has unset the ignore flag, indicating that the request has been fulfilled
                if (req.head.flags & OPTION_FLAG_IGNORE) == 0 {
                    ptr.cast::<Option<T>>().write(Some(T::from_request(req)));
                }
            };

            self.impls.insert(id, (idx, ctor_fn));
        }
        self
    }

    pub fn resolve(self) -> crate::result::Result<RequestResults> {
        let Self {
            mut requests,
            strings,
            impls,
        } = self;

        while let Err(e) = crate::result::Error::from_code(unsafe {
            sys::GetSystemInfo(KSlice::from_slice_mut(&mut requests))
        }) {
            match e {
                crate::result::Error::InsufficientLength => {
                    let mut work_done = false;

                    for (offset, string) in &mut self.strings {
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
    strings: Vec<(StringIndex, Vec<i8>)>,
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

        let mut buf = MaybeUninit::uninit();

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

        let mut buf = MaybeUninit::uninit();

        ctor_fn(buf.as_mut_ptr().cast(), &self.requests[idx]);

        unsafe { buf.assume_init() }
    }
}
