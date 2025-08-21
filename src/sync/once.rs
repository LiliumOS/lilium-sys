use core::{
    cell::UnsafeCell,
    mem::{ManuallyDrop, MaybeUninit},
    ops::ControlFlow,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::sys::{
    event::{AwaitAddress, NotifyAddress},
    kstr::KCSlice,
};

const INIT: usize = 0x0001;
const LOCK: usize = 0x0002;
const EVENT: usize = 0x0004;

pub struct OnceLock<T> {
    init: AtomicUsize,
    val: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Sync for OnceLock<T> {}

struct Guard<'a>(&'a AtomicUsize);

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        let val = self.0.swap(0, Ordering::Release);
        if (val & EVENT) != 0 {
            unsafe {
                NotifyAddress(self.0.as_ptr(), !0, 0, KCSlice::empty());
            }
        }
    }
}

impl<'a> Guard<'a> {
    pub fn finish_init(self) {
        let this = ManuallyDrop::new(self);
        let val = this.0.swap(INIT, Ordering::Release);
        if (val & EVENT) != 0 {
            unsafe {
                NotifyAddress(this.0.as_ptr(), !0, 0, KCSlice::empty());
            }
        }
    }
}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        Self {
            init: AtomicUsize::new(0),
            val: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub const fn new_init(val: T) -> Self {
        Self {
            init: AtomicUsize::new(INIT),
            val: UnsafeCell::new(MaybeUninit::new(val)),
        }
    }

    fn is_init(&self) -> bool {
        (self.init.load(Ordering::Acquire) & INIT) != 0
    }

    // Returns None if the lock was released because the
    fn lock(&self) -> Option<Guard> {
        let mut val = self.init.fetch_or(LOCK, Ordering::Acquire);
        let mut step = 0;

        while (val & LOCK) != 0 {
            if step < 128 {
                core::hint::spin_loop();
                val = self.init.fetch_or(LOCK, Ordering::Acquire);
                step += 1;
            } else {
                val = self.init.fetch_or(EVENT, Ordering::Relaxed);
                val |= EVENT;
                if unsafe { AwaitAddress(self.init.as_ptr(), &mut val, 0, KCSlice::empty()) } == 0 {
                    step = 32
                }
                val = self.init.fetch_or(LOCK, Ordering::Acquire)
            }
        }

        if (val & INIT) != 0 {
            let val = self.init.swap(INIT, Ordering::Relaxed);
            if (val & EVENT) != 0 {
                unsafe {
                    NotifyAddress(self.init.as_ptr(), !0, 0, KCSlice::empty());
                }
            }
            None
        } else {
            Some(Guard(&self.init))
        }
    }

    fn is_init_mut(&mut self) -> bool {
        (*self.init.get_mut() & INIT) != 0
    }

    fn try_init_impl<R, F: FnOnce() -> ControlFlow<R, T>>(&self, init_fn: F) -> ControlFlow<R, &T> {
        if self.is_init() {
            return ControlFlow::Continue(unsafe { &*(self.val.get().cast()) });
        }

        let Some(guard) = self.lock() else {
            return ControlFlow::Continue(unsafe { &*(self.val.get().cast()) });
        };

        match init_fn() {
            ControlFlow::Continue(c) => {
                let ptr = self.val.get().cast::<T>();
                unsafe {
                    ptr.write(c);
                }
                guard.finish_init();
                ControlFlow::Continue(unsafe { &*ptr })
            }
            ControlFlow::Break(err) => ControlFlow::Break(err),
        }
    }
}
