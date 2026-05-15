pub mod entry;

pub use entry::{
    AgentConversationEntry, AgentConversationEntryId, AgentConversationNavigationSubject,
};

use crate::ai::active_agent_views_model::ActiveAgentViewsModel;
use crate::ai::agent::conversation::{AIConversationId, ConversationStatus};
use crate::ai::blocklist::{BlocklistAIHistoryEvent, BlocklistAIHistoryModel};
use crate::ai::conversation_navigation::ConversationNavigationData;
use crate::workspace::{RestoreConversationLayout, WorkspaceAction};
use std::collections::HashMap;
use warp_core::ui::theme::{color::internal_colors, WarpTheme};
use warpui::color::ColorU;
use warpui::{AppContext, Entity, ModelContext, SingletonEntity};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentRunDisplayStatus {
    ConversationInProgress,
    ConversationSucceeded,
    ConversationError,
    ConversationBlocked { blocked_action: String },
    ConversationCancelled,
}

impl AgentRunDisplayStatus {
    pub fn from_conversation_status(status: &ConversationStatus) -> Self {
        match status {
            ConversationStatus::InProgress => Self::ConversationInProgress,
            ConversationStatus::Success => Self::ConversationSucceeded,
            ConversationStatus::Error => Self::ConversationError,
            ConversationStatus::Cancelled => Self::ConversationCancelled,
            ConversationStatus::Blocked { blocked_action } => Self::ConversationBlocked {
                blocked_action: blocked_action.clone(),
            },
        }
    }

    pub fn to_conversation_status(&self) -> ConversationStatus {
        match self {
            AgentRunDisplayStatus::ConversationInProgress => ConversationStatus::InProgress,
            AgentRunDisplayStatus::ConversationSucceeded => ConversationStatus::Success,
            AgentRunDisplayStatus::ConversationError => ConversationStatus::Error,
            AgentRunDisplayStatus::ConversationBlocked { blocked_action } => {
                ConversationStatus::Blocked {
                    blocked_action: blocked_action.clone(),
                }
            }
            AgentRunDisplayStatus::ConversationCancelled => ConversationStatus::Cancelled,
        }
    }

    pub fn is_cancellable(&self) -> bool {
        matches!(self, AgentRunDisplayStatus::ConversationInProgress)
    }

    pub fn status_icon_and_color(
        &self,
        theme: &WarpTheme,
    ) -> (crate::ui_components::icons::Icon, ColorU) {
        match self {
            AgentRunDisplayStatus::ConversationInProgress => (
                crate::ui_components::icons::Icon::ClockLoader,
                theme.ansi_fg_magenta(),
            ),
            AgentRunDisplayStatus::ConversationSucceeded => (
                crate::ui_components::icons::Icon::Check,
                theme.ansi_fg_green(),
            ),
            AgentRunDisplayStatus::ConversationError => (
                crate::ui_components::icons::Icon::Triangle,
                theme.ansi_fg_red(),
            ),
            AgentRunDisplayStatus::ConversationBlocked { .. } => (
                crate::ui_components::icons::Icon::StopFilled,
                theme.ansi_fg_yellow(),
            ),
            AgentRunDisplayStatus::ConversationCancelled => (
                crate::ui_components::icons::Icon::StopFilled,
                internal_colors::neutral_5(theme),
            ),
        }
    }
}

impl std::fmt::Display for AgentRunDisplayStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRunDisplayStatus::ConversationInProgress => write!(f, "In progress"),
            AgentRunDisplayStatus::ConversationSucceeded => write!(f, "Done"),
            AgentRunDisplayStatus::ConversationError => write!(f, "Error"),
            AgentRunDisplayStatus::ConversationBlocked { .. } => write!(f, "Blocked"),
            AgentRunDisplayStatus::ConversationCancelled => write!(f, "Cancelled"),
        }
    }
}

pub struct ConversationMetadata {
    pub nav_data: ConversationNavigationData,
}

pub struct AgentConversationsModel {
    conversations: HashMap<AIConversationId, ConversationMetadata>,
}

pub enum AgentConversationsModelEvent {
    ConversationsLoaded,
    ConversationUpdated,
    ConversationArtifactsUpdated,
}

impl Entity for AgentConversationsModel {
    type Event = AgentConversationsModelEvent;
}

impl SingletonEntity for AgentConversationsModel {}

impl AgentConversationsModel {
    pub fn new(ctx: &mut ModelContext<Self>) -> Self {
        ctx.subscribe_to_model(
            &BlocklistAIHistoryModel::handle(ctx),
            |model, event, ctx| {
                model.handle_history_event(event, ctx);
            },
        );

        Self {
            conversations: HashMap::new(),
        }
    }

    pub fn sync_conversations(&mut self, ctx: &mut ModelContext<Self>) {
        let nav_data_list = ConversationNavigationData::all_conversations(ctx);
        self.conversations.clear();
        for nav_data in nav_data_list {
            self.conversations
                .insert(nav_data.id, ConversationMetadata { nav_data });
        }
        ctx.emit(AgentConversationsModelEvent::ConversationsLoaded);
    }

