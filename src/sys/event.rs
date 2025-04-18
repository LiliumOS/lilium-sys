use core::ffi::c_ulong;

use super::{
    handle::HandlePtr,
    kstr::{KCSlice, KSlice},
    process::EventJoinProcess,
    result::SysResult,
    thread::{EventJoinThread, ThreadHandle},
    time::Duration,
};

def_option! {
    pub union ThreadBlockingOption(64) {

    }
}

def_option! {
    pub union AwaitAddrOption(64) {
        pub blocking: ThreadBlockingOption,
    }
}

def_option! {
    pub union NotifyAddressOption(64) {

    }
}

def_option_type! {
    pub struct EventAwaitAddress("58039808-7b8f-5895-9961-e1f9804b1f8c"){
        pub address: *mut c_ulong,
        pub ignore_mask: c_ulong,
        pub await_options: KCSlice<AwaitAddrOption>,
    }
}

def_option_type! {
    pub struct EventSleepThread("6c222e98-9d03-5f9f-babb-563695db3f4c") {
        pub sleep_duration: Duration,
    }
}

def_option! {
    pub union BlockingEvent(32) {
        pub await_addr: EventAwaitAddress,
        pub join_thread: EventJoinThread,
        pub join_process: EventJoinProcess,
    }
}

#[cfg(any(feature = "thread", doc))]
unsafe extern "system" {
    /// Atomically checks that `ptr.read()&!ignore_mask == current.read()` and blocks the thread if it does, until a notification arrives.
    /// If the check fails, `current` is set to the loaded `ptr`.
    /// If a notification was already pending, returns immediately, consuming that notification.
    ///
    /// `ignore_mask` controls which bits to ignore when performing the check. It also controls when to wake the thread. See [`NotifyAddress`] for details.
    ///
    /// Properties: Blocking, Interruptable
    ///
    /// The Notification is based on the mapping referred to by `ptr`, taking thread-private mappings into account.
    /// Shared mappings are not permitted on the memory presently.
    ///
    /// # Synchronization
    ///
    /// All accesses of `ptr` are done atomically, as though by [`AtomicUsize::load`][core::sync::atomic::AtomicUsize::load].
    /// A succesful [`AwaitAddress`] call *synchronizes-with* the [`NotifyAddress`] call that wakes it.
    ///
    /// Like all blocking functions, if the routine is interrupted (see [`InterruptThread`]), it returns `INTERRUPTED` and *synchronizes-with* the operation that causes the interruption.
    ///
    /// # Errors
    ///
    /// Returns `INVALID_MEMORY` if `ptr` is not aligned to the word/pointer size, if a memory access violation occurs, or if `ptr` points to a shared mapping (not process-private or thread-private and backed by a `IOHandle`).
    /// Note that calling `AwaitAddress` requires `ptr` to be Writable, and not merely readable.
    ///
    /// Returns `INVALID_STATE` if the value check fails.
    ///
    /// Returns `INTERRUPTED` or `TIMEOUT` if the blocking completes abruptly due to the thread being interrupted or the blocking-timeout expiring.
    ///
    /// Returns `INVALID_OPTION` if any option is invalid.
    ///
    pub unsafe fn AwaitAddress(
        ptr: *mut usize,
        current: *mut usize,
        ignore_mask: usize,
        options: KCSlice<AwaitAddrOption>,
    ) -> SysResult;

    /// Notifies up to `count` threads blocking on `ptr`. If fewer than `count` threads are blocking on `ptr`, then all of those threads are awoken. If no such threads are blocking,
    ///  the notification is made pending and will be consumed by the first thread that tries to block on `ptr`.
    ///
    /// `wake_mask` is used in the following way:
    /// * If any bits were set in the `ignore_mask` of [`AwaitAddress`] on a given thread, the notification is ignored if those bits are set in `wake_mask`.
    ///
    /// A `count` of `0` is a valid input. It will wake no threads if any are blocked, and will wake the next one to block on the address otherwise.
    ///
    /// A Positive return is the number of threads that was currently blocked on the Address.
    ///
    /// The Notification is based on the mapping referred to by `ptr`, taking thread-private mappings into account.
    /// Shared mappings are not permitted on the memory presently.
    ///
    /// ## Errors
    /// Returns `INVALID_MEMORY` if `ptr` is not aligned to the word/pointer size, if a memory access violation occurs, or if `ptr` points to a shared mapping (not process-private or thread-private and backed by a `IOHandle`).
    /// Note that calling `AwaitAddress` requires `ptr` to be Writable, and not merely readable.
    ///
    /// Returns `INVALID_OPTION` if any option is invalid.
    pub unsafe fn NotifyAddress(
        ptr: *mut usize,
        count: usize,
        wake_mask: usize,
        options: KCSlice<NotifyAddressOption>,
    ) -> SysResult;

    pub fn SetBlockingTimeout(dur: *const Duration) -> SysResult;
    pub fn SleepThread(dur: *const Duration, options: KCSlice<ThreadBlockingOption>) -> SysResult;
    pub safe fn PauseThread(options: KCSlice<ThreadBlockingOption>) -> SysResult;
    pub fn InterruptThread(th: HandlePtr<ThreadHandle>) -> SysResult;
    pub safe fn Interrupted() -> SysResult;
    pub safe fn ClearBlockingTimeout();

    /// Blocks the current thread until each event specified in `events` is notified.
    /// Returns the total number of events notified (same as the length of the slice unless optional events are included).
    ///
    /// The optional flag is cleared on each `event` that was marked optional and not ignored.
    ///
    /// A succesful return from this system call *synchronizes-with* the notification that completed each event that was not ignored.
    ///
    /// Blocking on an empty array returns immediately. It is unspecified whether the interrupted flag is checked before returning in this case.
    ///
    /// # Blocking Timeout/Interrupted
    /// Non-wait events that wake a thread (the thread being interrupted or the blocking timeout expiring) will wake the [`BlockOnEventsAll`] system call as a whole.
    /// In this case, the system call itself returns the appropriate error
    ///
    /// # Errors
    ///
    /// Errors from each event are notified as `INVALID_OPTION` errors. The error context specifies a more precise error. This error occurs even for recognized-but-optional
    ///
    /// If either array points to invalid memory, `INVALID_MEMORY` is returned. When the `events` slice is checked for write access is unspecified.
    ///
    /// If any option is malformed, or is unrecognized and not marked as optional, `INVALID_OPTION` is returned.
    pub unsafe fn BlockOnEventsAll(
        events: KSlice<BlockingEvent>,
        general_options: KCSlice<ThreadBlockingOption>,
    ) -> SysResult;

    /// Blocks the current thread until any event specified in `events` is notified.
    /// Returns the index in the array of the event which was notified.
    ///
    /// A succesful return from this system call *synchronizes-with* the notification that completed the indicated event.
    ///
    /// If multiple events are notified simultaneously, which event unblocks the system call is unspecified. The return only *synchronizes-with* the corresponding notification.
    ///
    /// Calling this function with an empty array is equivalent to [`PauseThread`] - IE. the thread will never unblock naturally.
    /// Note that it is an error to call the function with only optional events that get ignored, ``
    ///
    /// # Blocking Timeout/Interrupted
    /// Non-wait events that wake a thread (the thread being interrupted or the blocking timeout expiring) will wake the [`BlockOnEventsAll`] system call as a whole.
    /// In this case, the system call itself returns the appropriate error
    ///
    /// # Errors
    ///
    /// Errors from each event are notified as `INVALID_OPTION` errors. The error context specifies a more precise error. This error occurs even for recognized-but-optional
    ///
    /// If either array points to invalid memory, `INVALID_MEMORY` is returned. When the `events` slice is checked for write access is unspecified.
    ///
    /// If any option is malformed, or is unrecognized and not marked as optional, `INVALID_OPTION` is returned.
    ///
    /// If every event in `events` is marked optional and is ignored,  `DEADLOCKED` is returned.
    pub unsafe fn BlockOnEventsAny(
        events: KSlice<BlockingEvent>,
        general_options: KCSlice<ThreadBlockingOption>,
    ) -> SysResult;
}
