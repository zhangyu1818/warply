//! Settings for Blocklist AI.
//!
//! These settings configure ACP AgentView behavior, terminal suggestions, and local AI UX.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::ai::acp::registry::DEFAULT_AGENT_ID;
use cfg_if::cfg_if;
use lazy_static::lazy_static;
use regex::Regex;
use warpui::{AppContext, SingletonEntity};

use settings::{define_settings_group, Setting, SupportedPlatforms};
use warp_core::execution_mode::AppExecutionMode;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

/// The default mode for new terminal sessions.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Default mode for new sessions.",
    rename_all = "snake_case"
)]
pub enum DefaultSessionMode {
    /// New sessions start in the terminal mode (default).
    #[default]
    Terminal,
    /// New sessions start in agent view.
    Agent,
    /// New sessions open a user-defined tab config.
    /// The specific config is identified by the companion `default_tab_config_path` setting.
    TabConfig,
    /// New sessions open in a local Docker sandbox.
    DockerSandbox,
}

settings::macros::implement_setting_for_enum!(
    DefaultSessionMode,
    AISettings,
    SupportedPlatforms::ALL,
    private: false,
    toml_path: "general.default_session_mode",
    description: "The default mode for new terminal sessions.",
);

impl DefaultSessionMode {
    /// Display name for the settings dropdown.
    pub fn display_name(&self) -> &'static str {
        match self {
            DefaultSessionMode::Terminal => "Terminal",
            DefaultSessionMode::Agent => "Agent",
            DefaultSessionMode::TabConfig => "Tab Config",
            DefaultSessionMode::DockerSandbox => "Local Docker Sandbox",
        }
    }
}

pub type AcpDefaultConfigOptionsMap = HashMap<String, String>;

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Terminal suggestion effort level.",
    rename_all = "snake_case"
)]
pub enum TerminalSuggestionEffort {
    #[default]
    Default,
    Low,
    Medium,
    High,
    XHigh,
}

impl TerminalSuggestionEffort {
    pub fn display_name(&self) -> &'static str {
        match self {
            TerminalSuggestionEffort::Default => "Default",
            TerminalSuggestionEffort::Low => "Low",
            TerminalSuggestionEffort::Medium => "Medium",
            TerminalSuggestionEffort::High => "High",
            TerminalSuggestionEffort::XHigh => "XHigh",
        }
    }

    pub fn config_value(&self) -> Option<&'static str> {
        match self {
            TerminalSuggestionEffort::Default => None,
            TerminalSuggestionEffort::Low => Some("low"),
            TerminalSuggestionEffort::Medium => Some("medium"),
            TerminalSuggestionEffort::High => Some("high"),
            TerminalSuggestionEffort::XHigh => Some("xhigh"),
        }
    }
}

/// Controls how agent thinking/reasoning traces are displayed after streaming.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Controls how agent thinking is displayed after streaming.",
    rename_all = "snake_case"
)]
pub enum ThinkingDisplayMode {
    /// Show reasoning blocks while streaming, then collapse them when complete (default).
    #[default]
    ShowAndCollapse,
    /// Always keep reasoning blocks expanded, even after streaming finishes.
    AlwaysShow,
    /// Never show reasoning blocks.
    NeverShow,
}

settings::macros::implement_setting_for_enum!(
    ThinkingDisplayMode,
    AISettings,
    SupportedPlatforms::ALL,
    private: false,
    toml_path: "agents.acp.thinking_display_mode",
    description: "Controls how agent thinking traces are displayed after streaming.",
);

impl ThinkingDisplayMode {
    /// Display name for the settings dropdown.
    pub fn display_name(&self) -> &'static str {
        match self {
            ThinkingDisplayMode::ShowAndCollapse => "Show & collapse",
            ThinkingDisplayMode::AlwaysShow => "Always show",
            ThinkingDisplayMode::NeverShow => "Never show",
        }
    }

    pub fn command_palette_description(&self) -> &'static str {
        match self {
            ThinkingDisplayMode::ShowAndCollapse => "Set agent thinking display: show & collapse",
            ThinkingDisplayMode::AlwaysShow => "Set agent thinking display: always show",
            ThinkingDisplayMode::NeverShow => "Set agent thinking display: never show",
        }
    }

    pub fn should_render(&self) -> bool {
        !matches!(self, ThinkingDisplayMode::NeverShow)
    }

    pub fn should_keep_expanded(&self) -> bool {
        matches!(self, ThinkingDisplayMode::AlwaysShow)
    }
}

