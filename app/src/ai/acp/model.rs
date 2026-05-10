use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;

use agent_client_protocol::schema::{
    CancelNotification, ClientCapabilities, ContentBlock, CreateTerminalRequest,
    CreateTerminalResponse, FileSystemCapabilities, InitializeRequest, KillTerminalRequest,
    KillTerminalResponse, NewSessionRequest, PromptRequest, ProtocolVersion, ReadTextFileRequest,
    ReadTextFileResponse, ReleaseTerminalRequest, ReleaseTerminalResponse,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome, SessionConfigId, SessionConfigKind, SessionConfigOption,
    SessionConfigSelectOptions, SessionConfigValueId, SessionNotification,
    SetSessionConfigOptionRequest, StopReason, TerminalExitStatus, TerminalId,
    TerminalOutputRequest, TerminalOutputResponse, TextContent, WaitForTerminalExitRequest,
    WaitForTerminalExitResponse, WriteTextFileRequest, WriteTextFileResponse,
};
use agent_client_protocol::{Agent, Client, ConnectionTo};
use agent_client_protocol_tokio::AcpAgent;
use anyhow::{anyhow, Context as _};
use futures::channel::{mpsc, mpsc::UnboundedSender, oneshot};
use futures::StreamExt;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use warpui::{Entity, EntityId, ModelContext, SingletonEntity};

use crate::ai::agent::conversation::AIConversationId;
use crate::ai::agent::CancellationReason;
use crate::ai::agent::RenderableAIError;
use crate::ai::blocklist::{AcpResponseStreamTarget, BlocklistAIHistoryModel, ResponseStreamId};
use crate::ai::llms::LLMId;
use crate::settings::{AISettings, AcpAgentBackend};

