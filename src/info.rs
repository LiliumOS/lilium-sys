use core::{any::TypeId, mem::MaybeUninit};

use alloc::collections::BTreeMap;
use bytemuck::Zeroable;

use crate::{
    sys::{
        info as sys,
        kstr::KSlice,
        option::{ExtendedOptionHead, OPTION_FLAG_IGNORE},
    },
    uuid::Uuid,
};

pub trait FromRequest: Any {
    const REQ_ID: Uuid;

    /// # Safety
    /// `x` must correspond to the [`SysInfoRequest`] corresponding to [`Self::REQ_ID`][FromRequest::REQ_ID] that was fulfilled.
    unsafe fn from_request(x: &sys::SysInfoRequest) -> Self;
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ArchInfo {
    pub arch_id: Uuid,
    pub version: u32,
}

impl FromRequest for ArchInfo {
    const REQ_ID: Uuid = sys::SYSINFO_REQUEST_ARCH_INFO;

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

pub struct RequestBuilder {
    requests: Vec<sys::SysInfoRequest>,
    impls: BTreeMap<TypeId, (usize, fn(*mut (), &sys::SysInfoRequest))>,
}

impl RequestBuilder {
    pub const fn new() -> Self {
        Self {
            requests: Vec::new(),
            impls: BTreeMap::new(),
        }
    }

    pub fn request<T: FromRequest>(mut self) -> Self {
        let id = TypeId::of::<T>();
        if !self.impls.contains_key(&id) {
            let idx = self.requests.len();
            self.requests.push(sys::SysInfoRequest {
                head: ExtendedOptionHead {
                    ty: T::REQ_ID,
                    flags: 0,
                    ..Zeroable::zeroed()
                },
            });
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
            impls,
        } = self;

        crate::result::Error::from_code(unsafe {
            sys::GetSystemInfo(KSlice::from_slice_mut(&mut requests))
        })?;

        Ok(RequestResults { requests, impls })
    }
}

#[derive(Clone)]
pub struct RequestResults {
    requests: Vec<sys::SysInfoRequest>,
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
