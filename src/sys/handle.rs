
use super::result::SysResult;

pub struct Handle(());


#[repr(transparent)]
pub struct HandlePtr<T>(*mut T);


impl<T> core::fmt::Pointer for HandlePtr<T>{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

impl<T> core::fmt::Debug for HandlePtr<T>{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

impl<T> Clone for HandlePtr<T>{
    fn clone(&self) -> Self{
        *self
    }
}

impl<T> Copy for HandlePtr<T>{}

impl<T> core::hash::Hash for HandlePtr<T>{
    fn hash<H: core::hash::Hasher>(&self, hasher: &mut H){
        self.0.hash(hasher);
    }
}

impl<T> core::cmp::PartialEq for HandlePtr<T>{
    fn eq(&self, other: &Self) -> bool{
        self.0.eq(&other.0)
    }
}

impl<T> core::cmp::Eq for HandlePtr<T>{}


impl<T> HandlePtr<T>{
    pub const fn null() -> Self{
        Self(core::ptr::null_mut())
    }
    pub const fn cast<U>(self) -> HandlePtr<U>{
        HandlePtr(self.0.cast())
    }
}


#[repr(transparent)]
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct SharedHandlePtr(*mut Handle);

impl core::fmt::Pointer for SharedHandlePtr{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

unsafe impl Send for SharedHandlePtr{}
unsafe impl Sync for SharedHandlePtr{}

extern "C"{
    fn ShareHandle(shared_handle: *mut SharedHandlePtr,hdl: HandlePtr<Handle>) -> SysResult;
    fn UnshareHandle(hdl: HandlePtr<Handle>) -> SysResult;
    fn UpgradeSharedHandle(hdlout: HandlePtr<Handle>, shared_handle: SharedHandlePtr) -> SysResult;
}