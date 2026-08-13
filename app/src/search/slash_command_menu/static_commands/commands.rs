use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::search::slash_command_menu::{StaticCommand, static_commands::Argument};
use crate::ui_components::color_dot;

use super::Availability;

pub static AGENT: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/agent",
    description: "Start a new conversation",
    icon_path: "bundled/svg/agentmode.svg",
    availability: Availability::AI_ENABLED,
    auto_enter_ai_mode: false,
    argument: Some(Argument::optional().with_execute_on_selection()),
});

pub const CREATE_DOCKER_SANDBOX: StaticCommand = StaticCommand {
    name: "/docker-sandbox",
    description: "Create a new docker sandbox terminal session",
    icon_path: "bundled/svg/docker.svg",
    availability: Availability::LOCAL,
    auto_enter_ai_mode: false,
    argument: None,
};

pub static ADD_PROMPT: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/add-prompt",
    description: "Add new Agent prompt",
    icon_path: "bundled/svg/prompt.svg",
    availability: Availability::AI_ENABLED,
    auto_enter_ai_mode: false,
    argument: None,
});

pub const ADD_RULE: StaticCommand = StaticCommand {
    name: "/add-rule",
    description: "Add a new global rule for the agent",
    icon_path: "bundled/svg/book-open.svg",
    availability: Availability::AI_ENABLED,
    auto_enter_ai_mode: false,
    argument: None,
};

pub static EDIT: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/open-file",
    description: "Open a file in Warp's code editor",
    icon_path: "bundled/svg/file-code-02.svg",
    availability: Availability::LOCAL,
    auto_enter_ai_mode: false,
    argument: Some(
        Argument::optional().with_hint_text("<path/to/file[:line[:col]]> or \"@\" to search"),
    ),
});

pub static RENAME_TAB: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/rename-tab",
    description: "Rename the current tab",
    icon_path: "bundled/svg/pencil-line.svg",
    availability: Availability::ALWAYS,
    auto_enter_ai_mode: false,
    argument: Some(Argument::required().with_hint_text("<tab name>")),
});

static SET_TAB_COLOR_HINT: LazyLock<String> = LazyLock::new(|| {
    let mut hint = String::from("<");
    for color in color_dot::TAB_COLOR_OPTIONS {
        hint.push_str(&color.to_string().to_ascii_lowercase());
        hint.push('|');
    }
    hint.push_str("none>");
    hint
});

pub static SET_TAB_COLOR: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/set-tab-color",
    description: "Set the color of the current tab",
    icon_path: "bundled/svg/ellipse.svg",
    availability: Availability::ALWAYS,
    auto_enter_ai_mode: false,
    argument: Some(Argument::required().with_hint_text(SET_TAB_COLOR_HINT.as_str())),
});

pub static FORK: LazyLock<StaticCommand> = LazyLock::new(|| {
    let hint_text = "<optional prompt to send in forked conversation>";
    StaticCommand {
        name: "/fork",
        description: "Fork the current conversation in a new pane or a new tab",
        icon_path: "bundled/svg/arrow-split.svg",
        availability: Availability::AGENT_VIEW
            | Availability::ACTIVE_CONVERSATION
            | Availability::NO_LRC_CONTROL
            | Availability::AI_ENABLED,
        auto_enter_ai_mode: true,
        argument: Some(Argument::optional().with_hint_text(hint_text)),
    }
});

pub const OPEN_CODE_REVIEW: StaticCommand = StaticCommand {
    name: "/open-code-review",
    description: "Open code review",
    icon_path: "bundled/svg/diff.svg",
    availability: Availability::REPOSITORY,
    auto_enter_ai_mode: false,
    argument: None,
};

pub const INIT_NAME: &str = "/init";

pub const OPEN_PROJECT_RULES: StaticCommand = StaticCommand {
    name: "/open-project-rules",
    description: "Open the project rules file (AGENTS.md)",
    icon_path: "bundled/svg/file-code-02.svg",
    availability: Availability::REPOSITORY.union(Availability::AI_ENABLED),
    auto_enter_ai_mode: false,
    argument: None,
};

pub const OPEN_SETTINGS_FILE: StaticCommand = StaticCommand {
    name: "/open-settings-file",
    description: "Open settings file (TOML)",
    icon_path: "bundled/svg/file-code-02.svg",
    availability: Availability::LOCAL,
    auto_enter_ai_mode: false,
    argument: None,
};

pub const OPEN_REPO: StaticCommand = StaticCommand {
    name: "/open-repo",
    description: "Switch to another indexed repository",
    icon_path: "bundled/svg/folder.svg",
    availability: Availability::LOCAL,
    auto_enter_ai_mode: false,
    argument: None,
};

pub const OPEN_RULES: StaticCommand = StaticCommand {
    name: "/open-rules",
    description: "View all of your global and project rules",
    icon_path: "bundled/svg/book-open.svg",
    availability: Availability::AI_ENABLED,
    auto_enter_ai_mode: false,
    argument: None,
};

