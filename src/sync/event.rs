use core::{
    convert::Infallible,
    mem::{zeroed, MaybeUninit},
};

use crate::{
    result::Error,
    sys::{
        event::{self as sys, BlockingEvent, BlockingEventUnknown},
        kstr::{KCSlice, KSlice},
        option::{ExtendedOptionHead, OPTION_FLAG_IGNORE},
    },
    time::Clock,
    uuid::Uuid,
};

///
/// # Safety
/// Implementors must satisfy the following constraints:
/// * [`Event::init`] must produce a [`BlockingEvent`] that can be safely passed to [`sys::BlockOnEventsAny`] or [`sys::BlockOnEventsAll`],
/// * [`Event::get`] must not panic for any input that satisfies the precondition
pub unsafe trait Event {
    /// The value type which can be extracted from this event
    type Val;

    /// Initializes the [`BlockingEvent`] from `self`
    fn init(&mut self) -> BlockingEvent;

    /// Extracts the inner value from a fully initialized event.
    ///
    /// # Safety
    /// `blob` must refer to a value of type [`BlockingEvent`] which was returned from [`Event::init`] then passed to a succesful return from [`sys::BlockOnEventsAll`]
    ///  or a succesful return from [`sys::BlockOnEventsAny`] that returned the appropriate event index.
    unsafe fn get(&mut self, blob: &BlockingEvent) -> Self::Val;
}

pub struct Optional<E>(pub E);

unsafe impl<E> Event for Optional<E>
where
    E: Event,
{
    type Val = Option<E::Val>;
    fn init(&mut self) -> BlockingEvent {
        let mut event = self.0.init();
        unsafe {
            event.head.flags |= OPTION_FLAG_IGNORE;
        }
        event
    }

    unsafe fn get(&mut self, blob: &BlockingEvent) -> Self::Val {
        if (unsafe { blob.head.flags } & OPTION_FLAG_IGNORE) == 0 {
            Some(unsafe { E::get(&mut self.0, blob) })
        } else {
            None
        }
    }
}

/// Trait for types that provide a list of events.
/// This can either be an array or a tuple of [`Event`] types.
///
/// The trait cannot be implemented ouside of this trait.
///
/// Currently, tuples of at most length 12 are supported.
/// Tuples with length 0 are supported as well - for [`block_on_all`] it will succeed immediately (except that it may check for the thread's interrupted flag), for [`block_on_any`] it will never return succesfully.
pub unsafe trait EventList {
    #[doc(hidden)]
    type AllResult;
    #[doc(hidden)]
    type AnyResult;
    #[doc(hidden)]
    type EventArray: AsMut<[BlockingEvent]>;
    #[doc(hidden)]
    fn build_list(&mut self) -> Self::EventArray;
    #[doc(hidden)]
    unsafe fn all_result(&mut self, v: &Self::EventArray) -> Self::AllResult;
    #[doc(hidden)]
    unsafe fn any_result(&mut self, v: &Self::EventArray, n: usize) -> Self::AnyResult;
}

#[doc(hidden)]
pub enum EventListResult<
    A = Infallible,
    B = Infallible,
    C = Infallible,
    D = Infallible,
    E = Infallible,
    F = Infallible,
    G = Infallible,
    H = Infallible,
    I = Infallible,
    J = Infallible,
    K = Infallible,
    L = Infallible,
> {
    _0(A),
    _1(B),
    _2(C),
    _3(D),
    _4(E),
    _5(F),
    _6(G),
    _7(H),
    _8(I),
    _9(J),
    _10(K),
    _11(L),
    __ExhaustivePatternsNonExhaustive(Infallible),
}

/// Use to match on the result of [`block_on_any`] called on a tuple.
/// Invocations are of the form [`event_result!(n => $inner_pat)`][event_result] where `n` is an literal integer index up to the tuple length, and `$inner_pat` is inner pattern to match.
///
/// ## Exhaustive Match
/// Note that due to the implementation and limitations in Rust's type system, you must include a `_` pattern to match remaining patterns (including a potentially unbounded number of unreachable patterns).
/// If you match all elements of the result that correspond to the tuple, you can avoid this with the unstable `exhaustive_patterns` feature (this requires a nightly compiler).
///
#[macro_export]
macro_rules! event_result {
    ($lit:literal => $pat:ident) => {
        $crate::_paste::paste! {
            [<_ $lit>]($pat)
        }
    };
}

/// Blocks on several events at once, returning on an error or else when all provided events are notified.
/// `events` may be either a tuple or an array of types which implement the [`Event`] trait.
/// The result is the same kind of type (tuple or array) containing the result values from each event.
pub fn block_on_all<E: EventList>(mut events: E) -> crate::result::Result<E::AllResult> {
    let mut list = events.build_list();

    let slice: &mut [BlockingEvent] = list.as_mut();
    let kslice = KSlice::from_slice_mut(slice);

    Error::from_code(unsafe { sys::BlockOnEventsAll(kslice, KCSlice::empty()) })?;

    Ok(unsafe { events.all_result(&list) })
}

