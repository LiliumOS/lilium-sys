use super::{
    handle::{Handle, HandlePtr},
    result::SysResult,
};

#[repr(transparent)]
pub struct SocketHandle(Handle);

#[repr(C)]
pub struct sockaddr {}

#[allow(improper_ctypes)]
extern "C" {
    pub fn CreateServerSocket(servout: *mut HandlePtr<SocketHandle>) -> SysResult;

    pub fn ConnectAnon(
        sockout: *mut HandlePtr<SocketHandle>,
        server: HandlePtr<SocketHandle>,
    ) -> SysResult;

}