pub static NEW: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/new",
    description: "Start a new conversation (alias for /agent)",
    icon_path: "bundled/svg/new-conversation.svg",
    availability: Availability::NO_LRC_CONTROL | Availability::AI_ENABLED,
    auto_enter_ai_mode: false,
    argument: Some(Argument::optional().with_execute_on_selection()),
});

pub const PLAN_NAME: &str = "/plan";
pub const PR_COMMENTS_NAME: &str = "/pr-comments";

/// If `query` starts with the given command `name` followed by a space,
/// returns the remainder of the query. Otherwise returns `None`.
pub fn strip_command_prefix(query: &str, name: &str) -> Option<String> {
    query
        .strip_prefix(name)
        .and_then(|rest| rest.strip_prefix(' '))
        .map(|rest| rest.to_string())
}

pub static QUEUE: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/queue",
    description: "Queue a prompt to send after the agent finishes responding",
    icon_path: "bundled/svg/clock-plus.svg",
    availability: Availability::AGENT_VIEW
        | Availability::ACTIVE_CONVERSATION
        | Availability::AI_ENABLED,
    auto_enter_ai_mode: true,
    argument: Some(Argument::required().with_hint_text("<prompt to send when agent is done>")),
});

pub const FORK_FROM: StaticCommand = StaticCommand {
    name: "/fork-from",
    description: "Fork conversation from a specific query",
    icon_path: "bundled/svg/arrow-split.svg",
    availability: Availability::AGENT_VIEW
        .union(Availability::NO_LRC_CONTROL)
        .union(Availability::AI_ENABLED),
    auto_enter_ai_mode: true,
    argument: None,
};

pub const CONVERSATIONS: StaticCommand = StaticCommand {
    name: "/conversations",
    description: "Open conversation history",
    icon_path: "bundled/svg/conversation.svg",
    availability: Availability::AI_ENABLED,
    auto_enter_ai_mode: false,
    argument: None,
};

pub static PROMPTS: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/prompts",
    description: "Search saved prompts",
    icon_path: "bundled/svg/prompt.svg",
    availability: Availability::AI_ENABLED,
    auto_enter_ai_mode: false,
    argument: None,
});

pub const REWIND: StaticCommand = StaticCommand {
    name: "/rewind",
    description: "Rewind to a previous point in the conversation",
    icon_path: "bundled/svg/clock-rewind.svg",
    availability: Availability::AGENT_VIEW.union(Availability::AI_ENABLED),
    auto_enter_ai_mode: true,
    argument: None,
};

pub const EXPORT_TO_CLIPBOARD: StaticCommand = StaticCommand {
    name: "/export-to-clipboard",
    description: "Export current conversation to clipboard in markdown format",
    icon_path: "bundled/svg/copy.svg",
    availability: Availability::AGENT_VIEW.union(Availability::AI_ENABLED),
    auto_enter_ai_mode: true,
    argument: None,
};

pub static EXPORT_TO_FILE: LazyLock<StaticCommand> = LazyLock::new(|| StaticCommand {
    name: "/export-to-file",
    description: "Export current conversation to a markdown file",
    icon_path: "bundled/svg/download-01.svg",
    availability: Availability::AGENT_VIEW | Availability::AI_ENABLED,
    auto_enter_ai_mode: true,
    argument: Some(Argument::optional().with_hint_text("<optional filename>")),
});

pub static COMMAND_REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

/// A unique identifier for a static slash command.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct SlashCommandId(Uuid);

impl SlashCommandId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SlashCommandId {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Registry {
    commands: HashMap<SlashCommandId, StaticCommand>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        for command in all_commands().into_iter() {
            debug_assert!(
                !command
                    .availability
                    .contains(Availability::TERMINAL_VIEW | Availability::AGENT_VIEW),
                "command `{}` sets both TERMINAL_VIEW and AGENT_VIEW, which is unsatisfiable",
                command.name,
            );
            commands.insert(SlashCommandId::new(), command);
        }
        Self { commands }
    }

    pub fn all_commands_by_id(&self) -> impl Iterator<Item = (SlashCommandId, &StaticCommand)> {
        self.commands.iter().map(|(id, cmd)| (*id, cmd))
    }

    pub fn all_commands(&self) -> impl Iterator<Item = &StaticCommand> {
        self.commands.values()
    }

    pub fn get_command(&self, id: &SlashCommandId) -> Option<&StaticCommand> {
        self.commands.get(id)
    }

    pub fn get_command_with_name(&self, name: &str) -> Option<&StaticCommand> {
        self.commands.values().find(|command| command.name == name)
    }

    #[cfg(test)]
    pub fn get_command_id_with_name(&self, name: &str) -> Option<&SlashCommandId> {
        self.commands
            .iter()
            .find(|(_, command)| command.name == name)
            .map(|(id, _)| id)
    }
}

