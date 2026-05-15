use std::path::PathBuf;

use crate::settings::AISettings;
use crate::settings::{
    AgentModeCommandExecutionPredicate, DEFAULT_COMMAND_EXECUTION_ALLOWLIST,
    DEFAULT_COMMAND_EXECUTION_DENYLIST,
};
use serde::{Deserialize, Serialize};
use warpui::{AppContext, SingletonEntity};

pub const PROFILE_NAME_MAX_LENGTH: usize = 50;

pub mod editor;
pub mod profiles;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionPermission {
    AgentDecides,
    AlwaysAllow,
    AlwaysAsk,
}

impl ActionPermission {
    pub fn description(&self) -> &'static str {
        match self {
            ActionPermission::AgentDecides => {
                "The Agent chooses the safest path: acting on its own when confident, and asking for approval when uncertain."
            }
            ActionPermission::AlwaysAllow => {
                "Give the Agent full autonomy  — no manual approval ever required."
            }
            ActionPermission::AlwaysAsk => {
                "Require explicit approval before the Agent takes any action."
            }
        }
    }

    pub fn is_always_ask(&self) -> bool {
        matches!(self, Self::AlwaysAsk)
    }

    pub fn is_always_allow(&self) -> bool {
        matches!(self, Self::AlwaysAllow)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteToPtyPermission {
    AlwaysAllow,
    #[default]
    AlwaysAsk,
    AskOnFirstWrite,
}

impl WriteToPtyPermission {
    pub fn description(&self) -> &'static str {
        match self {
            WriteToPtyPermission::AlwaysAllow => ActionPermission::AlwaysAllow.description(),
            WriteToPtyPermission::AskOnFirstWrite => {
                "The agent will ask for permission the first time it needs to interact with a running command. After that, it will continue automatically for the rest of that command."
            }
            WriteToPtyPermission::AlwaysAsk => {
                "The agent will always ask for permission to interact with a running command."
            }
        }
    }

    pub fn is_always_allow(&self) -> bool {
        matches!(self, Self::AlwaysAllow)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputerUsePermission {
    #[default]
    Never,
    AlwaysAsk,
    AlwaysAllow,
}

impl ComputerUsePermission {
    pub fn description(&self) -> &'static str {
        match self {
            ComputerUsePermission::Never => {
                "Computer use tools are disabled and will not be available to the Agent."
            }
            ComputerUsePermission::AlwaysAsk => {
                "Require explicit approval before the Agent uses computer use tools."
            }
            ComputerUsePermission::AlwaysAllow => {
                "Give the Agent full autonomy to use computer use tools without approval."
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Never)
    }

    pub fn is_always_allow(&self) -> bool {
        matches!(self, Self::AlwaysAllow)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AskUserQuestionPermission {
    /// Never pause; skip questions and continue with best judgment.
    Never,
    /// Pause and wait for the user, unless auto-approve mode is enabled.
    #[default]
    AskExceptInAutoApprove,
    /// Always pause and wait for the user to answer before continuing, even in auto-approve mode.
    AlwaysAsk,
}

impl AskUserQuestionPermission {
    pub fn description(&self) -> &'static str {
        match self {
            AskUserQuestionPermission::AskExceptInAutoApprove => {
                "The Agent may ask a question and pause for your response, but will continue automatically when auto-approve is on."
            }
            AskUserQuestionPermission::Never => {
                "The Agent will not ask questions and will continue with its best judgment."
            }
            AskUserQuestionPermission::AlwaysAsk => {
                "The Agent may ask a question and will pause for your response even when auto-approve is on."
            }
        }
    }
}

/// Core data structure representing an AI execution profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AIExecutionProfile {
    pub name: String,
    pub is_default_profile: bool,
    pub apply_code_diffs: ActionPermission,
    pub read_files: ActionPermission,

    pub execute_commands: ActionPermission,
    pub write_to_pty: WriteToPtyPermission,
    pub ask_user_question: AskUserQuestionPermission,

    /// Always ask for permission for these commands
    pub command_denylist: Vec<AgentModeCommandExecutionPredicate>,

    /// When the execute_commands is set to AlwaysAsk, autoexecute these commands
    pub command_allowlist: Vec<AgentModeCommandExecutionPredicate>,

    /// When the read_files is set to AlwaysAsk, autoread from these directories
    pub directory_allowlist: Vec<PathBuf>,

    pub computer_use: ComputerUsePermission,

    /// Whether the agent may use web search when helpful for completing tasks
    pub web_search_enabled: bool,
}

impl Default for AIExecutionProfile {
    fn default() -> Self {
        Self {
            name: Default::default(),
            is_default_profile: false,
            apply_code_diffs: ActionPermission::AgentDecides,
            read_files: ActionPermission::AgentDecides,
            execute_commands: ActionPermission::AlwaysAsk,
            write_to_pty: WriteToPtyPermission::AlwaysAsk,
            ask_user_question: AskUserQuestionPermission::AskExceptInAutoApprove,
            command_denylist: DEFAULT_COMMAND_EXECUTION_DENYLIST.clone(),
            command_allowlist: Vec::new(),
            directory_allowlist: Vec::new(),
            computer_use: ComputerUsePermission::Never,
            web_search_enabled: true,
        }
    }
}

impl AIExecutionProfile {
    pub fn create_default_from_settings(app: &AppContext) -> Self {
        let ai_settings = AISettings::as_ref(app);
        Self {
            name: "Default".to_string(),
            is_default_profile: true,
            command_denylist: ai_settings.agent_mode_command_execution_denylist.clone(),
            command_allowlist: ai_settings
                .agent_mode_command_execution_allowlist
                .iter()
                .filter(|cmd| !DEFAULT_COMMAND_EXECUTION_ALLOWLIST.contains(cmd))
                .cloned()
                .collect(),
            directory_allowlist: ai_settings.agent_mode_coding_file_read_allowlist.clone(),
            ..Default::default()
        }
    }

    /// This creates a CLI-specific profile that will never ask the user for permission,
    /// since we cannot do so in a non-interactive setting.
    pub fn create_default_cli_profile(
        is_sandboxed: bool,
        computer_use_override: Option<bool>,
    ) -> Self {
        let command_denylist = if is_sandboxed {
            Vec::new()
        } else {
            DEFAULT_COMMAND_EXECUTION_DENYLIST.to_vec()
        };

        let computer_use_permission = match computer_use_override {
            Some(true) => ComputerUsePermission::AlwaysAllow,
            Some(false) => ComputerUsePermission::Never,
            None => {
                if is_sandboxed {
                    ComputerUsePermission::AlwaysAllow
                } else {
                    ComputerUsePermission::Never
                }
            }
        };

        Self {
            name: "Default (CLI)".to_owned(),
            is_default_profile: true,
            apply_code_diffs: ActionPermission::AlwaysAllow,
            read_files: ActionPermission::AlwaysAllow,
            execute_commands: ActionPermission::AlwaysAllow,
            write_to_pty: WriteToPtyPermission::AlwaysAllow,
            ask_user_question: AskUserQuestionPermission::Never,
            command_denylist,
            command_allowlist: DEFAULT_COMMAND_EXECUTION_ALLOWLIST.to_vec(),
            directory_allowlist: Vec::new(),
            computer_use: computer_use_permission,
            web_search_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AIExecutionProfile, ComputerUsePermission};

    #[test]
    fn cli_profile_honors_computer_use_override() {
        let profile = AIExecutionProfile::create_default_cli_profile(false, Some(true));
        assert_eq!(profile.computer_use, ComputerUsePermission::AlwaysAllow);
    }

    #[test]
    fn cli_profile_allows_computer_use_by_default_only_when_sandboxed() {
        let sandboxed = AIExecutionProfile::create_default_cli_profile(true, None);
        let unsandboxed = AIExecutionProfile::create_default_cli_profile(false, None);

        assert_eq!(sandboxed.computer_use, ComputerUsePermission::AlwaysAllow);
        assert_eq!(unsandboxed.computer_use, ComputerUsePermission::Never);
    }
}
