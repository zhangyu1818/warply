use std::collections::{HashMap, HashSet};

use agent_client_protocol::schema::ToolCallUpdate;
use anyhow::anyhow;
use chrono::{DateTime, Local, NaiveDateTime};
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use warpui::{AppContext, Entity, EntityId, ModelContext, SingletonEntity};

#[cfg(feature = "local_fs")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "local_fs")]
use diesel::SqliteConnection;

use crate::ai::acp::{AcpPermissionRequest, AcpPlan, AcpToolCall};
use crate::ai::agent::conversation::ConversationStatus;
use crate::ai::agent::conversation::UpdateConversationError;
use crate::ai::agent::task::TaskId;
use crate::ai::agent::AIAgentExchangeId;
use crate::ai::agent::CancellationReason;
use crate::ai::artifacts::Artifact;
use crate::ai::document::ai_document_model::AIDocumentModel;
use crate::ai::llms::LLMId;
use crate::input_suggestions::HistoryOrder;
use crate::persistence::model::AgentConversationData;
use crate::persistence::ModelEvent;
use crate::terminal::model::block::BlockId;
use crate::terminal::view::blocklist_filter;
use crate::GlobalResourceHandlesProvider;
use crate::{
    ai::agent::{
        conversation::{AIConversation, AIConversationId},
        AIAgentActionId, AIAgentExchange, AIAgentInput, AIAgentOutputStatus, FinishedAIAgentOutput,
        RenderableAIError,
    },
    persistence::model::AgentConversation,
    ui_components::icons::Icon,
};

#[cfg(feature = "local_fs")]
use crate::persistence::{database_file_path, establish_ro_connection};

use super::controller::response_stream::ResponseStreamId;
use super::persistence::{PersistedAIInput, PersistedAIInputType};
use super::RequestInput;

mod conversation_loader;
pub use conversation_loader::{
    convert_persisted_conversation_to_ai_conversation_with_metadata, RestoredConversationData,
};

pub(super) const MAX_HISTORICAL_CONVERSATIONS: usize = 100;

pub struct AcpResponseStreamTarget {
    pub stream_id: ResponseStreamId,
    pub conversation_id: AIConversationId,
    pub terminal_view_id: EntityId,
    pub model_id: LLMId,
    pub display_name: String,
}

#[derive(Debug, Clone)]
pub struct AIConversationMetadata {
    pub id: AIConversationId,

    pub title: String,

    pub initial_query: String,

    pub last_modified_at: NaiveDateTime,

    pub initial_working_directory: Option<String>,

    pub has_local_data: bool,

    pub artifacts: Vec<Artifact>,
}

