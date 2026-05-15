//! Action buttons row for conversation details panel.

use warp_core::ui::theme::AnsiColorIdentifier;
use warpui::elements::{ChildView, CrossAxisAlignment, Empty, Flex, ParentElement};
use warpui::{AppContext, Element, Entity, TypedActionView, View, ViewContext, ViewHandle};

use crate::ai::agent::conversation::AIConversationId;
use crate::ui_components::icons::Icon;
use crate::view_components::action_button::{ActionButton, ButtonSize, SecondaryTheme};
use crate::workspace::WorkspaceAction;

const BUTTON_SPACING: f32 = 4.;

/// Per-button config for the action buttons row.
/// Each field controls one button independently.
#[derive(Debug, Clone, Default)]
pub struct ActionButtonsConfig {
    pub open_action: Option<WorkspaceAction>,
    pub fork_conversation_id: Option<AIConversationId>,
}

impl ActionButtonsConfig {
    /// Returns true if no buttons will be rendered.
    pub fn is_empty(&self) -> bool {
        self.open_action.is_none() && self.fork_conversation_id.is_none()
    }

    pub fn for_conversation(
        conversation_id: AIConversationId,
        open_action: Option<WorkspaceAction>,
    ) -> Self {
        Self {
            open_action,
            fork_conversation_id: Some(conversation_id),
        }
    }
}

/// Events emitted by the action buttons.
#[derive(Debug, Clone)]
pub enum AgentDetailsButtonEvent {
    Open,
    ForkConversation { conversation_id: AIConversationId },
}

/// Actions dispatched by button clicks (internal).
#[derive(Debug, Clone)]
pub enum AgentDetailsAction {
    Open,
    ForkConversation,
}

/// Reusable action buttons row for details panel.
pub struct ConversationActionButtonsRow {
    config: ActionButtonsConfig,
    open_button: ViewHandle<ActionButton>,
    fork_conversation_button: ViewHandle<ActionButton>,
}

impl ConversationActionButtonsRow {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let open_button = ctx.add_typed_action_view(|_| {
            Self::make_action_button(
                Icon::LinkExternal,
                "Open conversation",
                None,
                AgentDetailsAction::Open,
            )
        });

        let fork_conversation_button = ctx.add_typed_action_view(|_| {
            Self::make_action_button(
                Icon::ArrowSplit,
                "Fork conversation",
                None,
                AgentDetailsAction::ForkConversation,
            )
        });

        Self {
            config: ActionButtonsConfig::default(),
            open_button,
            fork_conversation_button,
        }
    }

    /// Set the config and rerender.
    pub fn set_config(&mut self, config: ActionButtonsConfig, ctx: &mut ViewContext<Self>) {
        self.config = config;
        ctx.notify();
    }

    /// Returns true if no buttons will be rendered.
    pub fn is_empty(&self) -> bool {
        self.config.is_empty()
    }

    fn make_action_button(
        icon: Icon,
        tooltip: &str,
        icon_color: Option<AnsiColorIdentifier>,
        action: AgentDetailsAction,
    ) -> ActionButton {
        let mut button = ActionButton::new("", SecondaryTheme)
            .with_icon(icon)
            .with_size(ButtonSize::Small)
            .with_tooltip(tooltip)
            .on_click(move |ctx| {
                ctx.dispatch_typed_action(action.clone());
            });
        if let Some(color) = icon_color {
            button = button.with_icon_ansi_color(color);
        }
        button
    }
}

impl Entity for ConversationActionButtonsRow {
    type Event = AgentDetailsButtonEvent;
}

impl View for ConversationActionButtonsRow {
    fn ui_name() -> &'static str {
        "ConversationActionButtonsRow"
    }

    fn render(&self, _app: &AppContext) -> Box<dyn Element> {
        if self.config.is_empty() {
            return Empty::new().finish();
        }

        let mut row = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_spacing(BUTTON_SPACING);

        if self.config.open_action.is_some() {
            row.add_child(ChildView::new(&self.open_button).finish());
        }
        if self.config.fork_conversation_id.is_some() {
            row.add_child(ChildView::new(&self.fork_conversation_button).finish());
        }
        row.finish()
    }
}

impl TypedActionView for ConversationActionButtonsRow {
    type Action = AgentDetailsAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            AgentDetailsAction::Open => {
                if self.config.open_action.is_some() {
                    ctx.emit(AgentDetailsButtonEvent::Open);
                }
            }
            AgentDetailsAction::ForkConversation => {
                if let Some(conversation_id) = self.config.fork_conversation_id {
                    ctx.emit(AgentDetailsButtonEvent::ForkConversation { conversation_id });
                }
            }
        }
    }
}