use super::backend::adapter_is_available;
use super::events::AcpEvent;
use super::mapping::{map_session_update, session_update_label};
use super::{
    AcpCommands, AcpPermissionRequest, AcpPermissionSelection, AcpPlan, AcpSessionInfo,
    AcpSessionState, AcpTerminalTrace,
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AcpAgentState {
    Idle,
    Starting,
    Ready,
    Running,
    Failed(String),
}

pub struct AcpAgentModel {
    state: AcpAgentState,
    #[allow(dead_code)]
    events: Vec<AcpEvent>,
    session_state_by_conversation: HashMap<AIConversationId, AcpSessionState>,
    pending_permission_responses: HashMap<String, oneshot::Sender<AcpPermissionSelection>>,
    pending_session_cancels: HashMap<AIConversationId, UnboundedSender<()>>,
    allow_adapter_execution: bool,
}

#[derive(Clone)]
pub(crate) struct AcpRunTarget {
    pub conversation_id: AIConversationId,
    pub response_stream_id: ResponseStreamId,
    pub terminal_view_id: EntityId,
    pub model_id: LLMId,
    pub display_name: String,
}

impl AcpRunTarget {
    fn response_stream_target(&self) -> AcpResponseStreamTarget {
        AcpResponseStreamTarget {
            stream_id: self.response_stream_id.clone(),
            conversation_id: self.conversation_id,
            terminal_view_id: self.terminal_view_id,
            model_id: self.model_id.clone(),
            display_name: self.display_name.clone(),
        }
    }
}

enum AcpRuntimeEvent {
    Event(AcpEvent),
    PermissionRequested {
        request: AcpPermissionRequest,
        response: oneshot::Sender<AcpPermissionSelection>,
    },
}

impl AcpAgentModel {
    pub fn new(_: &mut ModelContext<Self>) -> Self {
        Self {
            state: AcpAgentState::Idle,
            events: Vec::new(),
            session_state_by_conversation: HashMap::new(),
            pending_permission_responses: HashMap::new(),
            pending_session_cancels: HashMap::new(),
            allow_adapter_execution: true,
        }
    }

    #[cfg(test)]
    pub fn new_for_test(_: &mut ModelContext<Self>) -> Self {
        Self {
            state: AcpAgentState::Idle,
            events: Vec::new(),
            session_state_by_conversation: HashMap::new(),
            pending_permission_responses: HashMap::new(),
            pending_session_cancels: HashMap::new(),
            allow_adapter_execution: false,
        }
    }

    #[cfg(test)]
    pub fn state(&self) -> AcpAgentState {
        self.state.clone()
    }

    #[allow(dead_code)]
    pub fn events(&self) -> &[AcpEvent] {
        &self.events
    }

    pub(crate) fn session_state(
        &self,
        conversation_id: AIConversationId,
    ) -> Option<&AcpSessionState> {
        self.session_state_by_conversation.get(&conversation_id)
    }

    pub(crate) fn has_active_session_for_conversation(
        &self,
        conversation_id: AIConversationId,
    ) -> bool {
        self.pending_session_cancels.contains_key(&conversation_id)
    }

    pub(crate) fn available_commands_for_conversation(
        &self,
        conversation_id: AIConversationId,
    ) -> &[agent_client_protocol::schema::AvailableCommand] {
        self.session_state_by_conversation
            .get(&conversation_id)
            .map(|state| state.commands.commands.as_slice())
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn set_available_commands_for_test(
        &mut self,
        conversation_id: AIConversationId,
        commands: Vec<agent_client_protocol::schema::AvailableCommand>,
    ) {
        self.session_state_by_conversation
            .entry(conversation_id)
            .or_default()
            .commands = AcpCommands::new(commands);
    }

    pub fn select_permission_option(&mut self, request_id: &str, option_id: String) -> bool {
        let Some(response) = self.pending_permission_responses.remove(request_id) else {
            log::warn!("ACP: permission selection for unknown request_id={request_id}");
            return false;
        };

        response
            .send(AcpPermissionSelection::Selected { option_id })
            .is_ok()
    }

    #[allow(dead_code)]
    pub fn cancel_permission_request(&mut self, request_id: &str) -> bool {
        let Some(response) = self.pending_permission_responses.remove(request_id) else {
            log::warn!("ACP: permission cancel for unknown request_id={request_id}");
            return false;
        };

        response.send(AcpPermissionSelection::Cancelled).is_ok()
    }

    pub fn cancel_session(&mut self, conversation_id: AIConversationId) -> bool {
        let Some(cancel) = self.pending_session_cancels.remove(&conversation_id) else {
            log::warn!("ACP: cancel requested for unknown conversation={conversation_id:?}");
            return false;
        };

        log::info!("ACP: cancel requested conversation={conversation_id:?}");
        cancel.unbounded_send(()).is_ok()
    }

    #[allow(dead_code)]
    pub fn submit_prompt(&mut self, prompt: String, cwd: PathBuf, ctx: &mut ModelContext<Self>) {
        self.submit_prompt_internal(
            prompt.clone(),
            vec![ContentBlock::Text(TextContent::new(prompt))],
            cwd,
            None,
            ctx,
        );
    }

    pub(crate) fn submit_prompt_for_run_target(
        &mut self,
        display_prompt: String,
        content_blocks: Vec<ContentBlock>,
        cwd: PathBuf,
        target: AcpRunTarget,
        ctx: &mut ModelContext<Self>,
    ) {
        self.submit_prompt_internal(display_prompt, content_blocks, cwd, Some(target), ctx);
    }

    fn submit_prompt_internal(
        &mut self,
        display_prompt: String,
        content_blocks: Vec<ContentBlock>,
        cwd: PathBuf,
        target: Option<AcpRunTarget>,
        ctx: &mut ModelContext<Self>,
    ) {
        let settings = AISettings::as_ref(ctx);
        let backend = *settings.acp_agent_backend;
        let default_config_options = settings.acp_default_config_options.clone();
        log::info!(
            "ACP: submit requested backend={} command={} cwd={} prompt_bytes={} target={} default_config_options={}",
            backend.display_name(),
            backend.adapter_command(),
            cwd.display(),
            display_prompt.len(),
            target_summary(target.as_ref()),
            default_config_options.len(),
        );
        if !self.allow_adapter_execution {
            log::info!("ACP: adapter execution disabled; simulating session start");
            self.handle_event(AcpEvent::SessionStarted, target.as_ref(), ctx);
            return;
        }

        if !adapter_is_available(backend) {
            log::warn!(
                "ACP: adapter missing command={} install_command={}",
                backend.adapter_command(),
                backend.install_command(),
            );
            self.handle_event(
                AcpEvent::AdapterMissing {
                    command: backend.adapter_command().to_string(),
                    install_command: backend.install_command().to_string(),
                },
                target.as_ref(),
                ctx,
            );
            return;
        }

        self.state = AcpAgentState::Starting;
        let (events_tx, events_rx) = futures::channel::mpsc::unbounded();
        let (cancel_tx, cancel_rx) = futures::channel::mpsc::unbounded();
        if let Some(target) = target.as_ref() {
            self.pending_session_cancels
                .insert(target.conversation_id, cancel_tx);
        }
        let stream_target = target.clone();
        ctx.spawn_stream_local(
            events_rx,
            move |me, event, ctx| {
                me.handle_runtime_event(event, stream_target.as_ref(), ctx);
            },
            |_, _| {},
        );

        let completion_target = target;
        log::info!(
            "ACP: spawning adapter task backend={} command={}",
            backend.display_name(),
            backend.adapter_command(),
        );
        ctx.spawn(
            run_one_prompt(
                backend,
                default_config_options,
                display_prompt,
                content_blocks,
                cwd,
                events_tx,
                cancel_rx,
            ),
            move |me, result, ctx| match result {
                Ok(()) => {
                    log::info!("ACP: adapter task completed successfully");
                }
                Err(err) => {
                    log::error!("ACP: adapter task failed: {err:#}");
                    me.handle_event(
                        AcpEvent::Failed {
                            message: err.to_string(),
                        },
                        completion_target.as_ref(),
                        ctx,
                    );
                }
            },
        );
    }

    fn handle_event(
        &mut self,
        event: AcpEvent,
        target: Option<&AcpRunTarget>,
        ctx: &mut ModelContext<Self>,
    ) {
        let event_summary = acp_event_summary(&event);
        self.state = match &event {
            AcpEvent::AdapterMissing { command, .. } => {
                AcpAgentState::Failed(format!("{command} is not installed"))
            }
            AcpEvent::SessionStarted => AcpAgentState::Starting,
            AcpEvent::UserTextDelta { .. }
            | AcpEvent::AssistantTextDelta { .. }
            | AcpEvent::AssistantThoughtDelta { .. }
            | AcpEvent::ToolCallStarted { .. }
            | AcpEvent::ToolCallUpdated { .. }
            | AcpEvent::TerminalUpdated { .. }
            | AcpEvent::PlanUpdated { .. }
            | AcpEvent::AvailableCommandsUpdated { .. }
            | AcpEvent::CurrentModeUpdated { .. }
            | AcpEvent::ConfigOptionsUpdated { .. }
            | AcpEvent::SessionInfoUpdated { .. }
            | AcpEvent::PermissionRequested { .. } => AcpAgentState::Running,
            AcpEvent::Completed => AcpAgentState::Ready,
            AcpEvent::Cancelled => AcpAgentState::Ready,
            AcpEvent::Failed { message } => AcpAgentState::Failed(message.clone()),
        };
        log::info!(
            "ACP: event received {event_summary} target={} state={:?}",
            target_summary(target),
            self.state,
        );

        if let Some(target) = target {
            self.apply_session_event_to_state(&event, target.conversation_id);
            self.apply_event_to_history(&event, target, ctx);
            if matches!(
                event,
                AcpEvent::Completed | AcpEvent::Cancelled | AcpEvent::Failed { .. }
            ) {
                self.pending_session_cancels.remove(&target.conversation_id);
            }
        }

        ctx.emit(event.clone());
        self.events.push(event);
    }

    fn apply_session_event_to_state(
        &mut self,
        event: &AcpEvent,
        conversation_id: AIConversationId,
    ) {
        let state = self
            .session_state_by_conversation
            .entry(conversation_id)
            .or_default();
        match event {
            AcpEvent::SessionStarted => {
                *state = AcpSessionState::default();
            }
            AcpEvent::AvailableCommandsUpdated { commands } => {
                state.commands = AcpCommands::new(commands.clone());
            }
            AcpEvent::CurrentModeUpdated { update } => {
                state.config = state.config.clone().with_current_mode(update.clone());
            }
            AcpEvent::ConfigOptionsUpdated { update } => {
                state.config = state.config.clone().with_config(update.clone());
            }
            AcpEvent::SessionInfoUpdated { update } => {
                state.info = Some(AcpSessionInfo {
                    info: update.clone(),
                });
            }
            AcpEvent::TerminalUpdated { terminal_id, trace } => {
                state
                    .terminal_traces
                    .insert(terminal_id.clone(), trace.clone());
            }
            _ => {}
        }
    }

    fn handle_runtime_event(
        &mut self,
        event: AcpRuntimeEvent,
        target: Option<&AcpRunTarget>,
        ctx: &mut ModelContext<Self>,
    ) {
        match event {
            AcpRuntimeEvent::Event(event) => self.handle_event(event, target, ctx),
            AcpRuntimeEvent::PermissionRequested { request, response } => {
                self.pending_permission_responses
                    .insert(request.request_id.clone(), response);
                self.handle_event(AcpEvent::PermissionRequested { request }, target, ctx);
            }
        }
    }

    fn apply_event_to_history(
        &self,
        event: &AcpEvent,
        target: &AcpRunTarget,
        ctx: &mut ModelContext<Self>,
    ) {
        match event {
            AcpEvent::SessionStarted => {
                log::info!(
                    "ACP: initializing AI history output stream={:?} conversation={:?}",
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    history.initialize_local_output_for_response_stream(
                        &target.response_stream_id,
                        target.conversation_id,
                        target.terminal_view_id,
                        target.model_id.clone(),
                        target.display_name.clone(),
                        ctx,
                    );
                });
            }
            AcpEvent::UserTextDelta { .. } => {}
            AcpEvent::AssistantTextDelta { text } => {
                log::info!(
                    "ACP: appending assistant text bytes={} stream={:?} conversation={:?}",
                    text.len(),
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let history_target = target.response_stream_target();
                    history.append_local_text_delta_to_response_stream(&history_target, text, ctx);
                });
            }
            AcpEvent::AssistantThoughtDelta { text } => {
                log::info!(
                    "ACP: appending assistant thought bytes={} stream={:?} conversation={:?}",
                    text.len(),
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let history_target = target.response_stream_target();
                    history.append_local_thought_delta_to_response_stream(
                        &history_target,
                        text,
                        ctx,
                    );
                });
            }
            AcpEvent::ToolCallStarted { tool_call } => {
                let mut tool_call = tool_call.clone();
                if let Some(state) = self
                    .session_state_by_conversation
                    .get(&target.conversation_id)
                {
                    for (terminal_id, trace) in &state.terminal_traces {
                        tool_call.set_terminal_trace(terminal_id.clone(), trace.clone());
                    }
                }
                log::info!(
                    "ACP: upserting tool call id={} status={:?} stream={:?} conversation={:?}",
                    tool_call.id,
                    tool_call.status,
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let history_target = target.response_stream_target();
                    history.upsert_acp_tool_call_to_response_stream(
                        &history_target,
                        tool_call,
                        ctx,
                    );
                });
            }
            AcpEvent::ToolCallUpdated { update } => {
                log::info!(
                    "ACP: updating tool call id={} stream={:?} conversation={:?}",
                    update.tool_call_id.0.as_ref(),
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let history_target = target.response_stream_target();
                    history.update_acp_tool_call_to_response_stream(
                        &history_target,
                        update.clone(),
                        ctx,
                    );
                    if let Some(state) = self
                        .session_state_by_conversation
                        .get(&target.conversation_id)
                    {
                        for (terminal_id, trace) in &state.terminal_traces {
                            history.update_acp_terminal_trace_to_response_stream(
                                &target.response_stream_id,
                                target.conversation_id,
                                target.terminal_view_id,
                                terminal_id.clone(),
                                trace.clone(),
                                ctx,
                            );
                        }
                    }
                });
            }
            AcpEvent::TerminalUpdated { terminal_id, trace } => {
                log::info!(
                    "ACP: updating terminal trace terminal_id={} output_bytes={} stream={:?} conversation={:?}",
                    terminal_id,
                    trace.output.len(),
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    history.update_acp_terminal_trace_to_response_stream(
                        &target.response_stream_id,
                        target.conversation_id,
                        target.terminal_view_id,
                        terminal_id.clone(),
                        trace.clone(),
                        ctx,
                    );
                });
            }
            AcpEvent::AvailableCommandsUpdated { .. }
            | AcpEvent::CurrentModeUpdated { .. }
            | AcpEvent::ConfigOptionsUpdated { .. } => {}
            AcpEvent::SessionInfoUpdated { update } => {
                if let Some(title) = update.title.value().filter(|title| !title.is_empty()) {
                    log::info!(
                        "ACP: updating conversation title bytes={} conversation={:?}",
                        title.len(),
                        target.conversation_id,
                    );
                    BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                        history.set_acp_conversation_title(
                            target.conversation_id,
                            target.terminal_view_id,
                            title.to_string(),
                            ctx,
                        );
                    });
                }
            }
            AcpEvent::PlanUpdated { plan } => {
                if plan.entries.is_empty() {
                    return;
                }
                log::info!(
                    "ACP: setting plan entries={} stream={:?} conversation={:?}",
                    plan.entries.len(),
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let history_target = target.response_stream_target();
                    history.set_acp_plan_for_response_stream(
                        &history_target,
                        AcpPlan { plan: plan.clone() },
                        ctx,
                    );
                });
            }
            AcpEvent::PermissionRequested { request } => {
                log::info!(
                    "ACP: upserting permission request id={} options={} stream={:?} conversation={:?}",
                    request.request_id,
                    request.options.len(),
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let history_target = target.response_stream_target();
                    history.update_acp_tool_call_to_response_stream(
                        &history_target,
                        request.tool_call_update.clone(),
                        ctx,
                    );
                    history.upsert_acp_permission_to_response_stream(
                        &history_target,
                        request.clone(),
                        ctx,
                    );
                });
            }
            AcpEvent::Completed => {
                log::info!(
                    "ACP: completing AI history output stream={:?} conversation={:?}",
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    history.mark_response_stream_completed_successfully(
                        &target.response_stream_id,
                        target.conversation_id,
                        target.terminal_view_id,
                        ctx,
                    );
                    if let Some(conversation) = history.conversation_mut(&target.conversation_id) {
                        conversation.cleanup_completed_response_stream(&target.response_stream_id);
                    }
                });
            }
            AcpEvent::Cancelled => {
                log::info!(
                    "ACP: cancelling AI history output stream={:?} conversation={:?}",
                    target.response_stream_id,
                    target.conversation_id,
                );
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    history.mark_response_stream_cancelled(
                        &target.response_stream_id,
                        target.conversation_id,
                        target.terminal_view_id,
                        CancellationReason::ManuallyCancelled,
                        ctx,
                    );
                    if let Some(conversation) = history.conversation_mut(&target.conversation_id) {
                        conversation.cleanup_completed_response_stream(&target.response_stream_id);
                    }
                });
            }
            AcpEvent::AdapterMissing {
                command,
                install_command,
            } => {
                self.fail_history_target(
                    target,
                    format!("{command} is not installed. Install it with `{install_command}`."),
                    ctx,
                );
            }
            AcpEvent::Failed { message } => {
                log::warn!(
                    "ACP: failing AI history output stream={:?} conversation={:?}: {}",
                    target.response_stream_id,
                    target.conversation_id,
                    message,
                );
                self.fail_history_target(target, message.clone(), ctx);
            }
        }
    }

    fn fail_history_target(
        &self,
        target: &AcpRunTarget,
        message: String,
        ctx: &mut ModelContext<Self>,
    ) {
        BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
            history.initialize_local_output_for_response_stream(
                &target.response_stream_id,
                target.conversation_id,
                target.terminal_view_id,
                target.model_id.clone(),
                target.display_name.clone(),
                ctx,
            );
            history.mark_response_stream_completed_with_error(
                RenderableAIError::Other {
                    error_message: message,
                    will_attempt_resume: false,
                    waiting_for_network: false,
                },
                &target.response_stream_id,
                target.conversation_id,
                target.terminal_view_id,
                ctx,
            );
            if let Some(conversation) = history.conversation_mut(&target.conversation_id) {
                conversation.cleanup_completed_response_stream(&target.response_stream_id);
            }
        });
    }
}

