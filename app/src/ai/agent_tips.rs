use crate::palette::PaletteMode;
use crate::terminal::input::SET_INPUT_MODE_AGENT_ACTION_NAME;
use crate::terminal::view::init::{
    CANCEL_COMMAND_KEYBINDING, SELECT_PREVIOUS_BLOCK_ACTION_NAME,
    TOGGLE_AUTOEXECUTE_MODE_KEYBINDING,
};
use crate::ui_events::PaletteSource;
use crate::util::bindings::trigger_to_keystroke;
use crate::workspace::view::{
    TOGGLE_COMMAND_PALETTE_KEYBINDING_NAME, TOGGLE_RIGHT_PANEL_BINDING_NAME,
};
use crate::workspace::WorkspaceAction;
use markdown_parser::FormattedTextFragment;
use std::sync::LazyLock;
use std::time::Duration;
use warpui::keymap::Keystroke;
use warpui::r#async::SpawnedFutureHandle;
use warpui::{AppContext, Entity, ModelContext, SingletonEntity};

/// Trait for tip implementations that can be displayed to users.
/// Tips provide helpful information with optional links and keybindings.
pub trait AITip: Clone {
    /// Returns the keystroke for this tip, if applicable.
    fn keystroke(&self, app: &AppContext) -> Option<Keystroke>;

    /// Returns the raw description text for this tip.
    fn description(&self) -> &str;

    /// Converts the tip to formatted text fragments for rendering.
    /// Default implementation adds "Tip: " prefix and parses backtick-wrapped text as inline code.
    fn to_formatted_text(&self, _app: &AppContext) -> Vec<FormattedTextFragment> {
        let text = format!("Tip: {}", self.description());

        // Style backtick-wrapped text as inline code
        let parts: Vec<&str> = text.split('`').collect();
        let mut fragments = Vec::new();
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if i % 2 == 0 {
                fragments.push(FormattedTextFragment::plain_text(part.to_string()));
            } else {
                fragments.push(FormattedTextFragment::inline_code(part.to_string()));
            }
        }
        fragments
    }

    /// Checks if this tip is applicable in the current context.
    /// Default implementation returns true (tip is always applicable).
    fn is_tip_applicable(
        &self,
        _current_working_directory: Option<&str>,
        _app: &AppContext,
    ) -> bool {
        true
    }
}

static DEFAULT_TIPS: LazyLock<Vec<AgentTip>> = LazyLock::new(|| {
    vec![
        AgentTip {
            description: "`/` to open the slash-command menu and access agent actions.".to_string(),
            link: None,
            binding_name: None,
            action: None,
        },
        AgentTip {
            description: "<keybinding> to toggle natural language detection and switch between agent and terminal input.".to_string(),
            link: None,
            binding_name: Some(SET_INPUT_MODE_AGENT_ACTION_NAME),
            action: None,
        },
        AgentTip {
            description: "<keybinding> to open the Command Palette and access Warp actions and shortcuts.".to_string(),
            link: None,
            binding_name: Some(TOGGLE_COMMAND_PALETTE_KEYBINDING_NAME),
            action: Some(WorkspaceAction::OpenPalette {
                mode: PaletteMode::Command,
                source: PaletteSource::AgentTip,
                query: None,
            }),
        },
        AgentTip {
            description: "Enter a new prompt to redirect the agent while it's running.".to_string(),
            link: None,
            binding_name: None,
            action: None,
        },
        AgentTip {
            description: "`@` to add local files or terminal blocks as prompt context.".to_string(),
            link: None,
            binding_name: None,
            action: None,
        },
        AgentTip {
            description: "<keybinding> to attach the prior command output as agent context.".to_string(),
            link: None,
            binding_name: Some(SELECT_PREVIOUS_BLOCK_ACTION_NAME),
            action: None,
        },
        AgentTip {
            description: "Right-click a block to copy a conversation's output.".to_string(),
            link: None,
            binding_name: None,
            action: None,
        },
        AgentTip {
            description: "<keybinding> to open the code review panel and review the agent's changes.".to_string(),
            link: None,
            binding_name: Some(TOGGLE_RIGHT_PANEL_BINDING_NAME),
            action: None,
        },
        AgentTip {
            description: "`/open-code-review` to inspect agent-generated diffs.".to_string(),
            link: None,
            binding_name: None,
            action: Some(WorkspaceAction::ToggleRightPanel),
        },
        AgentTip {
            description: "`/new` to start a new agent conversation with clean context.".to_string(),
            link: None,
            binding_name: None,
            action: None,
        },
        AgentTip {
            description: "Right-click selected text to attach it as agent context.".to_string(),
            link: None,
            binding_name: None,
            action: None,
        },
        AgentTip {
            description: "Use `AGENTS.md` to apply project-scoped rules.".to_string(),
            link: None,
            binding_name: None,
            action: None,
        },
        AgentTip {
            description: "`/init` to generate an `AGENTS.md` file for this project.".to_string(),
            link: None,
            binding_name: None,
            action: None,
        },
        AgentTip {
            description: "<keybinding> to auto-approve the agent's commands and diffs for the rest of the session.".to_string(),
            link: None,
            binding_name: Some(TOGGLE_AUTOEXECUTE_MODE_KEYBINDING),
            action: None,
        },
        AgentTip {
            description: "<keybinding> to cancel the current agent task.".to_string(),
            link: None,
            binding_name: Some(CANCEL_COMMAND_KEYBINDING),
            action: None,
        },
    ]
});