fn all_commands() -> Vec<StaticCommand> {
    let mut commands = vec![
        ADD_PROMPT.clone(),
        ADD_RULE,
        OPEN_PROJECT_RULES,
        OPEN_RULES,
        AGENT.clone(),
        NEW.clone(),
        RENAME_TAB.clone(),
        SET_TAB_COLOR.clone(),
        CONVERSATIONS,
        EXPORT_TO_CLIPBOARD,
    ];

    commands.push(CREATE_DOCKER_SANDBOX);

    commands.push(PROMPTS.clone());

    commands.push(OPEN_CODE_REVIEW);

    commands.push(QUEUE.clone());

    commands.push(FORK.clone());

    commands.push(FORK_FROM);

    commands.extend([EDIT.clone(), EXPORT_TO_FILE.clone()]);

    commands.push(REWIND);

    commands.push(OPEN_REPO);

    if cfg!(feature = "local_fs") {
        commands.push(OPEN_SETTINGS_FILE);
    }

    commands
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn command_names_are_unique() {
        let names = COMMAND_REGISTRY.all_commands().map(|command| command.name);
        let mut seen = HashSet::new();
        for name in names {
            assert!(seen.insert(name), "duplicate slash command name: {name}");
        }
    }

    #[test]
    fn static_command_registry_matches_reviewed_app_owned_commands() {
        let names = COMMAND_REGISTRY.all_commands().map(|command| command.name);
        let mut actual = HashSet::from_iter(names);
        let mut expected = HashSet::from([
            ADD_PROMPT.name,
            ADD_RULE.name,
            OPEN_PROJECT_RULES.name,
            OPEN_RULES.name,
            AGENT.name,
            NEW.name,
            RENAME_TAB.name,
            SET_TAB_COLOR.name,
            CONVERSATIONS.name,
            EXPORT_TO_CLIPBOARD.name,
            CREATE_DOCKER_SANDBOX.name,
            PROMPTS.name,
            OPEN_CODE_REVIEW.name,
            QUEUE.name,
            FORK.name,
            FORK_FROM.name,
            EDIT.name,
            EXPORT_TO_FILE.name,
            REWIND.name,
            OPEN_REPO.name,
        ]);

        if cfg!(feature = "local_fs") {
            expected.insert(OPEN_SETTINGS_FILE.name);
        } else {
            actual.remove(OPEN_SETTINGS_FILE.name);
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn agent_semantic_commands_are_not_registered_as_static_commands() {
        for name in [PLAN_NAME, INIT_NAME, "/create-new-project", "/pr-comments"] {
            assert!(
                COMMAND_REGISTRY.get_command_with_name(name).is_none(),
                "{name} should come from ACP available commands instead of the app static registry"
            );
        }
    }

    #[test]
    fn rename_tab_command_requires_argument() {
        let command = COMMAND_REGISTRY
            .get_command_with_name(RENAME_TAB.name)
            .expect("expected /rename-tab to be registered");
        let argument = command
            .argument
            .as_ref()
            .expect("expected /rename-tab to require an argument");

        assert!(!argument.is_optional);
        assert!(!argument.should_execute_on_selection);
        assert_eq!(argument.hint_text, Some("<tab name>"));
    }

    #[test]
    fn set_tab_color_command_requires_argument() {
        let command = COMMAND_REGISTRY
            .get_command_with_name(SET_TAB_COLOR.name)
            .expect("expected /set-tab-color to be registered");
        let argument = command
            .argument
            .as_ref()
            .expect("expected /set-tab-color to require an argument");

        assert!(!argument.is_optional);
        assert!(!argument.should_execute_on_selection);

        let hint = argument
            .hint_text
            .expect("/set-tab-color hint text is set dynamically");
        for color in color_dot::TAB_COLOR_OPTIONS {
            let lower = color.to_string().to_ascii_lowercase();
            assert!(hint.contains(&lower), "hint should mention `{lower}`");
        }
        assert!(hint.contains("none"), "hint should mention `none`");
    }

    #[test]
    fn strip_command_prefix_no_match() {
        let result = strip_command_prefix("just a normal query", "/plan");
        assert_eq!(result, None);
    }

    #[test]
    fn strip_command_prefix_empty() {
        let result = strip_command_prefix("", "/plan");
        assert_eq!(result, None);
    }

    #[test]
    fn strip_command_prefix_no_trailing_space() {
        // "/plan" alone (no trailing space) should NOT be stripped
        let result = strip_command_prefix("/plan", "/plan");
        assert_eq!(result, None);
    }

    #[test]
    fn strip_command_prefix_trailing_space_only() {
        // "/plan " with nothing after should strip to empty string
        let result = strip_command_prefix("/plan ", "/plan");
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn strip_command_prefix_substring_not_matched() {
        // "/planning" should not match "/plan"
        let result = strip_command_prefix("/planning something", "/plan");
        assert_eq!(result, None);
    }
}