#[allow(dead_code)]
async fn run_one_prompt(
    backend: AcpAgentBackend,
    default_config_options: HashMap<String, String>,
    display_prompt: String,
    content_blocks: Vec<ContentBlock>,
    cwd: PathBuf,
    events: UnboundedSender<AcpRuntimeEvent>,
    mut cancel_rx: mpsc::UnboundedReceiver<()>,
) -> anyhow::Result<()> {
    log::info!(
        "ACP: starting adapter backend={} command={} cwd={} prompt_bytes={} default_config_options={}",
        backend.display_name(),
        backend.adapter_command(),
        cwd.display(),
        display_prompt.len(),
        default_config_options.len(),
    );
    let agent = AcpAgent::from_args([backend.adapter_command()])?;
    log::info!(
        "ACP: adapter process configured command={}",
        backend.adapter_command(),
    );
    if events
        .unbounded_send(AcpRuntimeEvent::Event(AcpEvent::SessionStarted))
        .is_err()
    {
        log::warn!("ACP: failed to publish session start event");
    }
    let notifications = events.clone();
    let permissions = events.clone();
    let terminal_manager = AcpTerminalManager::default();
    let create_terminal_manager = terminal_manager.clone();
    let create_terminal_events = events.clone();
    let terminal_output_manager = terminal_manager.clone();
    let terminal_output_events = events.clone();
    let wait_terminal_manager = terminal_manager.clone();
    let wait_terminal_events = events.clone();
    let kill_terminal_manager = terminal_manager.clone();
    let kill_terminal_events = events.clone();
    let release_terminal_manager = terminal_manager.clone();

    let was_cancelled = Client
        .builder()
        .on_receive_notification(
            async move |notification: SessionNotification, _connection| {
                let session_id = notification.session_id.0.clone();
                let update_kind = session_update_label(&notification.update);
                if let Some(event) = map_session_update(notification.update) {
                    let event_summary = acp_event_summary(&event);
                    log::info!(
                        "ACP: received session update session_id={} update={} mapped_event={event_summary}",
                        session_id,
                        update_kind,
                    );
                    if notifications
                        .unbounded_send(AcpRuntimeEvent::Event(event))
                        .is_err()
                    {
                        log::warn!(
                            "ACP: failed to publish mapped event for session_id={}",
                            session_id,
                        );
                    }
                } else {
                    log::info!(
                        "ACP: ignored session update session_id={} update={}",
                        session_id,
                        update_kind,
                    );
                }
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |request: RequestPermissionRequest, responder, _connection| {
                let session_id = request.session_id.0.clone();
                let request_id = request.tool_call.tool_call_id.0.to_string();
                let local_request = AcpPermissionRequest::from_acp(request.clone());
                log::info!(
                    "ACP: permission request session_id={} request_id={} options={}",
                    session_id,
                    request_id,
                    local_request.options.len(),
                );

                if request.options.is_empty() {
                    log::warn!(
                        "ACP: cancelling permission request with no options session_id={}",
                        session_id,
                    );
                    return responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ));
                }

                let (selection_tx, selection_rx) = oneshot::channel();
                if permissions
                    .unbounded_send(AcpRuntimeEvent::PermissionRequested {
                        request: local_request,
                        response: selection_tx,
                    })
                    .is_err()
                {
                    log::warn!(
                        "ACP: failed to publish permission request for session_id={}",
                        session_id,
                    );
                    return responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ));
                }

                match selection_rx.await.unwrap_or(AcpPermissionSelection::Cancelled) {
                    AcpPermissionSelection::Selected { option_id } => {
                    log::info!(
                        "ACP: selecting permission option session_id={} option_id={}",
                        session_id,
                        option_id,
                    );
                    responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                            option_id,
                        )),
                    ))
                    }
                    AcpPermissionSelection::Cancelled => {
                        log::info!(
                            "ACP: cancelling permission request session_id={} request_id={}",
                            session_id,
                            request_id,
                        );
                        responder.respond(RequestPermissionResponse::new(
                            RequestPermissionOutcome::Cancelled,
                        ))
                    }
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: CreateTerminalRequest, responder, _connection| {
                match create_terminal_manager.create_terminal(request).await {
                    Ok(response) => {
                        publish_terminal_trace(
                            &create_terminal_events,
                            &create_terminal_manager,
                            &response.terminal_id,
                        )
                        .await;
                        responder.respond(response)
                    }
                    Err(err) => responder.respond_with_error(
                        agent_client_protocol::util::internal_error(err.to_string()),
                    ),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: TerminalOutputRequest, responder, _connection| {
                let terminal_id = request.terminal_id.clone();
                match terminal_output_manager.terminal_output(request).await {
                    Ok(response) => {
                        publish_terminal_trace(
                            &terminal_output_events,
                            &terminal_output_manager,
                            &terminal_id,
                        )
                        .await;
                        responder.respond(response)
                    }
                    Err(err) => responder.respond_with_error(
                        agent_client_protocol::util::internal_error(err.to_string()),
                    ),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: WaitForTerminalExitRequest, responder, _connection| {
                let terminal_id = request.terminal_id.clone();
                match wait_terminal_manager.wait_for_terminal_exit(request).await {
                    Ok(response) => {
                        publish_terminal_trace(
                            &wait_terminal_events,
                            &wait_terminal_manager,
                            &terminal_id,
                        )
                        .await;
                        responder.respond(response)
                    }
                    Err(err) => responder.respond_with_error(
                        agent_client_protocol::util::internal_error(err.to_string()),
                    ),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: KillTerminalRequest, responder, _connection| {
                let terminal_id = request.terminal_id.clone();
                match kill_terminal_manager.kill_terminal(request).await {
                    Ok(response) => {
                        publish_terminal_trace(
                            &kill_terminal_events,
                            &kill_terminal_manager,
                            &terminal_id,
                        )
                        .await;
                        responder.respond(response)
                    }
                    Err(err) => responder.respond_with_error(
                        agent_client_protocol::util::internal_error(err.to_string()),
                    ),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: ReleaseTerminalRequest, responder, _connection| {
                match release_terminal_manager.release_terminal(request).await {
                    Ok(response) => responder.respond(response),
                    Err(err) => responder.respond_with_error(
                        agent_client_protocol::util::internal_error(err.to_string()),
                    ),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: ReadTextFileRequest, responder, _connection| {
                match read_text_file(request).await {
                    Ok(response) => responder.respond(response),
                    Err(err) => responder.respond_with_error(
                        agent_client_protocol::util::internal_error(err.to_string()),
                    ),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: WriteTextFileRequest, responder, _connection| {
                match write_text_file(request).await {
                    Ok(response) => responder.respond(response),
                    Err(err) => responder.respond_with_error(
                        agent_client_protocol::util::internal_error(err.to_string()),
                    ),
                }
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(agent, |connection: ConnectionTo<Agent>| async move {
            log::info!("ACP: sending initialize request");
            connection
                .send_request(
                    InitializeRequest::new(ProtocolVersion::V1)
                        .client_capabilities(acp_client_capabilities()),
                )
                .block_task()
                .await?;
            log::info!("ACP: initialize completed");

            log::info!("ACP: sending new_session request cwd={}", cwd.display());
            let session = connection
                .send_request(NewSessionRequest::new(cwd))
                .block_task()
                .await?;
            log::info!(
                "ACP: new_session completed session_id={} config_options={}",
                session.session_id.0,
                session.config_options.as_ref().map_or(0, Vec::len),
            );

            if let Some(config_options) = session.config_options.as_ref() {
                for (config_id, value) in
                    default_config_options_to_apply(config_options, &default_config_options)
                {
                    log::info!(
                        "ACP: applying session config session_id={} config_id={} value={}",
                        session.session_id.0,
                        config_id.0,
                        value.0,
                    );
                    connection
                        .send_request(SetSessionConfigOptionRequest::new(
                            session.session_id.clone(),
                            config_id,
                            value,
                        ))
                        .block_task()
                        .await?;
                }
            }

            let session_id = session.session_id.clone();
            log::info!(
                "ACP: sending prompt request session_id={} prompt_bytes={}",
                session_id.0,
                display_prompt.len(),
            );
            let prompt_task = connection
                .send_request(PromptRequest::new(session_id.clone(), content_blocks))
                .block_task();
            tokio::pin!(prompt_task);
            let prompt_response = tokio::select! {
                result = &mut prompt_task => result?,
                cancel = cancel_rx.next() => {
                    if cancel.is_some() {
                        log::info!(
                            "ACP: sending cancel notification session_id={}",
                            session_id.0,
                        );
                        connection.send_notification(CancelNotification::new(session_id.clone()))?;
                    }
                    prompt_task.await?
                }
            };
            log::info!(
                "ACP: prompt request completed session_id={} stop_reason={:?}",
                session_id.0,
                prompt_response.stop_reason,
            );

            Ok(prompt_response.stop_reason == StopReason::Cancelled)
        })
        .await?;

    let event = if was_cancelled {
        AcpEvent::Cancelled
    } else {
        AcpEvent::Completed
    };
    if events
        .unbounded_send(AcpRuntimeEvent::Event(event))
        .is_err()
    {
        log::warn!("ACP: failed to publish terminal event");
    }
    log::info!("ACP: connection completed");
    Ok(())
}

pub(super) fn default_config_options_to_apply(
    config_options: &[SessionConfigOption],
    defaults: &HashMap<String, String>,
) -> Vec<(SessionConfigId, SessionConfigValueId)> {
    config_options
        .iter()
        .filter_map(|config_option| {
            let default_value = defaults.get(&config_option.id.0.to_string())?;
            if !select_option_contains_value(config_option, default_value) {
                return None;
            }
            Some((
                config_option.id.clone(),
                SessionConfigValueId::new(default_value.clone()),
            ))
        })
        .collect()
}

fn select_option_contains_value(config_option: &SessionConfigOption, value: &str) -> bool {
    match &config_option.kind {
        SessionConfigKind::Select(select) => match &select.options {
            SessionConfigSelectOptions::Ungrouped(options) => options
                .iter()
                .any(|option| option.value.0.as_ref() == value),
            SessionConfigSelectOptions::Grouped(groups) => groups.iter().any(|group| {
                group
                    .options
                    .iter()
                    .any(|option| option.value.0.as_ref() == value)
            }),
            _ => false,
        },
        _ => false,
    }
}

fn target_summary(target: Option<&AcpRunTarget>) -> String {
    target.map_or_else(
        || "none".to_owned(),
        |target| {
            format!(
                "conversation={:?} stream={:?} terminal={:?}",
                target.conversation_id, target.response_stream_id, target.terminal_view_id,
            )
        },
    )
}

fn acp_event_summary(event: &AcpEvent) -> String {
    match event {
        AcpEvent::AdapterMissing { command, .. } => {
            format!("adapter_missing command={command}")
        }
        AcpEvent::SessionStarted => "session_started".to_owned(),
        AcpEvent::UserTextDelta { text } => {
            format!("user_text_delta bytes={}", text.len())
        }
        AcpEvent::AssistantTextDelta { text } => {
            format!("assistant_text_delta bytes={}", text.len())
        }
        AcpEvent::AssistantThoughtDelta { text } => {
            format!("assistant_thought_delta bytes={}", text.len())
        }
        AcpEvent::ToolCallStarted { tool_call } => {
            format!("tool_call_started id={}", tool_call.id)
        }
        AcpEvent::ToolCallUpdated { update } => {
            format!("tool_call_updated id={}", update.tool_call_id.0)
        }
        AcpEvent::TerminalUpdated { terminal_id, trace } => {
            format!(
                "terminal_updated id={} output_bytes={}",
                terminal_id,
                trace.output.len()
            )
        }
        AcpEvent::PlanUpdated { plan } => {
            format!("plan_updated entries={}", plan.entries.len())
        }
        AcpEvent::AvailableCommandsUpdated { commands } => {
            format!("available_commands_updated count={}", commands.len())
        }
        AcpEvent::CurrentModeUpdated { update } => {
            format!("current_mode_updated mode={}", update.current_mode_id.0)
        }
        AcpEvent::ConfigOptionsUpdated { update } => {
            format!(
                "config_options_updated count={}",
                update.config_options.len()
            )
        }
        AcpEvent::SessionInfoUpdated { .. } => "session_info_updated".to_owned(),
        AcpEvent::PermissionRequested { request } => format!(
            "permission_requested request_id={} options={}",
            request.request_id,
            request.options.len()
        ),
        AcpEvent::Cancelled => "cancelled".to_owned(),
        AcpEvent::Completed => "completed".to_owned(),
        AcpEvent::Failed { message } => format!("failed message={message}"),
    }
}

impl Entity for AcpAgentModel {
    type Event = AcpEvent;
}

impl SingletonEntity for AcpAgentModel {}

fn acp_client_capabilities() -> ClientCapabilities {
    ClientCapabilities::new()
        .terminal(true)
        .fs(FileSystemCapabilities::new()
            .read_text_file(true)
            .write_text_file(true))
}

#[derive(Clone, Default)]
struct AcpTerminalManager {
    terminals: Arc<Mutex<HashMap<String, AcpTerminalHandle>>>,
}

struct AcpTerminalHandle {
    child: Mutex<Child>,
    output: Arc<Mutex<AcpTerminalOutput>>,
    read_tasks: Mutex<Vec<JoinHandle<()>>>,
    command: String,
    cwd: Option<String>,
    exit_status: Mutex<Option<TerminalExitStatus>>,
}

#[derive(Default)]
struct AcpTerminalOutput {
    output: String,
    truncated: bool,
    limit: Option<usize>,
}

impl AcpTerminalManager {
    async fn create_terminal(
        &self,
        request: CreateTerminalRequest,
    ) -> anyhow::Result<CreateTerminalResponse> {
        let mut command = Command::new(&request.command);
        command
            .args(&request.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(cwd) = &request.cwd {
            command.current_dir(cwd);
        }
        for env in &request.env {
            command.env(&env.name, &env.value);
        }

        let mut child = command
            .spawn()
            .with_context(|| format!("failed to start terminal command {}", request.command))?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let terminal_id = TerminalId::new(uuid::Uuid::new_v4().to_string());
        let output = Arc::new(Mutex::new(AcpTerminalOutput {
            output: String::new(),
            truncated: false,
            limit: request
                .output_byte_limit
                .map(|limit| usize::try_from(limit).unwrap_or(usize::MAX)),
        }));
        let mut read_tasks = Vec::new();
        if let Some(stdout) = stdout {
            read_tasks.push(tokio::spawn(read_terminal_stream(
                stdout,
                Arc::clone(&output),
            )));
        }
        if let Some(stderr) = stderr {
            read_tasks.push(tokio::spawn(read_terminal_stream(
                stderr,
                Arc::clone(&output),
            )));
        }

        let command_line = terminal_command_line(&request.command, &request.args);
        let handle = AcpTerminalHandle {
            child: Mutex::new(child),
            output,
            read_tasks: Mutex::new(read_tasks),
            command: command_line,
            cwd: request.cwd.map(|cwd| cwd.display().to_string()),
            exit_status: Mutex::new(None),
        };
        self.terminals
            .lock()
            .await
            .insert(terminal_id.0.to_string(), handle);

        Ok(CreateTerminalResponse::new(terminal_id))
    }

    async fn terminal_output(
        &self,
        request: TerminalOutputRequest,
    ) -> anyhow::Result<TerminalOutputResponse> {
        let terminal_id = request.terminal_id.0.to_string();
        self.refresh_terminal_exit_status(&terminal_id).await?;
        let terminals = self.terminals.lock().await;
        let handle = terminals
            .get(&terminal_id)
            .ok_or_else(|| anyhow!("unknown terminal {}", terminal_id))?;
        let output = handle.output.lock().await;
        let exit_status = handle.exit_status.lock().await.clone();

        Ok(
            TerminalOutputResponse::new(output.output.clone(), output.truncated)
                .exit_status(exit_status),
        )
    }

    async fn wait_for_terminal_exit(
        &self,
        request: WaitForTerminalExitRequest,
    ) -> anyhow::Result<WaitForTerminalExitResponse> {
        let terminal_id = request.terminal_id.0.to_string();
        let terminals = self.terminals.lock().await;
        let handle = terminals
            .get(&terminal_id)
            .ok_or_else(|| anyhow!("unknown terminal {}", terminal_id))?;

        let status = {
            let mut child = handle.child.lock().await;
            terminal_exit_status(child.wait().await?)
        };
        *handle.exit_status.lock().await = Some(status.clone());
        let read_tasks = std::mem::take(&mut *handle.read_tasks.lock().await);
        for task in read_tasks {
            let _ = task.await;
        }

        Ok(WaitForTerminalExitResponse::new(status))
    }

    async fn kill_terminal(
        &self,
        request: KillTerminalRequest,
    ) -> anyhow::Result<KillTerminalResponse> {
        let terminal_id = request.terminal_id.0.to_string();
        let terminals = self.terminals.lock().await;
        let handle = terminals
            .get(&terminal_id)
            .ok_or_else(|| anyhow!("unknown terminal {}", terminal_id))?;
        handle.child.lock().await.kill().await?;

        Ok(KillTerminalResponse::new())
    }

    async fn release_terminal(
        &self,
        request: ReleaseTerminalRequest,
    ) -> anyhow::Result<ReleaseTerminalResponse> {
        let terminal_id = request.terminal_id.0.to_string();
        let handle = self
            .terminals
            .lock()
            .await
            .remove(&terminal_id)
            .ok_or_else(|| anyhow!("unknown terminal {}", terminal_id))?;
        if handle.exit_status.lock().await.is_none() {
            let _ = handle.child.lock().await.kill().await;
        }

        Ok(ReleaseTerminalResponse::new())
    }

    async fn terminal_trace(&self, terminal_id: &TerminalId) -> Option<AcpTerminalTrace> {
        let terminal_id = terminal_id.0.to_string();
        let _ = self.refresh_terminal_exit_status(&terminal_id).await;
        let terminals = self.terminals.lock().await;
        let handle = terminals.get(&terminal_id)?;
        let output = handle.output.lock().await;
        let exit_status = handle.exit_status.lock().await.clone();
        Some(AcpTerminalTrace {
            command: Some(handle.command.clone()),
            cwd: handle.cwd.clone(),
            output: output.output.clone(),
            exit_code: exit_status.and_then(|status| status.exit_code.map(i64::from)),
        })
    }

    async fn refresh_terminal_exit_status(&self, terminal_id: &str) -> anyhow::Result<()> {
        let terminals = self.terminals.lock().await;
        let handle = terminals
            .get(terminal_id)
            .ok_or_else(|| anyhow!("unknown terminal {}", terminal_id))?;
        if handle.exit_status.lock().await.is_some() {
            return Ok(());
        }
        let maybe_status = handle.child.lock().await.try_wait()?;
        if let Some(status) = maybe_status {
            *handle.exit_status.lock().await = Some(terminal_exit_status(status));
        }
        Ok(())
    }
}

async fn read_terminal_stream<R>(mut stream: R, output: Arc<Mutex<AcpTerminalOutput>>)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(len) => {
                let text = String::from_utf8_lossy(&buffer[..len]);
                let mut output = output.lock().await;
                output.output.push_str(&text);
                apply_terminal_output_limit(&mut output);
            }
            Err(_) => break,
        }
    }
}

fn apply_terminal_output_limit(output: &mut AcpTerminalOutput) {
    let Some(limit) = output.limit else {
        return;
    };
    if output.output.len() <= limit {
        return;
    }

    let mut start = output.output.len() - limit;
    while start < output.output.len() && !output.output.is_char_boundary(start) {
        start += 1;
    }
    output.output.drain(..start);
    output.truncated = true;
}

fn terminal_exit_status(status: ExitStatus) -> TerminalExitStatus {
    TerminalExitStatus::new().exit_code(status.code().and_then(|code| u32::try_from(code).ok()))
}

fn terminal_command_line(command: &str, args: &[String]) -> String {
    let mut command_line = command.to_string();
    for arg in args {
        command_line.push(' ');
        command_line.push_str(arg);
    }
    command_line
}

async fn read_text_file(request: ReadTextFileRequest) -> anyhow::Result<ReadTextFileResponse> {
    let content = tokio::fs::read_to_string(&request.path)
        .await
        .with_context(|| format!("failed to read {}", request.path.display()))?;
    if request.line.is_none() && request.limit.is_none() {
        return Ok(ReadTextFileResponse::new(content));
    }

    let start = request.line.unwrap_or(1).saturating_sub(1) as usize;
    let limit = request.limit.map_or(usize::MAX, |limit| limit as usize);
    let content = content
        .split_inclusive('\n')
        .skip(start)
        .take(limit)
        .collect::<String>();

    Ok(ReadTextFileResponse::new(content))
}

async fn write_text_file(request: WriteTextFileRequest) -> anyhow::Result<WriteTextFileResponse> {
    tokio::fs::write(&request.path, request.content)
        .await
        .with_context(|| format!("failed to write {}", request.path.display()))?;
    Ok(WriteTextFileResponse::new())
}

async fn publish_terminal_trace(
    events: &UnboundedSender<AcpRuntimeEvent>,
    manager: &AcpTerminalManager,
    terminal_id: &TerminalId,
) {
    if let Some(trace) = manager.terminal_trace(terminal_id).await {
        let _ = events.unbounded_send(AcpRuntimeEvent::Event(AcpEvent::TerminalUpdated {
            terminal_id: terminal_id.0.to_string(),
            trace,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn test_select_permission_option_resolves_pending_request() {
        let (tx, rx) = oneshot::channel();
        let mut model = AcpAgentModel {
            state: AcpAgentState::Idle,
            events: Vec::new(),
            session_state_by_conversation: HashMap::new(),
            pending_permission_responses: HashMap::from([("request-1".to_string(), tx)]),
            pending_session_cancels: HashMap::new(),
            allow_adapter_execution: false,
        };

        assert!(model.select_permission_option("request-1", "allow-once".to_string()));
        assert!(!model.select_permission_option("request-1", "allow-once".to_string()));

        match block_on(rx).unwrap() {
            AcpPermissionSelection::Selected { option_id } => {
                assert_eq!(option_id, "allow-once");
            }
            AcpPermissionSelection::Cancelled => panic!("expected selected permission option"),
        }
    }

    #[test]
    fn test_cancel_session_sends_pending_cancel_signal() {
        let conversation_id = AIConversationId::new();
        let (tx, mut rx) = mpsc::unbounded();
        let mut model = AcpAgentModel {
            state: AcpAgentState::Idle,
            events: Vec::new(),
            session_state_by_conversation: HashMap::new(),
            pending_permission_responses: HashMap::new(),
            pending_session_cancels: HashMap::from([(conversation_id, tx)]),
            allow_adapter_execution: false,
        };

        assert!(model.cancel_session(conversation_id));
        assert!(!model.cancel_session(conversation_id));
        assert!(block_on(rx.next()).is_some());
    }

    #[test]
    fn test_session_state_tracks_commands_mode_config_and_info() {
        use agent_client_protocol::schema::{
            AvailableCommand, ConfigOptionUpdate, CurrentModeUpdate, SessionConfigOption,
            SessionConfigSelectOption, SessionInfoUpdate,
        };

        let conversation_id = AIConversationId::new();
        let mut model = AcpAgentModel {
            state: AcpAgentState::Idle,
            events: Vec::new(),
            session_state_by_conversation: HashMap::new(),
            pending_permission_responses: HashMap::new(),
            pending_session_cancels: HashMap::new(),
            allow_adapter_execution: false,
        };

        model.apply_session_event_to_state(
            &AcpEvent::AvailableCommandsUpdated {
                commands: vec![AvailableCommand::new("review", "Review changes")],
            },
            conversation_id,
        );
        model.apply_session_event_to_state(
            &AcpEvent::CurrentModeUpdated {
                update: CurrentModeUpdate::new("plan"),
            },
            conversation_id,
        );
        model.apply_session_event_to_state(
            &AcpEvent::ConfigOptionsUpdated {
                update: ConfigOptionUpdate::new(vec![SessionConfigOption::select(
                    "model",
                    "Model",
                    "gpt-5.5",
                    vec![SessionConfigSelectOption::new("gpt-5.5", "GPT-5.5")],
                )]),
            },
            conversation_id,
        );
        model.apply_session_event_to_state(
            &AcpEvent::SessionInfoUpdated {
                update: SessionInfoUpdate::new().title("ACP Session"),
            },
            conversation_id,
        );

        let state = model.session_state(conversation_id).unwrap();
        assert_eq!(state.commands.commands[0].name, "review");
        assert_eq!(
            state
                .config
                .current_mode
                .as_ref()
                .unwrap()
                .current_mode_id
                .0
                .as_ref(),
            "plan"
        );
        assert_eq!(
            state.config.config.as_ref().unwrap().config_options[0]
                .id
                .0
                .as_ref(),
            "model"
        );
        assert_eq!(
            state.info.as_ref().unwrap().info.title.value().unwrap(),
            "ACP Session"
        );
    }

    #[test]
    fn test_acp_client_capabilities_advertise_implemented_terminal_and_fs() {
        let capabilities = acp_client_capabilities();

        assert!(capabilities.terminal);
        assert!(capabilities.fs.read_text_file);
        assert!(capabilities.fs.write_text_file);
    }

    #[tokio::test]
    async fn test_acp_terminal_manager_runs_command_and_reports_output() {
        let manager = AcpTerminalManager::default();
        let response = manager
            .create_terminal(
                agent_client_protocol::schema::CreateTerminalRequest::new("session-1", "/bin/sh")
                    .args(vec!["-c".to_string(), "printf hi".to_string()]),
            )
            .await
            .unwrap();

        let exit = manager
            .wait_for_terminal_exit(
                agent_client_protocol::schema::WaitForTerminalExitRequest::new(
                    "session-1",
                    response.terminal_id.clone(),
                ),
            )
            .await
            .unwrap();
        let output = manager
            .terminal_output(agent_client_protocol::schema::TerminalOutputRequest::new(
                "session-1",
                response.terminal_id,
            ))
            .await
            .unwrap();

        assert_eq!(output.output, "hi");
        assert_eq!(output.exit_status.unwrap().exit_code, Some(0));
        assert_eq!(exit.exit_status.exit_code, Some(0));
    }

    #[tokio::test]
    async fn test_acp_fs_handlers_read_ranges_and_write_files() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("note.txt");

        write_text_file(agent_client_protocol::schema::WriteTextFileRequest::new(
            "session-1",
            path.clone(),
            "one\ntwo\nthree\n",
        ))
        .await
        .unwrap();
        let response = read_text_file(
            agent_client_protocol::schema::ReadTextFileRequest::new("session-1", path)
                .line(2)
                .limit(1),
        )
        .await
        .unwrap();

        assert_eq!(response.content, "two\n");
    }
}
