use crate::{
    cloud_object::{Owner, Space},
    identity::LocalIdentityProvider,
};
use warpui::{AppContext, Entity, ModelContext, SingletonEntity};

pub struct UserWorkspaces;

impl UserWorkspaces {
    #[cfg(test)]
    pub fn default_mock(_ctx: &mut ModelContext<Self>) -> Self {
        Self
    }

    pub fn new(_ctx: &mut ModelContext<Self>) -> Self {
        Self
    }

    pub fn all_user_spaces(&self, _ctx: &AppContext) -> Vec<Space> {
        vec![Space::Personal]
    }

    pub fn current_user_owner(&self, ctx: &AppContext) -> Owner {
        Owner::User {
            user_uid: LocalIdentityProvider::as_ref(ctx).get().user_id(),
        }
    }

    pub fn owner_to_space(&self, owner: Owner, ctx: &AppContext) -> Space {
        match owner {
            Owner::User { user_uid } => {
                let current_user = LocalIdentityProvider::as_ref(ctx).get().user_id();
                debug_assert_eq!(user_uid, current_user);
                Space::Personal
            }
        }
    }
}

impl Entity for UserWorkspaces {
    type Event = ();
}

impl SingletonEntity for UserWorkspaces {}
