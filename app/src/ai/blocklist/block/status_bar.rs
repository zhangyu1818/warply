use std::{collections::HashSet, sync::Arc, time::Duration};

use super::{
    cli_controller::{CLISubagentController, CLISubagentEvent},
    model::{AIBlockModel, AIBlockModelImpl, AIBlockOutputStatus},
    view_impl::common::{render_warping_indicator, ButtonProps, WarpingProps},
};
use crate::terminal::input::buffer_model::InputBufferUpdateEvent;
use crate::{
    ai::blocklist::agent_view::{
        agent_view_bg_fill, AgentMessageBar, AgentViewController, EphemeralMessageModel,
    },
    terminal::input::{
        buffer_model::InputBufferModel, slash_command_model::SlashCommandModel,
        suggestions_mode_model::InputSuggestionsModeModel,
    },
};

use crate::{
    ai::{
        acp::model::AcpAgentModel,
        agent::{
            conversation::AIConversationId, AIAgentExchangeId, AIAgentOutput,
            AIAgentOutputMessageType, CancellationReason, SummarizationType,
        },
        blocklist::{
            agent_view::shortcuts::AgentShortcutViewModel,
            model::AIBlockModelHelper,
            summarization_cancel_dialog::{
                self, SummarizationCancelDialog, SummarizationCancelDialogEvent,
            },
            BlocklistAIActionEvent, BlocklistAIActionModel, BlocklistAIContextEvent,
            BlocklistAIContextModel, BlocklistAIController, BlocklistAIHistoryEvent,
            BlocklistAIInputEvent, BlocklistAIInputModel, ResponseStreamId,
        },
    },
    settings::InputModeSettings,
    settings_view::keybindings::KeybindingChangedNotifier,
    terminal::{
        model::block::LONG_RUNNING_COMMAND_DURATION_MS,
        model_events::{ModelEvent, ModelEventDispatcher},
        TerminalModel, CANCEL_COMMAND_KEYBINDING,
    },
    util::bindings::keybinding_name_to_keystroke,
    BlocklistAIHistoryModel,
};
use instant::Instant;
use parking_lot::FairMutex;
use pathfinder_color::ColorU;
use warp_core::ui::theme::Fill;
use warpui::elements::shimmering_text::ShimmeringTextStateHandle;
use warpui::{
    elements::{Container, Empty, Flex, MouseStateHandle, ParentElement},
    keymap::Keystroke,
    presenter::ChildView,
    r#async::SpawnedFutureHandle,
    AppContext, Element, Entity, EntityId, ModelHandle, SingletonEntity, View, ViewContext,
    ViewHandle,
};
use warpui::{r#async::Timer, TypedActionView};

pub fn init(app: &mut AppContext) {
    summarization_cancel_dialog::init(app);
}

#[derive(Default)]
struct StateHandles {
    stop_button: MouseStateHandle,
}

pub struct BlocklistAIStatusBar {
    active_exchange_model: Option<Box<dyn AIBlockModel<View = BlocklistAIStatusBar>>>,
    action_model: ModelHandle<BlocklistAIActionModel>,
    controller: ModelHandle<BlocklistAIController>,
    cli_subagent_controller: ModelHandle<CLISubagentController>,
    input_model: ModelHandle<BlocklistAIInputModel>,
    agent_view_controller: ModelHandle<AgentViewController>,
    terminal_model: Arc<FairMutex<TerminalModel>>,
    shimmering_text_handle: ShimmeringTextStateHandle,
    state_handles: StateHandles,

    stop_keystroke: Option<Keystroke>,
    // Whether the summarization cancellation confirmation dialog is open.
    is_summarization_cancel_dialog_open: bool,
    summarization_cancel_dialog: ViewHandle<SummarizationCancelDialog>,

    /// Handle for the periodic timer that updates the summarization timer UI.
    summarization_timer_handle: Option<SpawnedFutureHandle>,
    summarization_start_time: Option<Instant>,
    /// Handle for the 1-second periodic timer that refreshes the "Last read …" suffix in
    /// the warping indicator while the active block has a recorded LRC snapshot.
    last_read_refresh_handle: Option<SpawnedFutureHandle>,

    latest_response_stream_id: Option<ResponseStreamId>,

    ephemeral_message_model: ModelHandle<EphemeralMessageModel>,
    agent_message_bar: ViewHandle<AgentMessageBar>,
}

impl BlocklistAIStatusBar {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        controller: ModelHandle<BlocklistAIController>,
        agent_view_controller: ModelHandle<AgentViewController>,
        cli_subagent_controller: ModelHandle<CLISubagentController>,
        action_model: ModelHandle<BlocklistAIActionModel>,
        context_model: ModelHandle<BlocklistAIContextModel>,
        input_model: ModelHandle<BlocklistAIInputModel>,
        input_buffer_model: ModelHandle<InputBufferModel>,
        model_event_dispatcher: &ModelHandle<ModelEventDispatcher>,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        shortcut_view_model: ModelHandle<AgentShortcutViewModel>,
        input_suggestions_model: ModelHandle<InputSuggestionsModeModel>,
        slash_command_model: ModelHandle<SlashCommandModel>,
        ephemeral_message_model: ModelHandle<EphemeralMessageModel>,
        terminal_view_id: EntityId,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        let history_model = BlocklistAIHistoryModel::handle(ctx);
        ctx.subscribe_to_model(&history_model, move |me, _, event, ctx| {
            if event
                .terminal_view_id()
                .is_some_and(|id| id != terminal_view_id)
            {
                return;
            }
            match event {
                BlocklistAIHistoryEvent::AppendedExchange {
                    response_stream_id,
                    exchange_id,
                    conversation_id,
                    ..
                } => {
                    if let Some(response_stream_id) = response_stream_id.clone() {
                        me.latest_response_stream_id = Some(response_stream_id);
                    }
                    me.reset_model_for_exchange(*exchange_id, *conversation_id, ctx);
                }
                BlocklistAIHistoryEvent::ClearedConversationsInTerminalView { .. } => {
                    me.active_exchange_model = None;
                    ctx.notify();
                }
                BlocklistAIHistoryEvent::ClearedActiveConversation {
                    conversation_id, ..
                }
                | BlocklistAIHistoryEvent::RemoveConversation {
                    conversation_id, ..
                } => {
                    if me.active_exchange_model.as_ref().is_some_and(|model| {
                        model
                            .conversation_id(ctx)
                            .is_some_and(|id| id == *conversation_id)
                    }) {
                        me.active_exchange_model = None;
                        ctx.notify();
                    }
                }
                BlocklistAIHistoryEvent::UpdatedConversationStatus { .. } => {
                    ctx.notify();
                }
                BlocklistAIHistoryEvent::SetActiveConversation {
                    conversation_id, ..
                } => {
                    let Some(conversation) =
                        BlocklistAIHistoryModel::as_ref(ctx).conversation(conversation_id)
                    else {
                        return;
                    };
                    let Some(new_latest_exchange_id) =
                        conversation.latest_exchange().map(|exchange| exchange.id)
                    else {
                        return;
                    };

                    me.reset_model_for_exchange(new_latest_exchange_id, conversation.id(), ctx);
                }
                BlocklistAIHistoryEvent::UpdatedAutoexecuteOverride { .. } => ctx.notify(),
                _ => (),
            }
        });
        ctx.subscribe_to_model(&context_model, |_, _, event, ctx| {
            if matches!(
                event,
                BlocklistAIContextEvent::PendingQueryStateUpdated
                    | BlocklistAIContextEvent::QueueNextPromptToggled
            ) {
                ctx.notify();
            }
        });
        ctx.subscribe_to_model(&input_model, |_, _, event, ctx| {
            if let BlocklistAIInputEvent::InputTypeChanged { .. } = event {
                ctx.notify();
            }
        });
        ctx.subscribe_to_model(
            &cli_subagent_controller,
            move |me, _, event, ctx| match event {
                CLISubagentEvent::UpdatedControl { .. }
                | CLISubagentEvent::ToggledHideResponses => {
                    ctx.notify();
                }
                CLISubagentEvent::UpdatedLastSnapshot => {
                    let has_active_snapshot = me.should_refresh_last_read_timer(ctx);
                    if has_active_snapshot {
                        me.start_last_read_timer(ctx);
                    } else {
                        me.stop_last_read_timer();
                    }
                    ctx.notify();
                }
                _ => {}
            },
        );
        ctx.subscribe_to_model(&input_buffer_model, |me, _, event, ctx| {
            let InputBufferUpdateEvent {
                old_content: old,
                new_content: new,
                ..
            } = event;
            if !me.input_model.as_ref(ctx).is_ai_input_enabled() && old.is_empty() != new.is_empty()
            {
                ctx.notify();
            }
        });
        ctx.subscribe_to_model(&agent_view_controller, |_, _, _, ctx| ctx.notify());

        let input_mode_settings = InputModeSettings::handle(ctx);
        ctx.subscribe_to_model(&input_mode_settings, |_, _, _, ctx| ctx.notify());
        let stop_keystroke = keybinding_name_to_keystroke(CANCEL_COMMAND_KEYBINDING, ctx);
        ctx.subscribe_to_model(&KeybindingChangedNotifier::handle(ctx), |me, _, _, ctx| {
            me.stop_keystroke = keybinding_name_to_keystroke(CANCEL_COMMAND_KEYBINDING, ctx);
            ctx.notify();
        });

        let summarization_cancel_dialog =
            ctx.add_typed_action_view(|_| SummarizationCancelDialog::default());
        ctx.subscribe_to_view(
            &summarization_cancel_dialog,
            |me, _, event, ctx| match event {
                SummarizationCancelDialogEvent::ConfirmCancel => {
                    me.cancel_active_request_or_action(ctx);
                    me.close_summarization_cancel_dialog(ctx);
                }
                SummarizationCancelDialogEvent::Continue => {
                    me.close_summarization_cancel_dialog(ctx);
                }
            },
        );

        ctx.subscribe_to_model(&action_model, |_, _, event, ctx| match event {
            BlocklistAIActionEvent::ExecutingAction(..)
            | BlocklistAIActionEvent::FinishedAction { .. } => ctx.notify(),
            _ => (),
        });
        ctx.subscribe_to_model(model_event_dispatcher, |me, _, event, ctx| match event {
            ModelEvent::AfterBlockStarted { block_id, .. } => {
                let terminal_model = me.terminal_model.lock();
                if terminal_model
                    .block_list()
                    .block_with_id(block_id)
                    .is_some_and(|block| block.agent_interaction_metadata().is_some())
                {
                    ctx.spawn(
                        Timer::after(Duration::from_millis(LONG_RUNNING_COMMAND_DURATION_MS)),
                        |_, _, ctx| ctx.notify(),
                    );
                }
            }
            ModelEvent::BlockCompleted(_) => {
                ctx.notify();
            }
            _ => (),
        });

        ctx.subscribe_to_model(&ephemeral_message_model, |_, _, _, ctx| {
            ctx.notify();
        });

        let agent_message_bar = ctx.add_view(|ctx| {
            AgentMessageBar::new(
                agent_view_controller.clone(),
                ephemeral_message_model.clone(),
                shortcut_view_model.clone(),
                input_buffer_model,
                input_model.clone(),
                input_suggestions_model,
                slash_command_model,
                context_model.clone(),
                terminal_model.clone(),
                ctx,
            )
        });

        Self {
            active_exchange_model: None,
            shimmering_text_handle: ShimmeringTextStateHandle::new(),
            action_model,
            input_model,
            terminal_model,
            controller,
            agent_view_controller,
            cli_subagent_controller,
            state_handles: Default::default(),
            stop_keystroke,
            summarization_cancel_dialog,
            latest_response_stream_id: None,
            is_summarization_cancel_dialog_open: false,
            summarization_timer_handle: None,
            summarization_start_time: None,
            last_read_refresh_handle: None,
            ephemeral_message_model,
            agent_message_bar,
        }
    }

    pub fn should_show_summarization_cancel_dialog(&self, app: &AppContext) -> bool {
        self.is_summarization_cancel_dialog_open
            && self
                .active_exchange_model
                .as_ref()
                .is_some_and(|model| model.is_conversation_summarization_active(app))
    }
    pub fn summarization_cancel_dialog_handle(&self) -> &ViewHandle<SummarizationCancelDialog> {
        &self.summarization_cancel_dialog
    }

    pub fn handle_ctrl_c(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(model) = self.active_exchange_model.as_ref() else {
            return;
        };

        if model.is_conversation_summarization_active(ctx) {
            if self.is_summarization_cancel_dialog_open {
                self.cancel_active_request_or_action(ctx);
                self.close_summarization_cancel_dialog(ctx);
                return;
            }

            self.open_summarization_cancel_dialog(ctx);
            return;
        }

        self.cancel_active_request_or_action(ctx);
        ctx.notify();
    }

    pub fn notify_and_notify_children(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.notify();
        self.agent_message_bar.update(ctx, |_, ctx| ctx.notify());
    }

    fn reset_model_for_exchange(
        &mut self,
        exchange_id: AIAgentExchangeId,
        conversation_id: AIConversationId,
        ctx: &mut ViewContext<Self>,
    ) {
        let history_model = BlocklistAIHistoryModel::as_ref(ctx);
        let conversation = history_model.conversation(&conversation_id);
        let exchange =
            conversation.and_then(|conversation| conversation.exchange_with_id(exchange_id));

        if self.active_exchange_model.as_ref().is_none_or(|model| {
            model.exchange_id(ctx).is_none_or(|id| id != exchange_id)
                || model
                    .conversation(ctx)
                    .is_none_or(|conversation| conversation.id() != conversation_id)
        }) {
            let Some(conversation) = conversation else {
                self.active_exchange_model = None;
                ctx.notify();
                return;
            };
            self.active_exchange_model = exchange
                .and_then(|e| {
                    AIBlockModelImpl::<BlocklistAIStatusBar>::new(
                        e.id,
                        conversation.id(),
                        false,
                        false,
                        ctx,
                    )
                    .ok()
                })
                .map(|model| {
                    model.on_updated_output(
                        Box::new(|me, ctx| me.on_updated_active_exchange_output(ctx)),
                        ctx,
                    );
                    Box::new(model) as Box<dyn AIBlockModel<View = BlocklistAIStatusBar>>
                });
            self.is_summarization_cancel_dialog_open = false;
            self.stop_summarization_timer();
        }
    }

    fn on_updated_active_exchange_output(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(model) = self.active_exchange_model.as_ref() else {
            return;
        };
        let status = model.status(ctx);

        // Auto-clear summarization confirmation dialog if summarization is no longer active
        if self.is_summarization_cancel_dialog_open
            && !model.is_conversation_summarization_active(ctx)
        {
            self.is_summarization_cancel_dialog_open = false;
            ctx.emit(
                BlocklistAIStatusBarEvent::SummarizationCancelDialogToggled { is_open: false },
            );
            self.stop_summarization_timer();
        }

        match status {
            AIBlockOutputStatus::PartiallyReceived { output }
            | AIBlockOutputStatus::Complete { output } => {
                let output = output.get();
                self.handle_updated_output(&output, ctx);
            }
            AIBlockOutputStatus::Cancelled { partial_output, .. } => {
                if let Some(output) = partial_output.as_ref() {
                    let output = output.get();
                    self.handle_updated_output(&output, ctx);
                }
            }
            AIBlockOutputStatus::Pending | AIBlockOutputStatus::Failed { .. } => (),
        }

        ctx.notify();
    }

    /// Closes the summarization cancel dialog.
    fn close_summarization_cancel_dialog(&mut self, ctx: &mut ViewContext<Self>) {
        self.is_summarization_cancel_dialog_open = false;
        ctx.focus_self();
        ctx.emit(BlocklistAIStatusBarEvent::SummarizationCancelDialogToggled { is_open: false });
        ctx.notify();
    }

    /// Opens the summarization cancel dialog.
    fn open_summarization_cancel_dialog(&mut self, ctx: &mut ViewContext<Self>) {
        self.is_summarization_cancel_dialog_open = true;
        ctx.focus(&self.summarization_cancel_dialog);
        ctx.emit(BlocklistAIStatusBarEvent::SummarizationCancelDialogToggled { is_open: true });
        ctx.notify();
    }

    fn handle_updated_output(&mut self, output: &AIAgentOutput, ctx: &mut ViewContext<Self>) {
        // Register element state for reasoning messages and track summarization timing.
        for message in &output.messages {
            // Track summarization start time and token count when summarization message arrives
            if let AIAgentOutputMessageType::Summarization {
                finished_duration,
                summarization_type,
                ..
            } = &message.message
            {
                // Only track conversation summarization, not tool call result summarization
                if matches!(summarization_type, SummarizationType::ConversationSummary) {
                    if finished_duration.is_none() {
                        // Starting summarization - record start time and start periodic updates
                        if self.summarization_start_time.is_none() {
                            self.summarization_start_time = Some(instant::Instant::now());
                            self.start_summarization_timer(ctx);
                            ctx.notify();
                        }
                    } else if self.summarization_start_time.is_some() {
                        self.stop_summarization_timer();
                    }
                }
            }
        }
    }

    /// Cancels either the in-flight request stream or a pending/running action if present.
    /// If neither is found but the conversation is still in progress (e.g., a subagent is running),
    /// cancels the entire conversation's progress.
    fn cancel_active_request_or_action(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(model) = self.active_exchange_model.as_ref() else {
            return;
        };
        if model.status(ctx).is_streaming() {
            if let Some(response_stream_id) = self.latest_response_stream_id.as_ref() {
                self.controller.update(ctx, |controller, ctx| {
                    controller.cancel_request(
                        response_stream_id,
                        CancellationReason::ManuallyCancelled,
                        ctx,
                    );
                });
            }
        } else {
            let Some(conversation_id) = model.conversation_id(ctx) else {
                return;
            };
            let Some(output) = model.status(ctx).output_to_render() else {
                return;
            };
            let actions = output
                .get()
                .actions()
                .map(|action| action.id.clone())
                .collect::<HashSet<_>>();
            if let Some(active_action_id) = self
                .action_model
                .as_ref(ctx)
                .get_pending_or_running_action_id(ctx)
                .filter(|id| actions.contains(id))
                .cloned()
            {
                self.action_model.update(ctx, |action_model, ctx| {
                    action_model.cancel_action_with_id(
                        conversation_id,
                        &active_action_id,
                        CancellationReason::ManuallyCancelled,
                        ctx,
                    );
                });
            } else if model
                .conversation(ctx)
                .is_some_and(|c| c.status().is_in_progress())
            {
                // No streaming request or pending action, but conversation is still in progress.
                // This happens when a subagent (e.g., computer use or advice) is running.
                // Cancel the entire conversation's progress.
                self.controller.update(ctx, |controller, ctx| {
                    controller.cancel_conversation_progress(
                        conversation_id,
                        CancellationReason::ManuallyCancelled,
                        ctx,
                    );
                });
            }
        }
    }

    /// Starts the periodic timer that updates the summarization UI while summarization is active.
    fn start_summarization_timer(&mut self, ctx: &mut ViewContext<Self>) {
        // Don't start a new timer if one is already running
        if self.summarization_timer_handle.is_some() {
            return;
        }

        // Start a new timer that keeps the elapsed-time indicator fresh.
        let handle = ctx.spawn(
            async move {
                Timer::after(Duration::from_secs(1)).await;
            },
            |me, _unit, ctx| {
                // Clear the handle first so we can restart
                me.summarization_timer_handle = None;

                // Check if summarization is still active
                if me.summarization_start_time.is_some() {
                    ctx.notify();
                    // Restart the timer for the next update
                    me.start_summarization_timer(ctx);
                }
            },
        );

        self.summarization_timer_handle = Some(handle);
    }

    fn stop_summarization_timer(&mut self) {
        self.summarization_start_time = None;
        if let Some(handle) = self.summarization_timer_handle.take() {
            handle.abort();
        }
    }

    fn should_refresh_last_read_timer(&self, ctx: &ViewContext<Self>) -> bool {
        let active_block_id = self
            .terminal_model
            .lock()
            .block_list()
            .active_block()
            .id()
            .clone();
        self.cli_subagent_controller
            .as_ref(ctx)
            .last_snapshot_at(&active_block_id)
            .is_some()
    }

    /// Starts the 1-second periodic timer that keeps the elapsed "Last read Xs ago" suffix
    /// updating in real time. No-ops if the timer is already running or if the active block
    /// no longer has a recorded snapshot.
    fn start_last_read_timer(&mut self, ctx: &mut ViewContext<Self>) {
        if self.last_read_refresh_handle.is_some() || !self.should_refresh_last_read_timer(ctx) {
            return;
        }
        let handle = ctx.spawn(
            async move {
                Timer::after(Duration::from_secs(1)).await;
            },
            |me, _, ctx| {
                me.last_read_refresh_handle = None;
                ctx.notify();
                me.start_last_read_timer(ctx);
            },
        );
        self.last_read_refresh_handle = Some(handle);
    }

    /// Stops and discards the last-read refresh timer.
    fn stop_last_read_timer(&mut self) {
        if let Some(handle) = self.last_read_refresh_handle.take() {
            handle.abort();
        }
    }

    fn render_warping_indicator_for_latest_exchange(
        &self,
        app: &AppContext,
    ) -> Option<Box<dyn Element>> {
        let model = self.active_exchange_model.as_ref()?;
        let conversation = model.conversation(app)?;
        let terminal_model = self.terminal_model.lock();
        let active_block = terminal_model.block_list().active_block();
        let has_expanded_requested_command_with_no_subagent = active_block
            .is_active_and_long_running()
            && active_block
                .agent_interaction_metadata()
                .is_some_and(|metadata| {
                    !metadata.should_hide_block() && metadata.long_running_control_state().is_none()
                });
        let should_render_warping = !model.request_type(app).is_passive()
            && !has_expanded_requested_command_with_no_subagent
            && (conversation.status().is_in_progress()
                || (active_block.is_agent_in_control() && !active_block.is_agent_blocked()));

        if !should_render_warping {
            return None;
        }

        let last_snapshot_at = self
            .cli_subagent_controller
            .as_ref(app)
            .last_snapshot_at(active_block.id());

        let default_warping_text = "Working...".to_owned();

        Some(render_warping_indicator(
            WarpingProps {
                model: model.as_ref(),
                terminal_model: &terminal_model,
                action_model: self.action_model.as_ref(app),
                shimmering_text_handle: &self.shimmering_text_handle,
                summarization_start_time: self.summarization_start_time,
                stop_button: Some(ButtonProps {
                    button_handle: &self.state_handles.stop_button,
                    keystroke: self.stop_keystroke.as_ref(),
                    is_active: false,
                }),
                default_warping_text,
                secondary_element: None,
                last_snapshot_at,
            },
            app,
        ))
    }
}

