use std::sync::Arc;
type RemoteServerIdentityKeyFn = dyn Fn() -> String + Send + Sync;
/// App-supplied identity and preference context for transport-agnostic
/// remote-server code.
///
/// Identity keys are non-secret stable partition keys used to select the
/// remote daemon's socket/PID directory.
#[derive(Clone)]
pub struct RemoteServerIdentityContext {
    remote_server_identity_key: Arc<RemoteServerIdentityKeyFn>,
}

impl RemoteServerIdentityContext {
    pub fn new(remote_server_identity_key: impl Fn() -> String + Send + Sync + 'static) -> Self {
        Self {
            remote_server_identity_key: Arc::new(remote_server_identity_key),
        }
    }

    pub fn remote_server_identity_key(&self) -> String {
        (self.remote_server_identity_key)()
    }
}