impl From<&AIConversation> for AIConversationMetadata {
    fn from(conversation: &AIConversation) -> Self {
        let title = conversation.title().unwrap_or_default().to_string();
        let initial_query: String = conversation.initial_query().unwrap_or_default();

        let last_modified_at = conversation
            .latest_exchange()
            .map(|exchange| exchange.start_time.naive_utc())
            .unwrap_or_else(|| chrono::Utc::now().naive_utc());

        let initial_working_directory = conversation
            .initial_working_directory()
            .or_else(|| conversation.current_working_directory());

        Self {
            id: conversation.id(),
            title,
            initial_query,
            last_modified_at,
            initial_working_directory,
            has_local_data: true,
            artifacts: conversation.artifacts().to_vec(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateHistoryError {
    #[error("Failed to update conversation: {0:?}")]
    Conversation(#[from] UpdateConversationError),
    #[error("Failed to find conversation with ID {0:?}")]
    ConversationNotFound(AIConversationId),
}

/// Responsible for managing the history of user and AI exchanges.
#[derive(Default)]
pub struct BlocklistAIHistoryModel {
    /// A [`HashMap`] mapping [`crate::terminal::TerminalView`] [`EntityId`]s to a [`Vec`] of
    /// live [`AIConversationId`] in that `TerminalView`.
    ///
    /// "Live" conversations are still visible and in the terminal view and selectable in the session, so
    /// clearing the blocklist removes the conversation from here.
    ///
    /// Note that when a terminal view is closed, we do not remove it from this map, so that it can be restored.
    live_conversation_ids_for_terminal_view: HashMap<EntityId, Vec<AIConversationId>>,

    /// A [`HashMap`] mapping [`crate::terminal::TerminalView`] [`EntityId`]s to a [`Vec`] of
    /// [`AIConversationId`] that were once live in that session, but were cleared from the blocklist.
    ///
    /// This is used to preserve queries for up-arrow history after clearing the blocklist.
    cleared_conversation_ids_for_terminal_view: HashMap<EntityId, Vec<AIConversationId>>,

    /// A [`HashMap`] mapping a [`AIConversationId`] to the [`AIConversation`] itself.
    /// Conversations may or may not be live in any open session. They will exist in this map if they
    /// have ever been loaded into memory.
    conversations_by_id: HashMap<AIConversationId, AIConversation>,

    /// The active conversation ID for a given terminal view.
    /// The active conversation is the one we're currently or have most recently streamed outputs for.
    /// If you want to get the conversation the next query will follow up in / what is selected in the input selector,
    /// use `context_model.selected_conversation_id` instead.
    active_conversation_for_terminal_view: HashMap<EntityId, AIConversationId>,

    /// The time at which each [`TerminalView`] was created. Note that this has no bearing on when
    /// any [`AIConversation`]s take place in the terminal view.
    terminal_view_created_at: HashMap<EntityId, DateTime<Local>>,

    /// A set of terminal views that are read-only conversation transcript viewers.
    /// This is view/UI state (not conversation state) and is used to filter transcript viewer
    /// conversations out of local history and navigation.
    conversation_transcript_viewer_terminal_view_ids: HashSet<EntityId>,

    /// AI queries that were read from the SQLite DB. These exchanges do not contain as much
    /// information as the other exchanges we store because they are only used for display in
    /// history.
    persisted_queries: Vec<PersistedAIInput>,

    /// Metadata for conversations. Does not include the actual content.
    all_conversations_metadata: HashMap<AIConversationId, AIConversationMetadata>,

    #[cfg(feature = "local_fs")]
    db_connection: Option<Arc<Mutex<SqliteConnection>>>,
}

impl BlocklistAIHistoryModel {
    pub(crate) fn new(
        persisted_queries: Vec<PersistedAIInput>,
        agent_conversations: &[AgentConversation],
    ) -> Self {
        #[cfg(feature = "local_fs")]
        let db_connection = database_file_path().to_str().and_then(|db_url| {
            establish_ro_connection(db_url)
                .ok()
                .map(|conn| Arc::new(Mutex::new(conn)))
        });

        let mut model = Self {
            persisted_queries,
            #[cfg(feature = "local_fs")]
            db_connection,
            ..Self::default()
        };

        // Initialize historical conversations from local DB
        model.initialize_historical_conversations(agent_conversations);

        model
    }

    #[cfg(test)]
    pub(crate) fn new_for_test() -> Self {
        Self::default()
    }

    /// Returns a flattened and ordered (oldest first) list of live conversations (not cleared) for the given terminal view ID.
    /// This works for terminal views that have been closed.
    pub fn all_live_conversations_for_terminal_view(
        &self,
        terminal_view_id: EntityId,
    ) -> impl Iterator<Item = &AIConversation> {
        self.live_conversation_ids_for_terminal_view
            .get(&terminal_view_id)
            .into_iter()
            .flat_map(|conversation_ids| {
                conversation_ids
                    .iter()
                    .filter_map(|conversation_id| self.conversation(conversation_id))
            })
    }

    /// Returns a flattened and ordered (oldest first) list of exchanges from live conversations (not cleared)
    /// in the given terminal view ID.
    /// This works for terminal views that have been closed.
    pub fn all_live_root_task_exchanges_for_terminal_view(
        &self,
        terminal_view_id: EntityId,
    ) -> impl Iterator<Item = &AIAgentExchange> {
        self.live_conversation_ids_for_terminal_view
            .get(&terminal_view_id)
            .into_iter()
            .flat_map(|conversation_ids| {
                conversation_ids.iter().flat_map(|conversation_id| {
                    self.conversations_by_id
                        .get(conversation_id)
                        .map(|conversation| conversation.root_task_exchanges())
                })
            })
            .flatten()
    }

    /// Returns a flattened and ordered (oldest first) list of exchanges from conversations
    /// that were cleared in the given terminal view ID, but are no longer live/visible.
    pub fn all_cleared_root_task_exchanges_for_terminal_view(
        &self,
        terminal_view_id: EntityId,
    ) -> impl Iterator<Item = &AIAgentExchange> {
        self.cleared_conversation_ids_for_terminal_view
            .get(&terminal_view_id)
            .into_iter()
            .flat_map(|conversation_ids| {
                conversation_ids.iter().flat_map(|conversation_id| {
                    self.conversations_by_id
                        .get(conversation_id)
                        .map(|conversation| conversation.root_task_exchanges())
                })
            })
            .flatten()
    }

    /// Returns a list of all conversations that have been cleared across all terminal views.
    pub fn all_cleared_conversations(&self) -> Vec<(EntityId, &AIConversation)> {
        self.cleared_conversation_ids_for_terminal_view
            .iter()
            .flat_map(|(terminal_view_id, conversation_ids)| {
                conversation_ids.iter().filter_map(|conversation_id| {
                    self.conversations_by_id
                        .get(conversation_id)
                        .map(|conversation| (*terminal_view_id, conversation))
                })
            })
            .collect::<Vec<_>>()
    }

    /// Returns a list of all live (not cleared) conversations across all terminal views,
    /// paired with the terminal view ID they belong to.
    /// This includes terminal views that have been closed.
    pub fn all_live_conversations(&self) -> Vec<(EntityId, &AIConversation)> {
        self.live_conversation_ids_for_terminal_view
            .iter()
            .flat_map(|(terminal_view_id, conversation_ids)| {
                conversation_ids.iter().filter_map(|conversation_id| {
                    self.conversations_by_id
                        .get(conversation_id)
                        .map(|conversation| (*terminal_view_id, conversation))
                })
            })
            .collect::<Vec<_>>()
    }

    /// Returns a conversation by ID by reading from memory. The conversation may not be available if:
    /// * The ID is invalid
    /// * The conversation has never been read into memory from db. Use load_conversation_from_db to handle reading from db.
    pub fn conversation(&self, conversation_id: &AIConversationId) -> Option<&AIConversation> {
        self.conversations_by_id.get(conversation_id)
    }

    pub fn conversation_mut(
        &mut self,
        conversation_id: &AIConversationId,
    ) -> Option<&mut AIConversation> {
        self.conversations_by_id.get_mut(conversation_id)
    }

    /// Returns the ID of the conversation that processed or is processing the response stream.
    ///
    /// A given response stream may only correspond to a single conversation at any given time,
    /// though the conversation to which it corresponds may change if a new conversation is started
    /// in the middle of the response, as is the case when the new conversation suggestion is
    /// accepted.
    pub fn conversation_for_response_stream(
        &self,
        response_stream_id: &ResponseStreamId,
    ) -> Option<AIConversationId> {
        self.conversations_by_id
            .iter()
            .find_map(|(conversation_id, conversation)| {
                if conversation.is_processing_response_stream(response_stream_id) {
                    Some(*conversation_id)
                } else {
                    None
                }
            })
    }

    pub fn conversation_status(
        &self,
        conversation_id: &AIConversationId,
    ) -> Option<&ConversationStatus> {
        self.conversation(conversation_id)
            .map(|conversation| conversation.status())
    }

    /// Returns the terminal view ID that owns the given conversation, if any.
    pub fn terminal_view_id_for_conversation(
        &self,
        conversation_id: &AIConversationId,
    ) -> Option<EntityId> {
        self.live_conversation_ids_for_terminal_view
            .iter()
            .find(|(_, conversation_ids)| conversation_ids.contains(conversation_id))
            .map(|(terminal_view_id, _)| *terminal_view_id)
    }

    /// Returns the conversation ID from the terminal view's history corresponding to the action,
    /// if any.
    pub fn conversation_id_for_action(
        &self,
        action_id: &AIAgentActionId,
        terminal_view_id: EntityId,
    ) -> Option<AIConversationId> {
        self.live_conversation_ids_for_terminal_view
            .get(&terminal_view_id)?
            .iter()
            .rev()
            .find(|conversation_id| {
                self.conversations_by_id
                    .get(conversation_id)
                    .is_some_and(|conversation| conversation.contains_action(action_id))
            })
            .copied()
    }

    /// The active conversation is the one we're currently or have most recently streamed outputs for.
    /// If you want to get the conversation the next query will follow up in / what is selected in the input selector,
    /// use `context_model.selected_conversation` instead.
    pub fn active_conversation(&self, terminal_view_id: EntityId) -> Option<&AIConversation> {
        self.active_conversation_id(terminal_view_id)
            .and_then(|id| self.conversation(&id))
    }

    /// True if this conversation was started from a passive entrypoint, AND the user has made no follow ups.
    pub fn is_entirely_passive_conversation(&self, conversation_id: &AIConversationId) -> bool {
        self.conversation(conversation_id)
            .is_some_and(|conversation| conversation.is_entirely_passive())
    }

    pub fn is_exchange_hidden(
        &self,
        conversation_id: AIConversationId,
        exchange_id: AIAgentExchangeId,
    ) -> bool {
        self.conversations_by_id
            .get(&conversation_id)
            .is_some_and(|c| c.is_exchange_hidden(exchange_id))
    }

    /// Add a new [`AIAgentExchange`] to the [`AIConversation`] with the given [`AIConversationId`].
    /// Emits an event with the new exchange.
    pub(super) fn update_conversation_for_new_request_input(
        &mut self,
        request_input: RequestInput,
        stream_id: ResponseStreamId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) -> Result<(), UpdateHistoryError> {
        let conversation = self
            .conversations_by_id
            .get_mut(&request_input.conversation_id)
            .ok_or(UpdateHistoryError::ConversationNotFound(
                request_input.conversation_id,
            ))?;
        conversation.update_for_new_request_input(
            request_input,
            stream_id,
            terminal_view_id,
            ctx,
        )?;
        Ok(())
    }

    pub fn restore_conversations(
        &mut self,
        terminal_view_id: EntityId,
        conversations: Vec<AIConversation>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.terminal_view_created_at
            .insert(terminal_view_id, Local::now());

        let mut conversation_ids = Vec::new();
        for conversation in conversations.into_iter() {
            let conversation_id = conversation.id();
            conversation_ids.push(conversation_id);
            self.live_conversation_ids_for_terminal_view
                .entry(terminal_view_id)
                .or_default()
                .push(conversation_id);

            let new_status = conversation.status().clone();
            self.conversations_by_id
                .insert(conversation_id, conversation);

            // Emit UpdatedConversationStatus for restored conversations so that
            // the workspace can set tab indicators appropriately
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationStatus {
                conversation_id,
                terminal_view_id,
                update: ConversationStatusUpdate::Restored,
                new_status,
            });
        }

        // Emit event so AI document views can populate their terminal view references
        ctx.emit(BlocklistAIHistoryEvent::RestoredConversations {
            terminal_view_id,
            conversation_ids,
        });
    }

    /// Sets the active conversation ID and transfers ownership from any other terminal view.
    pub fn set_active_conversation_id(
        &mut self,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        if !self
            .live_conversation_ids_for_terminal_view
            .get(&terminal_view_id)
            .is_some_and(|conversation_ids| conversation_ids.contains(&conversation_id))
        {
            log::error!(
                "Attempted to set active conversation ID for terminal view ID that does not own that conversation."
            );
            return;
        }

        // Track previous owners we removed the conversation from so we can
        // emit ownership-transfer events outside of the borrow of
        // `live_conversation_ids_for_terminal_view`. The conversation rendering
        // model assumes a single canonical owner per conversation, so each
        // previous owner needs a chance to drop its now-stale rendered AI
        // blocks.
        let mut previous_owners: Vec<EntityId> = Vec::new();
        for (other_terminal_view, other_terminal_view_live_conversation_ids) in self
            .live_conversation_ids_for_terminal_view
            .iter_mut()
            .filter(|(other_terminal_view_id, _)| **other_terminal_view_id != terminal_view_id)
        {
            if let Some(pos) = other_terminal_view_live_conversation_ids
                .iter()
                .position(|id| *id == conversation_id)
            {
                other_terminal_view_live_conversation_ids.remove(pos);
                previous_owners.push(*other_terminal_view);
            }

            if self
                .active_conversation_for_terminal_view
                .get(other_terminal_view)
                .is_some_and(|id| *id == conversation_id)
            {
                self.active_conversation_for_terminal_view
                    .remove(other_terminal_view);
                ctx.emit(BlocklistAIHistoryEvent::ClearedActiveConversation {
                    conversation_id,
                    terminal_view_id: *other_terminal_view,
                });
            }
        }
        for previous_terminal_view_id in previous_owners {
            ctx.emit(BlocklistAIHistoryEvent::ConversationOwnershipTransferred {
                conversation_id,
                previous_terminal_view_id,
                new_terminal_view_id: terminal_view_id,
            });
        }

        self.active_conversation_for_terminal_view
            .insert(terminal_view_id, conversation_id);

        ctx.emit(BlocklistAIHistoryEvent::SetActiveConversation {
            conversation_id,
            terminal_view_id,
        });
    }

    /// Marks a conversation as active for one terminal view without transferring ownership.
    pub fn mark_active_conversation_id(
        &mut self,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        if !self
            .live_conversation_ids_for_terminal_view
            .get(&terminal_view_id)
            .is_some_and(|conversation_ids| conversation_ids.contains(&conversation_id))
        {
            log::warn!(
                "mark_active_conversation_id: conversation {conversation_id:?} is not in \
                 terminal view {terminal_view_id:?} live list, skipping"
            );
            return;
        }

        self.active_conversation_for_terminal_view
            .insert(terminal_view_id, conversation_id);

        ctx.emit(BlocklistAIHistoryEvent::SetActiveConversation {
            conversation_id,
            terminal_view_id,
        });
    }

    /// Starts a new conversation in the given terminal view's history, effectively marking the
    /// existing conversation (if any) as completed.
    ///
    /// Returns the ID of the created conversation.
    ///
    /// Conversation completion is inferred if the conversation in question is _not_ the last
    /// element in the `conversations` vector.
    pub fn start_new_conversation(
        &mut self,
        terminal_view_id: EntityId,
        is_autoexecute_override: bool,
        ctx: &mut ModelContext<Self>,
    ) -> AIConversationId {
        let mut new_conversation = AIConversation::new();
        if is_autoexecute_override {
            new_conversation.toggle_autoexecute_override();
        }
        let new_conversation_id = new_conversation.id();
        self.live_conversation_ids_for_terminal_view
            .entry(terminal_view_id)
            .or_default()
            .push(new_conversation_id);
        self.conversations_by_id
            .insert(new_conversation_id, new_conversation);

        ctx.emit(BlocklistAIHistoryEvent::StartedNewConversation {
            new_conversation_id,
            terminal_view_id,
        });

        new_conversation_id
    }

    pub fn create_cli_subagent_task_for_conversation(
        &mut self,
        block_id: BlockId,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) -> Result<TaskId, UpdateHistoryError> {
        let conversation = self
            .conversations_by_id
            .get_mut(&conversation_id)
            .ok_or(UpdateHistoryError::ConversationNotFound(conversation_id))?;
        Ok(conversation.create_optimistic_cli_subagent_task(&block_id, terminal_view_id, ctx))
    }

    pub fn update_conversation_status(
        &mut self,
        terminal_view_id: EntityId,
        conversation_id: AIConversationId,
        status: ConversationStatus,
        ctx: &mut ModelContext<Self>,
    ) {
        self.update_conversation_status_with_error_message(
            terminal_view_id,
            conversation_id,
            status,
            None,
            ctx,
        );
    }

    pub fn update_conversation_status_with_error_message(
        &mut self,
        terminal_view_id: EntityId,
        conversation_id: AIConversationId,
        status: ConversationStatus,
        error_message: Option<String>,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            conversation.update_status_with_error_message(
                status,
                error_message,
                terminal_view_id,
                ctx,
            );
        }
    }

    pub fn on_forked_conversation(
        &mut self,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        // When a conversation is forked and restored into a new terminal view,
        // we want to emit UpdatedStreamingExchange events for every exchange
        // to ensure that all of the existing exchanges are persisted correctly.
        if let Some(conversation) = self.conversations_by_id.get(&conversation_id) {
            for exchange in conversation.all_exchanges().into_iter() {
                let is_hidden = conversation.is_exchange_hidden(exchange.id);
                ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                    exchange_id: exchange.id,
                    terminal_view_id,
                    conversation_id,
                    is_hidden,
                });
            }
        }
    }

    pub fn initialize_local_output_for_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        model_id: LLMId,
        display_name: String,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            if let Err(e) = conversation.initialize_local_output_for_response_stream(
                stream_id,
                terminal_view_id,
                model_id,
                display_name,
                ctx,
            ) {
                log::warn!("Failed to initialize ACP output: {e}");
            }
        }
    }

    pub fn append_local_text_delta_to_response_stream(
        &mut self,
        target: &AcpResponseStreamTarget,
        text_delta: &str,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&target.conversation_id) {
            if let Err(e) = conversation.append_local_text_delta_to_response_stream(
                &target.stream_id,
                target.terminal_view_id,
                text_delta,
                target.model_id.clone(),
                target.display_name.clone(),
                ctx,
            ) {
                log::warn!("Failed to append ACP text delta: {e}");
            }
        }
    }

    pub fn append_local_thought_delta_to_response_stream(
        &mut self,
        target: &AcpResponseStreamTarget,
        text_delta: &str,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&target.conversation_id) {
            if let Err(e) = conversation.append_local_thought_delta_to_response_stream(
                &target.stream_id,
                target.terminal_view_id,
                text_delta,
                target.model_id.clone(),
                target.display_name.clone(),
                ctx,
            ) {
                log::warn!("Failed to append ACP thought delta: {e}");
            }
        }
    }

    pub fn upsert_acp_tool_call_to_response_stream(
        &mut self,
        target: &AcpResponseStreamTarget,
        tool_call: AcpToolCall,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&target.conversation_id) {
            if let Err(e) = conversation.upsert_acp_tool_call_to_response_stream(
                &target.stream_id,
                target.terminal_view_id,
                tool_call,
                target.model_id.clone(),
                target.display_name.clone(),
                ctx,
            ) {
                log::warn!("Failed to upsert ACP tool call: {e}");
            }
        }
    }

    pub fn update_acp_tool_call_to_response_stream(
        &mut self,
        target: &AcpResponseStreamTarget,
        update: ToolCallUpdate,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&target.conversation_id) {
            if let Err(e) = conversation.update_acp_tool_call_to_response_stream(
                &target.stream_id,
                target.terminal_view_id,
                update,
                target.model_id.clone(),
                target.display_name.clone(),
                ctx,
            ) {
                log::warn!("Failed to update ACP tool call: {e}");
            }
        }
    }

    pub fn update_acp_terminal_trace_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        terminal_id: String,
        trace: crate::ai::acp::AcpTerminalTrace,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            if let Err(e) = conversation.update_acp_terminal_trace_to_response_stream(
                stream_id,
                terminal_view_id,
                terminal_id,
                trace,
                ctx,
            ) {
                log::warn!("Failed to update ACP terminal trace: {e}");
            }
        }
    }

    pub fn set_acp_plan_for_response_stream(
        &mut self,
        target: &AcpResponseStreamTarget,
        plan: AcpPlan,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&target.conversation_id) {
            if let Err(e) = conversation.set_acp_plan_for_response_stream(
                &target.stream_id,
                target.terminal_view_id,
                plan,
                target.model_id.clone(),
                target.display_name.clone(),
                ctx,
            ) {
                log::warn!("Failed to set ACP plan: {e}");
            }
        }
    }

    pub fn upsert_acp_permission_to_response_stream(
        &mut self,
        target: &AcpResponseStreamTarget,
        request: AcpPermissionRequest,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&target.conversation_id) {
            if let Err(e) = conversation.upsert_acp_permission_to_response_stream(
                &target.stream_id,
                target.terminal_view_id,
                request,
                target.model_id.clone(),
                target.display_name.clone(),
                ctx,
            ) {
                log::warn!("Failed to upsert ACP permission request: {e}");
            }
        }
    }

    pub fn update_acp_permission_selection_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        request_id: &str,
        option_id: String,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            if let Err(e) = conversation.update_acp_permission_selection_to_response_stream(
                stream_id,
                terminal_view_id,
                request_id,
                option_id,
                ctx,
            ) {
                log::warn!("Failed to update ACP permission selection: {e}");
            }
        }
    }

    pub fn set_acp_conversation_title(
        &mut self,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        title: String,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            conversation.set_display_title(title);
            ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationMetadata {
                terminal_view_id: Some(terminal_view_id),
                conversation_id,
            });
        }
    }

    pub fn assign_run_id_for_conversation(
        &mut self,
        conversation_id: AIConversationId,
        run_id: String,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) else {
            log::warn!(
                "assign_run_id_for_conversation: conversation {conversation_id:?} not found"
            );
            return;
        };
        conversation.set_run_id(run_id);
        ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationMetadata {
            terminal_view_id: Some(terminal_view_id),
            conversation_id,
        });
    }

    pub fn fork_conversation(
        &mut self,
        source_conversation: &AIConversation,
        prefix: &str,
        app: &AppContext,
    ) -> Result<AIConversation, anyhow::Error> {
        let Some(sqlite_sender) = GlobalResourceHandlesProvider::as_ref(app)
            .get()
            .model_event_sender
            .clone()
        else {
            return Err(anyhow!("No sqlite sender available."));
        };

        // We preserve reverted action IDs. Orphaned IDs (for actions not in fork) are harmless.
        // The reverted states are only copied to the new conversation if the revert happened before the user clicked fork,
        // but regardless of when the revert happened relative to the fork point.
        //
        // Example:
        // 1. Agent edit action
        // 2. Agent edit action
        // 3. User reverts edit from 1
        // 4. **User clicks fork**
        // 5. User reverts edit from 2
        //
        // In this example, the forked conversation will always show edit 1 as reverted and edit 2 as not reverted,
        // regardless of if the fork point is between 2 and 3 or 3 and 4. This is because we preserve all prior reverts,
        // either if they game before or after the fork point. However, once forked, we don't copy later reverts.
        let reverted_action_ids = if source_conversation.reverted_action_ids().is_empty() {
            None
        } else {
            Some(
                source_conversation
                    .reverted_action_ids()
                    .clone()
                    .into_iter()
                    .map_into()
                    .collect(),
            )
        };

        let acp_transcript_json = source_conversation
            .acp_transcript_json()
            .ok_or_else(|| anyhow!("Conversation has no ACP transcript."))?;
        let display_title = source_conversation
            .title()
            .map(|title| format!("{prefix}{title}"));
        let conversation_data = AgentConversationData {
            reverted_action_ids,
            artifacts_json: None,
            run_id: None,
            autoexecute_override: Some(source_conversation.autoexecute_override().into()),
            display_title,
            acp_transcript_json,
        };
        let forked_conversation_id = AIConversationId::new();
        if let Err(e) = sqlite_sender.send(ModelEvent::UpdateAgentConversation {
            conversation_id: forked_conversation_id.to_string(),
            conversation_data: conversation_data.clone(),
        }) {
            return Err(anyhow!("Failed to persist forked conversation: {e:?}."));
        }

        let forked_conversation =
            self.insert_forked_conversation(forked_conversation_id, conversation_data.clone())?;

        Ok(forked_conversation)
    }

    /// Forks an existing conversation at a specific exchange boundary.
    pub fn fork_conversation_at_exchange(
        &mut self,
        source_conversation: &AIConversation,
        from_exchange_id: AIAgentExchangeId,
        prefix: &str,
        app: &AppContext,
    ) -> Result<AIConversation, anyhow::Error> {
        let Some(sqlite_sender) = GlobalResourceHandlesProvider::as_ref(app)
            .get()
            .model_event_sender
            .clone()
        else {
            return Err(anyhow!("No sqlite sender available."));
        };

        // We preserve reverted action IDs. Orphaned IDs (for actions not in fork) are harmless.
        // The reverted states are only copied to the new conversation if the revert happened before the user clicked fork,
        // but regardless of when the revert happened relative to the fork point.
        //
        // Example:
        // 1. Agent edit action
        // 2. Agent edit action
        // 3. User reverts edit from 1
        // 4. **User clicks fork**
        // 5. User reverts edit from 2
        //
        // In this example, the forked conversation will always show edit 1 as reverted and edit 2 as not reverted,
        // regardless of if the fork point is between 2 and 3 or 3 and 4. This is because we preserve all prior reverts,
        // either if they game before or after the fork point. However, once forked, we don't copy later reverts.
        let reverted_action_ids = if source_conversation.reverted_action_ids().is_empty() {
            None
        } else {
            Some(
                source_conversation
                    .reverted_action_ids()
                    .clone()
                    .into_iter()
                    .map_into()
                    .collect(),
            )
        };

        let acp_transcript_json = source_conversation
            .acp_transcript_json_until_exchange(from_exchange_id)
            .ok_or_else(|| {
                anyhow!(
                    "No exchanges found for block in conversation {}.",
                    source_conversation.id()
                )
            })?;
        let display_title = source_conversation
            .title()
            .map(|title| format!("{prefix}{title}"));
        let conversation_data = AgentConversationData {
            reverted_action_ids,
            artifacts_json: None,
            run_id: None,
            autoexecute_override: Some(source_conversation.autoexecute_override().into()),
            display_title,
            acp_transcript_json,
        };

        let forked_conversation_id = AIConversationId::new();
        if let Err(e) = sqlite_sender.send(ModelEvent::UpdateAgentConversation {
            conversation_id: forked_conversation_id.to_string(),
            conversation_data: conversation_data.clone(),
        }) {
            return Err(anyhow!(
                "Failed to persist forked conversation at block: {e:?}."
            ));
        }

        let forked_conversation =
            self.insert_forked_conversation(forked_conversation_id, conversation_data)?;

        Ok(forked_conversation)
    }

    pub fn mark_response_stream_completed_successfully(
        &mut self,
        stream_id: &ResponseStreamId,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) else {
            return;
        };
        if let Err(e) = conversation.mark_request_completed(stream_id, terminal_view_id, ctx) {
            log::warn!("Failed to mark exchange as completed: {e}");
        }
    }

    pub fn set_exchange_time_to_first_token(
        &mut self,
        conversation_id: AIConversationId,
        exchange_id: AIAgentExchangeId,
        time_to_first_token_ms: i64,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            if let Ok(exchange) = conversation.get_exchange_to_update(exchange_id) {
                exchange.time_to_first_token_ms = Some(time_to_first_token_ms);
            }
        }
    }

    pub fn mark_response_stream_cancelled(
        &mut self,
        stream_id: &ResponseStreamId,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            if reason.is_reverted() {
                if let Err(e) =
                    conversation.mark_request_cancelled_due_to_revert(terminal_view_id, ctx)
                {
                    log::warn!("Failed to mark exchange as cancelled: {e}");
                }
            } else if let Err(e) =
                conversation.mark_request_cancelled(stream_id, terminal_view_id, reason, ctx)
            {
                log::warn!("Failed to mark exchange as cancelled: {e}");
            }
        }
        AIDocumentModel::handle(ctx).update(ctx, |model, ctx| {
            model.clear_streaming_documents_for_conversation(&conversation_id, ctx);
        });
    }

    pub fn mark_response_stream_completed_with_error(
        &mut self,
        error: RenderableAIError,
        stream_id: &ResponseStreamId,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            if let Err(e) = conversation.mark_request_completed_with_error(
                stream_id,
                error.clone(),
                terminal_view_id,
                ctx,
            ) {
                log::warn!("Failed to mark exchange as completed with error: {e}");
            }
        }
    }

    /// Handle clearing the blocklist for the terminal view.
    /// The terminal view will also cancel the active stream on processing the event emitted here.
    pub(crate) fn clear_conversations_in_terminal_view(
        &mut self,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        // Cancel the active stream when we clear conversations in this terminal view.
        let active_conversation_id = self
            .active_conversation_for_terminal_view
            .remove(&terminal_view_id);
        if let Some(cleared_conversation_ids) = self
            .live_conversation_ids_for_terminal_view
            .remove(&terminal_view_id)
        {
            self.cleared_conversation_ids_for_terminal_view
                .entry(terminal_view_id)
                .and_modify(|existing| existing.extend(cleared_conversation_ids.clone()))
                .or_insert(cleared_conversation_ids);
        }
        let cleared_conversation_ids = self
            .live_conversation_ids_for_terminal_view
            .remove(&terminal_view_id);
        if let Some(cleared_conversation_ids) = cleared_conversation_ids {
            self.cleared_conversation_ids_for_terminal_view
                .entry(terminal_view_id)
                .and_modify(|existing| existing.extend(cleared_conversation_ids.clone()))
                .or_insert(cleared_conversation_ids);
        }
        ctx.emit(
            BlocklistAIHistoryEvent::ClearedConversationsInTerminalView {
                terminal_view_id,
                active_conversation_id,
            },
        );
    }

    /// Handle removing a conversation from the history model, blocklist and in-memory.
    pub fn remove_conversation(
        &mut self,
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.remove_conversation_from_memory(conversation_id, Some(terminal_view_id), ctx);
    }

    /// Permanently delete a conversation.
    pub fn delete_conversation(
        &mut self,
        conversation_id: AIConversationId,
        terminal_view_id: Option<EntityId>,
        ctx: &mut ModelContext<Self>,
    ) {
        let conversation_title = self
            .conversations_by_id
            .get(&conversation_id)
            .and_then(|c| c.title().map(|t| t.to_string()));
        // Capture the run_id BEFORE the in-memory record is dropped so it
        // can be forwarded on the DeletedConversation event.
        let run_id = self
            .conversations_by_id
            .get(&conversation_id)
            .and_then(|c| c.run_id());

        self.remove_conversation_from_memory(conversation_id, terminal_view_id, ctx);

        // Delete persisted conversation from sqlite.
        let model_event_sender = GlobalResourceHandlesProvider::as_ref(ctx)
            .get()
            .model_event_sender
            .clone();
        let conversation_id_string = conversation_id.to_string();
        ctx.spawn(
            async move {
                if let Some(sender) = model_event_sender {
                    if let Err(e) = sender.send(ModelEvent::DeleteAIConversation {
                        conversation_id: conversation_id_string.clone(),
                    }) {
                        log::error!("Error sending DeleteAIConversation event: {e:?}");
                    }
                    if let Err(e) = sender.send(ModelEvent::DeleteAgentConversations {
                        conversation_ids: vec![conversation_id_string],
                    }) {
                        log::error!("Error sending DeleteAgentConversations event: {e:?}");
                    }
                }
            },
            |_, _, _| {},
        );

        // Only emit the event if we have a terminal_view_id, since the event is
        // filtered by terminal_view_id in handlers.
        if let Some(terminal_view_id) = terminal_view_id {
            ctx.emit(BlocklistAIHistoryEvent::DeletedConversation {
                terminal_view_id,
                conversation_id,
                conversation_title,
                run_id,
            });
        }
    }

    /// Remove a conversation from all in-memory storage.
    fn remove_conversation_from_memory(
        &mut self,
        conversation_id: AIConversationId,
        terminal_view_id: Option<EntityId>,
        ctx: &mut ModelContext<Self>,
    ) {
        // Capture the run_id BEFORE the in-memory record is dropped so the
        // RemoveConversation event can carry it (event subscribers can no
        // longer look it up via `conversation()` after this function returns).
        let run_id = self
            .conversations_by_id
            .get(&conversation_id)
            .and_then(|c| c.run_id());

        self.all_conversations_metadata.remove(&conversation_id);
        self.conversations_by_id.remove(&conversation_id);

        if let Some(terminal_view_id) = terminal_view_id {
            if self
                .active_conversation_for_terminal_view
                .get(&terminal_view_id)
                .is_some_and(|id| *id == conversation_id)
            {
                self.active_conversation_for_terminal_view
                    .remove(&terminal_view_id);
            }
            if let Some(vec) = self
                .live_conversation_ids_for_terminal_view
                .get_mut(&terminal_view_id)
            {
                vec.retain(|&id| id != conversation_id);
            }
            if let Some(vec) = self
                .cleared_conversation_ids_for_terminal_view
                .get_mut(&terminal_view_id)
            {
                vec.retain(|&id| id != conversation_id);
            }
            ctx.emit(BlocklistAIHistoryEvent::RemoveConversation {
                terminal_view_id,
                conversation_id,
                run_id,
            });
        }
    }

    /// Returns true if the conversation is live in any terminal view.
    pub fn is_conversation_live(&self, conversation_id: AIConversationId) -> bool {
        self.live_conversation_ids_for_terminal_view
            .values()
            .any(|conversation_ids| conversation_ids.contains(&conversation_id))
    }

    pub fn mark_terminal_view_as_conversation_transcript_viewer(
        &mut self,
        terminal_view_id: EntityId,
    ) {
        self.conversation_transcript_viewer_terminal_view_ids
            .insert(terminal_view_id);
    }

    pub fn is_terminal_view_conversation_transcript_viewer(
        &self,
        terminal_view_id: EntityId,
    ) -> bool {
        self.conversation_transcript_viewer_terminal_view_ids
            .contains(&terminal_view_id)
    }

    /// Returns [`AIQueryHistory`]s from all sources: live conversations, cleared conversations,
    /// and persisted queries from conversations not loaded in memory.
    ///
    /// When `terminal_view_id` is provided, queries from that terminal view are categorized as
    /// `CurrentSession` and all others as `DifferentSession`. When `None`, all queries are
    /// categorized as `DifferentSession`.
    ///
    pub(crate) fn all_ai_queries(
        &self,
        terminal_view_id: Option<EntityId>,
    ) -> impl Iterator<Item = AIQueryHistory> + '_ {
        // Collect all conversation IDs that are already in memory (live or cleared)
        // and build query vectors in the same loops
        let mut loaded_conversation_ids: HashSet<AIConversationId> = HashSet::new();

        let mut live_queries_vec = Vec::new();
        for (tv_id, conversation_ids) in self.live_conversation_ids_for_terminal_view.iter() {
            loaded_conversation_ids.extend(conversation_ids);

            let history_order = if terminal_view_id.is_some_and(|id| id == *tv_id) {
                HistoryOrder::CurrentSession
            } else {
                HistoryOrder::DifferentSession
            };

            for conversation_id in conversation_ids {
                if let Some(conversation) = self.conversations_by_id.get(conversation_id) {
                    for exchange in conversation.root_task_exchanges() {
                        if let Some(query) = ai_exchange_to_query_history(exchange, history_order) {
                            live_queries_vec.push(query);
                        }
                    }
                }
            }
        }

        let mut cleared_queries_vec = Vec::new();
        for (tv_id, conversation_ids) in self.cleared_conversation_ids_for_terminal_view.iter() {
            loaded_conversation_ids.extend(conversation_ids);

            let history_order = if terminal_view_id.is_some_and(|id| id == *tv_id) {
                HistoryOrder::CurrentSession
            } else {
                HistoryOrder::DifferentSession
            };

            for conversation_id in conversation_ids {
                if let Some(conversation) = self.conversations_by_id.get(conversation_id) {
                    for exchange in conversation.root_task_exchanges() {
                        if let Some(query) = ai_exchange_to_query_history(exchange, history_order) {
                            cleared_queries_vec.push(query);
                        }
                    }
                }
            }
        }

        // Add persisted queries from conversations not loaded in memory
        let persisted_queries_vec: Vec<_> = self
            .persisted_queries
            .iter()
            .filter(|persisted| !loaded_conversation_ids.contains(&persisted.conversation_id))
            .filter_map(|persisted| {
                persisted_ai_input_to_query_history(persisted, HistoryOrder::DifferentSession)
            })
            .collect();

        persisted_queries_vec
            .into_iter()
            .chain(cleared_queries_vec)
            .chain(live_queries_vec)
    }

    /// Returns `Some` with the [`AIConversationId`] of the active conversation inside the
    /// [`crate::terminal::TerminalView`] with the given [`EntityId`] if there is one. Returns
    /// `None` otherwise.
    /// The active conversation is the one we're currently or have most recently streamed outputs for.
    /// If you want to check what conversation the next query will follow up in / what is selected in the input selector,
    /// use `context_model.selected_conversation_id` instead.
    pub(crate) fn active_conversation_id(
        &self,
        terminal_view_id: EntityId,
    ) -> Option<AIConversationId> {
        let active_conversation_id = self
            .active_conversation_for_terminal_view
            .get(&terminal_view_id)
            .copied()?;

        let conversation_ids_for_terminal_view = self
            .live_conversation_ids_for_terminal_view
            .get(&terminal_view_id)?;

        if !conversation_ids_for_terminal_view.contains(&active_conversation_id) {
            log::warn!(
                "The active conversation ID {active_conversation_id:?} was not found in the list of conversation IDs for terminal view {terminal_view_id:?}. Conversation IDs: {conversation_ids_for_terminal_view:?}"
            );
            return None;
        }

        Some(active_conversation_id)
    }

    /// Set the hidden status of the exchange with the given ID.
    pub fn set_exchange_hidden_status(
        &mut self,
        terminal_view_id: EntityId,
        conversation_id: AIConversationId,
        exchange_id: AIAgentExchangeId,
        is_hidden: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) else {
            return;
        };
        conversation.set_is_exchange_hidden(exchange_id, is_hidden, terminal_view_id, ctx);
    }

    pub fn set_has_code_review_opened_to_true(&mut self, conversation_id: AIConversationId) {
        if let Some(conversation) = self.conversations_by_id.get_mut(&conversation_id) {
            conversation.mark_code_review_as_opened();
        }
    }

    pub fn toggle_autoexecute_override(
        &mut self,
        conversation_id: &AIConversationId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(conversation) = self.conversations_by_id.get_mut(conversation_id) else {
            return;
        };

        conversation.toggle_autoexecute_override();
        conversation.write_updated_conversation_state(ctx);
        ctx.emit(BlocklistAIHistoryEvent::UpdatedAutoexecuteOverride { terminal_view_id });
    }

    /// Truncates a conversation from the given exchange ID, removing all exchanges
    /// from that exchange onwards (inclusive). This is a lossy operation.
    ///
    /// Returns the set of exchange IDs that were removed.
    pub fn truncate_conversation_from_exchange(
        &mut self,
        conversation_id: AIConversationId,
        from_exchange_id: AIAgentExchangeId,
        ctx: &mut ModelContext<Self>,
    ) -> Result<HashSet<AIAgentExchangeId>, UpdateHistoryError> {
        let conversation = self
            .conversations_by_id
            .get_mut(&conversation_id)
            .ok_or(UpdateHistoryError::ConversationNotFound(conversation_id))?;

        let removed_exchange_ids = conversation.truncate_from_exchange(from_exchange_id, ctx)?;

        Ok(removed_exchange_ids)
    }

    /// Returns the latest exchange across all conversations in the terminal view.
    /// This is useful for determining if a specific exchange is the most recent one.
    /// Excludes passive code generation exchanges from consideration.
    pub fn latest_exchange_across_all_conversations(
        &self,
        terminal_view_id: EntityId,
    ) -> Option<&AIAgentExchange> {
        self.all_live_root_task_exchanges_for_terminal_view(terminal_view_id)
            .filter(|exchange| !exchange.has_passive_request())
            .max_by_key(|exchange| exchange.start_time)
    }

    /// Returns the conversation ID that contains the given exchange ID, if any.
    /// Searches through all conversations for a given terminal view.
    pub fn conversation_id_for_exchange(
        &self,
        exchange_id: AIAgentExchangeId,
        terminal_view_id: EntityId,
    ) -> Option<AIConversationId> {
        self.live_conversation_ids_for_terminal_view
            .get(&terminal_view_id)?
            .iter()
            .find(|conversation_id| {
                self.conversations_by_id
                    .get(conversation_id)
                    .is_some_and(|conversation| {
                        conversation.exchange_with_id(exchange_id).is_some()
                    })
            })
            .copied()
    }

    /// Returns local conversation metadata.
    pub fn get_local_conversations_metadata(
        &self,
    ) -> impl Iterator<Item = &AIConversationMetadata> {
        self.all_conversations_metadata.values()
    }

    /// Returns conversation metadata for a specific conversation ID.
    pub fn get_conversation_metadata(
        &self,
        conversation_id: &AIConversationId,
    ) -> Option<&AIConversationMetadata> {
        self.all_conversations_metadata.get(conversation_id)
    }

    /// Mark conversations as historical
    /// Historical conversations consist of non-live conversations that were read from disk on startup,
    /// and conversations (recorded here) that were live this session but have now been cleared.
    pub fn mark_conversations_historical_for_terminal_view(&mut self, terminal_view_id: EntityId) {
        if self.is_terminal_view_conversation_transcript_viewer(terminal_view_id) {
            // We don't mark conversation transcript viewer conversations as historical,
            // as they are stored separately and should not be persisted/displayed as regular user conversations.
            return;
        }

        // There's a slight concern here that the conversations we're preserving might not have persisted successfully
        // because of some unexpected error. Attempting to then restore these conversations would lead to unexpected behavior.
        // In the future it might be worthwhile to check that these conversations exist in the database before marking them as historical,
        // but for now this is an edge case that we don't need to worry about too much.
        let conversations_to_mark_historical: Vec<AIConversationMetadata> = self
            .all_live_conversations_for_terminal_view(terminal_view_id)
            .filter_map(|conversation| {
                let conversation_id = conversation.id();
                if !self.conversations_by_id.contains_key(&conversation_id)
                    || conversation.should_exclude_from_navigation()
                    || !blocklist_filter::conversation_would_render_in_blocklist(conversation)
                {
                    return None;
                }

                Some(conversation.into())
            })
            .collect();

        for metadata in conversations_to_mark_historical {
            self.all_conversations_metadata
                .insert(metadata.id, metadata);
        }
    }

    pub fn insert_forked_conversation(
        &mut self,
        conversation_id: AIConversationId,
        conversation_data: AgentConversationData,
    ) -> anyhow::Result<AIConversation> {
        let mut conversation = AIConversation::new_restored(conversation_id, conversation_data)?;

        // Assign fresh exchange IDs so persisted blocks do not collide.
        conversation.reassign_exchange_ids();

        self.conversations_by_id
            .insert(conversation_id, conversation.clone());

        let metadata = AIConversationMetadata::from(&conversation);
        self.all_conversations_metadata
            .insert(conversation_id, metadata);

        Ok(conversation)
    }
}

