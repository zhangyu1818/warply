use core::fmt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};
use warp_core::user_preferences::GetUserPreferences;
use warpui::{AppContext, Entity, EntityId, ModelContext, SingletonEntity};

use crate::LaunchMode;

use crate::settings::AgentModeCommandExecutionPredicate;

use super::{AIExecutionProfile, ActionPermission, WriteToPtyPermission};

const DEFAULT_PROFILE_PREF_KEY: &str = "AIExecutionProfile.Default";

/// ExecutionProfileId is the identifier that users of the AIExecutionProfilesModel use
/// to refer back to a specific profile. These are unique across the lifespan of the app.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClientProfileId(usize);

impl ClientProfileId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> ClientProfileId {
        static NEXT_PROFILE_ID: AtomicUsize = AtomicUsize::new(0);
        let raw = NEXT_PROFILE_ID.fetch_add(1, Ordering::Relaxed);
        ClientProfileId(raw)
    }
}

impl fmt::Display for ClientProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Debug)]
pub struct AIExecutionProfileInfo {
    id: ClientProfileId,
    data: AIExecutionProfile,
}

impl AIExecutionProfileInfo {
    pub fn id(&self) -> &ClientProfileId {
        &self.id
    }

    pub fn data(&self) -> &AIExecutionProfile {
        &self.data
    }
}

#[derive(Clone, Debug)]
pub struct DefaultProfileState {
    id: ClientProfileId,
    profile: AIExecutionProfile,
}

impl std::fmt::Display for DefaultProfileState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Local")
    }
}

impl DefaultProfileState {
    pub fn id(&self) -> ClientProfileId {
        self.id
    }
}

pub struct AIExecutionProfilesModel {
    default_profile_state: DefaultProfileState,
    active_profiles_per_session: HashMap<EntityId, ClientProfileId>,
}

impl AIExecutionProfilesModel {
    pub fn new(_launch_mode: &LaunchMode, ctx: &mut ModelContext<Self>) -> Self {
        let active_profiles_per_session: HashMap<EntityId, ClientProfileId> = HashMap::new();

        let default_profile_state = DefaultProfileState {
            id: ClientProfileId::new(),
            profile: Self::read_local_default_profile(ctx),
        };

        log::info!("Initialized execution profile model with state: {default_profile_state}",);

        Self {
            default_profile_state,
            active_profiles_per_session,
        }
    }

    fn read_local_default_profile(ctx: &AppContext) -> AIExecutionProfile {
        ctx.private_user_preferences()
            .read_value(DEFAULT_PROFILE_PREF_KEY)
            .ok()
            .flatten()
            .and_then(|value| serde_json::from_str::<AIExecutionProfile>(&value).ok())
            .unwrap_or_else(AIExecutionProfile::default_profile)
    }

    fn persist_local_default_profile(&self, ctx: &mut ModelContext<Self>) {
        let Ok(value) = serde_json::to_string(&self.default_profile_state.profile) else {
            log::error!("Failed to serialize local AI execution profile");
            return;
        };

        if let Err(e) = ctx
            .private_user_preferences()
            .write_value(DEFAULT_PROFILE_PREF_KEY, value)
        {
            log::error!("Failed to persist local AI execution profile: {e}");
        }
    }

    pub fn delete_profile(&mut self, profile_id: ClientProfileId, ctx: &mut ModelContext<Self>) {
        let id = self.default_profile_state.id();
        if id == profile_id {
            log::warn!("Attempted to delete default profile (id: {profile_id})");
            return;
        }

        self.active_profiles_per_session
            .retain(|_, active_profile_id| *active_profile_id != profile_id);

        ctx.emit(AIExecutionProfilesModelEvent::ProfileDeleted);
    }

    pub fn active_profile(
        &self,
        terminal_view_id: Option<EntityId>,
        ctx: &AppContext,
    ) -> AIExecutionProfileInfo {
        terminal_view_id
            .and_then(|id| self.active_profiles_per_session.get(&id))
            .and_then(|profile_id| self.get_profile_by_id(*profile_id, ctx))
            .unwrap_or_else(|| self.default_profile(ctx))
    }

    #[cfg(test)]
    pub fn default_profile_id(&self) -> ClientProfileId {
        self.default_profile_state.id()
    }

    pub fn default_profile(&self, _ctx: &AppContext) -> AIExecutionProfileInfo {
        AIExecutionProfileInfo {
            id: self.default_profile_state.id,
            data: self.default_profile_state.profile.clone(),
        }
    }

    /// Sets the active profile for a specific terminal view.
    pub fn set_active_profile(
        &mut self,
        terminal_view_id: EntityId,
        profile_id: ClientProfileId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.active_profiles_per_session
            .insert(terminal_view_id, profile_id);
        ctx.emit(AIExecutionProfilesModelEvent::UpdatedActiveProfile { terminal_view_id });
    }

    /// Returns a profile by its client ID.
    /// Returns None if the profile is not found.
    pub fn get_profile_by_id(
        &self,
        profile_id: ClientProfileId,
        _ctx: &AppContext,
    ) -> Option<AIExecutionProfileInfo> {
        if profile_id == self.default_profile_state.id {
            return Some(AIExecutionProfileInfo {
                id: self.default_profile_state.id,
                data: self.default_profile_state.profile.clone(),
            });
        }
        None
    }