#[derive(Clone, Debug)]
pub struct AgentTip {
    /// The text that will be displayed to the user. This is parsed such that:
    /// "Tip: " is added as a prefix,
    /// "<keybinding>" is replaced with user-defined and platform-specific keybinding referenced by binding_name,
    /// `text` that is wrapped in backticks is formatted as inline code
    pub description: String,
    pub link: Option<String>,
    pub binding_name: Option<&'static str>,
    pub action: Option<WorkspaceAction>,
}

impl AITip for AgentTip {
    fn keystroke(&self, app: &AppContext) -> Option<Keystroke> {
        let binding_name = self.binding_name?;

        if let Some(binding) = app.editable_bindings().find(|b| b.name == binding_name) {
            return trigger_to_keystroke(binding.trigger);
        }
        None
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn to_formatted_text(&self, app: &AppContext) -> Vec<FormattedTextFragment> {
        let mut text = format!("Tip: {}", self.description);

        // Replace <keybinding> with the actual keybinding string
        if let Some(keystroke) = self.keystroke(app) {
            text = text.replace("<keybinding>", &keystroke.displayed());
        }

        // Style backtick-wrapped text as inline code
        let parts: Vec<&str> = text.split('`').collect();
        let mut fragments = Vec::new();
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            if i % 2 == 0 {
                fragments.push(FormattedTextFragment::plain_text(part.to_string()));
            } else {
                fragments.push(FormattedTextFragment::inline_code(part.to_string()));
            }
        }

        fragments
    }

    fn is_tip_applicable(
        &self,
        _current_working_directory: Option<&str>,
        _app: &AppContext,
    ) -> bool {
        true
    }
}

impl WorkspaceAction {
    pub fn display_text(&self) -> Option<String> {
        match self {
            WorkspaceAction::OpenPalette { .. } => Some("Open palette".to_string()),
            WorkspaceAction::ToggleRightPanel => Some("Show diff view".to_string()),
            _ => None,
        }
    }
}

pub fn get_agent_tips(ctx: &AppContext) -> Vec<AgentTip> {
    let _ = ctx;
    DEFAULT_TIPS.clone()
}

/// A model for managing tips with cooldown logic.
/// Generic over any type implementing the AITip trait.
pub struct AITipModel<T: AITip> {
    tips: Vec<T>,
    current_tip: Option<T>,
    cooldown_handle: Option<SpawnedFutureHandle>,
}

impl<T: AITip + 'static> AITipModel<T> {
    /// Creates a new AITipModel with the given tips.
    /// Selects a random initial tip from the provided tips.
    ///
    /// # Panics
    /// Panics if the tips vector is empty.
    pub fn new(tips: Vec<T>) -> Self {
        use rand::seq::SliceRandom;
        debug_assert!(!tips.is_empty(), "AITipModel must have at least one tip");

        let mut rng = rand::thread_rng();
        let current_tip = tips.choose(&mut rng).cloned();

        Self {
            tips,
            current_tip,
            cooldown_handle: None,
        }
    }

    /// Returns the current tip, if one has been selected.
    pub fn current_tip(&self) -> Option<&T> {
        self.current_tip.as_ref()
    }
}

impl<T: AITip + 'static> Entity for AITipModel<T> {
    type Event = ();
}

// Specific implementation for AgentTip
impl AITipModel<AgentTip> {
    /// Creates a new AITipModel for AgentTips.
    /// This is the constructor used for the singleton model.
    pub fn new_for_agent_tips(ctx: &AppContext) -> Self {
        let tips = get_agent_tips(ctx);
        Self::new(tips)
    }

    /// Refreshes the current tip with a new random selection that is applicable
    /// for the given working directory.
    /// Only updates if not in cooldown period (60 seconds).
    pub fn maybe_refresh_tip(
        &mut self,
        current_working_directory: Option<&str>,
        ctx: &mut ModelContext<Self>,
    ) {
        // Don't update if cooldown is active
        if self.cooldown_handle.is_some() {
            return;
        }

        use rand::seq::SliceRandom;

        // Filter applicable tips based on working directory
        let available_tips: Vec<AgentTip> = self
            .tips
            .iter()
            .filter(|tip| tip.is_tip_applicable(current_working_directory, ctx))
            .cloned()
            .collect();

        // Select a random tip
        let mut rng = rand::thread_rng();
        self.current_tip = available_tips.choose(&mut rng).cloned();

        // Start 60-second cooldown
        let handle = ctx.spawn(
            async {
                warpui::r#async::Timer::after(Duration::from_secs(60)).await;
            },
            |me, _, _| {
                me.cooldown_handle = None;
            },
        );
        self.cooldown_handle = Some(handle);
        ctx.notify();
    }
}

impl SingletonEntity for AITipModel<AgentTip> {}