/// Whether an `UpdatedConversationStatus` event represents a restoration
/// (the conversation was re-loaded into a terminal view; the underlying
/// `ConversationStatus` did not change) or a real status set, in which case
/// the previous status is included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationStatusUpdate {
    Restored,
    Changed { prev_status: ConversationStatus },
}

#[derive(Clone, Debug)]
pub enum BlocklistAIHistoryEvent {
    /// A new conversation was started.
    StartedNewConversation {
        new_conversation_id: AIConversationId,
        terminal_view_id: EntityId,
    },

    CreatedSubtask {
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        task_id: TaskId,
    },

    AppendedExchange {
        exchange_id: AIAgentExchangeId,
        task_id: TaskId,
        terminal_view_id: EntityId,
        conversation_id: AIConversationId,
        is_hidden: bool,

        // Populated if this exchange is appended as a result of an in-flight API request.
        response_stream_id: Option<ResponseStreamId>,
    },

    ReassignedExchange {
        exchange_id: AIAgentExchangeId,
        terminal_view_id: EntityId,
        new_task_id: TaskId,
        new_conversation_id: AIConversationId,
    },

    /// Includes the terminal view's [`EntityId`] so we can disambiguate the source of the event
    /// because this [`BlocklistAIHistoryModel`] is global.
    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    UpdatedStreamingExchange {
        exchange_id: AIAgentExchangeId,
        terminal_view_id: EntityId,
        conversation_id: AIConversationId,
        is_hidden: bool,
    },

