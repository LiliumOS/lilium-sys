use core::{
    ffi::c_ulong,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::{
    result::{Error, Result},
    sys::time::{self as sys, ClockOffset, GetClockOffset},
    uuid::Uuid,
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(sys::Duration);

impl Duration {
    pub const ZERO: Self = Self(sys::Duration {
        seconds: 0,
        nanos_of_second: 0,
    });

    pub const fn from_seconds(seconds: i64) -> Self {
        Self(sys::Duration {
            seconds,
            nanos_of_second: 0,
        })
    }

    pub const fn from_seconds_and_nanos(mut seconds: i64, mut nanos: u32) -> Self {
        while nanos >= 1_000_000_000 {
            nanos -= 1_000_000_000;
            seconds += 1;
        }

        Self(sys::Duration {
            seconds,
            nanos_of_second: nanos,
        })
    }

    pub const fn from_system(mut dur: sys::Duration) -> Self {
        while dur.nanos_of_second >= 1_000_000_000 {
            dur.nanos_of_second -= 1_000_000_000;
            dur.seconds += 1;
        }

        Self(dur)
    }

    pub const fn into_system(self) -> sys::Duration {
        self.0
    }
}

impl AddAssign for Duration {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0.seconds += rhs.0.seconds;

        let mut nanos = self.0.nanos_of_second + rhs.0.nanos_of_second;

        if nanos >= 1_000_000_000 {
            nanos -= 1_000_000_000;
            self.0.seconds += 1;
        }

        self.0.nanos_of_second = nanos;
    }
}

impl Add for Duration {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign for Duration {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0.seconds -= rhs.0.seconds;

        let (mut nanos, wrap) = self
            .0
            .nanos_of_second
            .overflowing_sub(rhs.0.nanos_of_second);
        if wrap {
            nanos = nanos.wrapping_add(1_000_000_000);
            self.0.seconds -= 1;
        }

        self.0.nanos_of_second = nanos;
    }
}

impl Sub for Duration {
    type Output = Self;
    #[inline]
    fn sub(mut self, rhs: Self) -> Self {
        self -= rhs;
        self
    }
}

impl From<sys::Duration> for Duration {
    fn from(value: sys::Duration) -> Self {
        Self(value)
    }
}

impl From<core::time::Duration> for Duration {
    fn from(value: core::time::Duration) -> Self {
        let nanos = value.subsec_nanos();
        let seconds = value.as_secs();

        if seconds > i64::MAX {
            panic!("Too Long Duration")
        }

        Duration(sys::Duration {
            seconds: seconds as i64,
            nanos_of_second: nanos,
        })
    }
}

pub struct TimePoint<C>(sys::Duration, PhantomData<C>);

impl<C> Copy for TimePoint<C> {}

impl<C> Clone for TimePoint<C> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<C> core::fmt::Debug for TimePoint<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("TimePoint")
            .field(&self.0)
            .field(&self.1)
            .finish()
    }
}

impl<C> core::hash::Hash for TimePoint<C> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<C> PartialEq for TimePoint<C> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<C> Eq for TimePoint<C> {}

impl<C> Ord for TimePoint<C> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<C> PartialOrd for TimePoint<C> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "std")]
impl From<std::time::SystemTime> for TimePoint<SystemClock> {
    fn from(value: std::time::SystemTime) -> Self {
        let dur = value
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Time before Unix Epoch??");

        TimePoint(dur.into(), PhantomData)
    }
}

pub trait Clock {
    fn clock_uuid() -> Uuid;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn clock_uuid() -> Uuid {
        sys::CLOCK_EPOCH
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MonotonicClock;

impl Clock for MonotonicClock {
    fn clock_uuid() -> Uuid {
        sys::CLOCK_MONOTONIC
    }
}

impl<C> TimePoint<C> {
    pub const EPOCH: Self = Self(
        sys::Duration {
            seconds: 0,
            nanos_of_second: 0,
        },
        PhantomData,
    );

    pub const fn from_epoch_offset(dur: Duration) -> Self {
        Self(dur.0, PhantomData)
    }

    pub const fn since_epoch(self) -> Duration {
        Duration(self.0)
    }

    pub const fn between(self, other: Self) -> Duration {
        let mut dur = self.0;
        let other_dur = other.0;

        dur.seconds -= other_dur.seconds;

        let (mut nanos, wrap) = dur
            .nanos_of_second
            .overflowing_sub(other_dur.nanos_of_second);

        if wrap {
            nanos = nanos.wrapping_add(1_000_000_000);
            dur.seconds -= 1;
        }

        dur.nanos_of_second = nanos;

        Duration(dur)
    }
}

impl<C> Add<Duration> for TimePoint<C> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Duration) -> Self::Output {
        Self::from_epoch_offset(self.since_epoch() + rhs)
    }
}

impl<C> Add<TimePoint<C>> for Duration {
    type Output = TimePoint<C>;
    #[inline]
    fn add(self, rhs: TimePoint<C>) -> Self::Output {
        rhs + self
    }
}

impl<C> AddAssign<Duration> for TimePoint<C> {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl<C> Sub<Duration> for TimePoint<C> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Duration) -> Self::Output {
        Self::from_epoch_offset(self.since_epoch() - rhs)
    }
}

impl<C> SubAssign<Duration> for TimePoint<C> {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl<C> Sub for TimePoint<C> {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.since_epoch() - rhs.since_epoch()
    }
}

#[cfg(feature = "io")]
impl<C: Clock> TimePoint<C> {
    pub fn now() -> Result<Self> {
        let id = C::clock_uuid();

        let mut offset = MaybeUninit::uninit();

        Error::from_code(unsafe { GetClockOffset(offset.as_mut_ptr(), id) })?;

        // SAFETY: Because `GetClockOffset` didn't return an error, it initialized `offset`
        let dur = unsafe { offset.assume_init() };

        Ok(Self(dur, PhantomData))
    }

    pub fn since(self) -> Result<Duration> {
        let id = C::clock_uuid();
        let inner = self.since_epoch();

        let mut offset = MaybeUninit::uninit();

        Error::from_code(unsafe { GetClockOffset(offset.as_mut_ptr(), id) })?;

        // SAFETY: Because `GetClockOffset` didn't return an error, it initialized `offset`
        let dur = unsafe { offset.assume_init() };

        let dur = Duration(dur);

        Ok(dur - inner)
    }

    pub fn convert_to<C2: Clock>(self) -> Result<TimePoint<C2>> {
        let inner = self.since_epoch();

        let mut offsets = [
            ClockOffset {
                clockid: C::clock_uuid(),
            },
            ClockOffset {
                clockid: C2::clock_uuid(),
            },
        ];

        Error::from_code(unsafe {
            sys::GetClockOffsets(offsets.as_mut_ptr(), offsets.len() as c_ulong)
        })?;

        let current_dur = Duration(unsafe { offsets[0].offset });
        let current_dur2 = Duration(unsafe { offsets[1].offset });

        Ok(TimePoint::from_epoch_offset(
            (current_dur - inner) + current_dur2,
        ))
    }
}