impl View for BlocklistAIStatusBar {
    fn ui_name() -> &'static str {
        "BlocklistAIStatusBar"
    }

    fn render(&self, app: &AppContext) -> Box<dyn warpui::Element> {
        let agent_view_controller = self.agent_view_controller.as_ref(app);
        let status_element = if let (Some(warping_indicator), true) = (
            self.render_warping_indicator_for_latest_exchange(app),
            self.ephemeral_message_model
                .as_ref(app)
                .current_message()
                .is_none(),
        ) {
            warping_indicator
        } else if agent_view_controller.is_active() {
            return Flex::column()
                .with_child(ChildView::new(&self.agent_message_bar).finish())
                .finish();
        } else {
            return Empty::new().finish();
        };

        let background = if agent_view_controller.is_inline() {
            agent_view_bg_fill(app)
        } else {
            Fill::Solid(ColorU::transparent_black())
        };

        let mut container = Container::new(status_element).with_background(background);

        container = container.with_vertical_padding(8.);

        container.finish()
    }
}

#[derive(Debug, Clone)]
pub enum BlocklistAIStatusBarEvent {
    SummarizationCancelDialogToggled { is_open: bool },
}

impl Entity for BlocklistAIStatusBar {
    type Event = BlocklistAIStatusBarEvent;
}

#[derive(Debug, Clone)]
pub enum BlocklistAIStatusBarAction {
    Stop,
}

impl TypedActionView for BlocklistAIStatusBar {
    type Action = BlocklistAIStatusBarAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            BlocklistAIStatusBarAction::Stop => {
                self.cancel_active_acp_session(ctx);
            }
        }
    }
}

impl BlocklistAIStatusBar {
    fn cancel_active_acp_session(&self, ctx: &mut ViewContext<Self>) {
        let Some(conversation_id) = self
            .active_exchange_model
            .as_ref()
            .and_then(|model| model.conversation_id(ctx))
        else {
            log::warn!("ACP: stop requested without an active conversation");
            return;
        };

        AcpAgentModel::handle(ctx).update(ctx, |model, _| {
            model.cancel_session(conversation_id);
        });
    }
}
