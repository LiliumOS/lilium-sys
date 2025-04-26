use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

use lock_api::GuardSend;

use crate::{
    atomic::{AtomicTimedWaitEx, AtomicWaitEx, WaitError},
    sys::event,
    time::{Duration, MonotonicClock, TimePoint},
};

pub struct RawRwLock(AtomicUsize);

const LOCK_EXCLUSIVE: usize = 1 << (usize::BITS - 1);

unsafe impl lock_api::RawRwLock for RawRwLock {
    const INIT: Self = RawRwLock(AtomicUsize::new(0));

    type GuardMarker = GuardSend;

    fn lock_shared(&self) {
        let mut v = self.0.load(Ordering::Relaxed);
        let mut step = 0;
        loop {
            if (v & LOCK_EXCLUSIVE) != 0 {
                if step < 128 {
                    core::hint::spin_loop();
                    step += 1;
                } else {
                    let _ = self.0.wait(v);
                }
                v = self.0.load(Ordering::Relaxed);
                continue;
            }

            match self
                .0
                .compare_exchange_weak(v, v + 1, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(e) => v = e,
            }
        }
    }

    fn try_lock_shared(&self) -> bool {
        self.0
            .fetch_update(Ordering::Acquire, Ordering::Relaxed, |v| {
                if (v & LOCK_EXCLUSIVE) == 0 {
                    Some(v + 1)
                } else {
                    None
                }
            })
            .is_ok()
    }

    unsafe fn unlock_shared(&self) {
        let v = self.0.fetch_sub(1, Ordering::Release);
        if v == 1 || v == LOCK_EXCLUSIVE {
            // Optimize notification to only if someone (an exclusive) is waiting and we just released the last shared lock
            // Alternatively, notify a waiter if we've potentially blocked a shared waiter
            self.0.notify_one();
        }
    }

    fn lock_exclusive(&self) {
        let mut step = 0;
        while let Err(v) =
            self.0
                .compare_exchange_weak(0, LOCK_EXCLUSIVE, Ordering::Acquire, Ordering::Relaxed)
        {
            if v != 0 {
                if step < 128 {
                    core::hint::spin_loop();
                    step += 1;
                } else {
                    let _ = self.0.wait(v);
                }
            }
        }
    }

    fn try_lock_exclusive(&self) -> bool {
        self.0
            .compare_exchange(0, LOCK_EXCLUSIVE, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock_exclusive(&self) {
        let v = self.0.fetch_nand(LOCK_EXCLUSIVE, Ordering::Release);

        self.0.notify_one();
    }

    fn is_locked(&self) -> bool {
        self.0.load(Ordering::Relaxed) != 0
    }

    fn is_locked_exclusive(&self) -> bool {
        (self.0.load(Ordering::Relaxed) & LOCK_EXCLUSIVE) != 0
    }
}

unsafe impl lock_api::RawRwLockTimed for RawRwLock {
    type Duration = crate::time::Duration;
    type Instant = crate::time::TimePoint<MonotonicClock>;

    fn try_lock_shared_for(&self, timeout: Self::Duration) -> bool {
        let time = crate::time::TimePoint::now().expect("Clock could not be read") + timeout;
        self.try_lock_shared_until(time)
    }

    fn try_lock_shared_until(&self, timeout: Self::Instant) -> bool {
        let mut v = self.0.load(Ordering::Relaxed);
        let mut step = 0;
        loop {
            if (v & LOCK_EXCLUSIVE) != 0 {
                if step < 128 {
                    core::hint::spin_loop();
                    step += 1;
                } else {
                    if let Err(WaitError::Timeout) = self.0.wait_until(v, timeout) {
                        break false;
                    }
                }
                v = self.0.load(Ordering::Relaxed);
                continue;
            }

            match self
                .0
                .compare_exchange_weak(v, v + 1, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(_) => break true,
                Err(e) => v = e,
            }
        }
    }

    fn try_lock_exclusive_for(&self, timeout: Self::Duration) -> bool {
        let time = crate::time::TimePoint::now().expect("Clock could not be read") + timeout;
        self.try_lock_exclusive_until(time)
    }

    fn try_lock_exclusive_until(&self, timeout: Self::Instant) -> bool {
        let mut step = 0;
        while let Err(v) =
            self.0
                .compare_exchange_weak(0, LOCK_EXCLUSIVE, Ordering::Acquire, Ordering::Relaxed)
        {
            if v != 0 {
                if step < 128 {
                    core::hint::spin_loop();
                    step += 1;
                } else {
                    if let Err(WaitError::Timeout) = self.0.wait_until(v, timeout) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

pub type RwLock<T> = lock_api::RwLock<RawRwLock, T>;

pub struct RawMutex(AtomicUsize);

unsafe impl lock_api::RawMutex for RawMutex {
    const INIT: Self = RawMutex(AtomicUsize::new(0));
    type GuardMarker = GuardSend;

    fn is_locked(&self) -> bool {
        self.0.load(Ordering::Relaxed) != 0
    }

    fn try_lock(&self) -> bool {
        self.0.swap(LOCK_EXCLUSIVE, Ordering::Acquire) == 0
    }

    fn lock(&self) {
        let mut step = 0;
        while self.0.swap(LOCK_EXCLUSIVE, Ordering::Acquire) != 0 {
            if step < 0 {
                core::hint::spin_loop();
                step += 1;
            } else {
                let _ = self.0.wait(LOCK_EXCLUSIVE);
            }
        }
    }

    unsafe fn unlock(&self) {
        self.0.store(0, Ordering::Release)
    }
}

#[cfg(feature = "io")]
unsafe impl lock_api::RawMutexTimed for RawMutex {
    type Duration = crate::time::Duration;
    type Instant = crate::time::TimePoint<MonotonicClock>;
    fn try_lock_for(&self, timeout: Self::Duration) -> bool {
        let time = crate::time::TimePoint::now().expect("Clock could not be read") + timeout;

        self.try_lock_until(time)
    }

    fn try_lock_until(&self, timeout: Self::Instant) -> bool {
        let mut step = 0;
        while self.0.swap(LOCK_EXCLUSIVE, Ordering::Acquire) != 0 {
            if step < 0 {
                core::hint::spin_loop();
                step += 1;
            } else {
                if let Err(WaitError::Timeout) = self.0.wait_until(LOCK_EXCLUSIVE, timeout) {
                    return false;
                }
            }
        }

        true
    }
}

pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;

pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;

pub struct Condvar(AtomicUsize);

#[derive(Clone, Debug)]
pub struct WaitTimeoutResult<G>(pub G, bool);

impl<G> WaitTimeoutResult<G> {
    pub const fn timed_out(&self) -> bool {
        self.1
    }
}

impl Condvar {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub fn wait<'a, T: ?Sized + 'a>(&self, mut guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        MutexGuard::unlocked(&mut guard, || {
            let v = self.0.load(Ordering::Relaxed);
            let _ = self.0.wait(v);
        });
        guard
    }

    pub fn wait_timeout<'a, T: ?Sized + 'a>(
        &self,
        mut guard: MutexGuard<'a, T>,
        dur: impl Into<crate::time::Duration>,
    ) -> WaitTimeoutResult<MutexGuard<'a, T>> {
        let timeout = MutexGuard::unlocked(&mut guard, || {
            let v = self.0.load(Ordering::Relaxed);
            matches!(self.0.wait_for(v, dur), Err(WaitError::Timeout))
        });

        WaitTimeoutResult(guard, timeout)
    }

    pub fn notify_one(&self) {
        self.0.notify_one();
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub fn notify_all(&self) {
        self.0.notify_all();
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
