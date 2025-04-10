use core::ops::{BitAnd, Not};
pub use core::sync::atomic::{AtomicPtr, AtomicUsize};

use crate::{
    result::Error,
    sys::{
        error::{INTERRUPTED, INVALID_STATE, TIMEOUT},
        event as sys,
        kstr::KCSlice,
    },
    time::Duration,
};

mod private {
    pub trait Sealed {}
}

use private::Sealed;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum WaitError<S> {
    UnexpectedValue(S),
    Interrupted,
    Timeout,
}

/// Extension trait allowing wait/notify
pub trait AtomicWaitEx: Sealed + Sized {
    type Scalar: Copy + Sized;
    type Mask: Copy + Sized + BitAnd<Output = Self::Mask> + Not<Output = Self::Mask>;
    fn wait(&self, expected: Self::Scalar) -> Result<(), WaitError<Self::Scalar>>;
    fn wait_mask(
        &self,
        expected: Self::Scalar,
        mask: Self::Mask,
    ) -> Result<(), WaitError<Self::Scalar>>;
    fn notify(&self, count: usize) -> usize;
    fn notify_one(&self) -> bool {
        self.notify(1) != 0
    }
    fn notify_all(&self) -> usize {
        self.notify(usize::MAX)
    }
    fn notify_mask(&self, mask: Self::Mask, count: usize) -> usize;
    fn notify_mask_one(&self, mask: Self::Mask) -> bool {
        self.notify_mask(mask, 1) != 0
    }
    fn notify_mask_all(&self, mask: Self::Mask, count: usize) -> usize {
        self.notify_mask(mask, usize::MAX)
    }
}

pub trait AtomicTimedWaitEx: AtomicWaitEx {
    fn wait_for(
        &self,
        expected: Self::Scalar,
        dur: impl Into<crate::time::Duration>,
    ) -> Result<(), WaitError<Self::Scalar>>;
    fn wait_for_mask(
        &self,
        expected: Self::Scalar,
        mask: Self::Mask,
        dur: impl Into<crate::time::Duration>,
    ) -> Result<(), WaitError<Self::Scalar>>;

    #[cfg(feature = "io")]
    fn wait_until<C: crate::time::Clock>(
        &self,
        expected: Self::Scalar,
        point: impl Into<crate::time::TimePoint<C>>,
    ) -> Result<(), WaitError<Self::Scalar>> {
        let point: crate::time::TimePoint<C> = point.into();

        let dur = crate::time::TimePoint::now().expect("Failed to read current time");

        let dur = point - dur;

        self.wait_for(expected, dur)
    }

    #[cfg(feature = "io")]
    fn wait_until_mask<C: crate::time::Clock>(
        &self,
        expected: Self::Scalar,
        mask: Self::Mask,
        point: impl Into<crate::time::TimePoint<C>>,
    ) -> Result<(), WaitError<Self::Scalar>> {
        let point: crate::time::TimePoint<C> = point.into();

        let dur = crate::time::TimePoint::now().expect("Failed to read current time");

        let dur = point - dur;

        self.wait_for_mask(expected, mask, dur)
    }
}

impl Sealed for AtomicUsize {}

impl AtomicWaitEx for AtomicUsize {
    type Scalar = usize;
    type Mask = usize;

    fn wait(&self, mut expected: Self::Scalar) -> Result<(), WaitError<Self::Scalar>> {
        match Error::from_code(unsafe {
            sys::AwaitAddress(self.as_ptr(), &mut expected, 0, KCSlice::empty())
        }) {
            Ok(()) => Ok(()),
            Err(Error::InvalidState) => Err(WaitError::UnexpectedValue(expected)),
            Err(Error::Interrupted) => Err(WaitError::Interrupted),
            Err(Error::Timeout) => Err(WaitError::Timeout),
            Err(e) => panic!("Internal Error {}", e),
        }
    }

