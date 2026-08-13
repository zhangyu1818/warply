use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use parking_lot::FairMutex;
use warpui::r#async::Timer;
use warpui::clipboard::{ClipboardContent, ImageData};
use warpui::elements::{ChildView, Container, Element, Empty};
use warpui::{
    AppContext, Entity, EntityId, ModelHandle, SingletonEntity, View, ViewContext, ViewHandle,
};

use crate::ai::agent::ImageContext;
use crate::ai::blocklist::agent_view::agent_input_footer::{
    AgentInputFooter, AgentInputFooterEvent,
};
use crate::code_review::diff_state::GitDeltaPreference;
use crate::code_review::events::CodeReviewPaneEntrypoint;
use crate::settings::{AISettings, AISettingsChangedEvent, InputModeSettings};
use crate::terminal::cli_agent_sessions::{
    CLIAgentInputEntrypoint, CLIAgentRichInputCloseReason, CLIAgentSessionsModel,
};
use crate::terminal::model_events::{ModelEvent, ModelEventDispatcher};
use crate::terminal::{CLIAgent, TerminalModel};
use crate::util::image::{MAX_IMAGE_SIZE_BYTES_FOR_CLI_AGENT, MIME_SNIFF_BYTES, infer_mime_type};

use warp_terminal::model::escape_sequences::{BRACKETED_PASTE_END, BRACKETED_PASTE_START};

use super::{RichContentInsertionPosition, TerminalView};

const CLI_AGENT_PTY_WRITE_DELAY: Duration = Duration::from_millis(50);
const CLI_AGENT_BRACKETED_PASTE_ENTER_DELAY: Duration = Duration::from_millis(300);
const CLI_AGENT_IMAGE_PASTE_DELAY: Duration = Duration::from_millis(300);

#[allow(clippy::byte_char_slices)]
const CLI_AGENT_MODE_SWITCH_PREFIXES: &[u8] = &[b'!', b'&'];