/// Predicate types to match commands that can be executed by Agent Mode.
#[derive(Debug, Serialize, Deserialize, Clone)]
enum AgentModeCommandExecutionPredicateType {
    /// A regex with start (`^`) and end (`$`) anchors.
    ///
    /// We want regex rules to apply to the entire cmd string so we anchor them
    /// (there isn't any efficient way to apply to the entire cmd string at match-time).
    #[serde(with = "serde_regex")]
    AnchoredRegex(Regex),
}

impl AgentModeCommandExecutionPredicateType {
    fn new_regex(regex: &str) -> Result<Self, regex::Error> {
        // Redundant anchors aren't a problem so we can unconditionally add them.
        let anchored_regex = Regex::new(&format!("^{regex}$"))?;
        Ok(Self::AnchoredRegex(anchored_regex))
    }

    fn matches(&self, cmd: &str) -> bool {
        match self {
            Self::AnchoredRegex(regex) => regex.is_match(cmd),
        }
    }
}

impl PartialEq for AgentModeCommandExecutionPredicateType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AnchoredRegex(a), Self::AnchoredRegex(b)) => {
                // Indexing should be safe since they're guaranteed to have at least
                // the anchors around them.
                let a_unanchored = &a.as_str()[1..a.as_str().len() - 1];
                let b_unanchored = &b.as_str()[1..b.as_str().len() - 1];
                a_unanchored == b_unanchored
            }
        }
    }
}

impl std::fmt::Display for AgentModeCommandExecutionPredicateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnchoredRegex(regex) => {
                write!(f, "{}", &regex.as_str()[1..regex.as_str().len() - 1])
            }
        }
    }
}

/// A wrapper around [`AgentModeCommandExecutionPredicateType`] to enforce
/// the use of the provided constructors rather than direct construction of the variants.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(transparent)]
pub struct AgentModeCommandExecutionPredicate(AgentModeCommandExecutionPredicateType);

impl schemars::JsonSchema for AgentModeCommandExecutionPredicate {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("AgentModeCommandExecutionPredicate")
    }

    fn json_schema(gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // In the settings file, predicates are serialized as plain regex strings.
        gen.subschema_for::<String>()
    }
}

impl AgentModeCommandExecutionPredicate {
    pub fn new_regex(regex: &str) -> Result<Self, regex::Error> {
        Ok(Self(AgentModeCommandExecutionPredicateType::new_regex(
            regex,
        )?))
    }

    pub fn matches(&self, cmd: &str) -> bool {
        self.0.matches(cmd)
    }
}

impl std::fmt::Display for AgentModeCommandExecutionPredicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl settings_value::SettingsValue for AgentModeCommandExecutionPredicate {
    fn to_file_value(&self) -> serde_json::Value {
        serde_json::Value::String(self.to_string())
    }

    fn from_file_value(value: &serde_json::Value) -> Option<Self> {
        value.as_str().and_then(|s| Self::new_regex(s).ok())
    }
}

lazy_static! {
    // Matches optional args / options for a top-level command.
    static ref OPTIONAL_ARGS_REGEX: Regex = Regex::new(r"(\s.*)?").expect("Can parse optional args regex");
}

