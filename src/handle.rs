
mod private{
    pub trait Sealed{}
}

use core::{marker::PhantomData, ops::Deref};

use private::Sealed;

use crate::sys::{thread::{ThreadHandle, DetachThread}, debug::{DebugHandle, DebugDetach}, handle::HandlePtr, permission::{SecurityContext, DestroySecurityContext}};

pub trait HandleType: Sized + Sealed{
    unsafe fn destroy(ptr: HandlePtr<Self>);
}

pub trait UpcastHandle<T>: HandleType{}

impl Sealed for ThreadHandle{}
impl Sealed for DebugHandle{}
impl Sealed for SecurityContext{}

impl HandleType for ThreadHandle{
    unsafe fn destroy(ptr: HandlePtr<Self>){
        DetachThread(ptr);
    }
}

impl HandleType for DebugHandle{
    unsafe fn destroy(ptr: HandlePtr<Self>){
        DebugDetach(ptr);
    }
}

impl HandleType for SecurityContext{
    unsafe fn destroy(ptr: HandlePtr<Self>){
        DestroySecurityContext(ptr);
    }
}


#[repr(transparent)]
pub struct HandleRef<T>(HandlePtr<T>);

impl<T> HandleRef<T>{
    pub const fn as_raw(&self) -> HandlePtr<T>{
        self.0
    }
}


impl<T> HandleRef<T>{
    pub fn borrow<'a>(&'a self) -> BorrowedHandle<'a,T>{
        BorrowedHandle(self.0,PhantomData)
    }

    pub fn upcast<'a,U: HandleType>(&'a self) -> BorrowedHandle<'a,U> where T: UpcastHandle<U>{
        BorrowedHandle(self.0.cast(),PhantomData)
    }
}

impl<T> core::fmt::Debug for HandleRef<T>{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

impl<T> core::fmt::Pointer for HandleRef<T>{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}



#[repr(transparent)]
pub struct OwnedHandle<T: HandleType>(HandleRef<T>, PhantomData<T>);


impl<T: HandleType> OwnedHandle<T>{
    pub const unsafe fn take_ownership(hdl: HandlePtr<T>) -> Self{
        Self(HandleRef(hdl),PhantomData)
    }

    pub fn release_ownership(self) -> HandlePtr<T>{
        let ptr = self.0 .0;
        core::mem::forget(self);
        ptr
    }
}

impl<T: HandleType> core::fmt::Debug for OwnedHandle<T>{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

impl<T: HandleType> core::fmt::Pointer for OwnedHandle<T>{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

impl<T: HandleType> Deref for OwnedHandle<T>{
    type Target = HandleRef<T>;
    fn deref(&self) -> &HandleRef<T>{
        &self.0
    }
}

impl<T: HandleType> Drop for OwnedHandle<T>{
    fn drop(&mut self){
        unsafe{<T as HandleType>::destroy(self.0.0)}
    }
}

#[repr(transparent)]
pub struct BorrowedHandle<'a,T>(HandlePtr<T>,PhantomData<&'a T>);

impl<'a,T> Clone for BorrowedHandle<'a,T>{
    fn clone(&self) -> Self{
        *self
    }
}

impl<'a,T> Copy for BorrowedHandle<'a,T>{}

impl<'a,T: HandleType> BorrowedHandle<'a,T>{
    
}

impl<'a,T> Deref for BorrowedHandle<'a,T>{
    type Target = HandleRef<T>;
    fn deref(&self) -> &HandleRef<T>{
        unsafe{&*(core::ptr::addr_of!(self.0) as *const HandleRef<T>)}
    }
}

impl<'a,T> core::fmt::Debug for BorrowedHandle<'a,T>{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}

impl<'a,T> core::fmt::Pointer for BorrowedHandle<'a,T>{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result{
        self.0.fmt(f)
    }
}