    fn wait_mask(
        &self,
        mut expected: Self::Scalar,
        mask: Self::Mask,
    ) -> Result<(), WaitError<Self::Scalar>> {
        if mask == 0 {
            panic!("Non-zero mask expected")
        }

        match Error::from_code(unsafe {
            sys::AwaitAddress(self.as_ptr(), &mut expected, !mask, KCSlice::empty())
        }) {
            Ok(()) => Ok(()),
            Err(Error::InvalidState) => Err(WaitError::UnexpectedValue(expected)),
            Err(Error::Interrupted) => Err(WaitError::Interrupted),
            Err(Error::Timeout) => Err(WaitError::Timeout),
            Err(e) => panic!("Internal Error {}", e),
        }
    }

    fn notify(&self, count: usize) -> usize {
        let v = unsafe { sys::NotifyAddress(self.as_ptr(), count, 0, KCSlice::empty()) };

        v as usize
    }

    fn notify_mask(&self, mask: Self::Mask, count: usize) -> usize {
        let v = unsafe { sys::NotifyAddress(self.as_ptr(), count, mask, KCSlice::empty()) };

        v as usize
    }
}

impl<T> Sealed for AtomicPtr<T> {}

impl<T> AtomicWaitEx for AtomicPtr<T> {
    type Scalar = *mut T;
    type Mask = usize;

    fn wait(&self, expected: Self::Scalar) -> Result<(), WaitError<Self::Scalar>> {
        let mut expected = expected.addr();

        match Error::from_code(unsafe {
            sys::AwaitAddress(self.as_ptr().cast(), &mut expected, 0, KCSlice::empty())
        }) {
            Ok(()) => Ok(()),
            Err(Error::InvalidState) => Err(WaitError::UnexpectedValue(
                core::ptr::without_provenance_mut(expected),
            )),
            Err(Error::Interrupted) => Err(WaitError::Interrupted),
            Err(Error::Timeout) => Err(WaitError::Timeout),
            Err(e) => panic!("Internal Error {}", e),
        }
    }

    fn wait_mask(
        &self,
        mut expected: Self::Scalar,
        mask: Self::Mask,
    ) -> Result<(), WaitError<Self::Scalar>> {
        if mask == 0 {
            panic!("Non-zero mask expected")
        }

        match Error::from_code(unsafe {
            sys::AwaitAddress(
                self.as_ptr().cast(),
                core::ptr::from_mut(&mut expected).cast(),
                !mask,
                KCSlice::empty(),
            )
        }) {
            Ok(()) => Ok(()),
            Err(Error::InvalidState) => Err(WaitError::UnexpectedValue(expected)),
            Err(Error::Interrupted) => Err(WaitError::Interrupted),
            Err(Error::Timeout) => Err(WaitError::Timeout),
            Err(e) => panic!("Internal Error {}", e),
        }
    }

    fn notify(&self, count: usize) -> usize {
        let v = unsafe { sys::NotifyAddress(self.as_ptr().cast(), count, 0, KCSlice::empty()) };

        v as usize
    }

    fn notify_mask(&self, mask: Self::Mask, count: usize) -> usize {
        let v = unsafe { sys::NotifyAddress(self.as_ptr().cast(), count, mask, KCSlice::empty()) };

        v as usize
    }
}

impl<A: AtomicWaitEx> AtomicTimedWaitEx for A {
    fn wait_for(
        &self,
        expected: Self::Scalar,
        dur: impl Into<crate::time::Duration>,
    ) -> Result<(), WaitError<Self::Scalar>> {
        let timeout: crate::time::Duration = dur.into();
        let _ = unsafe { sys::SetBlockingTimeout(&timeout.into_system()) };

        self.wait(expected)
    }

    fn wait_for_mask(
        &self,
        expected: Self::Scalar,
        mask: Self::Mask,
        dur: impl Into<crate::time::Duration>,
    ) -> Result<(), WaitError<Self::Scalar>> {
        let timeout: crate::time::Duration = dur.into();
        let _ = unsafe { sys::SetBlockingTimeout(&timeout.into_system()) };

        self.wait_mask(expected, mask)
    }
}