    UpdatedConversationStatus {
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
        /// Distinguishes a restoration from a real status set.
        update: ConversationStatusUpdate,
        /// The conversation's status after this update.
        new_status: ConversationStatus,
    },

    /// The active conversation was set to another conversation in the history.
    SetActiveConversation {
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
    },

    /// `conversation_id` is no longer marked as active for the given terminal view.
    ClearedActiveConversation {
        conversation_id: AIConversationId,
        terminal_view_id: EntityId,
    },

    ClearedConversationsInTerminalView {
        terminal_view_id: EntityId,
        active_conversation_id: Option<AIConversationId>,
    },

    UpdatedTodoList {
        terminal_view_id: EntityId,
    },

    UpdatedAutoexecuteOverride {
        terminal_view_id: EntityId,
    },

    /// Emitted when a conversation is split into two (on suggest starting new conversation)
    SplitConversation {
        terminal_view_id: EntityId,
        old_conversation_id: AIConversationId,
        new_conversation_id: AIConversationId,
    },

    RemoveConversation {
        terminal_view_id: EntityId,
        conversation_id: AIConversationId,
        run_id: Option<String>,
    },

    DeletedConversation {
        terminal_view_id: EntityId,
        conversation_id: AIConversationId,
        conversation_title: Option<String>,
        run_id: Option<String>,
    },

