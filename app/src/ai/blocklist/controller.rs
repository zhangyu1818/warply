//! This module contains core business logic for Agent Mode, primarily sending input to the ACP
//! agent and applying protocol output to the local UI.
//!
//! The `BlocklistAIController` coordinates local state updates that power the Agent Mode UI.
pub mod input_context;
pub mod response_stream;
mod slash_command;
use input_context::{input_context_for_request, parse_context_attachments};
pub use slash_command::*;

use super::ResponseStreamId;
use super::{
    action_model::{BlocklistAIActionEvent, BlocklistAIActionModel},
    agent_view::{AgentViewController, AgentViewControllerEvent},
    context_model::BlocklistAIContextModel,
    history_model::BlocklistAIHistoryModel,
    BlocklistAIInputModel,
};
use crate::ai::acp::model::{AcpAgentModel, AcpRunTarget};
use crate::ai::agent::conversation::{AIConversation, AIConversationId, ConversationStatus};
use crate::ai::agent::task::TaskId;
use crate::ai::agent::{
    extract_user_query_mode, AIAgentActionResult, AIAgentActionResultType, AIAgentAttachment,
    AIAgentContext, AIAgentExchangeId, AIAgentInput, CancellationReason, RunningCommand,
    StaticQueryType, UserQueryMode,
};
use crate::ai::agent::{AnyFileContent, DocumentContentAttachmentSource, FileContext};
use crate::ai::document::ai_document_model::{
    AIDocumentId, AIDocumentModel, AIDocumentUserEditStatus,
};
use crate::ai::llms::LLMId;
use crate::global_resource_handles::GlobalResourceHandlesProvider;
use crate::persistence::ModelEvent;
use crate::settings::AISettings;
use crate::terminal::model::block::{formatted_terminal_contents_for_input, CURSOR_MARKER};
use crate::terminal::view::inline_banner::ZeroStatePromptSuggestionType;
use crate::terminal::{
    model::session::active_session::ActiveSession, model::terminal_model::TerminalModel,
};
use agent_client_protocol::schema::{
    BlobResourceContents, ContentBlock, EmbeddedResource, EmbeddedResourceResource, ImageContent,
    ResourceLink, TextContent, TextResourceContents,
};
use anyhow::anyhow;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Local};
use itertools::Itertools;
use parking_lot::FairMutex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use url::Url;
use warp_core::assertions::safe_assert;

use warpui::{AppContext, Entity, EntityId, ModelContext, ModelHandle, SingletonEntity};

pub enum BlocklistAIControllerEvent {
    /// Emitted when a request is sent to the AI agent API.
    SentRequest {
        contains_user_query: bool,
        /// True when this request is the first send of a previously queued prompt (e.g.
        /// via `/queue` or the auto-queue toggle) rather than a direct user submission.
        /// Subscribers that perform user-submission side effects (e.g. clearing the input
        /// buffer) should skip those effects when this is true — the user may have typed
        /// new input while the agent was busy and we don't want to wipe it.
        is_queued_prompt: bool,
        /// The model ID used for this request. None for slash commands that don't
        /// send a model request (e.g., /fork).
        model_id: LLMId,
        /// The ID of the response stream for this request.
        stream_id: ResponseStreamId,
    },

    /// Emitted when an AI output response is fully received, particularly relevant when output is
    /// being streamed.
    FinishedReceivingOutput {
        stream_id: ResponseStreamId,
        conversation_id: AIConversationId,
    },

    /// Emitted when the export-to-file slash command is executed.
    ExportConversationToFile { filename: Option<String> },
}

#[derive(Debug)]
pub struct RequestInput {
    pub conversation_id: AIConversationId,
    pub input_messages: HashMap<TaskId, Vec<AIAgentInput>>,
    pub working_directory: Option<String>,
    pub model_id: LLMId,
    pub coding_model_id: LLMId,
    pub cli_agent_model_id: LLMId,
    pub computer_use_model_id: LLMId,
    pub request_start_ts: DateTime<Local>,
}

struct AcpPromptPayload {
    content_blocks: Vec<ContentBlock>,
    display_prompt: String,
}

impl RequestInput {
    fn for_task(
        inputs: Vec<AIAgentInput>,
        task_id: TaskId,
        active_session: &ModelHandle<ActiveSession>,
        conversation_id: AIConversationId,
        app: &AppContext,
    ) -> Self {
        let mut me = Self::new_with_common_fields(conversation_id, active_session, app);
        me.input_messages.insert(task_id, inputs);
        me
    }

    fn for_actions_results(
        action_results: Vec<AIAgentActionResult>,
        context: Arc<[AIAgentContext]>,
        active_session: &ModelHandle<ActiveSession>,
        conversation_id: AIConversationId,
        app: &AppContext,
    ) -> Self {
        let mut me = Self::new_with_common_fields(conversation_id, active_session, app);
        for result in action_results.into_iter() {
            me.input_messages
                .entry(result.task_id.clone())
                .or_default()
                .push(AIAgentInput::ActionResult {
                    result,
                    context: context.clone(),
                });
        }
        me
    }

    pub fn all_inputs(&self) -> impl Iterator<Item = &AIAgentInput> {
        self.input_messages.values().flatten()
    }

    fn new_with_common_fields(
        conversation_id: AIConversationId,
        active_session: &ModelHandle<ActiveSession>,
        app: &AppContext,
    ) -> Self {
        let model_id = LLMId::from("auto");
        let coding_model_id = model_id.clone();
        let cli_agent_model_id = model_id.clone();
        let computer_use_model_id = model_id.clone();
        let working_directory = active_session
            .as_ref(app)
            .current_working_directory()
            .cloned();

        Self {
            conversation_id,
            input_messages: Default::default(),
            working_directory,
            model_id,
            coding_model_id,
            cli_agent_model_id,
            computer_use_model_id,
            request_start_ts: Local::now(),
        }
    }
}

/// Controller for Blocklist AI.
///
/// This is responsible for managing and updating blocklist AI state in a single terminal pane.
pub struct BlocklistAIController {
    active_session: ModelHandle<ActiveSession>,
    input_model: ModelHandle<BlocklistAIInputModel>,
    context_model: ModelHandle<BlocklistAIContextModel>,
    action_model: ModelHandle<BlocklistAIActionModel>,
    terminal_model: Arc<FairMutex<TerminalModel>>,

    /// The ID of the terminal view this controller is associated with.
    terminal_view_id: EntityId,

    /// Passive conversations explicitly requested to follow up after actions complete.
    pending_passive_follow_ups: HashSet<AIConversationId>,
}

enum InputQueryType {
    /// The user submitted query from the input. This may map to [`AIAgentInput::UserQuery`] but may
    /// map to other `AIAgentInput` types depending on various factors.
    UserSubmittedQueryFromInput {
        query: String,
        static_query_type: Option<StaticQueryType>,
        running_command: Option<RunningCommand>,
    },
    /// A custom [`AIInputType`].
    AIInputType { ai_input: AIAgentInput },
}