cfg_if! {
    // Compiling the regexes for the default command execution denylist can be slow
    // in an unoptimized build, so we use empty lists in unit tests.
    if #[cfg(test)] {
        lazy_static! {
            pub static ref DEFAULT_COMMAND_EXECUTION_DENYLIST: Vec<AgentModeCommandExecutionPredicate> = vec![];
        }
    } else {
        lazy_static! {
            pub static ref DEFAULT_COMMAND_EXECUTION_DENYLIST: Vec<AgentModeCommandExecutionPredicate> = vec![
                AgentModeCommandExecutionPredicate::new_regex(&format!("bash{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default bash rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("fish{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default fish rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("pwsh{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default pwsh rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("sh{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default sh rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("zsh{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default zsh rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("curl{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default curl rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("eval{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default eval rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("exec{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default exec rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("source{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default source rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("wget{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default wget rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("dig{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default dig rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("nslookup{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default nslookup rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("host{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default host rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("ssh{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default ssh rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("scp{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default scp rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("rsync{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default rsync rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("telnet{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default telnet rule into regex"),
                AgentModeCommandExecutionPredicate::new_regex(&format!("rm{}", OPTIONAL_ARGS_REGEX.as_str())).expect("Can parse default rm rule into regex"),
            ];
        }
    }
}

define_settings_group!(AISettings, settings: [
    acp_agent_backend: AcpAgentBackendSetting {
        type: String,
        default: DEFAULT_AGENT_ID.to_string(),
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "ai.acp.agent_backend",
        description: "The ACP agent backend.",
    }
    acp_default_config_options: AcpDefaultConfigOptions {
        type: HashMap<String, String>,
        default: HashMap::default(),
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
        toml_path: "ai.acp.default_config_options",
        description: "Default ACP session config option values keyed by ACP config option ID.",
    }
    terminal_suggestions_endpoint: TerminalSuggestionsEndpoint {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
        toml_path: "terminal_suggestions.endpoint",
        description: "OpenAI-compatible endpoint for Terminal Suggestions.",
    }
    terminal_suggestions_api_key: TerminalSuggestionsAPIKey {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
        toml_path: "terminal_suggestions.api_key",
        description: "API key for the OpenAI-compatible suggestions endpoint.",
    }
    terminal_suggestions_model: TerminalSuggestionsModel {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
        toml_path: "terminal_suggestions.model",
        description: "Model for Terminal Suggestions.",
    }
    terminal_suggestions_effort: TerminalSuggestionsEffort {
        type: TerminalSuggestionEffort,
        default: TerminalSuggestionEffort::Default,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "terminal_suggestions.effort",
        description: "Optional reasoning effort for Terminal Suggestions.",
    }
    terminal_next_command_enabled: TerminalNextCommandEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "terminal_suggestions.next_command_enabled",
        description: "Controls whether Next Command suggestions are enabled.",
    }
    terminal_prompt_suggestions_enabled: TerminalPromptSuggestionsEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "terminal_suggestions.prompt_suggestions_enabled",
        description: "Controls whether Prompt Suggestions are enabled.",
    }
    // If `false`, all AI features are disabled.
    is_any_ai_enabled: IsAnyAIEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "ai.enabled",
        description: "Controls whether all AI features are enabled.",
    },
    // This field should not be referenced directly to lookup active AI enablement -- use the
    // `is_active_ai_enabled()` getter.
    is_active_ai_enabled_internal: IsActiveAIEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "ai.active.enabled",
        description: "Controls whether proactive AI features like suggestions are enabled.",
    },
    autodetection_command_denylist: AICommandDenylist {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "ai.input.command_denylist",
        description: "Commands to exclude from AI natural language autodetection.",
    },
    // This field should not be referenced directly to lookup intelligent autosuggestion enablement
    // -- use the `is_intelligent_autosuggestions_enabled()` getter.
    intelligent_autosuggestions_enabled_internal: IntelligentAutosuggestionsEnabled {
        type: bool,
        default: true, // TODO(roland): revisit this when launched to stable
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "ai.active.intelligent_autosuggestions_enabled",
        description: "Controls whether AI-powered intelligent autosuggestions are enabled.",
    }
    // This field should not be referenced directly to lookup Code Suggestions
    // enablement -- use the `is_code_suggestions_enabled()` getter.
    code_suggestions_enabled_internal: CodeSuggestionsEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "ai.active.code_suggestions_enabled",
        description: "Controls whether AI code suggestions are enabled.",
    }
    // Whether or not the profile-level command autoexecution speedbump has been shown.
    //
    // Not a user-visible setting - persisted locally so the prompt is only shown once.
    has_shown_agent_mode_profile_command_autoexecution_speedbump: HasShownAgentModeProfileCommandAutoexecutionSpeedbump {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
    }
    // Whether or not we should show the speedbump for auto-writing to the PTY.
    //
    // Not a user-visible setting - persisted locally so the prompt is only shown once.
    should_show_agent_mode_write_to_pty_speedbump: ShouldShowAgentModeWriteToPtySpeedbump {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
    }
    // Whether or not we should show the speedbump for auto-reading files.
    //
    // Not a user-visible setting - persisted locally so the prompt is only shown once.
    should_show_agent_mode_autoread_files_speedbump: ShouldShowAgentModeCodingReadPermissionsNudge {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
    }
    // Whether or not the user wants agent mode requests to use their saved rules.
    memory_enabled: MemoryEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "agents.knowledge.rules_enabled",
        description: "Whether the agent uses your saved rules during requests.",
    }
    // Whether the agent mode setup banner has been shown for a given repo path.
    // Once shown, it will not be shown again for that repo.
    //
    // Not a user-visible setting - persisted locally so setup banners are not repeated.
    agent_mode_setup_banner_shown_for_repo_paths: AgentModeSetupBannerShownForRepoPaths {
        type: Vec<PathBuf>,
        default: vec![],
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
    }

    // Whether or not we should show the speedbump for showing code suggestion banners.
    // This includes both passive code diffs and suggested prompts (passive unit tests).
    //
    // Not a user-visible setting - persisted locally so the speedbump is not repeated.
    show_code_suggestion_speedbump: ShouldShowCodeSuggestionSpeedbump {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
    }

    // Tracks whether we've done the one-time auto-open of the conversation list for discoverability.
    // Once set to true, the conversation list visibility will be restored from workspace state.
    has_auto_opened_conversation_list: HasAutoOpenedConversationList {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        private: true,
    }

    // The raw stored default mode for new sessions. Use `default_session_mode()` to retrieve the
    // effective value, which is gated on AI availability.
    default_session_mode_internal: DefaultSessionMode,

    // The file path of the tab config used when default_session_mode_internal is TabConfig.
    // Only read when mode is TabConfig; ignored for all other modes.
    // Machine-local because tab config paths vary per machine.
    default_tab_config_path: DefaultTabConfigPath {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "general.default_tab_config_path",
    }

    // Controls how agent thinking/reasoning traces are displayed.
    thinking_display_mode: ThinkingDisplayMode,

    // Whether agent-executed shell commands should be included in command history
    // (up-arrow, Ctrl-R search, inline history menu).
    // When false, commands run by the AI agent are excluded from history.
    include_agent_commands_in_history: IncludeAgentCommandsInHistory {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "ai.input.include_agent_commands_in_history",
        description: "Whether agent-executed commands are included in command history.",
    }

    // Controls whether the conversation history view appears in the tools panel.
    show_conversation_history: ShowConversationHistory {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "ai.conversations.show_history",
        description: "Whether conversation history appears in the tools panel.",
    }

]);

impl AISettings {
    pub fn register_and_subscribe_to_events(app: &mut AppContext) {
        Self::register(app);
    }

    pub fn is_any_ai_enabled(&self, _app: &AppContext) -> bool {
        *self.is_any_ai_enabled
    }

    pub fn is_terminal_next_command_enabled(&self) -> bool {
        *self.terminal_next_command_enabled
    }

    pub fn is_terminal_prompt_suggestions_enabled(&self) -> bool {
        *self.terminal_prompt_suggestions_enabled
    }

    pub fn terminal_suggestions_config(
        &self,
    ) -> Option<crate::ai::terminal_suggestions::TerminalSuggestionsConfig> {
        let endpoint = self.terminal_suggestions_endpoint.trim();
        let model = self.terminal_suggestions_model.trim();
        if endpoint.is_empty() || model.is_empty() {
            return None;
        }

        Some(crate::ai::terminal_suggestions::TerminalSuggestionsConfig {
            endpoint: endpoint.to_string(),
            api_key: self.terminal_suggestions_api_key.trim().to_string(),
            model: model.to_string(),
            effort: *self.terminal_suggestions_effort,
        })
    }

    pub fn default_session_mode(&self, app: &AppContext) -> DefaultSessionMode {
        let mode = *self.default_session_mode_internal.value();
        match mode {
            DefaultSessionMode::Terminal
            | DefaultSessionMode::TabConfig
            | DefaultSessionMode::DockerSandbox => mode,
            DefaultSessionMode::Agent => {
                if self.is_any_ai_enabled(app) {
                    mode
                } else {
                    DefaultSessionMode::Terminal
                }
            }
        }
    }

    /// Returns the stored default tab config path (only meaningful when mode is `TabConfig`).
    pub fn default_tab_config_path(&self) -> &str {
        &self.default_tab_config_path
    }

    /// Looks up the `TabConfig` matching the stored `default_tab_config_path`.
    /// Returns `None` if the path is empty or no loaded config matches.
    pub fn resolved_default_tab_config(
        &self,
        app: &AppContext,
    ) -> Option<crate::tab_configs::TabConfig> {
        let path_str = self.default_tab_config_path.as_str();
        if path_str.is_empty() {
            return None;
        }
        let path = std::path::Path::new(path_str);
        crate::user_config::WarpConfig::as_ref(app)
            .tab_configs()
            .iter()
            .find(|config| config.source_path.as_deref().is_some_and(|p| p == path))
            .cloned()
    }

    pub fn is_active_ai_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app)
            && *self.is_active_ai_enabled_internal
            && AppExecutionMode::as_ref(app).allows_active_ai()
    }

    pub fn is_code_suggestions_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_active_ai_enabled(app) && *self.code_suggestions_enabled_internal
    }

    pub fn is_intelligent_autosuggestions_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_active_ai_enabled(app) && *self.intelligent_autosuggestions_enabled_internal
    }

    pub fn is_ai_autodetection_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_nld_in_terminal_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_memory_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app) && *self.memory_enabled
    }

    pub fn is_command_denylist_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_command_allowlist_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_directory_allowlist_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_execute_commands_permissions_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_write_to_pty_permissions_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_computer_use_permissions_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_read_files_permissions_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_code_diffs_permissions_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_ask_user_question_permissions_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn show_code_suggestion_speedbump(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app) && *self.show_code_suggestion_speedbump
    }
}

#[cfg(test)]
#[path = "ai_tests.rs"]
mod tests;