/// Blocks on several events at once, returning on an error or else when all provided events are notified.
/// `events` may be either a tuple or an array of types which implement the [`Event`] trait.
/// In the case of an array, the result is the [`Event::Val`] type.
/// In the case of a tuple, the result is an opaque enum that must be matched using [`event_result`].
/// The `N`th variant of the opaque enum contains the value of the `N`th element of `events` (See the note about "Exhaustive Match" on [`event_result`] for )
pub fn block_on_any<E: EventList>(mut events: E) -> crate::result::Result<E::AnyResult> {
    let mut list = events.build_list();

    let slice: &mut [BlockingEvent] = list.as_mut();
    let kslice = KSlice::from_slice_mut(slice);
    let n = unsafe { sys::BlockOnEventsAny(kslice, KCSlice::empty()) };
    Error::from_code(n)?;

    Ok(unsafe { events.any_result(&list, n as usize) })
}

unsafe impl<E: Event, const N: usize> EventList for [E; N] {
    type AllResult = [E::Val; N];
    type AnyResult = E::Val;
    type EventArray = [BlockingEvent; N];

    fn build_list(&mut self) -> Self::EventArray {
        let mut arr: [BlockingEvent; N] = unsafe { core::mem::zeroed() };

        for (a, b) in core::iter::zip(&mut arr, self) {
            *a = b.init();
        }

        arr
    }

    unsafe fn any_result(&mut self, v: &Self::EventArray, n: usize) -> Self::AnyResult {
        unsafe { self[n].get(&v[n]) }
    }

    unsafe fn all_result(&mut self, v: &Self::EventArray) -> Self::AllResult {
        let mut n = MaybeUninit::<[E::Val; N]>::uninit();

        let p = n.as_mut_ptr().cast::<E::Val>();

        for (i, (sys, event)) in core::iter::zip(v, self).enumerate() {
            unsafe { p.add(i).write(event.get(sys)) }
        }

        unsafe { n.assume_init() }
    }
}

macro_rules! impl_event_list_for_tuple{
    ($($id:ident),*) => {
        #[allow(non_snake_case)]
        unsafe impl<$($id: Event),*> EventList for ($($id,)*) {
            type AllResult = ($($id :: Val,)*);
            type AnyResult = EventListResult<$($id :: Val),*>;
            type EventArray = [BlockingEvent; ${count($id)}];

            fn build_list(&mut self) -> Self::EventArray {
                let ($($id,)*) = self;

                [$($id . init()),*]
            }

            unsafe fn all_result(&mut self,#[allow(unused_variables)] v : &Self::EventArray) -> Self::AllResult {
                let ($($id,)*) = self;

                ($($id . get(&v[${index()}]),)*)
            }

            unsafe fn any_result(&mut self, #[allow(unused_variables)]  v: &Self::EventArray, n: usize) -> Self::AnyResult {
                let ($($id,)*) = self;

                match n {
                    $(${index()} => {
                        crate::_paste::paste!{EventListResult:: [<_ ${index()}>] ($id . get (&v[${index()}]))}
                    })*
                    _ => unsafe{core::hint::unreachable_unchecked()}
                }
            }
        }
    }
}

impl_event_list_for_tuple!();
impl_event_list_for_tuple!(A);
impl_event_list_for_tuple!(A, B);
impl_event_list_for_tuple!(A, B, C);
impl_event_list_for_tuple!(A, B, C, D);
impl_event_list_for_tuple!(A, B, C, D, E);
impl_event_list_for_tuple!(A, B, C, D, E, F);
impl_event_list_for_tuple!(A, B, C, D, E, F, G);
impl_event_list_for_tuple!(A, B, C, D, E, F, G, H);
impl_event_list_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_event_list_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_event_list_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_event_list_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

pub struct SleepFor(pub crate::time::Duration);

unsafe impl Event for SleepFor {
    type Val = ();

    fn init(&mut self) -> BlockingEvent {
        BlockingEvent {
            sleep_thread: sys::EventSleepThread {
                sleep_duration: self.0.into_system(),
                ..sys::EventSleepThread::NIL
            },
        }
    }

    unsafe fn get(&mut self, _: &BlockingEvent) -> Self::Val {
        ()
    }
}

#[cfg(feature = "io")]
pub struct SleepUntil<C>(pub crate::time::TimePoint<C>);

unsafe impl<C: Clock> Event for SleepUntil<C> {
    type Val = ();

    fn init(&mut self) -> BlockingEvent {
        BlockingEvent {
            sleep_thread_until: sys::EventSleepThreadUntil {
                sleep_epoch: self.0.since_epoch().into_system(),
                clock: crate::sys::handle::HandleOrId {
                    uuid: C::clock_uuid(),
                },
                ..sys::EventSleepThreadUntil::NIL
            },
        }
    }

    unsafe fn get(&mut self, _: &BlockingEvent) {
        ()
    }
}