enum WhichTask {
    NewConversation,
    Task {
        conversation_id: AIConversationId,
        task_id: TaskId,
    },
}

struct InputQuery {
    which_task: WhichTask,
    input_query: InputQueryType,
    additional_attachments: HashMap<String, AIAgentAttachment>,
}

impl InputQuery {
    fn query(&self) -> String {
        match &self.input_query {
            InputQueryType::UserSubmittedQueryFromInput { query, .. } => query.clone(),
            InputQueryType::AIInputType { ai_input } => ai_input.user_query().unwrap_or_default(),
        }
    }
}

impl BlocklistAIController {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_model: ModelHandle<BlocklistAIInputModel>,
        context_model: ModelHandle<BlocklistAIContextModel>,
        action_model: ModelHandle<BlocklistAIActionModel>,
        active_session: ModelHandle<ActiveSession>,
        agent_view_controller: ModelHandle<AgentViewController>,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        terminal_view_id: EntityId,
        ctx: &mut ModelContext<Self>,
    ) -> Self {
        ctx.subscribe_to_model(&action_model, move |me, event, ctx| {
            let BlocklistAIActionEvent::FinishedAction {
                conversation_id,
                cancellation_reason,
                ..
            } = event
            else {
                return;
            };
            let action_model = me.action_model.as_ref(ctx);
            if action_model.has_unfinished_actions_for_conversation(*conversation_id) {
                return;
            }

            let history_model = BlocklistAIHistoryModel::handle(ctx);
            let Some(is_entirely_passive_code_diff) = history_model
                .as_ref(ctx)
                .conversation(conversation_id)
                .map(|conversation| conversation.is_entirely_passive_code_diff())
            else {
                return;
            };

            let Some(finished_action_results) =
                action_model.get_finished_action_results(*conversation_id)
            else {
                return;
            };
            let is_passive_code_diff = is_entirely_passive_code_diff
                && finished_action_results.last().is_some_and(|result| {
                    matches!(result.result, AIAgentActionResultType::RequestFileEdits(_))
                });
            let has_manual_follow_up = me.pending_passive_follow_ups.contains(conversation_id);

            let is_lrc_command_completed =
                cancellation_reason.is_some_and(|reason| reason.is_lrc_command_completed());
            let should_trigger_follow_up_request = (!is_passive_code_diff
                && !is_lrc_command_completed
                && finished_action_results
                    .iter()
                    .any(|result| result.result.should_trigger_request_upon_completion()))
                || has_manual_follow_up;
            if !should_trigger_follow_up_request {
                // We also check if there's an in-flight req, because it's possible that this
                // subscription callback was queued in response to auto-cancelling pending actions
                // in the process of constructing a request. In such cases, we don't want to update
                // conversation status to Cancelled/Success.
                if !AcpAgentModel::as_ref(ctx).has_active_session_for_conversation(*conversation_id)
                {
                    // If the completed actions do not trigger a follow-up request, update conversation
                    // status based on the outcome of the actions.
                    //
                    // (It would otherwise remain `InProgress`, which would be correct, since we'd be
                    // immediately triggering a follow-up request).
                    //
                    // In practice, the only time where this codepath gets triggered is upon completion
                    // of a passive code diff action, where we don't autosend the next request.
                    //
                    // With passive code diffs, its most appropriate to mark the conversation
                    // successful if the passive diff was accepted. In practice, there's only ever
                    // one RequestFileEdits action, so `finished_action_results` at this point
                    // should only have a single element.
                    //
                    // If the user does end up following up on the passive diff-originated conversation,
                    // the status will once again be updated to `InProgress`.
                    let updated_conversation_status = if finished_action_results
                        .iter()
                        .all(|result| result.result.is_successful())
                        || is_lrc_command_completed
                    {
                        ConversationStatus::Success
                    } else {
                        // This is an imperfect heuristic that practically speaking should have no effect.
                        //
                        // If we actually need to differentiate between the state of a conversation
                        // where actions completed with mixed result statuses (e.g. a mix of
                        // cancelled, error, and success) _and_ we don't automatically send back action
                        // results to the agent, then it'd be worth considering adding a new status
                        // variant.
                        ConversationStatus::Cancelled
                    };
                    history_model.update(ctx, |history_model, ctx| {
                        history_model.update_conversation_status(
                            me.terminal_view_id,
                            *conversation_id,
                            updated_conversation_status,
                            ctx,
                        );
                    });
                }
                return;
            }
            me.send_follow_up_for_conversation(*conversation_id, ctx);
        });

        ctx.subscribe_to_model(&agent_view_controller, |me, event, ctx| {
            let AgentViewControllerEvent::ExitedAgentView {
                conversation_id,
                final_exchange_count,
                ..
            } = event
            else {
                return;
            };

            // If we exited a brand-new empty conversation, there's nothing meaningful to cancel.
            if *final_exchange_count == 0 {
                return;
            }

            let history = BlocklistAIHistoryModel::handle(ctx);
            let Some(conversation) = history.as_ref(ctx).conversation(conversation_id) else {
                return;
            };

            if conversation.status().is_in_progress() {
                me.cancel_conversation_progress(
                    *conversation_id,
                    CancellationReason::ManuallyCancelled,
                    ctx,
                );
            }
        });

        Self {
            input_model,
            context_model,
            action_model,
            active_session,
            terminal_model,
            terminal_view_id,
            pending_passive_follow_ups: HashSet::new(),
        }
    }

    /// Internal method to send a query to the AI model. External callers should use either
    /// `send_user_query_in_conversation`, `send_user_in_conversation`, or
    /// `send_custom_ai_input_query` instead.
    ///
    /// When the request is sent, a `BlocklistAIEvent::SentRequest` event is emitted containing the
    /// query itself as well as a oneshot `Receiver` that can be `await`-ed to receive the response
    /// from the AI.
    fn send_query(
        &mut self,
        input_query: InputQuery,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let query = input_query.query().to_owned();
        let (conversation_id, task_id) = match input_query.which_task {
            WhichTask::NewConversation => {
                let conversation = self.start_new_conversation_for_request(ctx);
                (conversation.id(), conversation.get_root_task_id().clone())
            }
            WhichTask::Task {
                conversation_id,
                task_id,
            } => (conversation_id, task_id),
        };

        let ai_history_model = BlocklistAIHistoryModel::as_ref(ctx);
        let active_conversation_id = ai_history_model.active_conversation_id(self.terminal_view_id);
        let cancellation_reason = CancellationReason::FollowUpSubmitted {
            is_for_same_conversation: active_conversation_id
                .is_some_and(|id| id == conversation_id),
        };
        if let Some(active_conversation_id) = active_conversation_id {
            self.cancel_conversation_progress(active_conversation_id, cancellation_reason, ctx);
        }

        let (query, user_query_mode) = extract_user_query_mode(query);

        let should_prepend_finished_action_results = matches!(
            input_query.input_query,
            InputQueryType::UserSubmittedQueryFromInput { .. }
        );

        let completed_action_results = self.action_model.update(ctx, |action_model, ctx| {
            action_model.cancel_all_pending_actions(
                conversation_id,
                Some(cancellation_reason),
                ctx,
            );
            action_model.drain_finished_action_results(conversation_id)
        });

        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            Some(conversation_id),
            vec![],
            ctx,
        );
        let mut inputs = if should_prepend_finished_action_results {
            completed_action_results
                .into_iter()
                .map(|result| AIAgentInput::ActionResult {
                    result,
                    context: context.clone(),
                })
                .collect_vec()
        } else {
            // Custom AI inputs like CodeReview and FetchReviewComments are encoded as
            // top-level request variants (`request::input::Type::CodeReview`,
            // `request::input::Type::FetchReviewComments`, etc.), and `convert_input`
            // only emits those variants in the single-input path.
            //
            // Tool call results are encoded differently: they only exist inside
            // `request::input::Type::UserInputs` as `user_input::Input::ToolCallResult`.
            // There is no proto request shape that can represent both a top-level
            // CodeReview-style input and a ToolCallResult in the same request.
            //
            // So if we prepend an ActionResult here, `convert_input` has to fall back
            // to the multi-input `UserInputs` path, where CodeReview / FetchReviewComments
            // are ignored entirely. The stale tool result is preserved, but the custom
            // AI input disappears from the request.
            vec![]
        };

        let additional_attachments = input_query.additional_attachments;
        let ai_input = match input_query.input_query {
            InputQueryType::UserSubmittedQueryFromInput {
                static_query_type,
                running_command,
                ..
            } => input_for_query(
                query,
                &task_id,
                conversation_id,
                static_query_type,
                user_query_mode,
                running_command,
                additional_attachments,
                self.context_model.as_ref(ctx),
                self.active_session.as_ref(ctx),
                ctx,
            ),
            InputQueryType::AIInputType { ai_input } => ai_input,
        };
        inputs.push(ai_input);

        let send_result = self.send_request_input(
            RequestInput::for_task(inputs, task_id, &self.active_session, conversation_id, ctx),
            is_queued_prompt,
            ctx,
        );

        // If the request failed, re-insert the dirty event so it isn't
        // silently lost.
        if let Err(e) = &send_result {
            log::error!("Failed to send agent request: {e:?}");
        }
    }

    /// Populates plan documents from user query to AIDocumentModel if not already present.
    /// Parses attachments from query and creates AI documents for any user-attached plans.
    /// This is split from parse_context_attachments to run later in the pipeline when new conversations are created.
    fn maybe_populate_plans_for_ai_document_model(
        &self,
        referenced_attachments: &HashMap<String, AIAgentAttachment>,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        for attachment in referenced_attachments.values() {
            let AIAgentAttachment::DocumentContent {
                document_id,
                content,
                source,
                ..
            } = attachment
            else {
                continue;
            };
            if !matches!(*source, DocumentContentAttachmentSource::UserAttached) {
                continue;
            }
            let document_id = match AIDocumentId::try_from(document_id.as_str()) {
                Ok(id) => id,
                Err(_) => {
                    log::warn!("Invalid ai_document_id in document content: {document_id}");
                    continue;
                }
            };

            // Skip if document already exists in the model
            let ai_document_model = AIDocumentModel::as_ref(ctx);
            if ai_document_model
                .get_current_document(&document_id)
                .is_some()
            {
                continue;
            }

            AIDocumentModel::handle(ctx).update(ctx, |model, model_ctx| {
                model.restore_document(
                    document_id,
                    conversation_id,
                    "Plan",
                    content,
                    Local::now(),
                    model_ctx,
                );
            });
        }
    }

    pub fn send_user_query_in_new_conversation(
        &mut self,
        query: String,
        static_query_type: Option<StaticQueryType>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_new_conversation_internal(
            query,
            static_query_type,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends the first submission of a previously queued user prompt into a new conversation.
    /// Same as [`Self::send_user_query_in_new_conversation`] but marks the emitted
    /// `SentRequest` event so UI subscribers (e.g. the input editor) know not to treat
    /// this as a direct user submission and therefore not clear the input buffer.
    pub fn send_queued_user_query_in_new_conversation(
        &mut self,
        query: String,
        static_query_type: Option<StaticQueryType>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_new_conversation_internal(
            query,
            static_query_type,
            /*is_queued_prompt*/ true,
            ctx,
        );
    }

    fn send_user_query_in_new_conversation_internal(
        &mut self,
        query: String,
        static_query_type: Option<StaticQueryType>,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let running_command = {
            let terminal_model = self.terminal_model.lock();
            get_running_command(&terminal_model)
        };
        if let Some(running_command) = running_command {
            let conversation_id = self.start_new_conversation_for_request(ctx).id();
            let history_model = BlocklistAIHistoryModel::handle(ctx);
            let task_id = match history_model.update(ctx, |history_model, ctx| {
                history_model.create_cli_subagent_task_for_conversation(
                    running_command.block_id.clone(),
                    conversation_id,
                    self.terminal_view_id,
                    ctx,
                )
            }) {
                Ok(task_id) => task_id,
                Err(e) => {
                    log::error!("Could not create CLI subagent task optimistically: {e:?}");
                    return;
                }
            };
            self.send_query(
                InputQuery {
                    which_task: WhichTask::Task {
                        conversation_id,
                        task_id,
                    },
                    input_query: InputQueryType::UserSubmittedQueryFromInput {
                        query,
                        static_query_type,
                        running_command: Some(running_command),
                    },
                    additional_attachments: HashMap::new(),
                },
                is_queued_prompt,
                ctx,
            );
        } else {
            self.send_query(
                InputQuery {
                    which_task: WhichTask::NewConversation,
                    input_query: InputQueryType::UserSubmittedQueryFromInput {
                        query,
                        static_query_type,
                        running_command: None,
                    },
                    additional_attachments: HashMap::new(),
                },
                is_queued_prompt,
                ctx,
            );
        }
    }

    /// Sends a query into an existing conversation as an agent-initiated request.
    /// This is the agent-initiated counterpart to `send_user_query_in_conversation`.
    pub fn send_agent_query_in_conversation(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            false,
            HashMap::new(),
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends the given user query to the AI model.
    pub fn send_user_query_in_conversation(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            false, // skip_running_command_detection
            HashMap::new(),
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends the first submission of a previously queued user prompt into an existing conversation.
    /// Same as [`Self::send_user_query_in_conversation`] but marks the emitted `SentRequest`
    /// event so UI subscribers (e.g. the input editor) know not to treat this as a direct
    /// user submission and therefore not clear the input buffer.
    pub fn send_queued_user_query_in_conversation(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            false, // skip_running_command_detection
            HashMap::new(),
            /*is_queued_prompt*/ true,
            ctx,
        );
    }

    /// Sends the given user query to the AI model, with additional referenced attachments.
    pub fn send_user_query_in_conversation_with_attachments(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        additional_attachments: HashMap<String, AIAgentAttachment>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            false, // skip_running_command_detection
            additional_attachments,
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends the given user query to the AI model, skipping long running command detection.
    /// We use this when we fork a conversation and immediately send an initial query, to avoid
    /// a race condition where restored command blocks may appear long running when the initial query is sent,
    /// causing the query to go to the lrc subagent.
    pub fn send_user_query_in_conversation_no_lrc_subagent(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_user_query_in_conversation_internal(
            query,
            conversation_id,
            true, // skip_running_command_detection
            HashMap::new(),
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn send_user_query_in_conversation_internal(
        &mut self,
        query: String,
        conversation_id: AIConversationId,
        skip_running_command_detection: bool,
        additional_attachments: HashMap<String, AIAgentAttachment>,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        // Ensure we capture all pending context blocks before promoting and attaching them to the conversation.
        let context_block_ids = self
            .context_model
            .as_ref(ctx)
            .pending_context_block_ids()
            .clone();

        let (promoted_blocks, task_id, running_command) = {
            let mut terminal_model = self.terminal_model.lock();
            terminal_model
                .block_list_mut()
                .associate_blocks_with_conversation(context_block_ids.iter(), conversation_id);

            // Promote all blocks that are pending for this conversation to attached.
            // This happens at query submission time, making blocks permanently associated with the conversation.
            let promoted_blocks = terminal_model
                .block_list_mut()
                .promote_blocks_to_attached_from_conversation(conversation_id);

            let active_block = terminal_model.block_list().active_block();
            let running_command_opt = if !skip_running_command_detection {
                get_running_command(&terminal_model)
            } else {
                None
            };

            let (task_id, running_command) = if let Some(running_command) = running_command_opt {
                let history_model = BlocklistAIHistoryModel::handle(ctx);
                match history_model.update(ctx, |history_model, ctx| {
                    history_model.create_cli_subagent_task_for_conversation(
                        running_command.block_id.clone(),
                        conversation_id,
                        self.terminal_view_id,
                        ctx,
                    )
                }) {
                    Ok(task_id) => (task_id, Some(running_command)),
                    Err(e) => {
                        log::error!("Could not create CLI subagent task optimistically: {e:?}");
                        return;
                    }
                }
            } else if let Some(task_id) = active_block
                .is_agent_monitoring()
                .then(|| active_block.agent_interaction_metadata())
                .flatten()
                .filter(|metadata| metadata.conversation_id() == &conversation_id)
                .and_then(|metadata| metadata.subagent_task_id().cloned())
            {
                (task_id, None)
            } else {
                let history_model = BlocklistAIHistoryModel::as_ref(ctx);
                let Some(conversation) = history_model.conversation(&conversation_id) else {
                    log::error!(
                        "Tried to send follow-up query for non-existent conversation: {conversation_id:?}"
                    );
                    return;
                };

                (conversation.get_root_task_id().clone(), None)
            };

            (promoted_blocks, task_id, running_command)
        };

        // Persist the updated visibility for each promoted block
        if !promoted_blocks.is_empty() {
            if let Some(sender) = GlobalResourceHandlesProvider::as_ref(ctx)
                .get()
                .model_event_sender
                .as_ref()
            {
                for (block_id, agent_view_visibility) in promoted_blocks {
                    if let Err(e) = sender.send(ModelEvent::UpdateBlockAgentViewVisibility {
                        block_id: block_id.to_string(),
                        agent_view_visibility: agent_view_visibility.into(),
                    }) {
                        log::error!("Error sending UpdateBlockAgentViewVisibility event: {e:?}");
                    }
                }
            }
        }

        self.send_query(
            InputQuery {
                which_task: WhichTask::Task {
                    conversation_id,
                    task_id,
                },
                input_query: InputQueryType::UserSubmittedQueryFromInput {
                    query,
                    static_query_type: None,
                    running_command,
                },
                additional_attachments,
            },
            is_queued_prompt,
            ctx,
        );
    }

    /// Sends a request triggered by a zero-state prompt suggestion.
    pub fn send_zero_state_prompt_suggestion(
        &mut self,
        query_type: ZeroStatePromptSuggestionType,
        ctx: &mut ModelContext<Self>,
    ) {
        self.send_query(
            InputQuery {
                which_task: WhichTask::NewConversation,
                input_query: InputQueryType::UserSubmittedQueryFromInput {
                    query: query_type.query().to_string(),
                    static_query_type: query_type.static_query_type(),
                    running_command: None,
                },
                additional_attachments: HashMap::new(),
            },
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    /// Sends a custom [`AIAgentInput`] query.
    pub fn send_custom_ai_input_query(
        &mut self,
        ai_input: AIAgentInput,
        ctx: &mut ModelContext<Self>,
    ) {
        let which_task = match self.context_model.as_ref(ctx).selected_conversation_id(ctx) {
            Some(id) => {
                let Some(conversation) = BlocklistAIHistoryModel::as_ref(ctx).conversation(&id)
                else {
                    log::error!(
                        "Tried to send custom AI input query as follow-up in non-existent conversation"
                    );
                    return;
                };
                WhichTask::Task {
                    conversation_id: conversation.id(),
                    task_id: conversation.get_root_task_id().clone(),
                }
            }
            None => WhichTask::NewConversation,
        };
        self.send_query(
            InputQuery {
                which_task,
                input_query: InputQueryType::AIInputType { ai_input },
                additional_attachments: HashMap::new(),
            },
            /*is_queued_prompt*/ false,
            ctx,
        )
    }

    pub fn send_slash_command_request(
        &mut self,
        slash_command: SlashCommandRequest,
        ctx: &mut ModelContext<Self>,
    ) {
        slash_command.send_request(self, /*is_queued_prompt*/ false, ctx);
    }

    /// Same as [`Self::send_slash_command_request`] but marks the emitted `SentRequest`
    /// event as a queued prompt submission so UI subscribers (e.g. the input editor)
    /// don't clear the input buffer on the auto-send.
    pub fn send_queued_slash_command_request(
        &mut self,
        slash_command: SlashCommandRequest,
        ctx: &mut ModelContext<Self>,
    ) {
        slash_command.send_request(self, /*is_queued_prompt*/ true, ctx);
    }

    /// Mark a conversation to follow up after its actions complete and attempt to send immediately
    /// if results are already available.
    pub fn request_follow_up_after_actions(
        &mut self,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.pending_passive_follow_ups.insert(conversation_id);

        if AcpAgentModel::as_ref(ctx).has_active_session_for_conversation(conversation_id) {
            return;
        }

        let has_pending_actions = self
            .action_model
            .as_ref(ctx)
            .get_pending_actions_for_conversation(&conversation_id)
            .next()
            .is_some();
        if has_pending_actions {
            return;
        }

        let finished_action_results = self
            .action_model
            .as_ref(ctx)
            .get_finished_action_results(conversation_id);
        if finished_action_results.is_some_and(|results| !results.is_empty()) {
            self.send_follow_up_for_conversation(conversation_id, ctx);
        }
    }

    /// Sends a custom AI input, building context from the current session.
    pub fn send_ai_input_with_context(
        &mut self,
        build_input: impl FnOnce(Arc<[AIAgentContext]>) -> AIAgentInput,
        ctx: &mut ModelContext<Self>,
    ) {
        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            None,
            vec![],
            ctx,
        );
        self.send_custom_ai_input_query(build_input(context), ctx);
    }

    fn send_follow_up_for_conversation(
        &mut self,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        if AcpAgentModel::as_ref(ctx).has_active_session_for_conversation(conversation_id) {
            return;
        }

        BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
            history.mark_active_conversation_id(conversation_id, self.terminal_view_id, ctx);
        });

        let finished_results = self.action_model.update(ctx, |action_model, _| {
            action_model.drain_finished_action_results(conversation_id)
        });
        if finished_results.is_empty() {
            return;
        }

        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            Some(conversation_id),
            vec![],
            ctx,
        );
        let request_input = RequestInput::for_actions_results(
            finished_results,
            context,
            &self.active_session,
            conversation_id,
            ctx,
        );

        let _ = self.send_request_input(request_input, /*is_queued_prompt*/ false, ctx);

        self.pending_passive_follow_ups.remove(&conversation_id);
    }

    pub fn resume_conversation(
        &mut self,
        conversation_id: AIConversationId,
        additional_context: Vec<AIAgentContext>,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(conversation) =
            BlocklistAIHistoryModel::as_ref(ctx).conversation(&conversation_id)
        else {
            log::error!("Tried to resume non-existent conversation: {conversation_id:?}");
            return;
        };
        let task_id = {
            let terminal_model = self.terminal_model.lock();
            let active_block = terminal_model.block_list().active_block();
            if let Some(agent_interaction_metadata) = active_block
                .agent_interaction_metadata()
                .filter(|metadata| {
                    metadata.conversation_id() == &conversation_id && metadata.is_agent_in_control()
                })
            {
                agent_interaction_metadata
                    .subagent_task_id()
                    .cloned()
                    .unwrap_or_else(|| conversation.get_root_task_id().clone())
            } else {
                conversation.get_root_task_id().clone()
            }
        };

        let context = input_context_for_request(
            false,
            self.context_model.as_ref(ctx),
            self.active_session.as_ref(ctx),
            Some(conversation_id),
            additional_context,
            ctx,
        );

        let inputs = vec![AIAgentInput::ResumeConversation { context }];
        let _ = self.send_request_input(
            RequestInput::for_task(inputs, task_id, &self.active_session, conversation_id, ctx),
            /*is_queued_prompt*/ false,
            ctx,
        );
    }

    fn start_new_conversation_for_request<'a>(
        &self,
        ctx: &'a mut ModelContext<Self>,
    ) -> &'a AIConversation {
        let is_autoexecute_override = self
            .context_model
            .as_ref(ctx)
            .pending_query_autoexecute_override(ctx)
            .is_autoexecute_any_action();
        let history_model = BlocklistAIHistoryModel::handle(ctx);
        let id = history_model.update(ctx, |history_model, ctx| {
            // We don't mark passive conversations as "the active conversation" (at least when they first appear).
            history_model.start_new_conversation(
                self.terminal_view_id,
                is_autoexecute_override,
                ctx,
            )
        });
        history_model
            .as_ref(ctx)
            .conversation(&id)
            .expect("Conversation exists- was just created.")
    }

    fn acp_model_info(app: &AppContext) -> (LLMId, String) {
        let settings = AISettings::as_ref(app);
        let backend_id = settings.acp_agent_backend.as_str();
        let display_name = settings
            .acp_default_config_options
            .get("model")
            .cloned()
            .unwrap_or_else(|| {
                crate::ai::acp::registry::AcpRegistryModel::as_ref(app)
                    .registry()
                    .launch_for_agent(backend_id)
                    .map(|launch| launch.display_name)
                    .unwrap_or_else(|| backend_id.to_string())
            });
        (LLMId::from(display_name.clone()), display_name)
    }

    fn acp_prompt_from_request(request_input: &RequestInput) -> AcpPromptPayload {
        let mut sections = request_input
            .all_inputs()
            .filter_map(|input| input.user_query())
            .collect_vec();

        if sections.is_empty() {
            sections.extend(request_input.all_inputs().map(ToString::to_string));
        }

        let context_sections = request_input
            .all_inputs()
            .flat_map(Self::acp_context_sections_for_input)
            .collect_vec();

        if !context_sections.is_empty() {
            sections.push(format!("Context:\n{}", context_sections.join("\n\n")));
        }

        let display_prompt = sections.join("\n\n");
        let mut content_blocks = vec![ContentBlock::Text(TextContent::new(display_prompt.clone()))];
        content_blocks.extend(
            request_input
                .all_inputs()
                .flat_map(Self::acp_rich_content_blocks_for_input),
        );
        AcpPromptPayload {
            content_blocks,
            display_prompt,
        }
    }

    fn acp_context_sections_for_input(input: &AIAgentInput) -> Vec<String> {
        let mut sections = input
            .context()
            .into_iter()
            .flatten()
            .filter_map(Self::acp_context_section)
            .collect_vec();

        if let AIAgentInput::UserQuery {
            referenced_attachments,
            ..
        } = input
        {
            sections.extend(
                referenced_attachments
                    .iter()
                    .filter_map(|(name, attachment)| {
                        Self::acp_attachment_section(name, attachment)
                    }),
            );
        }

        sections
    }

    fn acp_context_section(context: &AIAgentContext) -> Option<String> {
        match context {
            AIAgentContext::SelectedText(text) => Some(format!("Selected text:\n{text}")),
            AIAgentContext::File(file) => Some(Self::acp_file_context_section(file)),
            AIAgentContext::Block(block) => Some(format!(
                "Terminal block:\nCommand: {}\nExit code: {}\nOutput:\n{}",
                block.command,
                block.exit_code.value(),
                block.output
            )),
            AIAgentContext::Directory { pwd, .. } => {
                pwd.as_ref().map(|pwd| format!("Working directory:\n{pwd}"))
            }
            AIAgentContext::Git { head, branch } => Some(format!(
                "Git:\nHEAD: {head}\nBranch: {}",
                branch.as_deref().unwrap_or("")
            )),
            AIAgentContext::ProjectRules {
                root_path,
                active_rules,
                additional_rule_paths,
            } => {
                let mut parts = vec![format!("Project rules root:\n{root_path}")];
                parts.extend(active_rules.iter().map(Self::acp_file_context_section));
                if !additional_rule_paths.is_empty() {
                    parts.push(format!(
                        "Additional rule paths:\n{}",
                        additional_rule_paths.join("\n")
                    ));
                }
                Some(parts.join("\n\n"))
            }
            AIAgentContext::Codebase { path, name } => Some(format!("Codebase:\n{name}\n{path}")),
            AIAgentContext::CurrentTime { current_time } => {
                Some(format!("Current time:\n{current_time}"))
            }
            AIAgentContext::ExecutionEnvironment(execution_context) => Some(format!(
                "Execution environment:\nShell: {}\nVersion: {}",
                execution_context.shell_name,
                execution_context.shell_version.as_deref().unwrap_or("")
            )),
            AIAgentContext::Image(_) => None,
        }
    }

    fn acp_attachment_section(name: &str, attachment: &AIAgentAttachment) -> Option<String> {
        match attachment {
            AIAgentAttachment::PlainText(text) => Some(format!("Attachment {name}:\n{text}")),
            AIAgentAttachment::DocumentContent { content, .. } => {
                Some(format!("Document attachment {name}:\n{content}"))
            }
            AIAgentAttachment::DiffHunk {
                file_path,
                diff_content,
                ..
            } => Some(format!(
                "Diff attachment {name}:\n{file_path}\n{diff_content}"
            )),
            AIAgentAttachment::DiffSet { file_diffs, .. } => {
                let diffs = file_diffs
                    .iter()
                    .flat_map(|(path, hunks)| {
                        hunks
                            .iter()
                            .map(move |hunk| format!("{path}\n{}", hunk.diff_content))
                    })
                    .join("\n\n");
                (!diffs.is_empty()).then(|| format!("Diff attachment {name}:\n{diffs}"))
            }
            AIAgentAttachment::FilePathReference { .. } => None,
            AIAgentAttachment::Block(block) => Some(format!(
                "Terminal block attachment {name}:\nCommand: {}\nExit code: {}\nOutput:\n{}",
                block.command,
                block.exit_code.value(),
                block.output
            )),
        }
    }

    fn acp_rich_content_blocks_for_input(input: &AIAgentInput) -> Vec<ContentBlock> {
        let mut blocks = Vec::new();

        if let Some(contexts) = input.context() {
            blocks.extend(contexts.iter().filter_map(|context| match context {
                AIAgentContext::Image(image) => Some(ContentBlock::Image(ImageContent::new(
                    image.data.clone(),
                    image.mime_type.clone(),
                ))),
                _ => None,
            }));
        }

        if let AIAgentInput::UserQuery {
            referenced_attachments,
            ..
        } = input
        {
            blocks.extend(
                referenced_attachments
                    .iter()
                    .sorted_by(|(left, _), (right, _)| left.cmp(right))
                    .filter_map(|(name, attachment)| match attachment {
                        AIAgentAttachment::FilePathReference { file_path, .. } => {
                            Some(Self::acp_file_attachment_content_block(name, file_path))
                        }
                        _ => None,
                    }),
            );
        }

        blocks
    }

    fn acp_file_attachment_content_block(name: &str, file_path: &str) -> ContentBlock {
        let path = Path::new(file_path);
        let uri = Self::acp_file_uri(path, file_path);
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        if let Some(resource) = Self::acp_embedded_resource_for_file(path, &uri, &mime_type) {
            return ContentBlock::Resource(resource);
        }

        let size = std::fs::metadata(path)
            .ok()
            .and_then(|metadata| i64::try_from(metadata.len()).ok());

        ContentBlock::ResourceLink(
            ResourceLink::new(name.to_string(), uri)
                .mime_type(mime_type)
                .size(size)
                .title(name.to_string()),
        )
    }

    fn acp_embedded_resource_for_file(
        path: &Path,
        uri: &str,
        mime_type: &str,
    ) -> Option<EmbeddedResource> {
        let metadata = std::fs::metadata(path).ok()?;
        if !metadata.is_file() {
            return None;
        }

        let bytes = std::fs::read(path).ok()?;

        let resource = match String::from_utf8(bytes) {
            Ok(text) => EmbeddedResourceResource::TextResourceContents(
                TextResourceContents::new(text, uri.to_string()).mime_type(mime_type.to_string()),
            ),
            Err(err) => EmbeddedResourceResource::BlobResourceContents(
                BlobResourceContents::new(
                    general_purpose::STANDARD.encode(err.into_bytes()),
                    uri.to_string(),
                )
                .mime_type(mime_type.to_string()),
            ),
        };

        Some(EmbeddedResource::new(resource))
    }

    fn acp_file_uri(path: &Path, fallback: &str) -> String {
        Url::from_file_path(path)
            .map(|url| url.to_string())
            .unwrap_or_else(|_| fallback.to_string())
    }

    fn acp_file_context_section(file: &FileContext) -> String {
        let content = match &file.content {
            AnyFileContent::StringContent(content) => content.clone(),
            AnyFileContent::BinaryContent(content) => {
                format!("Binary content: {} bytes", content.len())
            }
        };
        format!("File: {}\n{content}", file.file_name)
    }

    /// Attempts to send a request to the AI model API. Adds context to the input if it
    /// contains a user query. Returns `Err` if the AI input was not able to be sent due to an
    /// existing in-flight request. Emits an event containing a receiver for the AI's output.
    /// If conversation_id is Some, we follow up in that conversation.
    /// If it's None or we can't find a conversation with that ID, we start a new one.
    /// Returns the conversation ID of affected conversation and response stream ID.
    ///
    ///  This function does not handle cancelling any in flight requests (and sending them back as
    /// input) for an existing conversation. Consider calling [`Self::send_custom_ai_input_query`] if
    /// you're trying to send a query with a custom [`AIAgentInput`] type where you'd like the "normal"
    /// flow that handles existing conversations properly.
    fn send_request_input(
        &mut self,
        mut request_input: RequestInput,
        is_queued_prompt: bool,
        ctx: &mut ModelContext<Self>,
    ) -> anyhow::Result<(AIConversationId, ResponseStreamId)> {
        let history_model = BlocklistAIHistoryModel::handle(ctx);
        let conversation_id = {
            let Some(conversation) = history_model
                .as_ref(ctx)
                .conversation(&request_input.conversation_id)
            else {
                return Err(anyhow!(
                    "Tried to send request for non-existent conversation with ID {:?}",
                    request_input.conversation_id
                ));
            };

            conversation.id()
        };

        if AcpAgentModel::as_ref(ctx).has_active_session_for_conversation(conversation_id) {
            const AI_INPUT_NOT_SENT_ERROR_STR: &str =
                "Not sending AI input because there is an in-flight request";
            safe_assert!(false, "{}", AI_INPUT_NOT_SENT_ERROR_STR);
            return Err(anyhow::anyhow!(AI_INPUT_NOT_SENT_ERROR_STR));
        }

        let (acp_model_id, acp_model_display_name) = Self::acp_model_info(ctx);
        request_input.model_id = acp_model_id.clone();
        request_input.coding_model_id = acp_model_id.clone();
        request_input.cli_agent_model_id = acp_model_id.clone();
        request_input.computer_use_model_id = acp_model_id.clone();

        let response_stream_id = ResponseStreamId::new();
        let input_contains_user_query = request_input
            .all_inputs()
            .any(|input| input.is_user_query());

        let is_passive_request = request_input
            .all_inputs()
            .any(|input| input.is_passive_request());
        let acp_prompt = Self::acp_prompt_from_request(&request_input);
        let acp_cwd = request_input
            .working_directory
            .as_deref()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("/"));
        log::info!(
            "ACP: controller prepared request conversation={:?} stream={:?} cwd={} prompt_bytes={} model={}",
            conversation_id,
            response_stream_id,
            acp_cwd.display(),
            acp_prompt.display_prompt.len(),
            acp_model_display_name,
        );

        for input in request_input.all_inputs() {
            if let AIAgentInput::UserQuery {
                referenced_attachments,
                ..
            } = input
            {
                self.maybe_populate_plans_for_ai_document_model(
                    referenced_attachments,
                    conversation_id,
                    ctx,
                );
            }
        }

        history_model.update(ctx, |history_model, ctx| {
            match history_model.update_conversation_for_new_request_input(
                request_input,
                response_stream_id.clone(),
                self.terminal_view_id,
                ctx,
            ) {
                Ok(_) => {
                    history_model.update_conversation_status(
                        self.terminal_view_id,
                        conversation_id,
                        ConversationStatus::InProgress,
                        ctx,
                    );
                }
                Err(e) => {
                    log::warn!("Failed to push new exchange to AI conversation: {e:?}");
                }
            }
        });

        AcpAgentModel::handle(ctx).update(ctx, |model, ctx| {
            log::info!(
                "ACP: controller submitting request conversation={:?} stream={:?}",
                conversation_id,
                response_stream_id,
            );
            model.submit_prompt_for_run_target(
                acp_prompt.display_prompt,
                acp_prompt.content_blocks,
                acp_cwd,
                AcpRunTarget {
                    conversation_id,
                    response_stream_id: response_stream_id.clone(),
                    terminal_view_id: self.terminal_view_id,
                    model_id: acp_model_id.clone(),
                    display_name: acp_model_display_name.clone(),
                },
                ctx,
            );
        });

        if input_contains_user_query {
            // Get the pending document ID before clearing context
            let pending_document_id = self.context_model.as_ref(ctx).pending_document_id();

            // Reset the context state to the default.
            self.context_model.update(ctx, |context_model, ctx| {
                context_model.reset_context_to_default(ctx);
            });

            // Update the document status to UpToDate after query submission
            if let Some(doc_id) = pending_document_id {
                AIDocumentModel::handle(ctx).update(ctx, |model, mctx| {
                    model.set_user_edit_status(&doc_id, AIDocumentUserEditStatus::UpToDate, mctx);
                });
            }
        }

        ctx.emit(BlocklistAIControllerEvent::SentRequest {
            contains_user_query: input_contains_user_query,
            is_queued_prompt,
            model_id: acp_model_id.clone(),
            stream_id: response_stream_id.clone(),
        });
        if !is_passive_request {
            history_model.update(ctx, |history_model, ctx| {
                history_model.mark_active_conversation_id(
                    conversation_id,
                    self.terminal_view_id,
                    ctx,
                )
            });
        }

        // Trigger a snapshot save to persist the agent view state when a user query is sent.
        // This ensures the agent view is restored if the app restarts.
        if input_contains_user_query {
            ctx.dispatch_global_action("workspace:save_app", ());
        }

        Ok((conversation_id, response_stream_id))
    }

    /// Cancels a pending AI request response stream, given the exchange ID, if it exists.
    /// Returns true if a pending stream was found and canceled, false otherwise.
    pub fn try_cancel_pending_response_stream(
        &mut self,
        stream_id: &ResponseStreamId,
        _reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        let Some(conversation_id) =
            BlocklistAIHistoryModel::as_ref(ctx).conversation_for_response_stream(stream_id)
        else {
            log::warn!("Could not find conversation for stream {stream_id:?}, cannot cancel");
            return false;
        };
        AcpAgentModel::handle(ctx).update(ctx, |model, _| model.cancel_session(conversation_id))
    }

    /// Cancels 'progress' for the active conversation if there is one:
    ///  * If there is an in-flight request, cancels it.
    ///  * Else, if the request finished, but actions from the response are pending or mid-execution, cancels all of them.
    pub fn cancel_conversation_progress(
        &mut self,
        conversation_id: AIConversationId,
        reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) {
        if !AcpAgentModel::handle(ctx).update(ctx, |model, _| model.cancel_session(conversation_id))
        {
            // Otherwise, cancel pending actions and update the input state.
            self.action_model.update(ctx, |action_model, ctx| {
                action_model.cancel_all_pending_actions(conversation_id, Some(reason), ctx);
            });
            self.set_input_mode_for_cancellation(ctx);
        }
    }

    /// Clears finished action results for a conversation. Used when reverting.
    pub fn clear_finished_action_results(
        &mut self,
        conversation_id: AIConversationId,
        ctx: &mut ModelContext<Self>,
    ) {
        self.action_model.update(ctx, |action_model, _| {
            action_model.clear_finished_action_results(conversation_id);
        });
    }

    /// Cancels the in-flight request for the given conversation, if there is one.
    ///
    /// Returns `true` if a request was actually cancelled.
    pub fn cancel_request(
        &mut self,
        response_stream_id: &ResponseStreamId,
        _reason: CancellationReason,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        let Some(conversation_id) = BlocklistAIHistoryModel::as_ref(ctx)
            .conversation_for_response_stream(response_stream_id)
        else {
            log::warn!(
                "Could not find conversation for stream {response_stream_id:?}, cannot cancel"
            );
            return false;
        };
        AcpAgentModel::handle(ctx).update(ctx, |model, _| model.cancel_session(conversation_id))
    }

    /// Sets the terminal input state after an AI request is cancelled.
    /// From the user perspective, we downgrade the level of autonomy so:
    /// * Executing a task automatically -> interactive AI input
    /// * Interactive AI input -> interactive shell input
    fn set_input_mode_for_cancellation(&mut self, ctx: &mut ModelContext<Self>) {
        // If the request was cancelled, default to shell mode with autodetection
        // enabled.
        self.input_model.update(ctx, |input_model, ctx| {
            input_model.set_input_config_for_classic_mode(
                input_model
                    .input_config()
                    .with_shell_type()
                    .unlocked_if_autodetection_enabled(false, ctx),
                ctx,
            );
        });
    }
}

