use super::{
    handle::{Handle, HandlePtr},
    result::SysResult,
};

#[repr(transparent)]
pub struct SocketHandle(Handle);

#[repr(transparent)]
pub struct ServerHandle(Handle);

#[repr(C)]
pub struct sockaddr {}

#[cfg(any(feature = "io", doc))]
unsafe extern "system" {
    pub fn CreateServerSocket(servout: *mut HandlePtr<ServerHandle>) -> SysResult;

    pub fn ConnectAnon(
        sockout: *mut HandlePtr<SocketHandle>,
        server: HandlePtr<ServerHandle>,
    ) -> SysResult;

}