fn cli_agent_paste_keystroke_bytes() -> Vec<u8> {
    vec![0x16]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RichInputSubmitStrategy {
    Inline,
    BracketedPaste,
    DelayedEnter,
    BracketedPasteDelayedEnter,
}

fn rich_input_submit_strategy(agent: CLIAgent) -> RichInputSubmitStrategy {
    match agent {
        CLIAgent::Codex => RichInputSubmitStrategy::BracketedPaste,
        CLIAgent::Copilot => RichInputSubmitStrategy::BracketedPasteDelayedEnter,
        CLIAgent::Claude
        | CLIAgent::OpenCode
        | CLIAgent::Gemini
        | CLIAgent::Auggie
        | CLIAgent::CursorCli => RichInputSubmitStrategy::DelayedEnter,
        CLIAgent::Amp
        | CLIAgent::Droid
        | CLIAgent::Pi
        | CLIAgent::Goose
        | CLIAgent::Vibe
        | CLIAgent::Unknown => RichInputSubmitStrategy::Inline,
    }
}

impl TerminalView {
    pub(super) fn register_subscriptions_for_cli_agent_footer(
        &mut self,
        ctx: &mut ViewContext<Self>,
    ) {
        let ai_settings = AISettings::handle(ctx);
        ctx.subscribe_to_model(&ai_settings, |me, _, event, ctx| match event {
            AISettingsChangedEvent::ShouldRenderCLIAgentToolbar { .. }
            | AISettingsChangedEvent::CLIAgentToolbarEnabledCommands { .. } => {
                me.maybe_show_cli_agent_footer_in_blocklist(ctx);
            }
            _ => (),
        });

        ctx.subscribe_to_view(&self.cli_agent_footer, |me, _, event, ctx| {
            me.handle_cli_agent_footer_event(event, ctx);
        });

        let input_mode_settings = InputModeSettings::handle(ctx);
        let mut was_pinned_to_top = input_mode_settings
            .as_ref(ctx)
            .input_mode
            .is_pinned_to_top();
        ctx.subscribe_to_model(&input_mode_settings, move |me, settings_handle, _, ctx| {
            let is_pinned_to_top = settings_handle.as_ref(ctx).is_pinned_to_top();
            if was_pinned_to_top != is_pinned_to_top {
                was_pinned_to_top = is_pinned_to_top;
                me.maybe_show_cli_agent_footer_in_blocklist(ctx);
            }
        });
    }

    fn handle_cli_agent_footer_event(
        &mut self,
        event: &CLIAgentFooterEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            CLIAgentFooterEvent::WriteToPty(text) => {
                self.write_user_bytes_to_pty(text.as_bytes().to_vec(), ctx);
            }
            CLIAgentFooterEvent::InsertIntoRichInput(text) => {
                self.input.update(ctx, |input, ctx| {
                    input.insert_into_cli_agent_rich_input(text, ctx);
                });
            }
            CLIAgentFooterEvent::ToggleCodeReviewPane(cli_agent) => {
                self.toggle_code_review_pane(
                    GitDeltaPreference::Always,
                    CodeReviewPaneEntrypoint::CLIAgentView,
                    Some(*cli_agent),
                    true,
                    ctx,
                );
            }
            CLIAgentFooterEvent::ToggleFileExplorer(cli_agent) => {
                let _ = cli_agent;
                self.toggle_file_tree(ctx);
            }
            CLIAgentFooterEvent::OpenRichInput => {
                if self.has_active_cli_agent_input_session(ctx) {
                    self.close_cli_agent_rich_input_and_disable_auto_toggle(ctx);
                } else {
                    self.open_cli_agent_rich_input(CLIAgentInputEntrypoint::FooterButton, ctx);
                }
            }
            CLIAgentFooterEvent::HideRichInput => {
                self.close_cli_agent_rich_input_and_disable_auto_toggle(ctx);
            }
        }
    }

    pub(super) fn has_active_cli_agent_input_session(&self, app: &AppContext) -> bool {
        CLIAgentSessionsModel::as_ref(app).is_input_open(self.view_id)
    }

    pub(super) fn should_render_cli_agent_footer(
        &self,
        _model: &TerminalModel,
        app: &AppContext,
    ) -> bool {
        CLIAgentSessionsModel::as_ref(app)
            .session(self.view_id)
            .is_some()
            && *AISettings::as_ref(app).should_render_cli_agent_footer
    }

    pub(super) fn maybe_show_cli_agent_footer_in_blocklist(&mut self, ctx: &mut ViewContext<Self>) {
        self.hide_cli_agent_footer_in_blocklist(ctx);
        let (should_render_footer, is_alt_screen_active) = {
            let model = self.model.lock();
            (
                self.should_render_cli_agent_footer(&model, ctx),
                model.is_alt_screen_active(),
            )
        };
        if is_alt_screen_active || !should_render_footer {
            return;
        }

        let should_insert_after_block = !InputModeSettings::as_ref(ctx).is_pinned_to_top();

        self.insert_rich_content(
            None,
            self.cli_agent_footer.clone(),
            None,
            RichContentInsertionPosition::Append {
                insert_below_long_running_block: should_insert_after_block,
            },
            ctx,
        );
    }

    pub(super) fn hide_cli_agent_footer_in_blocklist(&mut self, ctx: &mut ViewContext<Self>) {
        let mut model = self.model.lock();
        let block_list = model.block_list_mut();
        block_list.remove_rich_content(self.cli_agent_footer.id());
        ctx.notify();
    }

    pub(in crate::terminal) fn close_cli_agent_rich_input(
        &mut self,
        reason: CLIAgentRichInputCloseReason,
        ctx: &mut ViewContext<Self>,
    ) {
        self.close_cli_agent_rich_input_impl(true, reason, ctx);
    }

    pub(in crate::terminal) fn close_cli_agent_rich_input_and_disable_auto_toggle(
        &mut self,
        ctx: &mut ViewContext<Self>,
    ) {
        self.close_cli_agent_rich_input_impl(false, CLIAgentRichInputCloseReason::Manual, ctx);
    }

    fn close_cli_agent_rich_input_impl(
        &mut self,
        should_auto_toggle_input: bool,
        _reason: CLIAgentRichInputCloseReason,
        ctx: &mut ViewContext<Self>,
    ) {
        if !self.has_active_cli_agent_input_session(ctx) {
            return;
        }

        let draft = self.input.as_ref(ctx).buffer_text(ctx);
        let view_id = self.view_id;
        CLIAgentSessionsModel::handle(ctx).update(ctx, |sessions_model, ctx| {
            sessions_model.set_draft(view_id, draft);
            sessions_model.close_input(view_id, should_auto_toggle_input, ctx);
        });

        self.redetermine_terminal_focus(ctx);
        ctx.notify();
    }

    fn maybe_close_rich_input_after_submit(&mut self, ctx: &mut ViewContext<Self>) {
        let session = CLIAgentSessionsModel::as_ref(ctx).session(self.view_id);
        let has_plugin = session
            .as_ref()
            .is_some_and(|s| s.listener.is_some() && s.should_auto_toggle_input);
        let ai_settings = AISettings::as_ref(ctx);

        let should_close = if has_plugin && *ai_settings.auto_toggle_rich_input {
            false
        } else {
            *ai_settings.auto_dismiss_rich_input_after_submit
        };

        if should_close {
            self.close_cli_agent_rich_input(CLIAgentRichInputCloseReason::Submit, ctx);
        } else {
            self.input.update(ctx, |input, ctx| {
                input.clear_buffer_and_reset_undo_stack(ctx);
            });
        }
    }

    pub(super) fn submit_cli_agent_rich_input(
        &mut self,
        text: String,
        ctx: &mut ViewContext<Self>,
    ) {
        if !self.has_active_cli_agent_input_session(ctx) || text.trim().is_empty() {
            return;
        }

        let view_id = self.view_id;
        CLIAgentSessionsModel::handle(ctx).update(ctx, |sessions_model, _| {
            sessions_model.clear_draft(view_id);
        });

        let strategy = CLIAgentSessionsModel::as_ref(ctx)
            .session(self.view_id)
            .map(|s| rich_input_submit_strategy(s.agent))
            .unwrap_or(RichInputSubmitStrategy::Inline);

        let text_bytes = text.into_bytes();

        self.input.update(ctx, |input, ctx| {
            input.clear_buffer_and_reset_undo_stack(ctx);
        });

        let images: Vec<_> = self
            .ai_context_model
            .as_ref(ctx)
            .pending_images()
            .into_iter()
            .cloned()
            .collect();
        if !images.is_empty() {
            self.ai_context_model.update(ctx, |model, ctx| {
                model.clear_pending_images(ctx);
            });
        }

        if text_bytes.len() > 1 && CLI_AGENT_MODE_SWITCH_PREFIXES.contains(&text_bytes[0]) {
            self.write_user_bytes_to_pty(vec![text_bytes[0]], ctx);
            let rest = text_bytes[1..].to_vec();
            ctx.spawn(
                Timer::after(CLI_AGENT_PTY_WRITE_DELAY),
                move |me, _, ctx| {
                    me.paste_images_then_submit_text(images, rest, strategy, ctx);
                },
            );
        } else {
            self.paste_images_then_submit_text(images, text_bytes, strategy, ctx);
        }
    }

    fn paste_images_then_submit_text(
        &mut self,
        images: Vec<ImageContext>,
        text_bytes: Vec<u8>,
        strategy: RichInputSubmitStrategy,
        ctx: &mut ViewContext<Self>,
    ) {
        if !self.has_active_cli_agent_input_session(ctx) {
            return;
        }

        if images.is_empty() {
            self.write_cli_agent_text_then_submit(text_bytes, strategy, ctx);
            return;
        }

        let spawner = ctx.spawner();
        ctx.spawn(
            async move {
                for image in images {
                    let raw_bytes =
                        match base64::engine::general_purpose::STANDARD.decode(&image.data) {
                            Ok(bytes) => bytes,
                            Err(_) => {
                                log::error!(
                                    "Failed to decode base64 image data for {}",
                                    image.file_name
                                );
                                continue;
                            }
                        };

                    let should_continue = spawner
                        .spawn(move |me, ctx| {
                            if !me.has_active_cli_agent_input_session(ctx) {
                                return false;
                            }
                            ctx.clipboard().write(ClipboardContent {
                                images: Some(vec![ImageData {
                                    data: raw_bytes,
                                    mime_type: image.mime_type,
                                    filename: Some(image.file_name),
                                }]),
                                ..Default::default()
                            });
                            me.write_user_bytes_to_pty(cli_agent_paste_keystroke_bytes(), ctx);
                            true
                        })
                        .await;

                    if !matches!(should_continue, Ok(true)) {
                        return false;
                    }

                    Timer::after(CLI_AGENT_IMAGE_PASTE_DELAY).await;
                }
                true
            },
            move |me, ok, ctx| {
                if !ok || !me.has_active_cli_agent_input_session(ctx) {
                    return;
                }
                me.write_cli_agent_text_then_submit(text_bytes, strategy, ctx);
            },
        );
    }

    pub(super) fn paste_dropped_images_to_cli_agent(
        &mut self,
        image_filepaths: Vec<String>,
        ctx: &mut ViewContext<Self>,
    ) {
        if image_filepaths.is_empty() {
            return;
        }
        let spawner = ctx.spawner();
        ctx.spawn(
            async move {
                for path_str in image_filepaths {
                    match async_fs::metadata(&path_str).await {
                        Ok(meta) if (meta.len() as usize) > MAX_IMAGE_SIZE_BYTES_FOR_CLI_AGENT => {
                            let filename = Path::new(&path_str)
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_else(|| path_str.clone());
                            let limit_mb = MAX_IMAGE_SIZE_BYTES_FOR_CLI_AGENT / 1_000_000;
                            let msg = format!(
                                "{filename} is too large to send to the agent (limit {limit_mb}MB)."
                            );
                            let _ = spawner
                                .spawn(move |me, ctx| {
                                    me.show_error_toast(msg, ctx);
                                })
                                .await;
                            continue;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Failed to stat dropped image {path_str}: {e}");
                            continue;
                        }
                    }

                    let bytes = match async_fs::read(&path_str).await {
                        Ok(b) => b,
                        Err(e) => {
                            log::error!("Failed to read dropped image {path_str}: {e}");
                            continue;
                        }
                    };
                    let path = Path::new(&path_str);
                    let filename = path.file_name().map(|n| n.to_string_lossy().into_owned());
                    let sniff_len = bytes.len().min(MIME_SNIFF_BYTES);
                    let mime_type = infer_mime_type(path, &bytes[..sniff_len]);

                    let should_continue = spawner
                        .spawn(move |me, ctx| {
                            if !me.has_active_cli_agent_session(ctx) {
                                return false;
                            }
                            let still_long_running = me
                                .model
                                .lock()
                                .block_list()
                                .active_block()
                                .is_active_and_long_running();
                            if !still_long_running {
                                return false;
                            }
                            ctx.clipboard().write(ClipboardContent {
                                images: Some(vec![ImageData {
                                    data: bytes,
                                    mime_type,
                                    filename,
                                }]),
                                ..Default::default()
                            });
                            me.write_user_bytes_to_pty(cli_agent_paste_keystroke_bytes(), ctx);
                            true
                        })
                        .await;

                    if !matches!(should_continue, Ok(true)) {
                        return;
                    }

                    Timer::after(CLI_AGENT_IMAGE_PASTE_DELAY).await;
                }
            },
            |_, _, _| {},
        );
    }

    fn write_cli_agent_text_then_submit(
        &mut self,
        text_bytes: Vec<u8>,
        strategy: RichInputSubmitStrategy,
        ctx: &mut ViewContext<Self>,
    ) {
        match strategy {
            RichInputSubmitStrategy::Inline => {
                let mut bytes = text_bytes;
                bytes.extend_from_slice(b"\r");
                self.write_user_bytes_to_pty(bytes, ctx);
                self.maybe_close_rich_input_after_submit(ctx);
            }
            RichInputSubmitStrategy::BracketedPaste => {
                let mut bytes = Vec::with_capacity(
                    BRACKETED_PASTE_START.len() + text_bytes.len() + BRACKETED_PASTE_END.len(),
                );
                bytes.extend_from_slice(BRACKETED_PASTE_START);
                bytes.extend_from_slice(&text_bytes);
                bytes.extend_from_slice(BRACKETED_PASTE_END);
                self.write_user_bytes_to_pty(bytes, ctx);
                self.write_user_bytes_to_pty(b"\r".to_vec(), ctx);
                self.maybe_close_rich_input_after_submit(ctx);
            }
            RichInputSubmitStrategy::DelayedEnter => {
                self.write_user_bytes_to_pty(text_bytes, ctx);
                ctx.spawn(
                    Timer::after(CLI_AGENT_PTY_WRITE_DELAY),
                    move |me, _, ctx| {
                        me.write_user_bytes_to_pty(b"\r".to_vec(), ctx);
                        me.maybe_close_rich_input_after_submit(ctx);
                    },
                );
            }
            RichInputSubmitStrategy::BracketedPasteDelayedEnter => {
                let mut bytes = Vec::with_capacity(
                    BRACKETED_PASTE_START.len() + text_bytes.len() + BRACKETED_PASTE_END.len(),
                );
                bytes.extend_from_slice(BRACKETED_PASTE_START);
                bytes.extend_from_slice(&text_bytes);
                bytes.extend_from_slice(BRACKETED_PASTE_END);
                self.write_user_bytes_to_pty(bytes, ctx);
                ctx.spawn(
                    Timer::after(CLI_AGENT_BRACKETED_PASTE_ENTER_DELAY),
                    move |me, _, ctx| {
                        me.write_user_bytes_to_pty(b"\r".to_vec(), ctx);
                        me.maybe_close_rich_input_after_submit(ctx);
                    },
                );
            }
        }
    }

    pub(in crate::terminal) fn open_cli_agent_rich_input(
        &mut self,
        entrypoint: CLIAgentInputEntrypoint,
        ctx: &mut ViewContext<Self>,
    ) {
        if self.has_active_cli_agent_input_session(ctx) {
            return;
        }

        let Some(_cli_agent) = CLIAgentSessionsModel::as_ref(ctx)
            .session(self.view_id)
            .map(|session| session.agent)
        else {
            return;
        };

        let ai_input_model = self.ai_input_model.as_ref(ctx);
        let previous_input_config = ai_input_model.input_config();
        let previous_was_lock_set_with_empty_buffer =
            ai_input_model.was_lock_set_with_empty_buffer();

        let view_id = self.view_id;
        CLIAgentSessionsModel::handle(ctx).update(ctx, |sessions_model, ctx| {
            sessions_model.open_input(
                view_id,
                entrypoint,
                previous_input_config,
                previous_was_lock_set_with_empty_buffer,
                true,
                ctx,
            );
        });

        self.redetermine_terminal_focus(ctx);
        ctx.notify();
    }
}

