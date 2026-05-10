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
    user_id: String,
    user_email: String,
}

impl RemoteServerIdentityContext {
    pub fn new(
        remote_server_identity_key: impl Fn() -> String + Send + Sync + 'static,
        user_id: String,
        user_email: String,
    ) -> Self {
        Self {
            remote_server_identity_key: Arc::new(remote_server_identity_key),
            user_id,
            user_email,
        }
    }

    pub fn remote_server_identity_key(&self) -> String {
        (self.remote_server_identity_key)()
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn user_email(&self) -> &str {
        &self.user_email
    }
}