impl Entity for BlocklistAIController {
    type Event = BlocklistAIControllerEvent;
}

#[derive(Clone)]
pub struct ClientIdentifiers {
    pub conversation_id: AIConversationId,
    pub client_exchange_id: AIAgentExchangeId,
    /// Not populated for restored AI blocks.
    pub response_stream_id: Option<ResponseStreamId>,
}

#[allow(clippy::too_many_arguments)]
fn input_for_query(
    query: String,
    task_id: &TaskId,
    conversation_id: AIConversationId,
    static_query_type: Option<StaticQueryType>,
    user_query_mode: UserQueryMode,
    running_command: Option<RunningCommand>,
    additional_attachments: HashMap<String, AIAgentAttachment>,
    context_model: &BlocklistAIContextModel,
    active_session: &ActiveSession,
    app: &AppContext,
) -> AIAgentInput {
    let context = input_context_for_request(
        true,
        context_model,
        active_session,
        Some(conversation_id),
        vec![],
        app,
    );
    let _ = task_id;
    let mut referenced_attachments = parse_context_attachments(&query, context_model, app);
    referenced_attachments.extend(additional_attachments);
    AIAgentInput::UserQuery {
        query,
        context,
        static_query_type,
        referenced_attachments,
        user_query_mode,
        running_command,
    }
}