pub(super) struct CLIAgentFooter {
    terminal_view_id: EntityId,
    terminal_model: Arc<FairMutex<TerminalModel>>,
    agent_input_footer: ViewHandle<AgentInputFooter>,
}

impl CLIAgentFooter {
    pub(crate) fn new(
        terminal_view_id: EntityId,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        model_event_dispatcher: &ModelHandle<ModelEventDispatcher>,
        agent_input_footer: ViewHandle<AgentInputFooter>,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        ctx.subscribe_to_view(&agent_input_footer, |me, _, event, ctx| {
            me.handle_agent_input_footer_event(event, ctx);
        });

        ctx.subscribe_to_model(model_event_dispatcher, |me, _, event, ctx| {
            if let ModelEvent::TerminalModeSwapped(..) = event {
                me.notify_and_notify_children(ctx);
            }
        });

        let cli_agent_sessions = CLIAgentSessionsModel::handle(ctx);
        ctx.subscribe_to_model(&cli_agent_sessions, move |me, _, event, ctx| {
            if event.terminal_view_id() != terminal_view_id {
                return;
            }
            me.notify_and_notify_children(ctx);
        });

        Self {
            terminal_view_id,
            terminal_model,
            agent_input_footer,
        }
    }

    fn handle_agent_input_footer_event(
        &mut self,
        event: &AgentInputFooterEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            AgentInputFooterEvent::WriteToPty(text) => {
                ctx.emit(CLIAgentFooterEvent::WriteToPty(text.clone()));
            }
            AgentInputFooterEvent::InsertIntoCLIRichInput(text) => {
                ctx.emit(CLIAgentFooterEvent::InsertIntoRichInput(text.clone()));
            }
            AgentInputFooterEvent::ToggleCodeReviewPane(agent) => {
                ctx.emit(CLIAgentFooterEvent::ToggleCodeReviewPane(*agent));
            }
            AgentInputFooterEvent::ToggleFileExplorer(agent) => {
                ctx.emit(CLIAgentFooterEvent::ToggleFileExplorer(*agent));
            }
            AgentInputFooterEvent::OpenRichInput => {
                ctx.emit(CLIAgentFooterEvent::OpenRichInput);
            }
            AgentInputFooterEvent::HideRichInput => {
                ctx.emit(CLIAgentFooterEvent::HideRichInput);
            }
            _ => {}
        }
    }

    pub(in crate::terminal) fn notify_and_notify_children(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.notify();
        self.agent_input_footer.update(ctx, |_, ctx| ctx.notify());
    }

    fn cli_agent(&self, app: &AppContext) -> Option<CLIAgent> {
        CLIAgentSessionsModel::as_ref(app)
            .session(self.terminal_view_id)
            .map(|session| session.agent)
    }
}

pub(super) enum CLIAgentFooterEvent {
    WriteToPty(String),
    InsertIntoRichInput(String),
    ToggleCodeReviewPane(CLIAgent),
    ToggleFileExplorer(CLIAgent),
    OpenRichInput,
    HideRichInput,
}

impl Entity for CLIAgentFooter {
    type Event = CLIAgentFooterEvent;
}

impl View for CLIAgentFooter {
    fn ui_name() -> &'static str {
        "CLIAgentFooter"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        if CLIAgentSessionsModel::as_ref(app).is_input_open(self.terminal_view_id)
            || self.cli_agent(app).is_none()
        {
            return Empty::new().finish();
        }

        let mut container = Container::new(ChildView::new(&self.agent_input_footer).finish())
            .with_horizontal_padding(*super::PADDING_LEFT);

        let terminal_model = self.terminal_model.lock();
        if terminal_model.is_alt_screen_active() {
            if let Some(bg_color) = terminal_model.alt_screen().inferred_bg_color() {
                container = container.with_background(bg_color);
            }
        }

        container.finish()
    }
}
