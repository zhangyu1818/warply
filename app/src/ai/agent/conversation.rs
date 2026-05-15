use crate::ai::acp::{AcpPermissionRequest, AcpPlan, AcpTerminalTrace, AcpToolCall};
use crate::ai::agent::comment::CodeReview;
use crate::ai::agent::util::parse_markdown_into_text_and_code_sections;
use crate::ai::artifacts::Artifact;
use crate::ai::blocklist::{RequestInput, ResponseStreamId};
use crate::ai::llms::LLMId;
use crate::terminal::general_settings::GeneralSettings;
use crate::terminal::model::block::BlockId;
use chrono::{DateTime, Local};
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};

use super::task_store::TaskStore;
use agent_client_protocol::schema::ToolCallUpdate;
use uuid::Uuid;
use vec1::{Size0Error, Vec1};
use warp_core::execution_mode::AppExecutionMode;
use warp_core::ui::appearance::Appearance;
use warp_core::ui::theme::color::internal_colors;
use warp_core::ui::theme::WarpTheme;
use warpui::color::ColorU;
use warpui::{EntityId, ModelContext, SingletonEntity};

use crate::ai::agent::CancellationReason;
use crate::{
    ai::{
        agent::{
            icons::{
                failed_icon, gray_stop_icon, in_progress_icon, succeeded_icon, yellow_stop_icon,
            },
            todos::AIAgentTodoList,
            AIAgentOutputMessage, AIAgentOutputMessageType, AIAgentText, MarkdownTextSection,
        },
        blocklist::{BlocklistAIHistoryEvent, ConversationStatusUpdate},
    },
    persistence::{
        model::{AgentConversationData, PersistedAutoexecuteMode},
        ModelEvent,
    },
    ui_components::icons::Icon,
    BlocklistAIHistoryModel, GlobalResourceHandlesProvider,
};

use super::task::UpdateTaskError;
use super::{
    task::{
        transaction::{SavedTask, Transaction},
        Task, TaskId,
    },
    AIAgentAction, AIAgentActionId, AIAgentContext, AIAgentExchange, AIAgentExchangeId,
    AIAgentInput, AIAgentOutputStatus, AIAgentTodo, AIAgentTodoId, FinishedAIAgentOutput,
    MessageId, RenderableAIError, UserQueryMode,
};
use super::{AIAgentOutput, OutputModelInfo, Shared};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Stopped,
}

impl TodoStatus {
    pub fn is_cancelled(&self) -> bool {
        matches!(self, TodoStatus::Cancelled)
    }
}

#[derive(Debug, Clone)]
struct AddedExchange {
    #[allow(dead_code)]
    task_id: TaskId,
    exchange_id: AIAgentExchangeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AcpTranscript {
    exchanges: Vec<AcpTranscriptExchange>,
}

impl AcpTranscript {
    fn from_conversation(conversation: &AIConversation) -> Option<Self> {
        let exchanges = conversation
            .root_task_exchanges()
            .filter_map(AcpTranscriptExchange::from_exchange)
            .collect::<Vec<_>>();

        if exchanges.is_empty() {
            None
        } else {
            Some(Self { exchanges })
        }
    }

    fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json)
            .map_err(|e| log::error!("Failed to deserialize ACP transcript: {e}"))
            .ok()
    }

    fn into_exchanges(self) -> Vec<AIAgentExchange> {
        self.exchanges
            .into_iter()
            .map(AcpTranscriptExchange::into_exchange)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AcpTranscriptExchange {
    id: AIAgentExchangeId,
    input: Vec<AcpTranscriptInput>,
    output: AcpTranscriptOutput,
    start_time: DateTime<Local>,
    finish_time: Option<DateTime<Local>>,
    working_directory: Option<String>,
    model_id: LLMId,
    coding_model_id: LLMId,
    cli_agent_model_id: LLMId,
    computer_use_model_id: LLMId,
}

impl AcpTranscriptExchange {
    fn from_exchange(exchange: &AIAgentExchange) -> Option<Self> {
        let input = exchange
            .input
            .iter()
            .filter_map(AcpTranscriptInput::from_input)
            .collect::<Vec<_>>();
        let output = exchange.output_status.output()?.get();
        let output = AcpTranscriptOutput::from_output(&output)?;

        if input.is_empty() {
            None
        } else {
            Some(Self {
                id: exchange.id,
                input,
                output,
                start_time: exchange.start_time,
                finish_time: exchange.finish_time,
                working_directory: exchange.working_directory.clone(),
                model_id: exchange.model_id.clone(),
                coding_model_id: exchange.coding_model_id.clone(),
                cli_agent_model_id: exchange.cli_agent_model_id.clone(),
                computer_use_model_id: exchange.computer_use_model_id.clone(),
            })
        }
    }

    fn into_exchange(self) -> AIAgentExchange {
        let (output, added_message_ids) = self.output.into_output();
        AIAgentExchange {
            id: self.id,
            input: self
                .input
                .into_iter()
                .map(AcpTranscriptInput::into_input)
                .collect(),
            output_status: AIAgentOutputStatus::Finished {
                finished_output: FinishedAIAgentOutput::Success {
                    output: Shared::new(output),
                },
            },
            added_message_ids,
            start_time: self.start_time,
            finish_time: self.finish_time,
            working_directory: self.working_directory,
            model_id: self.model_id,
            coding_model_id: self.coding_model_id,
            cli_agent_model_id: self.cli_agent_model_id,
            computer_use_model_id: self.computer_use_model_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AcpTranscriptInput {
    UserQuery {
        query: String,
        user_query_mode: UserQueryMode,
    },
}

impl AcpTranscriptInput {
    fn from_input(input: &AIAgentInput) -> Option<Self> {
        match input {
            AIAgentInput::UserQuery {
                query,
                user_query_mode,
                ..
            } => Some(Self::UserQuery {
                query: query.clone(),
                user_query_mode: *user_query_mode,
            }),
            _ => None,
        }
    }

    fn into_input(self) -> AIAgentInput {
        match self {
            Self::UserQuery {
                query,
                user_query_mode,
            } => AIAgentInput::UserQuery {
                query,
                context: Arc::default(),
                static_query_type: None,
                referenced_attachments: Default::default(),
                user_query_mode,
                running_command: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AcpTranscriptOutput {
    messages: Vec<AcpTranscriptOutputMessage>,
    model_id: Option<LLMId>,
    display_name: Option<String>,
}

impl AcpTranscriptOutput {
    fn from_output(output: &AIAgentOutput) -> Option<Self> {
        let messages = output
            .messages
            .iter()
            .filter_map(AcpTranscriptOutputMessage::from_message)
            .collect::<Vec<_>>();

        if messages.is_empty() {
            None
        } else {
            Some(Self {
                messages,
                model_id: output.model_info.as_ref().map(|info| info.model_id.clone()),
                display_name: output
                    .model_info
                    .as_ref()
                    .map(|info| info.display_name.clone()),
            })
        }
    }

    fn into_output(self) -> (AIAgentOutput, HashSet<MessageId>) {
        let mut output = AIAgentOutput::default();
        let mut added_message_ids = HashSet::new();

        for message in self.messages {
            let message = message.into_message();
            added_message_ids.insert(message.id.clone());
            output.messages.push(message);
        }

        if let (Some(model_id), Some(display_name)) = (self.model_id, self.display_name) {
            output.model_info = Some(OutputModelInfo {
                model_id,
                display_name,
            });
        }

        (output, added_message_ids)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AcpTranscriptOutputMessage {
    Text {
        id: String,
        markdown: String,
    },
    Reasoning {
        id: String,
        markdown: String,
    },
    ToolCall {
        id: String,
        tool_call: AcpToolCall,
    },
    Plan {
        id: String,
        plan: AcpPlan,
    },
    Permission {
        id: String,
        request: AcpPermissionRequest,
    },
}

impl AcpTranscriptOutputMessage {
    fn from_message(message: &AIAgentOutputMessage) -> Option<Self> {
        let id = message.id.to_string();
        match &message.message {
            AIAgentOutputMessageType::Text(text) => Some(Self::Text {
                id,
                markdown: acp_transcript_markdown(text),
            }),
            AIAgentOutputMessageType::Reasoning { text, .. } => Some(Self::Reasoning {
                id,
                markdown: acp_transcript_markdown(text),
            }),
            AIAgentOutputMessageType::AcpToolCall(tool_call) => Some(Self::ToolCall {
                id,
                tool_call: tool_call.clone(),
            }),
            AIAgentOutputMessageType::AcpPlan(plan) => Some(Self::Plan {
                id,
                plan: plan.clone(),
            }),
            AIAgentOutputMessageType::AcpPermission(request) => Some(Self::Permission {
                id,
                request: request.clone(),
            }),
            _ => None,
        }
    }

    fn into_message(self) -> AIAgentOutputMessage {
        match self {
            Self::Text { id, markdown } => AIAgentOutputMessage::text(
                MessageId::new(id),
                AIAgentText {
                    sections: parse_markdown_into_text_and_code_sections(&markdown),
                },
            ),
            Self::Reasoning { id, markdown } => AIAgentOutputMessage::reasoning(
                MessageId::new(id),
                AIAgentText {
                    sections: parse_markdown_into_text_and_code_sections(&markdown),
                },
                None,
            ),
            Self::ToolCall { id, tool_call } => {
                AIAgentOutputMessage::acp_tool_call(MessageId::new(id), tool_call)
            }
            Self::Plan { id, plan } => AIAgentOutputMessage::acp_plan(MessageId::new(id), plan),
            Self::Permission { id, request } => {
                AIAgentOutputMessage::acp_permission(MessageId::new(id), request)
            }
        }
    }
}

fn acp_transcript_markdown(text: &AIAgentText) -> String {
    text.sections
        .iter()
        .map(|section| format!("{}", MarkdownTextSection(section)))
        .join("\n")
}

#[derive(thiserror::Error, Debug)]
pub enum RestoreConversationError {
    #[error("Restored conversation has no root task")]
    NoRootTask,
}

#[derive(thiserror::Error, Debug)]
#[error("Subagent task not found")]
pub struct SubagentTaskNotFound;

/// An Agent Mode conversation.
#[derive(Debug, Clone)]
pub struct AIConversation {
    /// Unique ID for this conversation.
    id: AIConversationId,

    task_store: TaskStore,
    optimistic_cli_subagent_subtask_id: Option<TaskId>,

    /// TODO lists created during the conversation, ordered by creation time. The last list (if any) is the active list.
    todo_lists: Vec<AIAgentTodoList>,

    /// Current the code review in this conversation, `None` if the has never tried to address
    /// comments in this conversation.
    code_review: Option<CodeReview>,

    status: ConversationStatus,
    /// Optional detail for the current error status.
    status_error_message: Option<String>,

    /// Tracks whether the code review has been opened at least once for this conversation.
    has_opened_code_review: bool,

    /// The active transaction for this conversation, if any.
    transaction: Option<Transaction>,

    /// The per-conversation override on the user's usual autonomy settings.
    autoexecute_override: AIConversationAutoexecuteMode,

    added_exchanges_by_response: HashMap<ResponseStreamId, Vec1<AddedExchange>>,

    /// A set of the hidden exchanges.
    /// This is stored here instead of the AIAgentExchange because this is a view specific field.
    /// We cache this here because we don't have access to the block everywhere we are updating the
    /// persisted exchanges.
    hidden_exchanges: HashSet<AIAgentExchangeId>,

    /// A set of action IDs that have been reverted by the user.
    reverted_action_ids: HashSet<AIAgentActionId>,

    /// Fallback title used when no task description or initial query exists.
    fallback_display_title: Option<String>,

    /// Artifacts created during this conversation (plans, PRs, etc.).
    artifacts: Vec<Artifact>,
}

impl AIConversation {
    pub fn new() -> Self {
        let root_task = Task::new_optimistic_root();
        Self {
            id: AIConversationId::new(),
            task_store: TaskStore::with_root_task(root_task),
            optimistic_cli_subagent_subtask_id: None,
            code_review: None,
            todo_lists: vec![],
            status: ConversationStatus::InProgress,
            status_error_message: None,
            has_opened_code_review: false,
            transaction: None,
            autoexecute_override: Default::default(),
            added_exchanges_by_response: Default::default(),
            hidden_exchanges: Default::default(),
            reverted_action_ids: Default::default(),
            fallback_display_title: None,
            artifacts: Vec::new(),
        }
    }

    pub fn new_restored(
        id: AIConversationId,
        conversation_data: AgentConversationData,
    ) -> Result<Self, RestoreConversationError> {
        let acp_transcript = AcpTranscript::from_json(&conversation_data.acp_transcript_json)
            .ok_or(RestoreConversationError::NoRootTask)?;

        let mut root_task = Task::new_optimistic_root();
        let todo_lists = Vec::new();
        let root_task_id = root_task.id().clone();
        if let Some(title) = conversation_data.display_title.clone() {
            root_task.update_description(title);
        }

        let reverted_action_ids = conversation_data.reverted_action_ids.unwrap_or_default();
        let artifacts = conversation_data
            .artifacts_json
            .and_then(|json| {
                serde_json::from_str(&json)
                    .map_err(|e| log::error!("Failed to deserialize artifacts: {e}"))
                    .ok()
            })
            .unwrap_or_default();
        let autoexecute_override = conversation_data
            .autoexecute_override
            .map(Into::into)
            .unwrap_or_default();
        let fallback_display_title = conversation_data.display_title;

        let reverted_action_ids = reverted_action_ids.into_iter().map_into().collect();

        let mut task_store = TaskStore::with_root_task(root_task);
        for exchange in acp_transcript.into_exchanges() {
            task_store.append_exchange(&root_task_id, exchange);
        }

        let status = Self::derive_status_from_root_task(&task_store.root_task());

        Ok(Self {
            id,
            task_store,
            status,
            status_error_message: None,
            todo_lists,
            // TODO(alokedesai): Support session restoration for code review comments.
            code_review: None,
            has_opened_code_review: false,
            transaction: None,
            autoexecute_override,
            added_exchanges_by_response: Default::default(),
            hidden_exchanges: Default::default(),
            reverted_action_ids,
            optimistic_cli_subagent_subtask_id: None,
            fallback_display_title,
            artifacts,
        })
    }

    pub fn id(&self) -> AIConversationId {
        self.id
    }

    /// Assigns fresh exchange IDs to all exchanges in this conversation.
    /// Used when forking conversations to avoid ID collisions with persisted blocks.
    pub fn reassign_exchange_ids(&mut self) {
        let task_ids: Vec<TaskId> = self.task_store.tasks().map(|t| t.id().clone()).collect();
        for task_id in task_ids {
            self.task_store.modify_task(&task_id, |task| {
                task.reassign_exchange_ids();
            });
        }
    }

    /// Derive the conversation status from the root task's exchanges.
    /// Used when restoring conversations to determine if they were cancelled or completed successfully.
    fn derive_status_from_root_task(root_task: &Option<&Task>) -> ConversationStatus {
        let Some(root_task) = root_task else {
            return ConversationStatus::Success;
        };

        // Check the last exchange's output status
        if let Some(last_exchange) = root_task.last_exchange() {
            match &last_exchange.output_status {
                AIAgentOutputStatus::Finished {
                    finished_output: FinishedAIAgentOutput::Cancelled { .. },
                } => return ConversationStatus::Cancelled,
                AIAgentOutputStatus::Finished {
                    finished_output: FinishedAIAgentOutput::Error { .. },
                } => return ConversationStatus::Error,
                _ => {}
            }
        }

        // If not cancelled or errored, it's successful
        ConversationStatus::Success
    }

    /// Total agent response time for the last completed set of agent responses
    /// since the most recent user query.
    pub fn total_agent_response_time_since_last_user_query_ms(&self) -> i64 {
        let exchanges = self.all_exchanges();
        if exchanges.is_empty() {
            return 0;
        }

        // Walk backwards, accumulating durations until we find a user query
        let mut total_ms: i64 = 0;
        for exchange in exchanges.iter().rev() {
            total_ms += exchange
                .duration()
                .map(|duration| duration.num_milliseconds())
                .unwrap_or(0);

            if exchange.has_user_query() {
                break;
            }
        }

        total_ms
    }

    /// Wall-to-wall response time for the last completed set of agent responses.
    pub fn wall_to_wall_response_time_since_last_query(&self) -> Option<i64> {
        let exchanges = self.all_exchanges();
        let last_exchange = exchanges.last().copied()?;
        let finish_time = last_exchange.finish_time?;

        // Walk backwards to find the most recent exchange with a user query
        let start_time = exchanges.iter().rev().find_map(|exchange| {
            if exchange.has_user_query() {
                Some(exchange.start_time)
            } else {
                None
            }
        })?;

        let duration = finish_time.signed_duration_since(start_time);
        Some(duration.num_milliseconds())
    }

    pub fn status(&self) -> &ConversationStatus {
        &self.status
    }
    pub fn status_error_message(&self) -> Option<&str> {
        self.status_error_message.as_deref()
    }

    pub fn update_status(
        &mut self,
        status: ConversationStatus,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) {
        self.update_status_with_error_message(status, None, terminal_view_id, ctx);
    }

    pub fn update_status_with_error_message(
        &mut self,
        status: ConversationStatus,
        error_message: Option<String>,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) {
        self.status_error_message = if matches!(&status, ConversationStatus::Error) {
            error_message.filter(|message| !message.trim().is_empty())
        } else {
            None
        };
        let prev_status = self.status.clone();
        let new_status = status.clone();
        self.status = status;
        ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationStatus {
            conversation_id: self.id,
            terminal_view_id,
            update: ConversationStatusUpdate::Changed { prev_status },
            new_status,
        });
    }

    pub fn is_processing_response_stream(&self, stream_id: &ResponseStreamId) -> bool {
        self.added_exchanges_by_response.contains_key(stream_id)
    }

    /// Removes the response stream tracking entry after the stream has fully completed.
    pub fn cleanup_completed_response_stream(&mut self, stream_id: &ResponseStreamId) {
        self.added_exchanges_by_response.remove(stream_id);
    }

    pub fn new_exchange_ids_for_response(
        &self,
        stream_id: &ResponseStreamId,
    ) -> impl Iterator<Item = AIAgentExchangeId> + '_ {
        self.added_exchanges_by_response
            .get(stream_id)
            .into_iter()
            .flat_map(|added_exchanges| {
                added_exchanges
                    .iter()
                    .map(|new_exchange| new_exchange.exchange_id)
            })
    }

    pub fn all_tasks(&self) -> impl Iterator<Item = &Task> {
        self.task_store.tasks()
    }

    /// Returns the titles from the CreateDocuments request corresponding to the given action ID (if any).
    pub fn get_document_titles_for_action(
        &self,
        action_id: &AIAgentActionId,
    ) -> Option<Vec<String>> {
        for exchange in self.all_exchanges() {
            let Some(output) = exchange.output_status.output() else {
                continue;
            };

            for message in &output.get().messages {
                if let AIAgentOutputMessage {
                    message: AIAgentOutputMessageType::Action(action),
                    ..
                } = message
                {
                    if &action.id == action_id {
                        if let super::AIAgentActionType::CreateDocuments(
                            super::CreateDocumentsRequest { documents },
                        ) = &action.action
                        {
                            let titles = documents
                                .iter()
                                .map(|doc| doc.title.clone())
                                .collect::<Vec<_>>();
                            return Some(titles);
                        }
                    }
                }
            }
        }

        None
    }

    /// Returns the start timestamp of the earliest [`AIAgentExchange`] in the conversation, if
    /// any.
    pub fn start_ts(&self) -> Option<DateTime<Local>> {
        self.root_task_exchanges()
            .next()
            .map(|exchange| exchange.start_time)
    }

    pub fn has_opened_code_review(&self) -> bool {
        self.has_opened_code_review
    }

    pub fn mark_code_review_as_opened(&mut self) {
        self.has_opened_code_review = true;
    }

    /// Returns the IDs of comments that have been addressed in this conversation.
    pub fn addressed_comment_ids(&self) -> HashSet<crate::code_review::comments::CommentId> {
        self.code_review
            .as_ref()
            .map(|cr| cr.addressed_comments.iter().map(|c| c.id).collect())
            .unwrap_or_default()
    }

    pub fn is_entirely_passive_code_diff(&self) -> bool {
        let mut has_passive_code_diff_exchange = false;
        for exchange in self.root_task_exchanges() {
            has_passive_code_diff_exchange |= exchange.has_passive_code_diff();
            if exchange.has_user_query() {
                return false;
            }
        }
        has_passive_code_diff_exchange
    }

    pub fn is_entirely_passive(&self) -> bool {
        let mut has_passive_exchange = false;
        for exchange in self.root_task_exchanges() {
            has_passive_exchange |= exchange.has_passive_request();
            if exchange.has_user_query() {
                return false;
            }
        }
        has_passive_exchange
    }

    /// True if the conversation consists of just one exchange
    /// and that exchange is a passive suggestion.
    pub fn is_single_passive_exchange(&self) -> bool {
        self.task_store.task_count() == 1
            && self.is_entirely_passive()
            && self
                .get_root_task()
                .is_some_and(|task| task.exchanges_len() == 1)
    }

    /// True if the conversation started with a CLI subagent and was never continued.
    /// These conversations only have CLI subagent exchanges with no user queries,
    /// meaning they never hit the primary agent.
    pub fn is_orphaned_cli_subagent_conversation(&self) -> bool {
        // Check if conversation has only 1 task (root task) and it's a CLI subagent
        let started_with_cli_subagent = self.task_store.task_count() == 1
            && self
                .get_root_task()
                .is_some_and(|task| task.is_cli_subagent());

        if !started_with_cli_subagent {
            return false;
        }

        // Check if conversation was never continued (no user queries in any exchange)
        let never_continued = self
            .root_task_exchanges()
            .all(|exchange| !exchange.has_user_query());

        never_continued
    }

    /// Returns true if this conversation should be unconditionally excluded
    /// from conversation navigation and history.
    pub fn should_exclude_from_navigation(&self) -> bool {
        // Passive-only suggestions without any follow-up requests shouldn't be presented as
        // conversations.
        self.is_entirely_passive()
            // Orphaned CLI subagent conversations (invoked from within a terminal block) are
            // internal and shouldn't appear in navigation.
            || self.is_orphaned_cli_subagent_conversation()
    }

    pub fn is_exchange_hidden(&self, exchange_id: AIAgentExchangeId) -> bool {
        self.hidden_exchanges.contains(&exchange_id)
    }

    pub fn set_is_exchange_hidden(
        &mut self,
        exchange_id: AIAgentExchangeId,
        is_hidden: bool,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) {
        // If the status is not being modified, return.
        if is_hidden == self.hidden_exchanges.contains(&exchange_id) {
            return;
        }

        if is_hidden {
            self.hidden_exchanges.insert(exchange_id);
        } else {
            self.hidden_exchanges.remove(&exchange_id);
        }

        // If the status is being toggled, set the persisted exchange hidden status.
        // Find the exchange and the terminal view ID for the exchange and emit an event to update
        // the exchange hidden state.
        ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
            exchange_id,
            terminal_view_id,
            conversation_id: self.id,
            is_hidden,
        });
    }

    /// Returns an iterator over all exchanges in all tasks in this conversation.
    pub fn all_exchanges(&self) -> Vec<&AIAgentExchange> {
        self.task_store.all_exchanges().collect()
    }

    /// Returns a vector of vectors of exchanges, in linearized order as they appeared in the
    /// conversation, grouped by task ID.
    pub fn all_exchanges_by_task(&self) -> Vec<(TaskId, Vec<&AIAgentExchange>)> {
        self.task_store.all_exchanges_by_task()
    }

    pub fn root_task_exchanges(&self) -> impl Iterator<Item = &AIAgentExchange> {
        self.task_store
            .root_task()
            .into_iter()
            .flat_map(|task| task.exchanges())
    }

    pub fn exchange_count(&self) -> usize {
        self.task_store.exchange_count()
    }

    pub fn is_empty(&self) -> bool {
        self.exchange_count() == 0
    }

    pub fn exchanges_reversed(&self) -> impl Iterator<Item = &AIAgentExchange> {
        self.task_store
            .root_task()
            .into_iter()
            .flat_map(|task| task.exchanges_reversed())
    }
    pub fn exchange_with_id(&self, exchange_id: AIAgentExchangeId) -> Option<&AIAgentExchange> {
        for task in self.task_store.tasks() {
            if let Some(exchange) = task.exchanges().find(|exchange| exchange.id == exchange_id) {
                return Some(exchange);
            }
        }
        None
    }

    /// Returns the exchange that preceded the exchange with the given id, if there is one.
    pub fn previous_exchange(&self, exchange_id: &AIAgentExchangeId) -> Option<&AIAgentExchange> {
        self.exchanges_reversed()
            .skip_while(|e| e.id != *exchange_id)
            .nth(1)
    }

    /// Returns the last exchange that didn't contain a passive request.
    pub fn last_non_passive_exchange(&self) -> Option<&AIAgentExchange> {
        self.exchanges_reversed()
            .skip_while(|e| e.has_passive_request())
            .nth(0)
    }

    pub fn first_exchange(&self) -> Option<&AIAgentExchange> {
        self.task_store.first_exchange()
    }

    pub fn latest_exchange(&self) -> Option<&AIAgentExchange> {
        self.task_store.latest_exchange()
    }

    /// Get the auto-generated title of the given conversation
    /// (falling back to the first query if the title is empty).
    /// Get the title of the given conversation.
    /// Priority: auto-generated task description > initial query > fallback_display_title.
    pub fn title(&self) -> Option<String> {
        self.task_store
            .root_task()
            .and_then(|task| {
                if task.description().is_empty() {
                    self.initial_query()
                } else {
                    Some(task.description().to_owned())
                }
            })
            .or_else(|| self.fallback_display_title.clone())
    }

    /// Set a fallback title used when no task description or initial query exists.
    pub fn set_fallback_display_title(&mut self, title: String) {
        self.fallback_display_title = Some(title);
    }

    pub fn set_display_title(&mut self, title: String) {
        self.fallback_display_title = Some(title.clone());
        self.task_store
            .modify_root_task(|task| task.update_description(title));
    }

    /// Returns the last time this conversation was modified (i.e., when the latest exchange was started).
    pub fn last_modified_at(&self) -> Option<DateTime<Local>> {
        self.latest_exchange()
            .map(|e| e.finish_time.unwrap_or(e.start_time))
    }

    /// Returns artifacts created during this conversation.
    pub fn artifacts(&self) -> &[Artifact] {
        &self.artifacts
    }

    /// Adds an artifact to this conversation and persists the change.
    pub fn add_artifact(
        &mut self,
        artifact: Artifact,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) {
        self.artifacts.push(artifact.clone());
        self.write_updated_conversation_state(ctx);
        ctx.emit(BlocklistAIHistoryEvent::UpdatedConversationArtifacts {
            terminal_view_id,
            conversation_id: self.id,
            artifact,
        });
    }

    pub fn initial_query(&self) -> Option<String> {
        self.root_task_exchanges()
            .flat_map(|exchange| exchange.input.iter())
            .find_map(|input| {
                AIAgentInput::user_query(input)
                    .or_else(|| AIAgentInput::auto_code_diff_query(input).map(|s| s.to_string()))
            })
    }

    pub fn initial_user_query(&self) -> Option<String> {
        self.root_task_exchanges()
            .flat_map(|exchange| exchange.input.iter())
            .find_map(AIAgentInput::user_query)
    }

    /// Export the conversation to markdown format.
    /// This is used by both clipboard export and file export.
    pub fn export_to_markdown(
        &self,
        action_model: Option<&crate::ai::blocklist::BlocklistAIActionModel>,
    ) -> String {
        let mut result = Vec::new();
        for exchange in self.all_exchanges() {
            let formatted_exchange = exchange.format_for_copy(action_model);
            if !formatted_exchange.is_empty() {
                result.push(formatted_exchange);
            }
        }
        result.join("\n\n")
    }

    pub fn has_auto_code_diff_query(&self) -> bool {
        self.root_task_exchanges()
            .flat_map(|exchange| exchange.input.iter())
            .any(|input| input.is_auto_code_diff_query())
    }

    pub fn latest_user_query(&self) -> Option<String> {
        self.exchanges_reversed().find_map(|exchange| {
            exchange.input.iter().rev().find_map(|input| {
                AIAgentInput::user_query(input)
                    .map(|query| query.trim().to_owned())
                    .filter(|query| !query.is_empty())
            })
        })
    }

    /// Returns an iterator over the IDs of all UseComputer actions across all exchanges
    /// in this conversation.
    pub fn use_computer_action_ids(&self) -> impl Iterator<Item = AIAgentActionId> + '_ {
        self.all_exchanges().into_iter().flat_map(|exchange| {
            exchange
                .output_status
                .output()
                .into_iter()
                .flat_map(|output| {
                    output
                        .get()
                        .actions()
                        .filter(|a| matches!(a.action, super::AIAgentActionType::UseComputer(_)))
                        .map(|a| a.id.clone())
                        .collect::<Vec<_>>()
                })
        })
    }

    pub fn contains_action(&self, action_id: &AIAgentActionId) -> bool {
        self.task_store.tasks().any(|task| {
            task.exchanges()
            .any(|exchange| {
                let Some(output) = exchange.output_status.output()
                else {
                    return false;
                };
                output.get().messages.iter().any(|step| {
                    matches!(step, AIAgentOutputMessage{ message: AIAgentOutputMessageType::Action(AIAgentAction { id, .. }), .. } if id == action_id)
                })
            })
        })
    }

    /// Returns the exchange ID that contains the given action ID, if any.
    pub fn exchange_id_for_action(&self, action_id: &AIAgentActionId) -> Option<AIAgentExchangeId> {
        for task in self.task_store.tasks() {
            for exchange in task.exchanges() {
                let Some(output) = exchange.output_status.output() else {
                    continue;
                };
                let contains_action = output.get().messages.iter().any(|step| {
                    matches!(step, AIAgentOutputMessage{ message: AIAgentOutputMessageType::Action(AIAgentAction { id, .. }), .. } if id == action_id)
                });
                if contains_action {
                    return Some(exchange.id);
                }
            }
        }
        None
    }

    /// Returns the `AIAgentContext` objects attached to the exchange with the given ID, if any.
    pub fn context_for_exchange(
        &self,
        exchange_id: AIAgentExchangeId,
    ) -> impl Iterator<Item = &AIAgentContext> {
        context_in_exchanges(self.exchange_with_id(exchange_id).into_iter())
    }

    pub fn update_for_new_request_input(
        &mut self,
        request_input: RequestInput,
        stream_id: ResponseStreamId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        if let Some(request_info) = self.added_exchanges_by_response.remove(&stream_id) {
            log::error!(
                "Existing response stream info for stream id {stream_id:?}: {request_info:?}"
            );
        }

        let RequestInput {
            input_messages,
            working_directory,
            model_id,
            coding_model_id,
            cli_agent_model_id,
            computer_use_model_id,
            request_start_ts,
            ..
        } = request_input;

        for (task_id, inputs) in input_messages.into_iter() {
            let should_hide = false;
            let new_exchange = AIAgentExchange {
                id: AIAgentExchangeId::new(),
                input: inputs,
                output_status: AIAgentOutputStatus::Streaming { output: None },
                added_message_ids: HashSet::new(),
                start_time: request_start_ts,
                finish_time: None,
                working_directory: working_directory.clone(),
                // TODO(CORE-3546): fetch shell launch data from active session
                model_id: model_id.clone(),
                coding_model_id: coding_model_id.clone(),
                cli_agent_model_id: cli_agent_model_id.clone(),
                computer_use_model_id: computer_use_model_id.clone(),
            };

            let new_exchange_id = new_exchange.id;
            self.append_exchange_to_task(&task_id, new_exchange)?;

            self.added_exchanges_by_response.insert(
                stream_id.clone(),
                Vec1::new(AddedExchange {
                    task_id: task_id.clone(),
                    exchange_id: new_exchange_id,
                }),
            );

            if should_hide {
                self.hidden_exchanges.insert(new_exchange_id);
            }

            ctx.emit(BlocklistAIHistoryEvent::AppendedExchange {
                exchange_id: new_exchange_id,
                task_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden: should_hide,
                response_stream_id: Some(stream_id.clone()),
            });
        }
        Ok(())
    }

    pub fn append_reassigned_exchange(
        &mut self,
        response_stream_id: &ResponseStreamId,
        exchange: AIAgentExchange,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let root_task_id = self.task_store.root_task_id().clone();
        let exchange_id = exchange.id;
        if exchange.output_status.is_streaming() {
            if let Some(added_exchanges) =
                self.added_exchanges_by_response.get_mut(response_stream_id)
            {
                added_exchanges.push(AddedExchange {
                    task_id: root_task_id.clone(),
                    exchange_id,
                });
            } else {
                self.added_exchanges_by_response.insert(
                    response_stream_id.clone(),
                    Vec1::new(AddedExchange {
                        task_id: root_task_id.clone(),
                        exchange_id,
                    }),
                );
            }
        }

        self.append_exchange_to_task(&root_task_id, exchange)?;

        ctx.emit(BlocklistAIHistoryEvent::ReassignedExchange {
            exchange_id,
            terminal_view_id,
            new_task_id: root_task_id,
            new_conversation_id: self.id,
        });
        Ok(())
    }

    fn append_exchange_to_task(
        &mut self,
        task_id: &TaskId,
        exchange: AIAgentExchange,
    ) -> Result<(), UpdateConversationError> {
        for input in exchange.input.iter() {
            if let AIAgentInput::CodeReview {
                review_comments, ..
            } = input
            {
                let review_comments = review_comments
                    .comments
                    .clone()
                    .into_iter()
                    .map(|c| c.into())
                    .collect();

                if let Some(code_review) = self.code_review.as_mut() {
                    code_review.pending_comments.extend(review_comments);
                } else {
                    self.code_review = Some(CodeReview::new_with_pending_comments(review_comments));
                }
            }
        }

        if self.task_store.append_exchange(task_id, exchange) {
            Ok(())
        } else {
            Err(UpdateConversationError::NoActiveTask)
        }
    }

    pub fn remove_exchange(
        &mut self,
        exchange_id: AIAgentExchangeId,
    ) -> Result<AIAgentExchange, UpdateConversationError> {
        let mut response_entries_to_remove = vec![];
        for (stream_id, added_exchanges) in self.added_exchanges_by_response.iter_mut() {
            if let Some(idx) = added_exchanges
                .iter()
                .position(|new_exchange| new_exchange.exchange_id == exchange_id)
            {
                if let Err(Size0Error) = added_exchanges.remove(idx) {
                    response_entries_to_remove.push(stream_id.clone());
                }
            }
        }
        for response_id in response_entries_to_remove.into_iter() {
            self.added_exchanges_by_response.remove(&response_id);
        }

        // Find which task contains this exchange
        let task_id = self.task_store.tasks().find_map(|task| {
            task.exchanges()
                .any(|e| e.id == exchange_id)
                .then(|| task.id().clone())
        });

        if let Some(task_id) = task_id {
            if let Some(exchange) = self.task_store.remove_task_exchange(&task_id, exchange_id) {
                return Ok(exchange);
            }
        }
        Err(UpdateConversationError::ExchangeNotFound)
    }

    pub fn initialize_local_output_for_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        model_id: LLMId,
        display_name: String,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            self.get_exchange_to_update(new_exchange_info.exchange_id)?
                .init_output()?;

            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            if let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            {
                output.get_mut().model_info = Some(OutputModelInfo {
                    model_id: model_id.clone(),
                    display_name: display_name.clone(),
                });
            }

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn append_local_text_delta_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        text_delta: &str,
        model_id: LLMId,
        display_name: String,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        let message_id_prefix = format!("acp-{}-assistant", stream_id.as_str());
        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            else {
                return Err(UpdateConversationError::OutputNeverInitialized);
            };

            {
                let mut output = output.get_mut();
                output.model_info = Some(OutputModelInfo {
                    model_id: model_id.clone(),
                    display_name: display_name.clone(),
                });

                let message_id = if let Some(message) =
                    output.messages.last_mut().filter(|message| {
                        message.id.starts_with(&message_id_prefix)
                            && matches!(message.message, AIAgentOutputMessageType::Text(_))
                    }) {
                    if let AIAgentOutputMessageType::Text(AIAgentText { sections }) =
                        &mut message.message
                    {
                        let mut markdown = sections
                            .iter()
                            .map(|section| format!("{}", MarkdownTextSection(section)))
                            .join("\n");
                        markdown.push_str(text_delta);
                        *sections = parse_markdown_into_text_and_code_sections(&markdown);
                    }
                    message.id.clone()
                } else {
                    let message_id =
                        MessageId::new(format!("{message_id_prefix}-{}", output.messages.len()));
                    output.messages.push(AIAgentOutputMessage::text(
                        message_id.clone(),
                        AIAgentText {
                            sections: parse_markdown_into_text_and_code_sections(text_delta),
                        },
                    ));
                    message_id
                };
                exchange.added_message_ids.insert(message_id);
            }

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn append_local_thought_delta_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        text_delta: &str,
        model_id: LLMId,
        display_name: String,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        let message_id_prefix = format!("acp-{}-thought", stream_id.as_str());
        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            else {
                return Err(UpdateConversationError::OutputNeverInitialized);
            };

            {
                let mut output = output.get_mut();
                output.model_info = Some(OutputModelInfo {
                    model_id: model_id.clone(),
                    display_name: display_name.clone(),
                });

                let message_id = if let Some(message) =
                    output.messages.last_mut().filter(|message| {
                        message.id.starts_with(&message_id_prefix)
                            && matches!(message.message, AIAgentOutputMessageType::Reasoning { .. })
                    }) {
                    if let AIAgentOutputMessageType::Reasoning {
                        text: AIAgentText { sections },
                        ..
                    } = &mut message.message
                    {
                        let mut markdown = sections
                            .iter()
                            .map(|section| format!("{}", MarkdownTextSection(section)))
                            .join("\n");
                        markdown.push_str(text_delta);
                        *sections = parse_markdown_into_text_and_code_sections(&markdown);
                    }
                    message.id.clone()
                } else {
                    let message_id =
                        MessageId::new(format!("{message_id_prefix}-{}", output.messages.len()));
                    output.messages.push(AIAgentOutputMessage::reasoning(
                        message_id.clone(),
                        AIAgentText {
                            sections: parse_markdown_into_text_and_code_sections(text_delta),
                        },
                        None,
                    ));
                    message_id
                };
                exchange.added_message_ids.insert(message_id);
            }

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn upsert_acp_tool_call_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        tool_call: AcpToolCall,
        model_id: LLMId,
        display_name: String,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        let message_id =
            MessageId::new(format!("acp-{}-tool-{}", stream_id.as_str(), tool_call.id));
        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            else {
                return Err(UpdateConversationError::OutputNeverInitialized);
            };

            {
                let mut output = output.get_mut();
                output.model_info = Some(OutputModelInfo {
                    model_id: model_id.clone(),
                    display_name: display_name.clone(),
                });

                if let Some(message) = output
                    .messages
                    .iter_mut()
                    .find(|message| message.id == message_id)
                {
                    message.message = AIAgentOutputMessageType::AcpToolCall(tool_call.clone());
                } else {
                    output.messages.push(AIAgentOutputMessage::acp_tool_call(
                        message_id.clone(),
                        tool_call.clone(),
                    ));
                }
            }

            exchange.added_message_ids.insert(message_id.clone());

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn update_acp_tool_call_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        update: ToolCallUpdate,
        model_id: LLMId,
        display_name: String,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        let message_id = MessageId::new(format!(
            "acp-{}-tool-{}",
            stream_id.as_str(),
            update.tool_call_id.0.as_ref()
        ));
        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            else {
                return Err(UpdateConversationError::OutputNeverInitialized);
            };

            {
                let mut output = output.get_mut();
                output.model_info = Some(OutputModelInfo {
                    model_id: model_id.clone(),
                    display_name: display_name.clone(),
                });

                if let Some(message) = output
                    .messages
                    .iter_mut()
                    .find(|message| message.id == message_id)
                {
                    if let AIAgentOutputMessageType::AcpToolCall(tool_call) = &mut message.message {
                        tool_call.apply_update(update.clone());
                    }
                }
            }

            exchange.added_message_ids.insert(message_id.clone());
            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn update_acp_terminal_trace_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        terminal_id: String,
        trace: AcpTerminalTrace,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            else {
                return Err(UpdateConversationError::OutputNeverInitialized);
            };

            {
                let mut output = output.get_mut();
                for message in &mut output.messages {
                    if let AIAgentOutputMessageType::AcpToolCall(tool_call) = &mut message.message {
                        tool_call.set_terminal_trace(terminal_id.clone(), trace.clone());
                    }
                }
            }

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn set_acp_plan_for_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        plan: AcpPlan,
        model_id: LLMId,
        display_name: String,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        let message_id = MessageId::new(format!("acp-{}-plan", stream_id.as_str()));
        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            else {
                return Err(UpdateConversationError::OutputNeverInitialized);
            };

            {
                let mut output = output.get_mut();
                output.model_info = Some(OutputModelInfo {
                    model_id: model_id.clone(),
                    display_name: display_name.clone(),
                });

                if let Some(message) = output
                    .messages
                    .iter_mut()
                    .find(|message| message.id == message_id)
                {
                    message.message = AIAgentOutputMessageType::AcpPlan(plan.clone());
                } else {
                    output.messages.push(AIAgentOutputMessage::acp_plan(
                        message_id.clone(),
                        plan.clone(),
                    ));
                }
            }

            exchange.added_message_ids.insert(message_id.clone());

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn upsert_acp_permission_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        request: AcpPermissionRequest,
        model_id: LLMId,
        display_name: String,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        let message_id = MessageId::new(format!(
            "acp-{}-permission-{}",
            stream_id.as_str(),
            request.request_id
        ));
        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            else {
                return Err(UpdateConversationError::OutputNeverInitialized);
            };

            {
                let mut output = output.get_mut();
                output.model_info = Some(OutputModelInfo {
                    model_id: model_id.clone(),
                    display_name: display_name.clone(),
                });

                if let Some(message) = output
                    .messages
                    .iter_mut()
                    .find(|message| message.id == message_id)
                {
                    message.message = AIAgentOutputMessageType::AcpPermission(request.clone());
                } else {
                    output.messages.push(AIAgentOutputMessage::acp_permission(
                        message_id.clone(),
                        request.clone(),
                    ));
                }
            }

            exchange.added_message_ids.insert(message_id.clone());

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn update_acp_permission_selection_to_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        request_id: &str,
        option_id: String,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            return Err(UpdateConversationError::NoPendingRequest);
        };

        let message_id = MessageId::new(format!(
            "acp-{}-permission-{}",
            stream_id.as_str(),
            request_id
        ));
        for new_exchange_info in new_exchanges.iter() {
            let is_hidden = self
                .hidden_exchanges
                .contains(&new_exchange_info.exchange_id);
            let exchange = self.get_exchange_to_update(new_exchange_info.exchange_id)?;
            let AIAgentOutputStatus::Streaming {
                output: Some(output),
            } = &exchange.output_status
            else {
                return Err(UpdateConversationError::OutputNeverInitialized);
            };

            {
                let mut output = output.get_mut();
                if let Some(message) = output
                    .messages
                    .iter_mut()
                    .find(|message| message.id == message_id)
                {
                    if let AIAgentOutputMessageType::AcpPermission(request) = &mut message.message {
                        request.selected_option_id = Some(option_id.clone());
                    }
                }
            }

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id: new_exchange_info.exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        Ok(())
    }

    pub fn mark_request_completed(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(new_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            log::error!("No pending request info for completed request.");
            return Err(UpdateConversationError::NoPendingRequest);
        };

        let mut has_new_actions = false;
        for AddedExchange {
            exchange_id,
            task_id,
        } in new_exchanges.into_iter()
        {
            let completed_exchange = self.mark_exchange_completed(&task_id, exchange_id)?;
            let output = completed_exchange
                .output_status
                .output()
                .map(Shared::get_owned);
            if let Some(output_shared) = output {
                let output = output_shared.get();
                has_new_actions |= output.actions().next().is_some();
            }

            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden: self.is_exchange_hidden(exchange_id),
            });
        }
        self.write_updated_conversation_state(ctx);

        if !has_new_actions {
            // Update conversation-level status to success if the output has no actions.
            self.update_status(ConversationStatus::Success, terminal_view_id, ctx);
        }

        Ok(())
    }

    pub fn mark_completed_after_successful_split(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        // Remove the mapping between the response stream and this conversation, as the response stream is
        // now associated with a different one.
        if let Some(added_exchanges) = self.added_exchanges_by_response.remove(stream_id) {
            for AddedExchange {
                exchange_id,
                task_id,
            } in added_exchanges.into_iter()
            {
                self.mark_exchange_completed(&task_id, exchange_id)?;

                ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                    exchange_id,
                    terminal_view_id,
                    conversation_id: self.id,
                    is_hidden: self.is_exchange_hidden(exchange_id),
                });
            }
        }
        self.write_updated_conversation_state(ctx);

        // Update conversation-level status to success if the output has no actions.
        self.update_status(ConversationStatus::Success, terminal_view_id, ctx);
        Ok(())
    }

    pub fn mark_request_cancelled(
        &mut self,
        stream_id: &ResponseStreamId,
        terminal_view_id: EntityId,
        reason: CancellationReason,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(added_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            log::error!("No pending request info for completed request.");
            return Err(UpdateConversationError::NoPendingRequest);
        };
        if self.transaction.is_some() {
            self.commit_transaction()
        }

        for AddedExchange { exchange_id, .. } in added_exchanges.into_iter() {
            let exchange = self.get_exchange_to_update(exchange_id)?;
            let AIAgentOutputStatus::Streaming { output } = &exchange.output_status else {
                // Skip exchanges that are already finished (e.g., a root task exchange
                // that completed before a subagent exchange was cancelled).
                continue;
            };
            exchange.output_status = AIAgentOutputStatus::Finished {
                finished_output: FinishedAIAgentOutput::Cancelled {
                    output: output.as_ref().map(Shared::get_owned),
                    reason,
                },
            };

            exchange.finish_time = Some(Local::now());

            let is_hidden = self.is_exchange_hidden(exchange_id);
            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        self.write_updated_conversation_state(ctx);

        // Don't mark the conversation as Cancelled if we're just cancelling to send a follow-up
        // on the same conversation. The conversation will be immediately set back to InProgress.
        if !reason.is_follow_up_for_same_conversation() {
            self.update_status(ConversationStatus::Cancelled, terminal_view_id, ctx);
        }
        Ok(())
    }

    pub fn mark_request_cancelled_due_to_revert(
        &mut self,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        if self.transaction.is_some() {
            self.commit_transaction();
        }
        self.update_status(ConversationStatus::Success, terminal_view_id, ctx);
        Ok(())
    }

    pub fn mark_request_completed_with_error(
        &mut self,
        stream_id: &ResponseStreamId,
        error: RenderableAIError,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<(), UpdateConversationError> {
        let Some(added_exchanges) = self.added_exchanges_by_response.get(stream_id).cloned() else {
            log::error!("No pending request info for completed request.");
            return Err(UpdateConversationError::NoPendingRequest);
        };
        if self.transaction.is_some() {
            self.commit_transaction()
        }

        for AddedExchange { exchange_id, .. } in added_exchanges.into_iter() {
            let exchange = self.get_exchange_to_update(exchange_id)?;
            let AIAgentOutputStatus::Streaming { output } = &exchange.output_status else {
                return Err(UpdateConversationError::OutputAlreadyFinished);
            };
            exchange.output_status = AIAgentOutputStatus::Finished {
                finished_output: FinishedAIAgentOutput::Error {
                    output: output.as_ref().map(Shared::get_owned),
                    error: error.clone(),
                },
            };

            exchange.finish_time = Some(Local::now());

            let is_hidden = self.is_exchange_hidden(exchange_id);
            ctx.emit(BlocklistAIHistoryEvent::UpdatedStreamingExchange {
                exchange_id,
                terminal_view_id,
                conversation_id: self.id,
                is_hidden,
            });
        }

        self.write_updated_conversation_state(ctx);
        self.update_status_with_error_message(
            ConversationStatus::Error,
            Some(error.to_string()),
            terminal_view_id,
            ctx,
        );
        Ok(())
    }

    fn mark_exchange_completed(
        &mut self,
        _task_id: &TaskId,
        exchange_id: AIAgentExchangeId,
    ) -> Result<&AIAgentExchange, UpdateConversationError> {
        let exchange = self.get_exchange_to_update(exchange_id)?;
        let AIAgentOutputStatus::Streaming {
            output: Some(output),
        } = &exchange.output_status
        else {
            return Err(UpdateConversationError::OutputAlreadyFinished);
        };

        let output = output.get_owned();
        exchange.output_status = AIAgentOutputStatus::Finished {
            finished_output: FinishedAIAgentOutput::Success { output },
        };

        exchange.finish_time = Some(Local::now());

        let exchange = self
            .exchange_with_id(exchange_id)
            .ok_or(UpdateConversationError::ExchangeNotFound)?;
        Ok(exchange)
    }

    pub fn get_exchange_to_update(
        &mut self,
        exchange_id: AIAgentExchangeId,
    ) -> Result<&mut AIAgentExchange, UpdateConversationError> {
        self.task_store
            .exchange_mut(exchange_id)
            .ok_or(UpdateConversationError::ExchangeNotFound)
    }

    pub fn get_root_task(&self) -> Option<&Task> {
        self.task_store.root_task()
    }

    pub fn get_root_task_id(&self) -> &TaskId {
        self.task_store.root_task_id()
    }

    pub fn get_task(&self, task_id: &TaskId) -> Option<&Task> {
        self.task_store.get(task_id)
    }

    /// Optimistically creates a subtask for the CLISubagent task when a user query is sent while
    /// the a command is running but no subagent has been spawned yet.
    ///
    /// This is done in two scenarios:
    ///
    /// 1) The user enters agent mode while a user-executed command is running, and sends a query.
    /// 2) The agent has executed a long-running requested command, but before the response stream
    /// finishes (in which the CLI subagent would be spawned), the user pre-empts with a query.
    ///
    pub fn create_optimistic_cli_subagent_task(
        &mut self,
        block_id: &BlockId,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> TaskId {
        if self.optimistic_cli_subagent_subtask_id.take().is_some() {
            log::error!(
                "Tried to optimistically create new subtask for CLI agent when one exists already."
            );
        }

        let new_task = Task::new_optimistic_cli_agent_subtask(block_id.clone());
        let new_task_id = new_task.id().clone();
        self.optimistic_cli_subagent_subtask_id = Some(new_task_id.clone());
        self.task_store.insert(new_task);
        ctx.emit(BlocklistAIHistoryEvent::CreatedSubtask {
            conversation_id: self.id,
            terminal_view_id,
            task_id: new_task_id.clone(),
        });
        new_task_id
    }

    pub fn is_subagent_task_finished(
        &self,
        subagent_task_id: &TaskId,
    ) -> Result<bool, SubagentTaskNotFound> {
        let subagent_task = self
            .task_store
            .get(subagent_task_id)
            .ok_or(SubagentTaskNotFound)?;
        Ok(subagent_task.last_exchange().is_some_and(|exchange| {
            matches!(exchange.output_status, AIAgentOutputStatus::Finished { .. })
        }))
    }

    pub fn has_active_subagent(&self) -> bool {
        if self.optimistic_cli_subagent_subtask_id.is_some() {
            return true;
        }
        self.all_tasks().any(|task| {
            !task.is_root_task()
                && self
                    .is_subagent_task_finished(task.id())
                    .is_ok_and(|finished| !finished)
        })
    }

    pub fn todo_lists(&self) -> &Vec<AIAgentTodoList> {
        &self.todo_lists
    }

    pub fn active_todo_list(&self) -> Option<&AIAgentTodoList> {
        self.todo_lists.last()
    }

    pub fn active_todo(&self) -> Option<&AIAgentTodo> {
        self.active_todo_list()
            .and_then(|todo_list| todo_list.in_progress_item())
    }

    pub fn todo_status(&self, todo_id: &AIAgentTodoId) -> Option<TodoStatus> {
        for (i, list) in self.todo_lists.iter().rev().enumerate() {
            let is_active_list = i == 0;
            if let Some(pos) = list
                .pending_items()
                .iter()
                .position(|item| &item.id == todo_id)
            {
                if is_active_list {
                    if pos == 0 {
                        return if self.status.is_in_progress() {
                            Some(TodoStatus::InProgress)
                        } else {
                            Some(TodoStatus::Stopped)
                        };
                    } else {
                        return Some(TodoStatus::Pending);
                    }
                } else {
                    return Some(TodoStatus::Cancelled);
                }
            } else if list
                .completed_items()
                .iter()
                .any(|item| &item.id == todo_id)
            {
                return Some(TodoStatus::Completed);
            }
        }
        None
    }

    pub fn begin_transaction(&mut self) {
        if self.transaction.is_some() {
            log::error!("Transaction already in progress.");
            return;
        }
        self.transaction = Some(Transaction::new());
    }

    fn commit_transaction(&mut self) {
        // Clear the transaction if it exists.
        if self.transaction.take().is_none() {
            log::error!("No transaction in progress.");
        }
    }

    pub(crate) fn write_updated_conversation_state(
        &mut self,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) {
        // Check if session restoration is enabled before writing any state.
        if !*GeneralSettings::as_ref(ctx).restore_session
            || !AppExecutionMode::as_ref(ctx).can_save_session()
        {
            return;
        }

        let Some(sqlite_sender) = GlobalResourceHandlesProvider::as_ref(ctx)
            .get()
            .model_event_sender
            .clone()
        else {
            return;
        };

        let reverted_action_ids = if self.reverted_action_ids.is_empty() {
            None
        } else {
            Some(
                self.reverted_action_ids
                    .clone()
                    .into_iter()
                    .map_into()
                    .collect(),
            )
        };

        let artifacts_json = if self.artifacts.is_empty() {
            None
        } else {
            match serde_json::to_string(&self.artifacts) {
                Ok(json) => Some(json),
                Err(e) => {
                    log::error!(
                        "Failed to serialize artifacts when persisting conversation data: {e}"
                    );
                    None
                }
            }
        };
        let Some(acp_transcript_json) = self.acp_transcript_json() else {
            return;
        };

        let event = ModelEvent::UpdateAgentConversation {
            conversation_id: self.id.to_string(),
            conversation_data: AgentConversationData {
                reverted_action_ids,
                artifacts_json,
                autoexecute_override: Some(self.autoexecute_override.into()),
                display_title: self.fallback_display_title.clone(),
                acp_transcript_json,
            },
        };
        ctx.spawn(
            async move {
                if let Err(e) = sqlite_sender.send(event) {
                    log::warn!("Failed to send updated AI tasks to sqlite writer thread: {e:?}");
                }
            },
            |_, _, _| {},
        );
    }

    pub(crate) fn acp_transcript_json(&self) -> Option<String> {
        Self::serialize_acp_transcript(AcpTranscript::from_conversation(self)?)
    }

    pub(crate) fn acp_transcript_json_until_exchange(
        &self,
        from_exchange_id: AIAgentExchangeId,
    ) -> Option<String> {
        let mut found_from_exchange = false;
        let mut exchanges = Vec::new();
        for exchange in self.root_task_exchanges() {
            if found_from_exchange && exchange.has_user_query() {
                break;
            }
            if exchange.id == from_exchange_id {
                found_from_exchange = true;
            }
            if let Some(exchange) = AcpTranscriptExchange::from_exchange(exchange) {
                exchanges.push(exchange);
            }
        }

        found_from_exchange
            .then_some(AcpTranscript { exchanges })
            .and_then(Self::serialize_acp_transcript)
    }

    fn serialize_acp_transcript(transcript: AcpTranscript) -> Option<String> {
        serde_json::to_string(&transcript)
            .map_err(|e| log::error!("Failed to serialize ACP transcript: {e}"))
            .ok()
    }

    pub fn rollback_transaction(&mut self, response_stream_id: &ResponseStreamId) {
        let Some(transaction) = self.transaction.take() else {
            log::error!("No transaction in progress.");
            return;
        };
        let mut deleted_tasks = Vec::new();
        let mut updated_tasks = Vec::new();

        // For each saved task in the transaction:
        for (_, saved_task) in transaction.saved_tasks() {
            match saved_task {
                SavedTask::New(id) => {
                    // The task was added during the transaction, so we need to delete it
                    deleted_tasks.push(id);
                }
                SavedTask::Existing(saved_task) => {
                    // The task was updated during the transaction, so we need to restore it
                    updated_tasks.push(*saved_task);
                }
            }
        }

        updated_tasks.into_iter().for_each(|task| {
            log::debug!("Rolling back existing task: {:?}", task.id());
            self.task_store.insert(task);
        });
        deleted_tasks.into_iter().for_each(|task_id| {
            log::debug!("Rolling back new task: {task_id:?}");
            self.task_store.remove(&task_id);
        });

        if let Some(added_exchanges) = self
            .added_exchanges_by_response
            .get(response_stream_id)
            .cloned()
        {
            let mut updated_added_exchanges: Option<Vec1<AddedExchange>> = None;
            for added_exchange in added_exchanges.into_iter() {
                let does_exchange_exist = self
                    .task_store
                    .get(&added_exchange.task_id)
                    .and_then(|task| {
                        task.exchanges()
                            .find(|exchange| exchange.id == added_exchange.exchange_id)
                    })
                    .is_some();
                if does_exchange_exist {
                    if let Some(updated_added_exchanges) = updated_added_exchanges.as_mut() {
                        updated_added_exchanges.push(added_exchange);
                    } else {
                        updated_added_exchanges = Some(Vec1::new(added_exchange));
                    }
                }
            }
            if let Some(updated_added_exchanges) = updated_added_exchanges {
                self.added_exchanges_by_response
                    .insert(response_stream_id.clone(), updated_added_exchanges);
            }
        }
    }

    pub fn checkpoint_task(&mut self, task_id: &TaskId) {
        if let Some(transaction) = &mut self.transaction {
            if let Some(task) = self.task_store.get(task_id) {
                transaction.checkpoint_task(task);
            } else {
                transaction.checkpoint_new_task(task_id);
            }
        }
    }

    pub fn toggle_autoexecute_override(&mut self) {
        self.autoexecute_override =
            if self.autoexecute_override == AIConversationAutoexecuteMode::RespectUserSettings {
                AIConversationAutoexecuteMode::RunToCompletion
            } else {
                AIConversationAutoexecuteMode::RespectUserSettings
            };
    }

    pub fn autoexecute_override(&self) -> AIConversationAutoexecuteMode {
        self.autoexecute_override
    }

    pub fn autoexecute_any_action(&self) -> bool {
        self.autoexecute_override.is_autoexecute_any_action()
    }

    pub fn initial_working_directory(&self) -> Option<String> {
        self.task_store
            .root_task()
            .and_then(Task::initial_working_directory)
    }

    /// Returns the current working directory from the most recent exchange that has one.
    /// Scans exchanges in reverse order and returns the first populated working directory.
    pub fn current_working_directory(&self) -> Option<String> {
        self.task_store
            .all_exchanges_rev()
            .find_map(|exchange| exchange.working_directory.clone())
    }

    pub fn mark_action_as_reverted(
        &mut self,
        action_id: AIAgentActionId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) {
        self.reverted_action_ids.insert(action_id);
        self.write_updated_conversation_state(ctx);
    }

    pub fn is_action_reverted(&self, action_id: &AIAgentActionId) -> bool {
        self.reverted_action_ids.contains(action_id)
    }

    pub fn reverted_action_ids(&self) -> &HashSet<AIAgentActionId> {
        &self.reverted_action_ids
    }

    /// Truncates the conversation from the given exchange ID, removing all exchanges
    /// from that exchange onwards (inclusive). This is a lossy operation - the removed
    /// exchanges are permanently deleted from this conversation.
    ///
    /// Returns the set of exchange IDs that were removed.
    pub fn truncate_from_exchange(
        &mut self,
        from_exchange_id: AIAgentExchangeId,
        ctx: &mut ModelContext<BlocklistAIHistoryModel>,
    ) -> Result<HashSet<AIAgentExchangeId>, UpdateConversationError> {
        let all_exchanges: Vec<AIAgentExchangeId> =
            self.root_task_exchanges().map(|e| e.id).collect();

        let truncate_from_idx = all_exchanges
            .iter()
            .position(|id| *id == from_exchange_id)
            .ok_or(UpdateConversationError::ExchangeNotFound)?;

        let exchanges_to_remove: HashSet<AIAgentExchangeId> =
            all_exchanges[truncate_from_idx..].iter().copied().collect();

        if exchanges_to_remove.is_empty() {
            return Ok(exchanges_to_remove);
        }

        let message_ids_to_remove: HashSet<MessageId> = exchanges_to_remove
            .iter()
            .filter_map(|ex_id| self.exchange_with_id(*ex_id))
            .flat_map(|ex| ex.added_message_ids.iter().cloned())
            .collect();

        if self
            .task_store
            .modify_root_task(|root_task| {
                root_task.truncate_exchanges_from(from_exchange_id);
                root_task.remove_messages(&message_ids_to_remove);
            })
            .is_some()
        {
            self.todo_lists.clear();
        }

        // Make sure we don't have stale code review comment state
        self.code_review = None;

        self.added_exchanges_by_response
            .retain(|_, added_exchanges| {
                if added_exchanges
                    .iter()
                    .all(|added| exchanges_to_remove.contains(&added.exchange_id))
                {
                    return false;
                }
                let _ = added_exchanges
                    .retain(|added| !exchanges_to_remove.contains(&added.exchange_id));
                true
            });

        self.hidden_exchanges
            .retain(|ex_id| !exchanges_to_remove.contains(ex_id));

        // Stale ones are harmless, but might as well remove stale reverted action IDs
        let mut new_reverted_action_ids = std::mem::take(&mut self.reverted_action_ids);
        new_reverted_action_ids.retain(|id| self.contains_action(id));
        self.reverted_action_ids = new_reverted_action_ids;

        let root_task_is_empty = self
            .task_store
            .root_task()
            .is_none_or(|task| task.exchanges_len() == 0);

        // If all exchanges were removed, reset the root task to optimistic state.
        // This allows the next message to go through the normal "first message" flow
        // and promote the optimistic task.
        if root_task_is_empty {
            let root_task_id = self.task_store.root_task_id().clone();
            self.task_store.remove(&root_task_id);
            let new_root_task = Task::new_optimistic_root();
            self.task_store.set_root_task(new_root_task);
        }

        self.write_updated_conversation_state(ctx);

        Ok(exchanges_to_remove)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateConversationError {
    #[error("Exchange not found.")]
    ExchangeNotFound,
    #[error("Could not update task: {0:?}")]
    UpdateTask(#[from] UpdateTaskError),
    #[error("Task not found.")]
    TaskNotFound,
    #[error("Attempted to update already-finished output.")]
    OutputAlreadyFinished,
    #[error("Attempted to update output that was never initialized.")]
    OutputNeverInitialized,
    #[error("No active task")]
    NoActiveTask,
    #[error("No pending request.")]
    NoPendingRequest,
}

/// A globally unique ID for a conversation with an AI agent.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AIConversationId(Uuid);

impl Display for AIConversationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AIConversationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AIConversationId {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<String> for AIConversationId {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(Uuid::try_parse(&value)?))
    }
}

/// Returns an iterator over `AIAgentContext`s attached to inputs in the given `exchanges`, in the
/// same order in which they appeared.
pub(super) fn context_in_exchanges<'a>(
    exchanges: impl Iterator<Item = &'a AIAgentExchange> + 'a,
) -> impl Iterator<Item = &'a AIAgentContext> + 'a {
    exchanges.flat_map(|exchange| {
        exchange
            .input
            .iter()
            .filter_map(AIAgentInput::context)
            .flatten()
    })
}

impl AIAgentExchange {
    /// Returns an error if the output was already initialized.
    pub(super) fn init_output(&mut self) -> Result<(), UpdateTaskError> {
        match &mut self.output_status {
            AIAgentOutputStatus::Streaming { ref mut output } => {
                if output.is_none() {
                    *output = Some(Shared::new(AIAgentOutput {
                        messages: vec![],
                        citations: vec![],
                        model_info: None,
                    }));
                }
                Ok(())
            }
            AIAgentOutputStatus::Finished { .. } => Err(UpdateTaskError::OutputAlreadyFinished),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AIConversationAutoexecuteMode {
    #[default]
    RespectUserSettings,
    RunToCompletion,
}

impl AIConversationAutoexecuteMode {
    pub fn is_autoexecute_any_action(&self) -> bool {
        matches!(self, AIConversationAutoexecuteMode::RunToCompletion)
    }
}

impl From<PersistedAutoexecuteMode> for AIConversationAutoexecuteMode {
    fn from(value: PersistedAutoexecuteMode) -> Self {
        match value {
            PersistedAutoexecuteMode::RespectUserSettings => Self::RespectUserSettings,
            PersistedAutoexecuteMode::RunToCompletion => Self::RunToCompletion,
        }
    }
}

impl From<AIConversationAutoexecuteMode> for PersistedAutoexecuteMode {
    fn from(value: AIConversationAutoexecuteMode) -> Self {
        match value {
            AIConversationAutoexecuteMode::RespectUserSettings => Self::RespectUserSettings,
            AIConversationAutoexecuteMode::RunToCompletion => Self::RunToCompletion,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConversationStatus {
    /// Agent is running.
    InProgress,

    /// The last turn of the agent finished with success.
    Success,

    /// The last turn of the agent completed with error.
    Error,

    /// The last turn of the agent was cancelled by the user.
    Cancelled,

    /// The last turn of the agent resulted in an action whose execution is blocked by the user.
    Blocked { blocked_action: String },
}

impl std::fmt::Display for ConversationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversationStatus::InProgress => write!(f, "In progress"),
            ConversationStatus::Success => write!(f, "Done"),
            ConversationStatus::Error => write!(f, "Error"),
            ConversationStatus::Cancelled => write!(f, "Cancelled"),
            ConversationStatus::Blocked { .. } => write!(f, "Blocked"),
        }
    }
}

impl ConversationStatus {
    pub fn render_icon(&self, appearance: &Appearance) -> warpui::elements::Icon {
        match self {
            ConversationStatus::InProgress => in_progress_icon(appearance),
            ConversationStatus::Success => succeeded_icon(appearance),
            ConversationStatus::Blocked { .. } => yellow_stop_icon(appearance),
            ConversationStatus::Error => failed_icon(appearance),
            ConversationStatus::Cancelled => gray_stop_icon(appearance),
        }
    }

    pub fn status_icon_and_color(&self, theme: &WarpTheme) -> (Icon, ColorU) {
        match self {
            ConversationStatus::InProgress => (Icon::ClockLoader, theme.ansi_fg_magenta()),
            ConversationStatus::Success => (Icon::Check, theme.ansi_fg_green()),
            ConversationStatus::Error => (Icon::Triangle, theme.ansi_fg_red()),
            ConversationStatus::Cancelled => (Icon::StopFilled, internal_colors::neutral_5(theme)),
            ConversationStatus::Blocked { .. } => (Icon::StopFilled, theme.ansi_fg_yellow()),
        }
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self, ConversationStatus::InProgress)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, ConversationStatus::Blocked { .. })
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, ConversationStatus::Cancelled)
    }

    pub fn is_done(&self) -> bool {
        matches!(
            self,
            ConversationStatus::Success | ConversationStatus::Error | ConversationStatus::Cancelled
        )
    }

    pub fn is_error(&self) -> bool {
        matches!(self, ConversationStatus::Error)
    }
}
