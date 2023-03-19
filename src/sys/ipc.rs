use core::ffi::{c_long, c_void};

use super::{
    fs::FileHandle,
    handle::{Handle, HandlePtr},
    io::IOHandle,
    kstr::{KStrCPtr, KStrPtr},
    result::SysResult,
    thread::ThreadHandle,
};

#[repr(transparent)]
pub struct IPCConnectionHandle(Handle);

#[repr(transparent)]
pub struct IPCServerHandle(Handle);

/// These bits are set to one of the various MODE_ values indicated below
pub const FLAG_MODE_MASK: c_long = 0x3;
/// Hidden Channel - requires IPC_CONNECT_PRIVATE to the thread to open and IPC_DISCOVER_HIDDEN to refer to the channel by name (or to find the channel via EnumerateOnThread/EnumerateOnProcess).
/// Not available via `EnumerateByName`
pub const MODE_HIDDEN: c_long = 0x00;
/// Private Channel - requires IPC_CONNECT_PRIVATE to the thread to open or to refer to by name
pub const MODE_PRIVATE: c_long = 0x01;
/// Normal Channel - requires IPC_CONNECT to the thread to open or refer to by name
pub const MODE_REGULAR: c_long = 0x02;
/// Public Channel - requires IPC_CONNECT_PUBLIC to the thread to open or refer to by name.
pub const MODE_PUBLIC: c_long = 0x03;
/// If set, then the thread owning the IPC Channel Server is interrupted whether a client connects to the IPC Channel
pub const FLAG_INTERRUPT_ON_CONNECT: c_long = 0x04;
/// If non-zero, set to the signal value the thread recieves when a client connects to the IPC Channel
/// Value is just the SIG* constants shifted right by 3.
pub const SIGNAL_ON_CONNECT_MASK: c_long = 0xF8;

#[allow(improper_ctypes)]
extern "C" {
    pub fn OpenIPCServer(
        flags: c_long,
        common_name: KStrCPtr,
        handle_out: *mut HandlePtr<IPCServerHandle>,
    ) -> SysResult;
    pub fn AwaitConnection(
        server: HandlePtr<IPCServerHandle>,
        client: *mut HandlePtr<IPCConnectionHandle>,
    ) -> SysResult;
    pub fn PollConnect(
        server: HandlePtr<IPCServerHandle>,
        client: *mut HandlePtr<IPCConnectionHandle>,
    ) -> SysResult;
    pub fn ConnectTo(
        th: HandlePtr<ThreadHandle>,
        common_name: KStrCPtr,
        handle_out: *mut HandlePtr<IPCConnectionHandle>,
    ) -> SysResult;
    pub fn EnumerateOnThread(th: HandlePtr<ThreadHandle>, state: *mut *mut c_void) -> SysResult;
    pub fn EnumerateNext(th: HandlePtr<ThreadHandle>, state: *mut *mut c_void) -> SysResult;
    pub fn EnumerateGet(
        th: HandlePtr<ThreadHandle>,
        state: *mut *mut c_void,
        handle_out: *mut HandlePtr<IPCConnectionHandle>,
        name_out: KStrPtr,
    ) -> SysResult;

    pub fn ConnectToNamed(
        handle_out: *mut HandlePtr<IPCConnectionHandle>,
        resolution_base: HandlePtr<FileHandle>,
        name: KStrCPtr,
    ) -> SysResult;
    pub fn ConnectToFile(
        handle_out: *mut HandlePtr<IPCConnectionHandle>,
        file: HandlePtr<FileHandle>,
    ) -> SysResult;

    pub fn IsIPCConnection(iohdl: HandlePtr<IOHandle>) -> SysResult;
}
