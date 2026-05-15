use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    ai::{
        agent::conversation::AIConversationId,
        execution_profiles::{
            profiles::{AIExecutionProfilesModel, ClientProfileId},
            AIExecutionProfile, ActionPermission, AskUserQuestionPermission, WriteToPtyPermission,
        },
    },
    settings::AgentModeCommandExecutionPredicate,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use warp_completer::parsers::simple::decompose_command;
use warp_util::path::EscapeChar;
use warpui::{AppContext, Entity, EntityId, ModelContext, SingletonEntity};

use super::BlocklistAIHistoryModel;

/// Whether or not a command can be auto-executed, along with a detailed reason.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum CommandExecutionPermission {
    Allowed(CommandExecutionPermissionAllowedReason),
    Denied(CommandExecutionPermissionDeniedReason),
}

/// Why a command can be auto-executed.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum CommandExecutionPermissionAllowedReason {
    Dispatched,
    ExplicitlyAllowlisted,
    IsReadOnlyAndSettingEnabled,
    AgentDecided,
    AlwaysAllowed,
    RunToCompletion,
}

/// Why a command can't be auto-executed.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum CommandExecutionPermissionDeniedReason {
    AutonomyForceDisabled,
    AlwaysAskEnabled,
    ExplicitlyDenylisted,
    ContainsRedirection,
    AgentDecided,
}

impl CommandExecutionPermission {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed(..))
    }
}

/// Whether or not a file can be auto-read, along with a detailed reason.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum FileReadPermission {
    Allowed(FileReadPermissionAllowedReason),
    Denied(FileReadPermissionDeniedReason),
}

/// Why a file can be auto-read.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum FileReadPermissionAllowedReason {
    Dispatched,
    AlreadyReadInConvo,
    ExplicitlyAllowlisted,
    AutoreadSettingEnabled,
    AgentDecided,
    RunToCompletion,
}

/// Why a file can't be auto-read.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum FileReadPermissionDeniedReason {
    AutonomyForceDisabled,
    AlwaysAskEnabled,
    AgentDecided,
}

impl FileReadPermission {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed(..))
    }
}

/// Whether or not a file can be auto-written, along with a detailed reason.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum FileWritePermission {
    Allowed(FileWritePermissionAllowedReason),
    Denied(FileWritePermissionDeniedReason),
}

impl FileWritePermission {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed(..))
    }
}

/// Why a file can be written automatically.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum FileWritePermissionAllowedReason {
    Dispatched,
    AgentDecided,
    AutowriteSettingEnabled,
    RunToCompletion,
}

/// Why a file can't be written automatically.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum FileWritePermissionDeniedReason {
    AutonomyForceDisabled,
    AlwaysAskEnabled,
    AgentDecided,
}

pub struct BlocklistAIPermissions {
    /// A set of one-off files that the user has allowed Agent Mode
    /// to read for the duration of a given conversation.
    ///
    /// TODO: remove this once AM doesn't re-request access to the same file in a given convo.
    temporary_file_permissions: HashMap<AIConversationId, HashSet<PathBuf>>,
}

impl BlocklistAIPermissions {
    pub fn new(_ctx: &mut ModelContext<Self>) -> Self {
        Self {
            temporary_file_permissions: Default::default(),
        }
    }

    pub fn permissions_profile_for_id(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> AIExecutionProfile {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        let profile = profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx));
        let profile_data = profile.data();