    pub fn set_apply_code_diffs(
        &mut self,
        profile_id: ClientProfileId,
        apply_code_diffs: &ActionPermission,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                if profile.apply_code_diffs != *apply_code_diffs {
                    profile.apply_code_diffs = *apply_code_diffs;
                    return true;
                }
                false
            },
            ctx,
        );
    }

    pub fn set_read_files(
        &mut self,
        profile_id: ClientProfileId,
        read_files: &ActionPermission,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                if profile.read_files != *read_files {
                    profile.read_files = *read_files;
                    return true;
                }
                false
            },
            ctx,
        );
    }

    pub fn set_execute_commands(
        &mut self,
        profile_id: ClientProfileId,
        execute_commands: &ActionPermission,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                if profile.execute_commands != *execute_commands {
                    profile.execute_commands = *execute_commands;
                    return true;
                }
                false
            },
            ctx,
        );
    }

    pub fn set_write_to_pty(
        &mut self,
        profile_id: ClientProfileId,
        write_to_pty: &WriteToPtyPermission,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                if profile.write_to_pty != *write_to_pty {
                    profile.write_to_pty = *write_to_pty;
                    return true;
                }
                false
            },
            ctx,
        );
    }

    pub fn set_computer_use(
        &mut self,
        profile_id: ClientProfileId,
        permission: &super::ComputerUsePermission,
        ctx: &mut ModelContext<Self>,
    ) {
        let current_value = self
            .get_profile_by_id(profile_id, ctx)
            .map(|p| p.data().computer_use);

        self.edit_profile_internal(
            profile_id,
            |profile| {
                if profile.computer_use != *permission {
                    profile.computer_use = *permission;
                    return true;
                }
                false
            },
            ctx,
        );

        if current_value != Some(*permission) {}
    }

    pub fn set_ask_user_question(
        &mut self,
        profile_id: ClientProfileId,
        permission: super::AskUserQuestionPermission,
        ctx: &mut ModelContext<Self>,
    ) {
        let current_value = self
            .get_profile_by_id(profile_id, ctx)
            .map(|p| p.data().ask_user_question);

        self.edit_profile_internal(
            profile_id,
            |profile| {
                if profile.ask_user_question != permission {
                    profile.ask_user_question = permission;
                    return true;
                }
                false
            },
            ctx,
        );

        if current_value != Some(permission) {}
    }

    pub fn set_profile_name(
        &mut self,
        profile_id: ClientProfileId,
        name: &str,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                if profile.name != name {
                    profile.name = name.to_string();
                    return true;
                }
                false
            },
            ctx,
        );
    }

    pub fn add_to_command_allowlist(
        &mut self,
        profile_id: ClientProfileId,
        predicate: &AgentModeCommandExecutionPredicate,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                if !profile.command_allowlist.contains(predicate) {
                    profile.command_allowlist.push(predicate.clone());
                    return true;
                }
                false
            },
            ctx,
        );
    }

    pub fn remove_from_command_allowlist(
        &mut self,
        profile_id: ClientProfileId,
        predicate: &AgentModeCommandExecutionPredicate,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                let original_len = profile.command_allowlist.len();
                profile.command_allowlist.retain(|p| p != predicate);
                profile.command_allowlist.len() != original_len
            },
            ctx,
        );
    }

    pub fn add_to_directory_allowlist(
        &mut self,
        profile_id: ClientProfileId,
        path: &PathBuf,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                if !profile.directory_allowlist.contains(path) {
                    profile.directory_allowlist.push(path.clone());
                    return true;
                }
                false
            },
            ctx,
        );
    }

    pub fn remove_from_directory_allowlist(
        &mut self,
        profile_id: ClientProfileId,
        path: &PathBuf,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                let original_len = profile.directory_allowlist.len();
                profile.directory_allowlist.retain(|p| p != path);
                profile.directory_allowlist.len() != original_len
            },
            ctx,
        );
    }

    pub fn add_to_command_denylist(
        &mut self,
        profile_id: ClientProfileId,
        predicate: &AgentModeCommandExecutionPredicate,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                if !profile.command_denylist.contains(predicate) {
                    profile.command_denylist.push(predicate.clone());
                    return true;
                }
                false
            },
            ctx,
        );
    }

    pub fn remove_from_command_denylist(
        &mut self,
        profile_id: ClientProfileId,
        predicate: &AgentModeCommandExecutionPredicate,
        ctx: &mut ModelContext<Self>,
    ) {
        self.edit_profile_internal(
            profile_id,
            |profile| {
                let original_len = profile.command_denylist.len();
                profile.command_denylist.retain(|p| p != predicate);
                profile.command_denylist.len() != original_len
            },
            ctx,
        );
    }

    fn edit_profile_internal(
        &mut self,
        profile_id: ClientProfileId,
        edit_fn: impl FnOnce(&mut AIExecutionProfile) -> bool,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        if self.default_profile_state.id == profile_id {
            let mut new_profile = self.default_profile_state.profile.clone();
            let value_changed = edit_fn(&mut new_profile);
            if !value_changed {
                return false;
            }

            self.default_profile_state.profile = new_profile;
            self.persist_local_default_profile(ctx);
            ctx.emit(AIExecutionProfilesModelEvent::ProfileUpdated(profile_id));
            return true;
        }

        false
    }
}

#[allow(clippy::enum_variant_names)]
pub enum AIExecutionProfilesModelEvent {
    ProfileUpdated(ClientProfileId),
    ProfileDeleted,
    UpdatedActiveProfile { terminal_view_id: EntityId },
}

impl Entity for AIExecutionProfilesModel {
    type Event = AIExecutionProfilesModelEvent;
}

impl SingletonEntity for AIExecutionProfilesModel {}
