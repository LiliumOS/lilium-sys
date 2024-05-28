use core::marker::PhantomData;

use crate::{
    result::{Error, Result},
    sys::{handle::HandlePtr, thread as sys},
};

pub struct TlsKey<T>(isize, PhantomData<*mut T>);

unsafe impl<T> Send for TlsKey<T> {}
unsafe impl<T> Sync for TlsKey<T> {}

impl<T> Clone for TlsKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TlsKey<T> {}

impl<T> PartialEq for TlsKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for TlsKey<T> {}

impl<T> core::fmt::Debug for TlsKey<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("TlsKey").field(&(self.0 as *mut T)).finish()
    }
}

impl<T> core::fmt::Pointer for TlsKey<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("tls:")?;
        (self.0 as *mut T).fmt(f)
    }
}

impl<T> TlsKey<T> {
    pub fn try_alloc() -> Result<Self> {
        let layout = core::alloc::Layout::new::<T>();
        let size = layout.size();
        let align = layout.align();

        let key = if size < 16 || align <= 16 {
            unsafe { sys::tls_alloc_dyn(size) }
        } else {
            unsafe { sys::tls_alloc_dyn_aligned(size, align) }
        };

        Error::from_code(key)?;

        Ok(Self(key, PhantomData))
    }
    pub fn alloc() -> Self {
        match Self::try_alloc() {
            Ok(val) => val,
            Err(_) => alloc::alloc::handle_alloc_error(core::alloc::Layout::new::<T>()),
        }
    }

    pub fn get(&self) -> *mut T {
        get_tls_ptr_impl(self.0).cast()
    }

    pub unsafe fn dealloc(self) {
        sys::tls_free_dyn(self.0)
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "x86", all(target_arch = "x86_64", target_pointer_width = "64")))]{
        #[inline]
        fn get_tls_ptr_impl(key: isize) -> *mut (){
            let ret;

            unsafe{core::arch::asm!("lea {ptr}, fs:[{ptr}]", ptr = inout(reg) key=>ret, options(readonly, nostack, preserves_flags))}

            ret
        }
    }else if #[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]{
        #[inline]
        fn get_tls_ptr_impl(key: isize) -> *mut (){
            let ret;

            unsafe{core::arch::asm!("lea {ptr:e}, fs:[{ptr:e}]", ptr = inout(reg) key=>ret), options(readonly, nostack, preserves_flags)}

            ret
        }
    }else{
        #[inline]
        fn get_tls_ptr_impl(key: isize) -> *mut (){
            let mut base = core::ptr::null_mut::<u8>();
            let _ = unsafe{sys::GetTLSBaseAddr(HandlePtr::null(), &mut base)};
            base.offset(key).cast()
        }
    }

}