    pub fn register_view_open(
        &mut self,
        _window_id: warpui::WindowId,
        _view_id: warpui::EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.sync_conversations(ctx);
    }

    pub fn register_view_closed(
        &mut self,
        _window_id: warpui::WindowId,
        _view_id: warpui::EntityId,
        _ctx: &mut ModelContext<Self>,
    ) {
    }

    pub fn get_entries(&self, app: &AppContext) -> Vec<AgentConversationEntry> {
        let history_model = BlocklistAIHistoryModel::as_ref(app);
        let mut entries: Vec<_> = self
            .conversations
            .values()
            .map(|conversation| entry::entry_for_conversation(conversation, history_model))
            .collect();
        entries.sort_by(|a, b| b.display.last_updated.cmp(&a.display.last_updated));
        entries
    }

    pub fn get_entry_by_id(
        &self,
        id: &AgentConversationEntryId,
        app: &AppContext,
    ) -> Option<AgentConversationEntry> {
        let history_model = BlocklistAIHistoryModel::as_ref(app);
        match id {
            AgentConversationEntryId::Conversation(conversation_id) => self
                .conversations
                .get(conversation_id)
                .map(|conversation| entry::entry_for_conversation(conversation, history_model)),
        }
    }

    pub fn resolve_open_action(
        subject: AgentConversationNavigationSubject,
        restore_layout: Option<RestoreConversationLayout>,
        app: &AppContext,
    ) -> Option<WorkspaceAction> {
        let model = Self::as_ref(app);
        match subject {
            AgentConversationNavigationSubject::Entry(id) => model
                .get_entry_by_id(&id, app)
                .and_then(|entry| model.resolve_entry_open_action(&entry, restore_layout, app)),
        }
    }

    fn resolve_entry_open_action(
        &self,
        entry: &AgentConversationEntry,
        restore_layout: Option<RestoreConversationLayout>,
        app: &AppContext,
    ) -> Option<WorkspaceAction> {
        let active_views_model = ActiveAgentViewsModel::as_ref(app);
        let conversation_id = entry.conversation_id;

        if active_views_model.is_conversation_open(conversation_id, app) {
            if let Some(nav_data) = self
                .conversations
                .get(&conversation_id)
                .map(|metadata| &metadata.nav_data)
            {
                return Some(WorkspaceAction::RestoreOrNavigateToConversation {
                    conversation_id,
                    window_id: nav_data.window_id,
                    pane_view_locator: nav_data.pane_view_locator,
                    terminal_view_id: nav_data.terminal_view_id,
                    restore_layout,
                });
            }

            if let Some(terminal_view_id) =
                active_views_model.get_terminal_view_id_for_conversation(conversation_id, app)
            {
                return Some(WorkspaceAction::FocusTerminalViewInWorkspace { terminal_view_id });
            }
        }

        let nav_data = self
            .conversations
            .get(&conversation_id)
            .map(|metadata| &metadata.nav_data)?;

        Some(WorkspaceAction::RestoreOrNavigateToConversation {
            conversation_id,
            window_id: nav_data.window_id,
            pane_view_locator: None,
            terminal_view_id: nav_data.terminal_view_id,
            restore_layout,
        })
    }

    fn handle_history_event(
        &mut self,
        event: &BlocklistAIHistoryEvent,
        ctx: &mut ModelContext<Self>,
    ) {
        match event {
            BlocklistAIHistoryEvent::StartedNewConversation { .. }
            | BlocklistAIHistoryEvent::SetActiveConversation { .. }
            | BlocklistAIHistoryEvent::AppendedExchange { .. }
            | BlocklistAIHistoryEvent::SplitConversation { .. }
            | BlocklistAIHistoryEvent::RestoredConversations { .. }
            | BlocklistAIHistoryEvent::RemoveConversation { .. }
            | BlocklistAIHistoryEvent::DeletedConversation { .. }
            | BlocklistAIHistoryEvent::ClearedConversationsInTerminalView { .. }
            | BlocklistAIHistoryEvent::ClearedActiveConversation { .. } => {
                self.sync_conversations(ctx);
            }
            BlocklistAIHistoryEvent::UpdatedConversationStatus { .. } => {
                ctx.emit(AgentConversationsModelEvent::ConversationUpdated);
            }
            BlocklistAIHistoryEvent::UpdatedConversationArtifacts { .. } => {
                ctx.emit(AgentConversationsModelEvent::ConversationArtifactsUpdated);
            }
            BlocklistAIHistoryEvent::CreatedSubtask { .. }
            | BlocklistAIHistoryEvent::ReassignedExchange { .. }
            | BlocklistAIHistoryEvent::UpdatedTodoList { .. }
            | BlocklistAIHistoryEvent::UpdatedAutoexecuteOverride { .. }
            | BlocklistAIHistoryEvent::UpdatedConversationMetadata { .. }
            | BlocklistAIHistoryEvent::UpdatedStreamingExchange { .. }
            | BlocklistAIHistoryEvent::ConversationOwnershipTransferred { .. } => {}
        }
    }
}