fn get_running_command(terminal_model: &TerminalModel) -> Option<RunningCommand> {
    let active_block = terminal_model.block_list().active_block();
    if !active_block.is_active_and_long_running() || active_block.is_agent_monitoring() {
        return None;
    }
    let is_alt_screen_active = terminal_model.is_alt_screen_active();
    Some(RunningCommand {
        block_id: active_block.id().clone(),
        command: active_block.command_to_string(),
        grid_contents: if is_alt_screen_active {
            formatted_terminal_contents_for_input(
                terminal_model.alt_screen().grid_handler(),
                None,
                CURSOR_MARKER,
            )
        } else {
            formatted_terminal_contents_for_input(
                active_block.output_grid().grid_handler(),
                // TODO(vorporeal): This is probably too large.
                Some(1000),
                CURSOR_MARKER,
            )
        },
        cursor: CURSOR_MARKER.to_owned(),
        requested_command_id: active_block.requested_command_action_id().cloned(),
        is_alt_screen_active,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent::AnyFileContent;
    use crate::ai::block_context::BlockContext;
    use crate::terminal::model::block::BlockId;
    use crate::terminal::model::terminal_model::BlockIndex;
    use agent_client_protocol::schema::EmbeddedResourceResource;
    use warp_core::command::ExitCode;

    #[test]
    fn acp_prompt_preserves_user_context_inputs() {
        let task_id = TaskId::new("root-task".to_string());
        let selected_text = "selected text from terminal";
        let file_content = "fn helper() {}";
        let block_output = "test failed";
        let context = vec![
            AIAgentContext::SelectedText(selected_text.to_string()),
            AIAgentContext::File(FileContext::new(
                "src/lib.rs".to_string(),
                AnyFileContent::StringContent(file_content.to_string()),
                None,
                None,
            )),
            AIAgentContext::Block(Box::new(BlockContext {
                id: BlockId::default(),
                index: BlockIndex::zero(),
                command: "cargo test".to_string(),
                output: block_output.to_string(),
                exit_code: ExitCode::from(101),
                is_auto_attached: false,
                started_ts: None,
                finished_ts: None,
                pwd: Some("/repo".to_string()),
                shell: Some("zsh".to_string()),
                username: None,
                hostname: None,
                git_branch: None,
                os: None,
                session_id: None,
            })),
        ];
        let request_input = RequestInput {
            conversation_id: AIConversationId::new(),
            input_messages: HashMap::from([(
                task_id,
                vec![AIAgentInput::UserQuery {
                    query: "Fix the failing test".to_string(),
                    context: context.into(),
                    static_query_type: None,
                    referenced_attachments: Default::default(),
                    user_query_mode: UserQueryMode::Normal,
                    running_command: None,
                }],
            )]),
            working_directory: Some("/repo".to_string()),
            model_id: LLMId::from("test-model"),
            coding_model_id: LLMId::from("test-model"),
            cli_agent_model_id: LLMId::from("test-model"),
            computer_use_model_id: LLMId::from("test-model"),
            request_start_ts: Local::now(),
        };

        let payload = BlocklistAIController::acp_prompt_from_request(&request_input);

        assert!(payload.display_prompt.starts_with("Fix the failing test"));
        assert!(payload.display_prompt.contains(selected_text));
        assert!(payload.display_prompt.contains("src/lib.rs"));
        assert!(payload.display_prompt.contains(file_content));
        assert!(payload.display_prompt.contains("cargo test"));
        assert!(payload.display_prompt.contains(block_output));
        assert_eq!(payload.content_blocks.len(), 1);
        assert!(matches!(
            &payload.content_blocks[0],
            agent_client_protocol::schema::ContentBlock::Text(text)
                if text.text == payload.display_prompt
        ));
    }

    #[test]
    fn acp_prompt_includes_rich_attachment_content_blocks() {
        let temp = tempfile::TempDir::new().unwrap();
        let file_path = temp.path().join("notes.txt");
        std::fs::write(&file_path, "attached file body").unwrap();
        let task_id = TaskId::new("root-task".to_string());
        let image = crate::ai::agent::ImageContext {
            data: "base64-image".to_string(),
            mime_type: "image/png".to_string(),
            file_name: "image.png".to_string(),
            is_figma: false,
        };
        let request_input = RequestInput {
            conversation_id: AIConversationId::new(),
            input_messages: HashMap::from([(
                task_id,
                vec![AIAgentInput::UserQuery {
                    query: "Use the attachments".to_string(),
                    context: vec![AIAgentContext::Image(image)].into(),
                    static_query_type: None,
                    referenced_attachments: HashMap::from([(
                        "notes.txt".to_string(),
                        AIAgentAttachment::FilePathReference {
                            file_id: "file-1".to_string(),
                            file_name: "notes.txt".to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                    )]),
                    user_query_mode: UserQueryMode::Normal,
                    running_command: None,
                }],
            )]),
            working_directory: Some(temp.path().to_string_lossy().to_string()),
            model_id: LLMId::from("test-model"),
            coding_model_id: LLMId::from("test-model"),
            cli_agent_model_id: LLMId::from("test-model"),
            computer_use_model_id: LLMId::from("test-model"),
            request_start_ts: Local::now(),
        };

        let payload = BlocklistAIController::acp_prompt_from_request(&request_input);

        assert_eq!(payload.content_blocks.len(), 3);
        assert!(matches!(
            &payload.content_blocks[0],
            ContentBlock::Text(text) if text.text == "Use the attachments"
        ));
        assert!(matches!(
            &payload.content_blocks[1],
            ContentBlock::Image(image)
                if image.data == "base64-image" && image.mime_type == "image/png"
        ));
        assert!(matches!(
            &payload.content_blocks[2],
            ContentBlock::Resource(resource)
                if matches!(
                    &resource.resource,
                    EmbeddedResourceResource::TextResourceContents(resource)
                        if resource.text == "attached file body"
                            && resource.mime_type.as_deref() == Some("text/plain")
                )
        ));
    }
}
