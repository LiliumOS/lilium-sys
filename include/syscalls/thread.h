#define SUBSYSTEM_BASE (1 << 12)

#define SYS_StartThread (0)
// pub fn StartThread(
//     tsc: *const ThreadStartContext,
//     thout: *mut HandlePtr<ThreadHandle>,
//     options: KCSlice<ThreadStartOption>,
// ) -> SysResult;
// pub safe fn ParkThread() -> SysResult;
// pub fn UnparkThread(th: HandlePtr<ThreadHandle>) -> SysResult;
// pub safe fn YieldThread();
// pub fn AwaitAddress(addr: *mut c_void) -> SysResult;
// pub fn NotifyOne(addr: *mut c_void) -> SysResult;
// pub fn NotifyAll(addr: *mut c_void) -> SysResult;
// pub fn SetBlockingTimeout(dur: *const Duration) -> SysResult;
// pub fn SleepThread(dur: *const Duration) -> SysResult;
// pub safe fn PauseThread() -> SysResult;
// pub fn InterruptThread(th: HandlePtr<ThreadHandle>) -> SysResult;
// pub safe fn Interrupted() -> SysResult;
// pub safe fn ClearBlockingTimeout();
// pub fn ThreadExit(thr: c_int) -> !;
// pub fn GetCurrentThread() -> HandlePtr<ThreadHandle>;
// pub fn GetTLSBaseAddr(th: HandlePtr<ThreadHandle>, addrout: *mut *mut c_void) -> SysResult;
// pub fn SetTLSBaseAddr(th: HandlePtr<ThreadHandle>, addr: *mut c_void) -> SysResult;
// pub fn JoinThread(th: HandlePtr<ThreadHandle>) -> SysResult;
// pub fn DetachThread(th: HandlePtr<ThreadHandle>) -> SysResult;

// pub fn SendHandle(toth: HandlePtr<ThreadHandle>, sendhdl: HandlePtr<Handle>) -> SysResult;
// pub fn RecieveHandle(out: *mut HandlePtr<Handle>) -> SysResult;

// pub fn SetThreadName(th: HandlePtr<ThreadHandle>, name: KStrCPtr) -> SysResult;
// pub fn GetThreadName(th: HandlePtr<ThreadHandle>, name: *mut KStrPtr) -> SysResult;

// /// Sets the current thread to be the "control" thread of the current process.
// /// When the control thread exits or is killed by an exception, each other
// pub safe fn ControlProcessExit() -> SysResult;
// pub safe fn ReleaseProcessExit() -> SysResult;