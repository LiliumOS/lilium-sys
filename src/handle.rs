mod private {
    pub trait Sealed {}
}

use core::{borrow::Borrow, hash::Hash, marker::PhantomData, mem::MaybeUninit, ops::Deref};

use private::Sealed;

use crate::{
    sys::{
        debug::{DebugDetach, DebugHandle},
        device::DeviceHandle,
        fs::FileHandle,
        handle::{self as sys, HandlePtr},
        io::{CloseIOStream, IOHandle},
        permission::{DestroySecurityContext, SecurityContext},
        thread::{DetachThread, ThreadHandle},
    },
    thread::TlsKey,
};

use crate::result::{Error, Result};

pub trait HandleType: Sized + Sealed {
    unsafe fn destroy(ptr: HandlePtr<Self>);
}

pub trait UpcastHandle<T>: HandleType {}

impl Sealed for ThreadHandle {}
impl Sealed for DebugHandle {}
impl Sealed for SecurityContext {}
impl Sealed for IOHandle {}
impl Sealed for FileHandle {}
impl Sealed for DeviceHandle {}

impl HandleType for ThreadHandle {
    unsafe fn destroy(ptr: HandlePtr<Self>) {
        DetachThread(ptr);
    }
}

impl HandleType for DebugHandle {
    unsafe fn destroy(ptr: HandlePtr<Self>) {
        DebugDetach(ptr);
    }
}

impl HandleType for SecurityContext {
    unsafe fn destroy(ptr: HandlePtr<Self>) {
        DestroySecurityContext(ptr);
    }
}

impl HandleType for IOHandle {
    unsafe fn destroy(ptr: HandlePtr<Self>) {
        CloseIOStream(ptr);
    }
}

impl HandleType for FileHandle {
    unsafe fn destroy(ptr: HandlePtr<Self>) {
        CloseIOStream(ptr.cast());
    }
}

impl HandleType for DeviceHandle {
    unsafe fn destroy(ptr: HandlePtr<Self>) {
        CloseIOStream(ptr.cast());
    }
}

#[repr(transparent)]
pub struct HandleRef<T>(HandlePtr<T>);

impl<T> HandleRef<T> {
    pub const fn as_raw(&self) -> HandlePtr<T> {
        self.0
    }
}

impl<T> HandleRef<T> {
    pub fn borrow<'a>(&'a self) -> BorrowedHandle<'a, T> {
        BorrowedHandle(self.0, PhantomData)
    }

    pub fn upcast<'a, U: HandleType>(&'a self) -> BorrowedHandle<'a, U>
    where
        T: UpcastHandle<U>,
    {
        BorrowedHandle(self.0.cast(), PhantomData)
    }
}

impl<T> core::fmt::Debug for HandleRef<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> core::fmt::Pointer for HandleRef<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> PartialEq for HandleRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for HandleRef<T> {}

impl<T> Hash for HandleRef<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[repr(transparent)]
pub struct OwnedHandle<T: HandleType>(HandleRef<T>, PhantomData<T>);

impl<T: HandleType> OwnedHandle<T> {
    pub const unsafe fn take_ownership(hdl: HandlePtr<T>) -> Self {
        Self(HandleRef(hdl), PhantomData)
    }

    pub fn release_ownership(self) -> HandlePtr<T> {
        let ptr = self.0 .0;
        core::mem::forget(self);
        ptr
    }
}

impl<T: HandleType> core::fmt::Debug for OwnedHandle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: HandleType> core::fmt::Pointer for OwnedHandle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: HandleType> PartialEq for OwnedHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: HandleType> Eq for OwnedHandle<T> {}

impl<T: HandleType> Hash for OwnedHandle<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: HandleType> Deref for OwnedHandle<T> {
    type Target = HandleRef<T>;
    fn deref(&self) -> &HandleRef<T> {
        &self.0
    }
}

impl<T: HandleType> Drop for OwnedHandle<T> {
    fn drop(&mut self) {
        unsafe { <T as HandleType>::destroy(self.0 .0) }
    }
}

#[repr(transparent)]
pub struct BorrowedHandle<'a, T>(HandlePtr<T>, PhantomData<&'a T>);

impl<'a, T> Clone for BorrowedHandle<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for BorrowedHandle<'a, T> {}

impl<'a, T: HandleType> BorrowedHandle<'a, T> {}

impl<'a, T> Deref for BorrowedHandle<'a, T> {
    type Target = HandleRef<T>;
    fn deref(&self) -> &HandleRef<T> {
        unsafe { &*(core::ptr::addr_of!(self.0) as *const HandleRef<T>) }
    }
}

impl<'a, T> core::fmt::Debug for BorrowedHandle<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a, T> core::fmt::Pointer for BorrowedHandle<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a, T> PartialEq for BorrowedHandle<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a, T> Eq for BorrowedHandle<'a, T> {}

impl<'a, T> Hash for BorrowedHandle<'a, T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

pub unsafe trait AsHandle<'a, T> {
    fn as_handle(&self) -> HandlePtr<T>;
}

unsafe impl<'a, T> AsHandle<'a, T> for HandlePtr<T> {
    fn as_handle(&self) -> HandlePtr<T> {
        *self
    }
}

unsafe impl<'a, T> AsHandle<'a, T> for &'a HandleRef<T> {
    fn as_handle(&self) -> HandlePtr<T> {
        self.as_raw()
    }
}

unsafe impl<'a, T: HandleType> AsHandle<'a, T> for &'a OwnedHandle<T> {
    fn as_handle(&self) -> HandlePtr<T> {
        self.as_raw()
    }
}

unsafe impl<'a, T> AsHandle<'a, T> for BorrowedHandle<'a, T> {
    fn as_handle(&self) -> HandlePtr<T> {
        self.as_raw()
    }
}

unsafe impl<'a, T> AsHandle<'a, T> for &'a T
where
    T: AsHandle<'a, T>,
{
    fn as_handle(&self) -> HandlePtr<T> {
        <T as AsHandle<'a, T>>::as_handle(self)
    }
}

pub struct SharedHandle<T: HandleType>(sys::SharedHandlePtr, TlsKey<HandlePtr<T>>);

impl<T: HandleType> core::fmt::Debug for SharedHandle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: HandleType> Drop for SharedHandle<T> {
    fn drop(&mut self) {
        unsafe {
            self.1.dealloc();
        }
    }
}

impl<T: HandleType> SharedHandle<T> {
    pub fn share(file: OwnedHandle<T>) -> Result<Self> {
        let loc = TlsKey::<HandlePtr<T>>::try_alloc()?;

        let hdl = file.release_ownership();

        let bare_hdl = hdl.cast();

        let mut shared = MaybeUninit::uninit();

        Error::from_code(unsafe { sys::ShareHandle(shared.as_mut_ptr(), bare_hdl, 0) })?;
        let shared = unsafe { shared.assume_init() };

        unsafe {
            loc.get().write(bare_hdl.cast());
        }

        Ok(Self(shared, loc))
    }

    pub fn try_get(&self) -> Result<HandlePtr<T>> {
        let val = unsafe { self.1.get().read() };

        if val == HandlePtr::null() {
            let mut hdl = MaybeUninit::uninit();

            Error::from_code(unsafe {
                crate::sys::handle::UpgradeSharedHandle(hdl.as_mut_ptr(), self.0)
            })?;

            let hdl = unsafe { hdl.assume_init() }.cast();
            unsafe {
                self.1.get().write(hdl);
            }

            Ok(hdl)
        } else {
            Ok(val)
        }
    }
}