    /// Emitted when conversations are restored in a terminal view.
    RestoredConversations {
        terminal_view_id: EntityId,
        conversation_ids: Vec<AIConversationId>,
    },

    /// Emitted when conversation metadata is updated.
    /// `terminal_view_id` is None when updating historical-only conversations.
    UpdatedConversationMetadata {
        terminal_view_id: Option<EntityId>,
        conversation_id: AIConversationId,
    },

    /// Emitted when conversation artifacts are updated (plans, PRs, etc.)
    UpdatedConversationArtifacts {
        terminal_view_id: EntityId,
        conversation_id: AIConversationId,
        artifact: Artifact,
    },

    /// Emitted when a conversation moves between terminal views — i.e. when
    /// `set_active_conversation_id` removes the conversation from the live
    /// list of one or more `previous_terminal_view_id`s. The previous owners
    /// must drop any rendered AI blocks for this conversation so the new
    /// owner is the sole renderer; otherwise we end up with a transcript
    /// split across panes (some blocks in the old view, new exchanges in the
    /// new view). The `terminal_view_id()` accessor returns the previous
    /// owner so existing per-view event filters do the right thing.
    ConversationOwnershipTransferred {
        conversation_id: AIConversationId,
        previous_terminal_view_id: EntityId,
        new_terminal_view_id: EntityId,
    },
}

