use alloc::{string::String, vec::Vec};

use crate::{
    fs::PathBuf,
    security::SecurityContext,
    sys::{
        fs::FileHandle, handle::HandlePtr, isolation::NamespaceHandle, kstr::KStrCPtr,
        process::EnvironmentMapHandle,
    },
};

pub struct Command {
    resolution_base: HandlePtr<FileHandle>,
    cmd: PathBuf,
    env: HandlePtr<EnvironmentMapHandle>,
    namespace: HandlePtr<NamespaceHandle>,
    start_security_context: HandlePtr<SecurityContext>,
    args: Vec<String>,
}