        AIExecutionProfile {
            apply_code_diffs: self.get_apply_code_diffs_setting_for_profile(ctx, profile_id),
            read_files: self.get_read_files_setting_for_profile(ctx, profile_id),
            execute_commands: self.get_execute_commands_setting_for_profile(ctx, profile_id),
            write_to_pty: self.get_write_to_pty_setting_for_profile(ctx, profile_id),
            command_allowlist: self.get_execute_commands_allowlist_for_profile(ctx, profile_id),
            command_denylist: self.get_execute_commands_denylist_for_profile(ctx, profile_id),
            directory_allowlist: self.get_read_files_allowlist_for_profile(ctx, profile_id),
            computer_use: self.get_computer_use_setting_for_profile(ctx, profile_id),
            ask_user_question: self.get_ask_user_question_setting_for_profile(ctx, profile_id),

            name: profile_data.name.clone(),
            is_default_profile: profile_data.is_default_profile,
            web_search_enabled: profile_data.web_search_enabled,
        }
    }

    pub fn get_apply_code_diffs_setting_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> ActionPermission {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .apply_code_diffs
    }

    pub fn get_apply_code_diffs_setting(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> ActionPermission {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);

        self.get_apply_code_diffs_setting_for_profile(ctx, *active_profile.id())
    }

    pub fn get_read_files_setting_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> ActionPermission {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .read_files
    }

    pub fn get_read_files_setting(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> ActionPermission {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);
        self.get_read_files_setting_for_profile(ctx, *active_profile.id())
    }

    pub fn get_read_files_allowlist_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> Vec<PathBuf> {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .directory_allowlist
            .clone()
    }

    pub fn get_read_files_allowlist(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> Vec<PathBuf> {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);
        self.get_read_files_allowlist_for_profile(ctx, *active_profile.id())
    }

    pub fn get_execute_commands_setting_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> ActionPermission {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .execute_commands
    }

    pub fn get_execute_commands_setting(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> ActionPermission {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);
        self.get_execute_commands_setting_for_profile(ctx, *active_profile.id())
    }

    pub fn get_execute_commands_allowlist_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> Vec<AgentModeCommandExecutionPredicate> {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .command_allowlist
            .clone()
    }

    pub fn get_execute_commands_allowlist(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> Vec<AgentModeCommandExecutionPredicate> {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);
        self.get_execute_commands_allowlist_for_profile(ctx, *active_profile.id())
    }

    pub fn get_execute_commands_denylist_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> Vec<AgentModeCommandExecutionPredicate> {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .command_denylist
            .clone()
    }

    pub fn get_execute_commands_denylist(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> Vec<AgentModeCommandExecutionPredicate> {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);
        self.get_execute_commands_denylist_for_profile(ctx, *active_profile.id())
    }

    pub fn get_write_to_pty_setting_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> WriteToPtyPermission {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .write_to_pty
    }

    pub fn get_write_to_pty_setting(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> WriteToPtyPermission {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);
        self.get_write_to_pty_setting_for_profile(ctx, *active_profile.id())
    }

    pub fn can_write_to_pty(
        &self,
        conversation_id: &AIConversationId,
        terminal_view_id: Option<EntityId>,
        ctx: &AppContext,
    ) -> WriteToPtyPermission {
        if BlocklistAIHistoryModel::as_ref(ctx)
            .conversation(conversation_id)
            .is_some_and(|convo| convo.autoexecute_any_action())
        {
            return WriteToPtyPermission::AlwaysAllow;
        }
        self.get_write_to_pty_setting(ctx, terminal_view_id)
    }

    pub fn get_computer_use_setting_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> crate::ai::execution_profiles::ComputerUsePermission {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .computer_use
    }

    pub fn get_computer_use_setting(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> crate::ai::execution_profiles::ComputerUsePermission {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);
        self.get_computer_use_setting_for_profile(ctx, *active_profile.id())
    }

    pub fn get_ask_user_question_setting_for_profile(
        &self,
        ctx: &AppContext,
        profile_id: ClientProfileId,
    ) -> AskUserQuestionPermission {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        profiles_model
            .get_profile_by_id(profile_id, ctx)
            .unwrap_or_else(|| profiles_model.default_profile(ctx))
            .data()
            .ask_user_question
    }

    pub fn get_ask_user_question_setting(
        &self,
        ctx: &AppContext,
        terminal_view_id: Option<EntityId>,
    ) -> AskUserQuestionPermission {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(terminal_view_id, ctx);
        self.get_ask_user_question_setting_for_profile(ctx, *active_profile.id())
    }

    /// Returns whether or not Agent Mode can auto-read the given files.
    pub fn can_read_files_with_conversation(
        &self,
        conversation_id: &AIConversationId,
        paths: Vec<PathBuf>,
        terminal_view_id: Option<EntityId>,
        ctx: &AppContext,
    ) -> FileReadPermission {
        if BlocklistAIHistoryModel::as_ref(ctx)
            .conversation(conversation_id)
            .is_some_and(|convo| convo.autoexecute_any_action())
        {
            return FileReadPermission::Allowed(FileReadPermissionAllowedReason::RunToCompletion);
        }

        self.can_read_files(Some(conversation_id), paths, terminal_view_id, ctx)
    }

    /// Returns whether or not Warp can auto-read the given files.
    pub fn can_read_files(
        &self,
        conversation_id: Option<&AIConversationId>,
        paths: Vec<PathBuf>,
        terminal_view_id: Option<EntityId>,
        ctx: &AppContext,
    ) -> FileReadPermission {
        if paths.is_empty() {
            // We can vacuously read 0 files.
            return FileReadPermission::Allowed(
                FileReadPermissionAllowedReason::ExplicitlyAllowlisted,
            );
        }

        // Check if we've already been given permission to read these files in this conversation.
        if let Some(temp_permissions) =
            conversation_id.and_then(|id| self.temporary_file_permissions.get(id))
        {
            if paths.iter().all(|path| {
                temp_permissions
                    .iter()
                    .any(|allowed| path.starts_with(allowed))
            }) {
                return FileReadPermission::Allowed(
                    FileReadPermissionAllowedReason::AlreadyReadInConvo,
                );
            }
        }

        match self.get_read_files_setting(ctx, terminal_view_id) {
            ActionPermission::AgentDecides => {
                // For now, we always read files. We don't ask the user for permission.
                FileReadPermission::Allowed(FileReadPermissionAllowedReason::AgentDecided)
            }
            ActionPermission::AlwaysAllow => {
                FileReadPermission::Allowed(FileReadPermissionAllowedReason::AutoreadSettingEnabled)
            }
            ActionPermission::AlwaysAsk => {
                let allowlisted_paths = self.get_read_files_allowlist(ctx, terminal_view_id);
                if paths
                    .iter()
                    .all(|p| allowlisted_paths.iter().any(|dir| p.starts_with(dir)))
                {
                    FileReadPermission::Allowed(
                        FileReadPermissionAllowedReason::ExplicitlyAllowlisted,
                    )
                } else {
                    FileReadPermission::Denied(FileReadPermissionDeniedReason::AlwaysAskEnabled)
                }
            }
        }
    }

    /// Returns whether or not Agent Mode can automatically write to files.
    pub fn can_write_files(
        &self,
        conversation_id: &AIConversationId,
        _paths: &[PathBuf],
        terminal_view_id: Option<EntityId>,
        ctx: &AppContext,
    ) -> FileWritePermission {
        if BlocklistAIHistoryModel::as_ref(ctx)
            .conversation(conversation_id)
            .is_some_and(|convo| convo.autoexecute_any_action())
        {
            return FileWritePermission::Allowed(FileWritePermissionAllowedReason::RunToCompletion);
        }

        self.determine_write_permissions_from_active_profile(terminal_view_id, ctx)
    }

    fn determine_write_permissions_from_active_profile(
        &self,
        terminal_view_id: Option<EntityId>,
        ctx: &AppContext,
    ) -> FileWritePermission {
        match self.get_apply_code_diffs_setting(ctx, terminal_view_id) {
            ActionPermission::AgentDecides => {
                FileWritePermission::Denied(FileWritePermissionDeniedReason::AgentDecided)
            }
            ActionPermission::AlwaysAllow => FileWritePermission::Allowed(
                FileWritePermissionAllowedReason::AutowriteSettingEnabled,
            ),
            ActionPermission::AlwaysAsk => {
                FileWritePermission::Denied(FileWritePermissionDeniedReason::AlwaysAskEnabled)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// Returns whether or not Agent Mode can auto-execute the given command.
    pub fn can_autoexecute_command(
        &self,
        conversation_id: &AIConversationId,
        command: &str,
        escape_char: EscapeChar,
        is_read_only: bool,
        is_risky: Option<bool>,
        terminal_view_id: Option<EntityId>,
        ctx: &AppContext,
    ) -> CommandExecutionPermission {
        // Normalize line continuations based on shell type.
        // POSIX shells (bash/zsh/fish) use backslash, PowerShell uses backtick.
        let normalized_command = match escape_char {
            EscapeChar::Backslash => command.replace("\\\n", " "),
            EscapeChar::Backtick => command.replace("`\n", " "),
        };

        // The command string might be composed of multiple commands so let's
        // break it up first.
        let (commands, contains_redirection) = decompose_command(&normalized_command, escape_char);

        // The denylist takes precedence over all other conditions.
        let denylist = self.get_execute_commands_denylist(ctx, terminal_view_id);
        if commands
            .iter()
            .any(|c| denylist.iter().any(|d| d.matches(c)))
        {
            return CommandExecutionPermission::Denied(
                CommandExecutionPermissionDeniedReason::ExplicitlyDenylisted,
            );
        }

        if BlocklistAIHistoryModel::as_ref(ctx)
            .conversation(conversation_id)
            .is_some_and(|convo| convo.autoexecute_any_action())
        {
            return CommandExecutionPermission::Allowed(
                CommandExecutionPermissionAllowedReason::RunToCompletion,
            );
        }

        match self.get_execute_commands_setting(ctx, terminal_view_id) {
            ActionPermission::AgentDecides => {
                if is_risky == Some(false) {
                    return CommandExecutionPermission::Allowed(
                        CommandExecutionPermissionAllowedReason::AgentDecided,
                    );
                }

                if contains_redirection {
                    return CommandExecutionPermission::Denied(
                        CommandExecutionPermissionDeniedReason::ContainsRedirection,
                    );
                }

                let allowlist = self.get_execute_commands_allowlist(ctx, terminal_view_id);
                if commands.iter().all(|command| {
                    allowlist
                        .iter()
                        .any(|allowlist_item| allowlist_item.matches(command))
                }) {
                    return CommandExecutionPermission::Allowed(
                        CommandExecutionPermissionAllowedReason::ExplicitlyAllowlisted,
                    );
                }

                // For now, the heuristic is if the command is read only or if we're executing
                // a plan. Otherwise, we don't want to autoexecute.
                if is_read_only {
                    CommandExecutionPermission::Allowed(
                        CommandExecutionPermissionAllowedReason::AgentDecided,
                    )
                } else {
                    CommandExecutionPermission::Denied(
                        CommandExecutionPermissionDeniedReason::AgentDecided,
                    )
                }
            }
            ActionPermission::AlwaysAllow => CommandExecutionPermission::Allowed(
                CommandExecutionPermissionAllowedReason::AlwaysAllowed,
            ),
            ActionPermission::AlwaysAsk => {
                let allowlist = self.get_execute_commands_allowlist(ctx, terminal_view_id);

                if commands.iter().all(|command| {
                    allowlist
                        .iter()
                        .any(|allowlist_item| allowlist_item.matches(command))
                }) {
                    CommandExecutionPermission::Allowed(
                        CommandExecutionPermissionAllowedReason::ExplicitlyAllowlisted,
                    )
                } else {
                    CommandExecutionPermission::Denied(
                        CommandExecutionPermissionDeniedReason::AlwaysAskEnabled,
                    )
                }
            }
        }
    }

    /// Sets whether or not we should always allow writing to the PTY.
    pub fn set_always_allow_write_to_pty(
        &mut self,
        enabled: bool,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) -> Result<()> {
        let permission = if enabled {
            WriteToPtyPermission::AlwaysAllow
        } else {
            WriteToPtyPermission::AlwaysAsk
        };
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(Some(terminal_view_id), ctx);
        AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
            profiles_model.set_write_to_pty(*active_profile.id(), &permission, ctx);
        });
        Ok(())
    }

    /// Sets whether or not we should always allow reading files.
    pub fn set_always_allow_read_files(
        &mut self,
        enabled: bool,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) -> Result<()> {
        let permissions = if enabled {
            ActionPermission::AlwaysAllow
        } else {
            ActionPermission::AlwaysAsk
        };

        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(Some(terminal_view_id), ctx);
        AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
            profiles_model.set_read_files(*active_profile.id(), &permissions, ctx);
        });
        Ok(())
    }

    pub fn allow_read_files_for_directory(
        &mut self,
        path: PathBuf,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) -> Result<()> {
        let active_profile =
            AIExecutionProfilesModel::as_ref(ctx).active_profile(Some(terminal_view_id), ctx);
        AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
            profiles_model.set_read_files(*active_profile.id(), &ActionPermission::AlwaysAsk, ctx);
            profiles_model.add_to_directory_allowlist(*active_profile.id(), &path, ctx);
        });
        Ok(())
    }

    /// Gives Agent Mode temporary access to the provided `files`.
    /// The permissions are scoped to the given conversation.
    pub fn add_temporary_file_read_permissions<P: Into<PathBuf>>(
        &mut self,
        conversation_id: AIConversationId,
        files: impl IntoIterator<Item = P>,
    ) {
        self.temporary_file_permissions
            .entry(conversation_id)
            .or_default()
            .extend(files.into_iter().map(Into::into));
    }

    /// Returns whether the agent can ask the user a question in the given conversation.
    pub fn can_ask_user_question(
        &self,
        conversation_id: &AIConversationId,
        terminal_view_id: Option<EntityId>,
        ctx: &AppContext,
    ) -> bool {
        match self.get_ask_user_question_setting(ctx, terminal_view_id) {
            AskUserQuestionPermission::Never => false,
            AskUserQuestionPermission::AskExceptInAutoApprove => {
                !BlocklistAIHistoryModel::as_ref(ctx)
                    .conversation(conversation_id)
                    .is_some_and(|convo| convo.autoexecute_any_action())
            }
            AskUserQuestionPermission::AlwaysAsk => true,
        }
    }
}

impl Entity for BlocklistAIPermissions {
    type Event = ();
}

impl SingletonEntity for BlocklistAIPermissions {}

/// Returns true iff Agent Mode autonomy features are allowed on this client.
/// Granular permissions still need to be checked for specific autonomy features
/// (e.g. whether a command is auto-executable).
pub fn is_agent_mode_autonomy_allowed(_ctx: &AppContext) -> bool {
    true
}

#[cfg(test)]
#[path = "permissions_test.rs"]
mod tests;
