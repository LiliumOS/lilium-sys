use core::marker::PhantomData;

use crate::{
    result::{Error, Result},
    sys::{handle::HandlePtr, kstr::KCSlice, thread as sys, time::Duration},
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
        unsafe { sys::tls_free_dyn(self.0) }
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

pub fn sleep(dur: impl Into<crate::time::Duration>) {
    let dur: crate::time::Duration = dur.into();
    let mut dur = dur.into_system();

    while unsafe { crate::sys::event::SleepThread(&mut dur, KCSlice::empty()) } < 0 {}
}

#[cfg(feature = "io")]
pub fn sleep_for<C: crate::time::Clock>(tp: impl Into<crate::time::TimePoint<C>>) {
    use core::slice;

    use crate::{
        sys::{
            error::INVALID_OPTION,
            event::{BlockingEvent, EventSleepThreadUntil},
            kstr::KSlice,
        },
        time::TimePoint,
    };

    let tp: crate::time::TimePoint<C> = tp.into();
    let epoch = tp.since_epoch().into_system();

    let clock = C::CLOCK_ID;
    // SleepThreadUntil is only available via `BlockOnEvents*`.
    let mut event = BlockingEvent {
        sleep_thread_until: EventSleepThreadUntil {
            sleep_epoch: epoch,
            clock: crate::sys::handle::HandleOrId { uuid: clock },
            ..EventSleepThreadUntil::NIL
        },
    };
    loop {
        let mut i = unsafe {
            crate::sys::event::BlockOnEventsAll(
                KSlice::from_slice_mut(slice::from_mut(&mut event)),
                KCSlice::empty(),
            )
        };
        if i >= 0 {
            break;
        }

        if i == INVALID_OPTION {
            // Kernel or the clock does not support `EventSleepThreadUntil`, fallback to sleep-on:
            let now = TimePoint::<C>::now().expect("Bad Clock support");

            let dur = tp.between(now);
            break sleep(dur);
        }
    }
}