impl BlocklistAIHistoryEvent {
    /// Returns the terminal view ID associated with this event, if any.
    /// Returns `None` for events that apply globally (e.g., historical conversation metadata updates).
    pub fn terminal_view_id(&self) -> Option<EntityId> {
        match self {
            BlocklistAIHistoryEvent::StartedNewConversation {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::AppendedExchange {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::UpdatedConversationStatus {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::SetActiveConversation {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::ClearedActiveConversation {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::ClearedConversationsInTerminalView {
                terminal_view_id,
                ..
            }
            | BlocklistAIHistoryEvent::ReassignedExchange {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::UpdatedTodoList {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::UpdatedAutoexecuteOverride {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::SplitConversation {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::RemoveConversation {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::DeletedConversation {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::CreatedSubtask {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::RestoredConversations {
                terminal_view_id, ..
            }
            | BlocklistAIHistoryEvent::ConversationOwnershipTransferred {
                previous_terminal_view_id: terminal_view_id,
                ..
            }
            | BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
                terminal_view_id, ..
            } => Some(*terminal_view_id),
            // UpdatedConversationMetadata can have None when updating historical-only conversations
            BlocklistAIHistoryEvent::UpdatedConversationMetadata {
                terminal_view_id, ..
            } => *terminal_view_id,
        }
    }
}

impl Entity for BlocklistAIHistoryModel {
    type Event = BlocklistAIHistoryEvent;
}

impl SingletonEntity for BlocklistAIHistoryModel {}

/// Helper struct for showing AI history to the user. Guarantees that there is a user query and
/// contains less data than [`AIAgentExchange`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AIQueryHistory {
    /// The input originating from the user.
    pub query_text: String,

    /// The time the input was sent.
    pub start_time: DateTime<Local>,

    /// The status of the output streaming from the AI API.
    pub output_status: AIQueryHistoryOutputStatus,

    /// The working directory when the AI query was submitted.
    pub working_directory: Option<String>,

    /// The ordering category for this query in history.
    pub history_order: HistoryOrder,
}

impl AIQueryHistory {
    /// Creates a new [`AIQueryHistory`] for testing.
    #[cfg(test)]
    pub(crate) fn new_for_test(
        query_text: &str,
        start_time: DateTime<Local>,
        history_order: HistoryOrder,
    ) -> Self {
        Self {
            query_text: query_text.to_owned(),
            start_time,
            output_status: AIQueryHistoryOutputStatus::Pending,
            working_directory: None,
            history_order,
        }
    }
}

fn ai_exchange_to_query_history(
    value: &AIAgentExchange,
    history_order: HistoryOrder,
) -> Option<AIQueryHistory> {
    let query = value.input.iter().find_map(AIAgentInput::user_query)?;

    Some(AIQueryHistory {
        query_text: query,
        start_time: value.start_time,
        output_status: AIQueryHistoryOutputStatus::from(&value.output_status),
        working_directory: value.working_directory.clone(),
        history_order,
    })
}

fn persisted_ai_input_to_query_history(
    value: &PersistedAIInput,
    history_order: HistoryOrder,
) -> Option<AIQueryHistory> {
    // Extract the query text from the first Query input
    let query_text = value
        .inputs
        .iter()
        .map(|input| match input {
            PersistedAIInputType::Query { text, .. } => Some(text.clone()),
        })
        .next()
        .flatten()?;

    Some(AIQueryHistory {
        query_text,
        start_time: value.start_ts,
        output_status: value.output_status.clone(),
        working_directory: value.working_directory.clone(),
        history_order,
    })
}

/// Status of output streaming from the AI API.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AIQueryHistoryOutputStatus {
    /// We are waiting to or are currently streaming output.
    Pending,
    /// The user manually cancelled output streaming.
    Cancelled,
    /// Output streaming failed.
    Failed,
    /// Output streaming completed successfully.
    Completed,
}

impl AIQueryHistoryOutputStatus {
    /// Returns a string representation of the output status.
    pub(crate) fn display_text(&self) -> &'static str {
        match self {
            AIQueryHistoryOutputStatus::Completed => "Completed successfully",
            AIQueryHistoryOutputStatus::Pending => "Pending",
            AIQueryHistoryOutputStatus::Cancelled => "Cancelled by user",
            AIQueryHistoryOutputStatus::Failed => "Failed",
        }
    }

    pub(crate) fn icon(&self) -> Icon {
        match self {
            AIQueryHistoryOutputStatus::Completed => Icon::Check,
            AIQueryHistoryOutputStatus::Pending => Icon::Loading,
            AIQueryHistoryOutputStatus::Cancelled => Icon::SlashCircle,
            AIQueryHistoryOutputStatus::Failed => Icon::AlertTriangle,
        }
    }
}

impl From<&AIAgentOutputStatus> for AIQueryHistoryOutputStatus {
    fn from(status: &AIAgentOutputStatus) -> Self {
        match status {
            AIAgentOutputStatus::Streaming { .. } => Self::Pending,
            AIAgentOutputStatus::Finished {
                finished_output, ..
            } => match finished_output {
                FinishedAIAgentOutput::Cancelled { .. } => Self::Cancelled,
                FinishedAIAgentOutput::Error { .. } => Self::Failed,
                FinishedAIAgentOutput::Success { .. } => Self::Completed,
            },
        }
    }
}

/// The default prefix used when forking a conversation.
pub const FORK_PREFIX: &str = "(Fork) ";

/// The prefix used when saving a conversation before a rewind operation.
pub const PRE_REWIND_PREFIX: &str = "(Pre-Rewind) ";

#[cfg(test)]
#[path = "history_model_test.rs"]
mod tests;
