use std::sync::Arc;

use remote_server::identity::RemoteServerIdentityContext;

use crate::identity::local_identity::LocalIdentity;

pub fn remote_server_identity_context(
    local_identity: Arc<LocalIdentity>,
) -> RemoteServerIdentityContext {
    RemoteServerIdentityContext::new(move || remote_server_identity_key(&local_identity))
}

fn remote_server_identity_key(local_identity: &LocalIdentity) -> String {
    local_identity.user_id().as_string()
}
