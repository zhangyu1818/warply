use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;

use agent_client_protocol::schema::{
    CancelNotification, ClientCapabilities, ContentBlock, CreateTerminalRequest,
    CreateTerminalResponse, EmbeddedResource, EmbeddedResourceResource, FileSystemCapabilities,
    InitializeRequest, KillTerminalRequest, KillTerminalResponse, NewSessionRequest,
    PromptCapabilities, PromptRequest, ProtocolVersion, ReadTextFileRequest, ReadTextFileResponse,
    ReleaseTerminalRequest, ReleaseTerminalResponse, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, ResourceLink, SelectedPermissionOutcome,
    SessionConfigId, SessionConfigKind, SessionConfigOption, SessionConfigSelectOptions,
    SessionConfigValueId, SessionNotification, SetSessionConfigOptionRequest, StopReason,
    TerminalExitStatus, TerminalId, TerminalOutputRequest, TerminalOutputResponse,
    WaitForTerminalExitRequest, WaitForTerminalExitResponse, WriteTextFileRequest,
    WriteTextFileResponse,
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
use crate::settings::AISettings;
use crate::terminal::local_shell::LocalShellState;

use super::backend::{adapter_args, adapter_is_available};
use super::events::AcpEvent;
use super::mapping::{map_session_update, session_update_label};
use super::registry::{AcpAgentLaunch, AcpRegistryModel};
use super::{
    AcpCommands, AcpPermissionRequest, AcpPermissionSelection, AcpPlan, AcpSessionInfo,
    AcpSessionState, AcpTerminalTrace,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcpAgentState {
    Idle,
    Starting,
    Ready,
    Running,
    Failed(String),
}

pub struct AcpAgentModel {
    state: AcpAgentState,
    session_state_by_conversation: HashMap<AIConversationId, AcpSessionState>,
    conversation_sessions: HashMap<AIConversationId, AcpConversationSessionHandle>,
    pending_permission_responses: HashMap<String, oneshot::Sender<AcpPermissionSelection>>,
    pending_session_cancels: HashMap<AIConversationId, UnboundedSender<()>>,
    next_runtime_id: u64,
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
    Event {
        event: AcpEvent,
        target: AcpRunTarget,
    },
    PermissionRequested {
        request: AcpPermissionRequest,
        response: oneshot::Sender<AcpPermissionSelection>,
        target: AcpRunTarget,
    },
}

struct AcpConversationSessionHandle {
    runtime_id: u64,
    backend_id: String,
    default_config_options: HashMap<String, String>,
    prompt_tx: UnboundedSender<AcpPromptCommand>,
}

struct AcpPromptCommand {
    display_prompt: String,
    content_blocks: Vec<ContentBlock>,
    cwd: PathBuf,
    target: AcpRunTarget,
    cancel_rx: mpsc::UnboundedReceiver<()>,
}

impl AcpAgentModel {
    pub fn new(_: &mut ModelContext<Self>) -> Self {
        Self {
            state: AcpAgentState::Idle,
            session_state_by_conversation: HashMap::new(),
            conversation_sessions: HashMap::new(),
            pending_permission_responses: HashMap::new(),
            pending_session_cancels: HashMap::new(),
            next_runtime_id: 0,
            allow_adapter_execution: true,
        }
    }

    #[cfg(test)]
    pub fn new_for_test(_: &mut ModelContext<Self>) -> Self {
        Self {
            state: AcpAgentState::Idle,
            session_state_by_conversation: HashMap::new(),
            conversation_sessions: HashMap::new(),
            pending_permission_responses: HashMap::new(),
            pending_session_cancels: HashMap::new(),
            next_runtime_id: 0,
            allow_adapter_execution: false,
        }
    }

    #[cfg(test)]
    pub fn state(&self) -> AcpAgentState {
        self.state.clone()
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

    pub fn cancel_session(&mut self, conversation_id: AIConversationId) -> bool {
        let Some(cancel) = self.pending_session_cancels.remove(&conversation_id) else {
            log::warn!("ACP: cancel requested for unknown conversation={conversation_id:?}");
            return false;
        };

        log::info!("ACP: cancel requested conversation={conversation_id:?}");
        cancel.unbounded_send(()).is_ok()
    }

    pub(crate) fn submit_prompt_for_run_target(
        &mut self,
        display_prompt: String,
        content_blocks: Vec<ContentBlock>,
        cwd: PathBuf,
        target: AcpRunTarget,
        ctx: &mut ModelContext<Self>,
    ) {
        self.submit_prompt_internal(display_prompt, content_blocks, cwd, target, ctx);
    }

    fn submit_prompt_internal(
        &mut self,
        display_prompt: String,
        content_blocks: Vec<ContentBlock>,
        cwd: PathBuf,
        target: AcpRunTarget,
        ctx: &mut ModelContext<Self>,
    ) {
        let settings = AISettings::as_ref(ctx);
        let backend_id = settings.acp_agent_backend.to_string();
        let default_config_options = settings.acp_default_config_options.clone();
        let Some(launch) = AcpRegistryModel::as_ref(ctx)
            .registry()
            .launch_for_agent(&backend_id)
        else {
            self.handle_event(
                AcpEvent::Failed {
                    message: format!("No ACP registry launch configuration for {backend_id}"),
                },
                &target,
                ctx,
            );
            return;
        };
        let command = launch.command_line.join(" ");
        log::info!(
            "ACP: submit requested backend={} command={} cwd={} prompt_bytes={} target={} default_config_options={}",
            launch.display_name,
            command,
            cwd.display(),
            display_prompt.len(),
            target_summary(&target),
            default_config_options.len(),
        );
        if !self.allow_adapter_execution {
            log::info!("ACP: adapter execution disabled; simulating prompt start");
            self.handle_event(AcpEvent::PromptStarted, &target, ctx);
            return;
        }

        let adapter_path_env = LocalShellState::handle(ctx).update(ctx, |shell_state, ctx| {
            shell_state.get_interactive_path_env_var(ctx)
        });

        self.state = AcpAgentState::Starting;
        let (cancel_tx, cancel_rx) = futures::channel::mpsc::unbounded();
        self.pending_session_cancels
            .insert(target.conversation_id, cancel_tx);
        let prompt = AcpPromptCommand {
            display_prompt,
            content_blocks,
            cwd,
            target: target.clone(),
            cancel_rx,
        };

        if self
            .conversation_sessions
            .get(&target.conversation_id)
            .is_some_and(|session| {
                session.backend_id == backend_id
                    && session.default_config_options == default_config_options
            })
        {
            let session = self
                .conversation_sessions
                .get(&target.conversation_id)
                .expect("checked conversation session exists");
            let runtime_id = session.runtime_id;
            let prompt_tx = session.prompt_tx.clone();
            log::info!(
                "ACP: reusing adapter session runtime_id={} target={}",
                runtime_id,
                target_summary(&target),
            );
            if prompt_tx.unbounded_send(prompt).is_err() {
                self.conversation_sessions.remove(&target.conversation_id);
                self.handle_event(
                    AcpEvent::Failed {
                        message: "ACP adapter session is no longer available".to_string(),
                    },
                    &target,
                    ctx,
                );
            }
            return;
        }
        self.conversation_sessions.remove(&target.conversation_id);

        let (events_tx, events_rx) = futures::channel::mpsc::unbounded();
        ctx.spawn_stream_local(
            events_rx,
            move |me, event, ctx| me.handle_runtime_event(event, ctx),
            |_, _| {},
        );

        let (prompt_tx, prompt_rx) = futures::channel::mpsc::unbounded();
        let runtime_id = self.next_runtime_id;
        self.next_runtime_id = self.next_runtime_id.wrapping_add(1);
        self.conversation_sessions.insert(
            target.conversation_id,
            AcpConversationSessionHandle {
                runtime_id,
                backend_id,
                default_config_options: default_config_options.clone(),
                prompt_tx: prompt_tx.clone(),
            },
        );
        let completion_target = target.clone();
        let completion_conversation_id = target.conversation_id;
        log::info!(
            "ACP: spawning adapter task runtime_id={} backend={} command={}",
            runtime_id,
            launch.display_name,
            command,
        );
        ctx.spawn(
            run_conversation_session(
                launch,
                default_config_options,
                prompt_rx,
                events_tx,
                adapter_path_env,
            ),
            move |me, result, ctx| {
                match result {
                    Ok(()) => {
                        log::info!(
                            "ACP: adapter task completed successfully runtime_id={runtime_id}"
                        );
                    }
                    Err(err) => {
                        log::error!("ACP: adapter task failed runtime_id={runtime_id}: {err:#}");
                        me.handle_event(
                            AcpEvent::Failed {
                                message: err.to_string(),
                            },
                            &completion_target,
                            ctx,
                        );
                    }
                }

                if me
                    .conversation_sessions
                    .get(&completion_conversation_id)
                    .is_some_and(|session| session.runtime_id == runtime_id)
                {
                    me.conversation_sessions.remove(&completion_conversation_id);
                }
            },
        );

        if prompt_tx.unbounded_send(prompt).is_err() {
            self.conversation_sessions.remove(&target.conversation_id);
            self.handle_event(
                AcpEvent::Failed {
                    message: "ACP adapter session did not start".to_string(),
                },
                &target,
                ctx,
            );
        }
    }

    fn handle_event(
        &mut self,
        event: AcpEvent,
        target: &AcpRunTarget,
        ctx: &mut ModelContext<Self>,
    ) {
        let event_summary = acp_event_summary(&event);
        self.state = match &event {
            AcpEvent::AdapterMissing { command, .. } => {
                AcpAgentState::Failed(format!("{command} is not installed"))
            }
            AcpEvent::SessionStarted | AcpEvent::PromptStarted => AcpAgentState::Starting,
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

        self.apply_session_event_to_state(&event, target.conversation_id);
        self.apply_event_to_history(&event, target, ctx);
        if matches!(
            event,
            AcpEvent::Completed
                | AcpEvent::Cancelled
                | AcpEvent::Failed { .. }
                | AcpEvent::AdapterMissing { .. }
        ) {
            self.pending_session_cancels.remove(&target.conversation_id);
        }

        ctx.emit(event);
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

    fn handle_runtime_event(&mut self, event: AcpRuntimeEvent, ctx: &mut ModelContext<Self>) {
        match event {
            AcpRuntimeEvent::Event { event, target } => self.handle_event(event, &target, ctx),
            AcpRuntimeEvent::PermissionRequested {
                request,
                response,
                target,
            } => {
                self.pending_permission_responses
                    .insert(request.request_id.clone(), response);
                self.handle_event(AcpEvent::PermissionRequested { request }, &target, ctx);
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
            AcpEvent::PromptStarted => {
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
            AcpEvent::SessionStarted => {}
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

async fn run_conversation_session(
    launch: AcpAgentLaunch,
    default_config_options: HashMap<String, String>,
    mut prompt_rx: mpsc::UnboundedReceiver<AcpPromptCommand>,
    events: UnboundedSender<AcpRuntimeEvent>,
    adapter_path_env: impl std::future::Future<Output = Option<String>>,
) -> anyhow::Result<()> {
    let Some(first_prompt) = prompt_rx.next().await else {
        return Ok(());
    };
    let adapter_path_env = adapter_path_env.await;
    let command = launch.command_line.first().cloned().unwrap_or_default();
    let command_line = launch.command_line.join(" ");
    log::info!(
        "ACP: starting adapter backend={} command={} cwd={} prompt_bytes={} default_config_options={} has_path_env={}",
        launch.display_name,
        command_line,
        first_prompt.cwd.display(),
        first_prompt.display_prompt.len(),
        default_config_options.len(),
        adapter_path_env.is_some(),
    );
    if !adapter_is_available(&launch, adapter_path_env.as_deref()) {
        log::warn!(
            "ACP: adapter missing command={} install_command={}",
            command,
            launch.install_command,
        );
        if events
            .unbounded_send(AcpRuntimeEvent::Event {
                event: AcpEvent::AdapterMissing {
                    command,
                    install_command: launch.install_command.clone(),
                },
                target: first_prompt.target,
            })
            .is_err()
        {
            log::warn!("ACP: failed to publish adapter missing event");
        }
        return Ok(());
    }
    let agent = AcpAgent::from_args(adapter_args(&launch, adapter_path_env.as_deref()))?;
    log::info!("ACP: adapter process configured command={}", command_line,);
    let event_target = Arc::new(Mutex::new(first_prompt.target.clone()));
    let active_prompt_target = Arc::new(Mutex::new(Some(first_prompt.target.clone())));
    let notifications = events.clone();
    let notification_target = event_target.clone();
    let permissions = events.clone();
    let permission_target = event_target.clone();
    let terminal_manager = AcpTerminalManager::default();
    let create_terminal_manager = terminal_manager.clone();
    let create_terminal_events = events.clone();
    let create_terminal_target = event_target.clone();
    let terminal_output_manager = terminal_manager.clone();
    let terminal_output_events = events.clone();
    let terminal_output_target = event_target.clone();
    let wait_terminal_manager = terminal_manager.clone();
    let wait_terminal_events = events.clone();
    let wait_terminal_target = event_target.clone();
    let kill_terminal_manager = terminal_manager.clone();
    let kill_terminal_events = events.clone();
    let kill_terminal_target = event_target.clone();
    let release_terminal_manager = terminal_manager.clone();
    let connection_events = events.clone();
    let connection_active_prompt_target = active_prompt_target.clone();

    let connection_result = Client
        .builder()
        .on_receive_notification(
            async move |notification: SessionNotification, _connection| {
                let session_id = notification.session_id.0.clone();
                let update_kind = session_update_label(&notification.update);
                if let Some(event) = map_session_update(notification.update) {
                    let event_summary = acp_event_summary(&event);
                    let target = notification_target.lock().await.clone();
                    log::info!(
                        "ACP: received session update session_id={} update={} mapped_event={event_summary}",
                        session_id,
                        update_kind,
                    );
                    if notifications
                        .unbounded_send(AcpRuntimeEvent::Event { event, target })
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
                let target = permission_target.lock().await.clone();
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
                        target,
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
                        let target = create_terminal_target.lock().await.clone();
                        publish_terminal_trace(
                            &create_terminal_events,
                            &create_terminal_manager,
                            &response.terminal_id,
                            &target,
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
                        let target = terminal_output_target.lock().await.clone();
                        publish_terminal_trace(
                            &terminal_output_events,
                            &terminal_output_manager,
                            &terminal_id,
                            &target,
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
                        let target = wait_terminal_target.lock().await.clone();
                        publish_terminal_trace(
                            &wait_terminal_events,
                            &wait_terminal_manager,
                            &terminal_id,
                            &target,
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
                        let target = kill_terminal_target.lock().await.clone();
                        publish_terminal_trace(
                            &kill_terminal_events,
                            &kill_terminal_manager,
                            &terminal_id,
                            &target,
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
            let initialize = connection
                .send_request(
                    InitializeRequest::new(ProtocolVersion::V1)
                        .client_capabilities(acp_client_capabilities()),
                )
                .block_task()
                .await?;
            let prompt_capabilities = initialize.agent_capabilities.prompt_capabilities;
            log::info!("ACP: initialize completed");

            log::info!(
                "ACP: sending new_session request cwd={}",
                first_prompt.cwd.display()
            );
            let session = connection
                .send_request(NewSessionRequest::new(first_prompt.cwd.clone()))
                .block_task()
                .await?;
            log::info!(
                "ACP: new_session completed session_id={} config_options={}",
                session.session_id.0,
                session.config_options.as_ref().map_or(0, Vec::len),
            );
            publish_runtime_event(&events, &first_prompt.target, AcpEvent::SessionStarted);

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
            let mut next_prompt = Some(first_prompt);
            loop {
                let mut prompt = if let Some(prompt) = next_prompt.take() {
                    prompt
                } else if let Some(prompt) = prompt_rx.next().await {
                    prompt
                } else {
                    break;
                };

                *event_target.lock().await = prompt.target.clone();
                *active_prompt_target.lock().await = Some(prompt.target.clone());
                publish_runtime_event(&events, &prompt.target, AcpEvent::PromptStarted);

                let prompt_bytes = prompt.display_prompt.len();
                let content_blocks = adapt_prompt_content_blocks_for_capabilities(
                    prompt.content_blocks,
                    &prompt_capabilities,
                );
                log::info!(
                    "ACP: sending prompt request session_id={} prompt_bytes={} content_blocks={}",
                    session_id.0,
                    prompt_bytes,
                    content_blocks.len(),
                );
                let prompt_task = connection
                    .send_request(PromptRequest::new(session_id.clone(), content_blocks))
                    .block_task();
                tokio::pin!(prompt_task);
                let prompt_response = tokio::select! {
                    result = &mut prompt_task => result,
                    cancel = prompt.cancel_rx.next() => {
                        if cancel.is_some() {
                            log::info!(
                                "ACP: sending cancel notification session_id={}",
                                session_id.0,
                            );
                            connection.send_notification(CancelNotification::new(session_id.clone()))?;
                        }
                        prompt_task.await
                    }
                };
                let prompt_response = match prompt_response {
                    Ok(response) => response,
                    Err(err) => {
                        *active_prompt_target.lock().await = None;
                        publish_runtime_event(
                            &events,
                            &prompt.target,
                            AcpEvent::Failed {
                                message: err.to_string(),
                            },
                        );
                        return Ok(());
                    }
                };
                log::info!(
                    "ACP: prompt request completed session_id={} stop_reason={:?}",
                    session_id.0,
                    prompt_response.stop_reason,
                );

                let event = if prompt_response.stop_reason == StopReason::Cancelled {
                    AcpEvent::Cancelled
                } else {
                    AcpEvent::Completed
                };
                publish_runtime_event(&events, &prompt.target, event);
                *active_prompt_target.lock().await = None;
            }

            Ok(())
        })
        .await;

    if let Err(err) = connection_result {
        let active_target = connection_active_prompt_target.lock().await.clone();
        if let Some(target) = active_target {
            publish_runtime_event(
                &connection_events,
                &target,
                AcpEvent::Failed {
                    message: err.to_string(),
                },
            );
        } else {
            log::warn!("ACP: adapter connection ended while idle: {err:#}");
        }
        return Ok(());
    }

    log::info!("ACP: connection completed");
    Ok(())
}

fn publish_runtime_event(
    events: &UnboundedSender<AcpRuntimeEvent>,
    target: &AcpRunTarget,
    event: AcpEvent,
) {
    if events
        .unbounded_send(AcpRuntimeEvent::Event {
            event,
            target: target.clone(),
        })
        .is_err()
    {
        log::warn!("ACP: failed to publish runtime event");
    }
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

fn target_summary(target: &AcpRunTarget) -> String {
    format!(
        "conversation={:?} stream={:?} terminal={:?}",
        target.conversation_id, target.response_stream_id, target.terminal_view_id,
    )
}

fn acp_event_summary(event: &AcpEvent) -> String {
    match event {
        AcpEvent::AdapterMissing { command, .. } => {
            format!("adapter_missing command={command}")
        }
        AcpEvent::SessionStarted => "session_started".to_owned(),
        AcpEvent::PromptStarted => "prompt_started".to_owned(),
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

fn adapt_prompt_content_blocks_for_capabilities(
    content_blocks: Vec<ContentBlock>,
    capabilities: &PromptCapabilities,
) -> Vec<ContentBlock> {
    content_blocks
        .into_iter()
        .filter_map(|content_block| match content_block {
            ContentBlock::Image(image) if capabilities.image => Some(ContentBlock::Image(image)),
            ContentBlock::Image(_) => {
                log::warn!("ACP: omitting image prompt content because agent lacks image capability");
                None
            }
            ContentBlock::Audio(audio) if capabilities.audio => Some(ContentBlock::Audio(audio)),
            ContentBlock::Audio(_) => {
                log::warn!("ACP: omitting audio prompt content because agent lacks audio capability");
                None
            }
            ContentBlock::Resource(resource) if capabilities.embedded_context => {
                Some(ContentBlock::Resource(resource))
            }
            ContentBlock::Resource(resource) => embedded_resource_to_link(resource)
                .map(ContentBlock::ResourceLink)
                .or_else(|| {
                    log::warn!(
                        "ACP: omitting embedded resource because agent lacks embeddedContext capability"
                    );
                    None
                }),
            content_block => Some(content_block),
        })
        .collect()
}

fn embedded_resource_to_link(resource: EmbeddedResource) -> Option<ResourceLink> {
    match resource.resource {
        EmbeddedResourceResource::TextResourceContents(resource) => {
            let name = resource_name_from_uri(&resource.uri);
            let size = i64::try_from(resource.text.len()).ok();
            Some(
                ResourceLink::new(name.clone(), resource.uri)
                    .mime_type(resource.mime_type)
                    .size(size)
                    .title(name),
            )
        }
        EmbeddedResourceResource::BlobResourceContents(resource) => {
            let name = resource_name_from_uri(&resource.uri);
            Some(
                ResourceLink::new(name.clone(), resource.uri)
                    .mime_type(resource.mime_type)
                    .title(name),
            )
        }
        _ => None,
    }
}

fn resource_name_from_uri(uri: &str) -> String {
    uri.rsplit('/')
        .find(|part| !part.is_empty())
        .unwrap_or(uri)
        .to_string()
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
    target: &AcpRunTarget,
) {
    if let Some(trace) = manager.terminal_trace(terminal_id).await {
        publish_runtime_event(
            events,
            target,
            AcpEvent::TerminalUpdated {
                terminal_id: terminal_id.0.to_string(),
                trace,
            },
        );
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
            session_state_by_conversation: HashMap::new(),
            conversation_sessions: HashMap::new(),
            pending_permission_responses: HashMap::from([("request-1".to_string(), tx)]),
            pending_session_cancels: HashMap::new(),
            next_runtime_id: 0,
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
            session_state_by_conversation: HashMap::new(),
            conversation_sessions: HashMap::new(),
            pending_permission_responses: HashMap::new(),
            pending_session_cancels: HashMap::from([(conversation_id, tx)]),
            next_runtime_id: 0,
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
            session_state_by_conversation: HashMap::new(),
            conversation_sessions: HashMap::new(),
            pending_permission_responses: HashMap::new(),
            pending_session_cancels: HashMap::new(),
            next_runtime_id: 0,
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

    #[test]
    fn test_acp_prompt_capabilities_downgrade_unsupported_rich_blocks() {
        use agent_client_protocol::schema::{ImageContent, TextContent, TextResourceContents};

        let blocks = vec![
            ContentBlock::Text(TextContent::new("hello")),
            ContentBlock::Image(ImageContent::new("base64-image", "image/png")),
            ContentBlock::Resource(EmbeddedResource::new(
                EmbeddedResourceResource::TextResourceContents(
                    TextResourceContents::new("file body", "file:///tmp/file.txt")
                        .mime_type("text/plain"),
                ),
            )),
        ];

        let adapted =
            adapt_prompt_content_blocks_for_capabilities(blocks, &PromptCapabilities::new());

        assert_eq!(adapted.len(), 2);
        assert!(matches!(&adapted[0], ContentBlock::Text(text) if text.text == "hello"));
        assert!(matches!(
            &adapted[1],
            ContentBlock::ResourceLink(resource)
                if resource.uri == "file:///tmp/file.txt"
                    && resource.name == "file.txt"
                    && resource.mime_type.as_deref() == Some("text/plain")
        ));
    }

    #[test]
    fn test_acp_prompt_capabilities_keep_supported_rich_blocks() {
        use agent_client_protocol::schema::{ImageContent, TextContent, TextResourceContents};

        let blocks = vec![
            ContentBlock::Text(TextContent::new("hello")),
            ContentBlock::Image(ImageContent::new("base64-image", "image/png")),
            ContentBlock::Resource(EmbeddedResource::new(
                EmbeddedResourceResource::TextResourceContents(
                    TextResourceContents::new("file body", "file:///tmp/file.txt")
                        .mime_type("text/plain"),
                ),
            )),
        ];
        let capabilities = PromptCapabilities::new().image(true).embedded_context(true);

        let adapted = adapt_prompt_content_blocks_for_capabilities(blocks, &capabilities);

        assert_eq!(adapted.len(), 3);
        assert!(matches!(&adapted[0], ContentBlock::Text(text) if text.text == "hello"));
        assert!(matches!(
            &adapted[1],
            ContentBlock::Image(image)
                if image.data == "base64-image" && image.mime_type == "image/png"
        ));
        assert!(matches!(&adapted[2], ContentBlock::Resource(_)));
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

    #[tokio::test]
    async fn test_acp_runtime_reuses_session_for_multiple_prompts() {
        use agent_client_protocol::schema::TextContent;
        use serde_json::Value;
        use std::time::Duration;

        let temp = tempfile::TempDir::new().unwrap();
        let log_path = temp.path().join("requests.jsonl");
        let script_path = temp.path().join("fake_acp.py");
        std::fs::write(
            &script_path,
            r#"import json
import os
import sys

session_id = "session-1"
log_path = os.environ["ACP_TEST_LOG"]

def send(message):
    sys.stdout.write(json.dumps(message, separators=(",", ":")) + "\n")
    sys.stdout.flush()

for line in sys.stdin:
    message = json.loads(line)
    method = message.get("method")
    if method in ("initialize", "session/new", "session/prompt"):
        with open(log_path, "a", encoding="utf-8") as log:
            log.write(json.dumps({"method": method, "params": message.get("params")}, separators=(",", ":")) + "\n")
    if "id" not in message:
        continue
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"protocolVersion": 1, "agentCapabilities": {}}})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"sessionId": session_id}})
    elif method == "session/prompt":
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"stopReason": "end_turn"}})
    else:
        send({"jsonrpc": "2.0", "id": message["id"], "result": {}})
"#,
        )
        .unwrap();

        let launch = AcpAgentLaunch {
            agent_id: "fake-agent".to_string(),
            display_name: "Fake Agent".to_string(),
            command_line: vec![
                "/usr/bin/env".to_string(),
                "python3".to_string(),
                script_path.to_string_lossy().to_string(),
            ],
            env: vec![(
                "ACP_TEST_LOG".to_string(),
                log_path.to_string_lossy().to_string(),
            )],
            install_command: "python3".to_string(),
        };
        let (events_tx, _events_rx) = mpsc::unbounded();
        let (prompt_tx, prompt_rx) = mpsc::unbounded();
        let conversation_id = AIConversationId::new();
        let target = |display_name: &str| AcpRunTarget {
            conversation_id,
            response_stream_id: ResponseStreamId::new(),
            terminal_view_id: EntityId::new(),
            model_id: LLMId::from("fake-model"),
            display_name: display_name.to_string(),
        };
        let prompt = |text: &str, target: AcpRunTarget| {
            let (_cancel_tx, cancel_rx) = mpsc::unbounded();
            AcpPromptCommand {
                display_prompt: text.to_string(),
                content_blocks: vec![ContentBlock::Text(TextContent::new(text.to_string()))],
                cwd: temp.path().to_path_buf(),
                target,
                cancel_rx,
            }
        };

        prompt_tx
            .unbounded_send(prompt("first", target("first")))
            .unwrap();
        prompt_tx
            .unbounded_send(prompt("second", target("second")))
            .unwrap();
        drop(prompt_tx);

        tokio::time::timeout(
            Duration::from_secs(5),
            run_conversation_session(
                launch,
                HashMap::new(),
                prompt_rx,
                events_tx,
                std::future::ready(None),
            ),
        )
        .await
        .unwrap()
        .unwrap();

        let requests = std::fs::read_to_string(log_path).unwrap();
        let records = requests
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        let new_session_count = records
            .iter()
            .filter(|record| record["method"] == "session/new")
            .count();
        let prompt_session_ids = records
            .iter()
            .filter(|record| record["method"] == "session/prompt")
            .map(|record| record["params"]["sessionId"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();

        assert_eq!(new_session_count, 1);
        assert_eq!(prompt_session_ids, vec!["session-1", "session-1"]);
    }

    #[tokio::test]
    async fn test_acp_runtime_reports_connection_failure_to_active_prompt() {
        use agent_client_protocol::schema::TextContent;
        use std::time::Duration;

        let temp = tempfile::TempDir::new().unwrap();
        let script_path = temp.path().join("fake_acp_exit.py");
        std::fs::write(
            &script_path,
            r#"import json
import sys

session_id = "session-1"
prompt_count = 0

def send(message):
    sys.stdout.write(json.dumps(message, separators=(",", ":")) + "\n")
    sys.stdout.flush()

for line in sys.stdin:
    message = json.loads(line)
    method = message.get("method")
    if "id" not in message:
        continue
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"protocolVersion": 1, "agentCapabilities": {}}})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"sessionId": session_id}})
    elif method == "session/prompt":
        prompt_count += 1
        if prompt_count > 1:
            sys.stderr.write("adapter failed during prompt\n")
            sys.stderr.flush()
            sys.exit(1)
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"stopReason": "end_turn"}})
    else:
        send({"jsonrpc": "2.0", "id": message["id"], "result": {}})
"#,
        )
        .unwrap();

        let launch = AcpAgentLaunch {
            agent_id: "fake-agent".to_string(),
            display_name: "Fake Agent".to_string(),
            command_line: vec![
                "/usr/bin/env".to_string(),
                "python3".to_string(),
                script_path.to_string_lossy().to_string(),
            ],
            env: vec![],
            install_command: "python3".to_string(),
        };
        let (events_tx, mut events_rx) = mpsc::unbounded();
        let (prompt_tx, prompt_rx) = mpsc::unbounded();
        let conversation_id = AIConversationId::new();
        let target = AcpRunTarget {
            conversation_id,
            response_stream_id: ResponseStreamId::new(),
            terminal_view_id: EntityId::new(),
            model_id: LLMId::from("fake-model"),
            display_name: "Fake Model".to_string(),
        };
        let first_stream_id = target.response_stream_id.clone();
        let second_target = AcpRunTarget {
            conversation_id,
            response_stream_id: ResponseStreamId::new(),
            terminal_view_id: EntityId::new(),
            model_id: LLMId::from("fake-model"),
            display_name: "Fake Model".to_string(),
        };
        let second_stream_id = second_target.response_stream_id.clone();
        let (_cancel_tx, cancel_rx) = mpsc::unbounded();
        prompt_tx
            .unbounded_send(AcpPromptCommand {
                display_prompt: "first".to_string(),
                content_blocks: vec![ContentBlock::Text(TextContent::new("first"))],
                cwd: temp.path().to_path_buf(),
                target,
                cancel_rx,
            })
            .unwrap();
        let (_cancel_tx, cancel_rx) = mpsc::unbounded();
        prompt_tx
            .unbounded_send(AcpPromptCommand {
                display_prompt: "second".to_string(),
                content_blocks: vec![ContentBlock::Text(TextContent::new("second"))],
                cwd: temp.path().to_path_buf(),
                target: second_target,
                cancel_rx,
            })
            .unwrap();
        drop(prompt_tx);

        tokio::time::timeout(
            Duration::from_secs(5),
            run_conversation_session(
                launch,
                HashMap::new(),
                prompt_rx,
                events_tx,
                std::future::ready(None),
            ),
        )
        .await
        .unwrap()
        .unwrap();

        let mut events = Vec::new();
        while let Some(event) = events_rx.next().await {
            if let AcpRuntimeEvent::Event { event, target } = event {
                events.push((target.response_stream_id, event));
            }
        }

        assert!(events
            .iter()
            .any(|(stream_id, event)| stream_id == &first_stream_id
                && matches!(event, AcpEvent::Completed)));
        assert!(!events
            .iter()
            .any(|(stream_id, event)| stream_id == &first_stream_id
                && matches!(event, AcpEvent::Failed { .. })));
        assert!(events
            .iter()
            .any(|(stream_id, event)| stream_id == &second_stream_id
                && matches!(event, AcpEvent::Failed { .. })));
    }
}
