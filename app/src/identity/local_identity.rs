use std::sync::Arc;

use parking_lot::RwLock;
use warpui::{AppContext, Entity, SingletonEntity};

use super::{user::User, UserUid};

pub struct LocalIdentity {
    user: RwLock<User>,
}

impl LocalIdentity {
    fn new(_ctx: &AppContext) -> Self {
        Self {
            user: RwLock::new(User::local()),
        }
    }

    #[cfg(any(test, feature = "integration_tests"))]
    pub fn new_for_test() -> Self {
        Self {
            user: RwLock::new(User::test()),
        }
    }

    #[cfg_attr(target_family = "wasm", allow(dead_code))]
    pub fn initialize(ctx: &AppContext) -> Self {
        Self::new(ctx)
    }

    pub fn username_for_display(&self) -> Option<String> {
        Some(self.user.read().username_for_display().to_owned())
    }

    pub fn user_email(&self) -> Option<String> {
        Some(self.user.read().metadata.email.clone())
    }

    pub fn user_id(&self) -> Option<UserUid> {
        Some(self.user.read().local_id)
    }
}

pub struct LocalIdentityProvider {
    local_identity: Arc<LocalIdentity>,
}

impl LocalIdentityProvider {
    pub fn new(local_identity: Arc<LocalIdentity>) -> Self {
        Self { local_identity }
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self {
            local_identity: Arc::new(LocalIdentity::new_for_test()),
        }
    }

    pub fn get(&self) -> &Arc<LocalIdentity> {
        &self.local_identity
    }
}

impl Entity for LocalIdentityProvider {
    type Event = ();
}

impl SingletonEntity for LocalIdentityProvider {}
