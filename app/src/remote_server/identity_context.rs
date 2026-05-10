use std::sync::Arc;

use remote_server::identity::RemoteServerIdentityContext;

use crate::identity::local_identity::LocalIdentity;

pub fn server_api_identity_context(
    local_identity: Arc<LocalIdentity>,
) -> RemoteServerIdentityContext {
    let identity_local_identity = local_identity.clone();
    let user_id_local_identity = local_identity.clone();
    let user_email_local_identity = local_identity;

    let user_id = user_id_local_identity
        .user_id()
        .map(|uid| uid.as_string())
        .unwrap_or_default();
    let user_email = user_email_local_identity.user_email().unwrap_or_default();

    RemoteServerIdentityContext::new(
        move || remote_server_identity_key(&identity_local_identity),
        user_id,
        user_email,
    )
}

fn remote_server_identity_key(local_identity: &LocalIdentity) -> String {
    local_identity
        .user_id()
        .map(|uid| uid.as_string())
        .unwrap_or_else(|| "local".to_owned())
}
