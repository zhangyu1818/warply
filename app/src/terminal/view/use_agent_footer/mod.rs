//! Footer bar for "Use agent" functionality during long-running commands.
//!
//! This module provides a footer that appears at the bottom of active long running blocks,
//! offering users the option to bring in the agent.

use crate::terminal::cli_agent_sessions::CLIAgentSessionsModel;
mod warpify_footer;

pub use crate::terminal::CLIAgent;
use warpify_footer::{WarpifyFooterView, WarpifyFooterViewEvent};

use std::sync::{Arc, LazyLock};

use parking_lot::FairMutex;
use pathfinder_color::ColorU;
use warp_core::{
    settings::Setting,
    ui::{
        appearance::Appearance,
        color::contrast::{
            high_enough_contrast, pick_best_foreground_color, MinimumAllowedContrast,
        },
        theme::{color::internal_colors, Fill as ThemeFill},
    },
};

use warpui::{
    elements::{
        ChildView, Container, CrossAxisAlignment, Empty, Expanded, Flex, MainAxisSize,
        ParentElement,
    },
    keymap::Keystroke,
    AppContext, Element, Entity, EntityId, ModelHandle, SingletonEntity, TypedActionView, View,
    ViewContext, ViewHandle,
};

use crate::{
    ai::blocklist::{agent_view::agent_view_bg_fill, block::cli_controller::CLISubagentEvent},
    cmd_or_ctrl_shift,
    settings::{AISettings, AISettingsChangedEvent, InputModeSettings},
    terminal::{
        model_events::{ModelEvent, ModelEventDispatcher},
        TerminalModel,
    },
    ui_components::{blended_colors, icons::Icon},
    view_components::action_button::{
        ActionButton, ActionButtonTheme, ButtonSize, KeystrokeSource, TooltipAlignment,
    },
};

use super::{RichContentInsertionPosition, TerminalAction, TerminalView};
use crate::terminal::view::block_banner::WarpificationMode;

static USE_AGENT_KEYSTROKE: LazyLock<Keystroke> =
    LazyLock::new(|| Keystroke::parse(cmd_or_ctrl_shift("enter")).expect("valid keystroke"));

impl TerminalView {
    pub(super) fn register_subscriptions_for_use_agent_footer(
        &mut self,
        ctx: &mut ViewContext<Self>,
    ) {
        let ai_settings = AISettings::handle(ctx);
        ctx.subscribe_to_model(&ai_settings, |me, _, event, ctx| match event {
            AISettingsChangedEvent::IsAnyAIEnabled { .. } => {
                me.maybe_show_use_agent_footer_in_blocklist(ctx);
            }
            AISettingsChangedEvent::ShouldRenderUseAgentToolbarForUserCommands { .. } => {
                // When the setting is re-enabled (e.g. from the AI settings page),
                // reset the pane-scoped dismissal so the footer can reappear.
                if *AISettings::as_ref(ctx)
                    .should_render_use_agent_footer_for_user_commands
                    .value()
                {
                    me.use_agent_footer.update(ctx, |footer, _| {
                        footer.did_user_dismiss = false;
                    });
                }
                me.maybe_show_use_agent_footer_in_blocklist(ctx);
            }
            _ => (),
        });

        ctx.subscribe_to_view(&self.use_agent_footer, |me, _, event, ctx| {
            me.handle_use_agent_footer_event(event, ctx);
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
                me.maybe_show_use_agent_footer_in_blocklist(ctx);
            }
        });

        ctx.subscribe_to_model(
            &self.cli_subagent_controller,
            |me, _, event, ctx| match event {
                CLISubagentEvent::SpawnedSubagent { .. } => {
                    me.hide_use_agent_footer_in_blocklist(ctx);
                }
                CLISubagentEvent::UpdatedControl { .. } => {
                    me.maybe_show_use_agent_footer_in_blocklist(ctx);
                }
                _ => (),
            },
        );
    }

    fn handle_use_agent_footer_event(
        &mut self,
        event: &UseAgentToolbarEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            UseAgentToolbarEvent::Dismiss => {
                self.hide_use_agent_footer_in_blocklist(ctx);
                ctx.notify();
            }
            UseAgentToolbarEvent::Warpify { mode } => {
                self.hide_use_agent_footer_in_blocklist(ctx);
                match mode {
                    WarpificationMode::Ssh { .. } => {
                        self.handle_action(&TerminalAction::WarpifySSHSession, ctx);
                    }
                    WarpificationMode::Subshell { .. } => {
                        self.handle_action(&TerminalAction::TriggerSubshellBootstrap, ctx);
                    }
                }
            }
            UseAgentToolbarEvent::UseAgent => {
                self.hide_use_agent_footer_in_blocklist(ctx);
                self.handle_action(&TerminalAction::SetInputModeAgent, ctx);
            }
        }
    }

    /// Checks if the footer should be rendered.
    pub(super) fn should_render_use_agent_footer(
        &self,
        model: &TerminalModel,
        app: &AppContext,
    ) -> bool {
        let ai_settings = AISettings::as_ref(app);

        // If a warpify mode is set, that means ssh or subshell is detected and we should show the footer.
        if self
            .use_agent_footer
            .as_ref(app)
            .warpify_mode(app)
            .is_some()
        {
            return true;
        }

        let active_block = model.block_list().active_block();
        if CLIAgentSessionsModel::as_ref(app)
            .session(self.view_id)
            .is_some()
        {
            return false;
        }

        // All other footer variants require the global AI setting to be on.
        if !ai_settings.is_any_ai_enabled(app) {
            return false;
        }

        if !active_block.is_eligible_for_agent_handoff() {
            // For regular commands (not agent handoff), check the "Use Agent" footer setting.
            // Agent handoff blocks always show the footer regardless of this setting.
            let is_user_command = active_block.requested_command_action_id().is_none();
            if is_user_command
                && (self.use_agent_footer.as_ref(app).did_user_dismiss()
                    || !*ai_settings.should_render_use_agent_footer_for_user_commands)
            {
                return false;
            }
        }

        !self.is_input_box_visible(model, app)
            && (active_block.is_eligible_to_tag_in_agent()
                || active_block.is_eligible_for_agent_handoff())
    }

    /// Returns the detected CLI agent for the active block's command, if any.
    ///
    /// This method resolves aliases before detecting the CLI agent. For example,
    /// if a user has aliased `foo` to `claude`, running `foo` will detect Claude.
    pub(super) fn detect_cli_agent_from_model(
        &self,
        model: &TerminalModel,
        ctx: &AppContext,
    ) -> Option<(CLIAgent, Option<String>)> {
        let active_block = model.block_list().active_block();

        if !active_block.is_active_and_long_running() {
            return None;
        }

        let command = active_block.command_with_secrets_obfuscated(false);

        let detected = self.active_block_session_id().and_then(|session_id| {
            self.sessions.read(ctx, |sessions, _| {
                let session = sessions.get(session_id)?;
                CLIAgent::detect(
                    &command,
                    Some(session.shell_family().escape_char()),
                    Some(session.aliases()),
                    ctx,
                )
            })
        });

        detected.map(|agent| (agent, None))
    }

    /// Hides the agent input and re-shows the 'Use agent' footer at the bottom of the block.
    pub(super) fn tag_out_agent_for_user_long_running_command(
        &mut self,
        ctx: &mut ViewContext<Self>,
    ) {
        if !self
            .model
            .lock()
            .block_list()
            .active_block()
            .is_agent_tagged_in()
        {
            return;
        }

        self.model
            .lock()
            .block_list_mut()
            .active_block_mut()
            .set_is_agent_tagged_in(false);

        if !self.model.lock().is_alt_screen_active() {
            self.maybe_show_use_agent_footer_in_blocklist(ctx);
        }

        self.input.update(ctx, |input, ctx| {
            input.set_input_mode_terminal(false, ctx);
        });
        self.redetermine_terminal_focus(ctx);

        ctx.notify();
    }

    pub(super) fn maybe_show_use_agent_footer_in_blocklist(&mut self, ctx: &mut ViewContext<Self>) {
        // This is a bit of a hack- but it ensures we never show more than one footer in the
        // blocklist.
        self.hide_use_agent_footer_in_blocklist(ctx);
        let (should_render_footer, is_alt_screen_active) = {
            let model = self.model.lock();
            (
                self.should_render_use_agent_footer(&model, ctx),
                model.is_alt_screen_active(),
            )
        };
        if is_alt_screen_active || !should_render_footer {
            return;
        }

        let should_insert_after_block = !InputModeSettings::as_ref(ctx).is_pinned_to_top();

        self.insert_rich_content(
            None,
            self.use_agent_footer.clone(),
            None,
            RichContentInsertionPosition::Append {
                insert_below_long_running_block: should_insert_after_block,
            },
            ctx,
        );
    }

    pub(super) fn hide_use_agent_footer_in_blocklist(&mut self, ctx: &mut ViewContext<Self>) {
        let mut model = self.model.lock();
        let block_list = model.block_list_mut();
        block_list.remove_rich_content(self.use_agent_footer.id());
        ctx.notify();
    }
}

/// Footer rendered at the bottom of the active long running block or alt screen element.
///
/// For regular commands, displays a 'Use agent' keystroke button to enter agent mode.
pub struct UseAgentToolbar {
    terminal_view_id: EntityId,
    terminal_model: Arc<FairMutex<TerminalModel>>,

    // Standard "Use agent" UI
    button: ViewHandle<ActionButton>,
    give_control_back_button: ViewHandle<ActionButton>,
    dismiss_button: ViewHandle<ActionButton>,
    dont_show_again_button: ViewHandle<ActionButton>,

    // Warpify footer UI (shown when a subshell/SSH command is detected).
    warpify_footer_view: ViewHandle<WarpifyFooterView>,

    // `true` if the user has dismissed the footer.
    //
    // Footer dismissal is terminal pane-scoped, e.g. dismissal hides the footer for this
    // specific terminal pane for the lifetime of the pane.
    did_user_dismiss: bool,
}

impl UseAgentToolbar {
    pub(crate) fn new(
        terminal_view_id: EntityId,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        model_event_dispatcher: &ModelHandle<ModelEventDispatcher>,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        let button_size = ButtonSize::XSmall;

        let button = ctx.add_typed_action_view(|ctx| {
            ActionButton::new(
                "Use agent",
                AgentFooterButtonTheme::new(Some(terminal_model.clone())),
            )
            .with_icon(Icon::AgentMode)
            .with_keybinding(KeystrokeSource::Fixed(USE_AGENT_KEYSTROKE.clone()), ctx)
            .with_size(button_size)
            .with_tooltip("Ask the agent to assist")
            .with_tooltip_alignment(TooltipAlignment::Left)
            .on_click(|ctx| {
                ctx.dispatch_typed_action(TerminalAction::SetInputModeAgent);
            })
        });
        let give_control_back_button = ctx.add_typed_action_view(|ctx| {
            ActionButton::new(
                "Give control back to agent",
                AgentFooterButtonTheme::new(Some(terminal_model.clone())),
            )
            .with_icon(Icon::AgentMode)
            .with_keybinding(KeystrokeSource::Fixed(USE_AGENT_KEYSTROKE.clone()), ctx)
            .with_size(button_size)
            .with_tooltip("Ask the agent to resume")
            .with_tooltip_alignment(TooltipAlignment::Left)
            .on_click(|ctx| {
                ctx.dispatch_typed_action(TerminalAction::SetInputModeAgent);
            })
        });
        let dismiss_button = ctx.add_typed_action_view(|_| {
            ActionButton::new(
                "Dismiss",
                AgentFooterButtonTheme::new(Some(terminal_model.clone())),
            )
            .on_click(|ctx| {
                ctx.dispatch_typed_action(UseAgentToolbarAction::Dismiss { permanently: false });
            })
            .with_size(button_size)
        });
        let dont_show_again_button = ctx.add_typed_action_view(|_| {
            ActionButton::new(
                "Don't show again",
                AgentFooterButtonTheme::new(Some(terminal_model.clone())),
            )
            .on_click(|ctx| {
                ctx.dispatch_typed_action(UseAgentToolbarAction::Dismiss { permanently: true });
            })
            .with_size(button_size)
        });

        let warpify_footer_view =
            ctx.add_typed_action_view(|ctx| WarpifyFooterView::new(terminal_model.clone(), ctx));

        ctx.subscribe_to_view(&warpify_footer_view, |me, _, event, ctx| {
            me.handle_warpify_footer_event(event, ctx);
        });

        ctx.subscribe_to_model(model_event_dispatcher, |me, _, event, ctx| {
            if let ModelEvent::TerminalModeSwapped(..) = event {
                me.notify_and_notify_children(ctx);
            }
        });

        // Re-render when the CLI agent session state changes (e.g. status updates
        // from the plugin, session started/ended).
        let cli_agent_sessions = CLIAgentSessionsModel::handle(ctx);
        ctx.subscribe_to_model(&cli_agent_sessions, move |me, _, event, ctx| {
            if event.terminal_view_id() != terminal_view_id {
                return;
            }
            me.notify_and_notify_children(ctx);
        });

        Self {
            terminal_view_id,
            button,
            give_control_back_button,
            dismiss_button,
            dont_show_again_button,
            warpify_footer_view,
            terminal_model,
            did_user_dismiss: false,
        }
    }

    fn handle_warpify_footer_event(
        &mut self,
        event: &WarpifyFooterViewEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            WarpifyFooterViewEvent::Warpify { mode } => {
                ctx.emit(UseAgentToolbarEvent::Warpify { mode: mode.clone() });
            }
            WarpifyFooterViewEvent::UseAgent => {
                ctx.emit(UseAgentToolbarEvent::UseAgent);
            }
            WarpifyFooterViewEvent::Dismiss => {
                ctx.emit(UseAgentToolbarEvent::Dismiss);
            }
        }
    }

    pub(in crate::terminal) fn notify_and_notify_children(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.notify();
        self.warpify_footer_view.update(ctx, |_, ctx| ctx.notify());
        self.button.update(ctx, |_, ctx| ctx.notify());
        self.give_control_back_button
            .update(ctx, |_, ctx| ctx.notify());
        self.dismiss_button.update(ctx, |_, ctx| ctx.notify());
        self.dont_show_again_button
            .update(ctx, |_, ctx| ctx.notify());
    }

    /// Returns whether the user has dismissed this footer.
    pub fn did_user_dismiss(&self) -> bool {
        self.did_user_dismiss
    }

    fn cli_agent(&self, app: &AppContext) -> Option<CLIAgent> {
        CLIAgentSessionsModel::as_ref(app)
            .session(self.terminal_view_id)
            .map(|session| session.agent)
    }

    /// Sets the current warpification mode. When set, the footer shows the
    /// warpify view instead of the regular "Use agent" view.
    pub(in crate::terminal) fn set_warpify_mode(
        &mut self,
        mode: WarpificationMode,
        ctx: &mut ViewContext<Self>,
    ) {
        self.warpify_footer_view.update(ctx, |view, ctx| {
            view.set_mode(mode, ctx);
        });
        ctx.notify();
    }

    /// Clears the warpification mode so the footer reverts to its default behavior.
    pub(in crate::terminal) fn clear_warpify_mode(&mut self, ctx: &mut ViewContext<Self>) {
        self.warpify_footer_view.update(ctx, |view, ctx| {
            view.clear_mode(ctx);
        });
        ctx.notify();
    }

    /// Returns the current warpification mode, if set.
    pub(in crate::terminal) fn warpify_mode(&self, app: &AppContext) -> Option<WarpificationMode> {
        self.warpify_footer_view.as_ref(app).mode().cloned()
    }
}

/// Events emitted by UseAgentToolbar.
pub enum UseAgentToolbarEvent {
    /// The footer was dismissed.
    Dismiss,
    /// User chose to warpify the subshell/SSH session.
    Warpify { mode: WarpificationMode },
    /// User chose to use the agent.
    UseAgent,
}

impl Entity for UseAgentToolbar {
    type Event = UseAgentToolbarEvent;
}

impl View for UseAgentToolbar {
    fn ui_name() -> &'static str {
        "UseAgentToolbar"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        // If a warpify mode is set, delegate rendering to the warpify footer view.
        if self.warpify_footer_view.as_ref(app).mode().is_some() {
            return ChildView::new(&self.warpify_footer_view).finish();
        }

        if self.cli_agent(app).is_some() {
            return Empty::new().finish();
        }

        let terminal_model = self.terminal_model.lock();

        let active_block = terminal_model.block_list().active_block();
        let show_give_control_back_button = active_block.is_eligible_for_agent_handoff();
        let show_dismiss_actions = active_block.requested_command_action_id().is_none();

        let mut button_row = Flex::row()
            .with_spacing(4.)
            .with_main_axis_size(MainAxisSize::Max)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                ChildView::new(if show_give_control_back_button {
                    &self.give_control_back_button
                } else {
                    &self.button
                })
                .finish(),
            );

        if show_dismiss_actions {
            button_row = button_row
                .with_child(Expanded::new(1., Empty::new().finish()).finish())
                .with_child(ChildView::new(&self.dismiss_button).finish());

            if !show_give_control_back_button {
                button_row =
                    button_row.with_child(ChildView::new(&self.dont_show_again_button).finish());
            }
        }

        let mut container = Container::new(button_row.finish())
            .with_horizontal_padding(*super::PADDING_LEFT)
            .with_vertical_padding(4.);

        if terminal_model.is_alt_screen_active() {
            if let Some(bg_color) = terminal_model.alt_screen().inferred_bg_color() {
                container = container.with_background(bg_color);
            }
        } else if terminal_model.block_list().agent_view_state().is_inline() {
            container = container.with_background(agent_view_bg_fill(app));
        }

        container.finish()
    }
}

#[derive(Debug, Clone)]
pub enum UseAgentToolbarAction {
    Dismiss { permanently: bool },
}

impl TypedActionView for UseAgentToolbar {
    type Action = UseAgentToolbarAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        let UseAgentToolbarAction::Dismiss { permanently } = action;
        self.did_user_dismiss = true;
        ctx.emit(UseAgentToolbarEvent::Dismiss);

        if *permanently {
            AISettings::handle(ctx).update(ctx, |settings, ctx| {
                if let Err(_e) = settings
                    .should_render_use_agent_footer_for_user_commands
                    .set_value(false, ctx)
                {}
            });
        }

        ctx.notify();
    }
}

#[derive(Clone)]
pub(super) struct AgentFooterButtonTheme {
    /// When set, enables alt-screen contrast adjustment for text and border.
    terminal_model: Option<Arc<FairMutex<TerminalModel>>>,
}

impl AgentFooterButtonTheme {
    pub fn new(terminal_model: Option<Arc<FairMutex<TerminalModel>>>) -> Self {
        Self { terminal_model }
    }

    /// Returns the inferred background colour of the alt screen, if active.
    fn inferred_alt_screen_bg(&self) -> Option<ColorU> {
        let terminal_model = self.terminal_model.as_ref()?;
        let terminal_model = terminal_model.lock();
        terminal_model
            .is_alt_screen_active()
            .then(|| terminal_model.alt_screen().inferred_bg_color())
            .flatten()
    }

    /// Picks a colour that contrasts well against `bg`, choosing between two
    /// neutral candidates.
    fn contrast_adjusted_color(
        bg: ColorU,
        default: ColorU,
        contrast: MinimumAllowedContrast,
        appearance: &Appearance,
    ) -> ColorU {
        if high_enough_contrast(default, bg, contrast) {
            default
        } else {
            pick_best_foreground_color(
                bg,
                blended_colors::neutral_2(appearance.theme()),
                blended_colors::neutral_6(appearance.theme()),
                contrast,
            )
        }
    }
}

impl ActionButtonTheme for AgentFooterButtonTheme {
    fn background(&self, hovered: bool, appearance: &Appearance) -> Option<ThemeFill> {
        if hovered {
            Some(internal_colors::fg_overlay_2(appearance.theme()))
        } else {
            None
        }
    }

    fn border(&self, appearance: &Appearance) -> Option<ColorU> {
        let color = appearance.theme().outline().into_solid();
        if let Some(bg) = self.inferred_alt_screen_bg() {
            return Some(Self::contrast_adjusted_color(
                bg,
                color,
                MinimumAllowedContrast::NonText,
                appearance,
            ));
        }
        Some(color)
    }

    fn text_color(
        &self,
        _hovered: bool,
        _background: Option<ThemeFill>,
        appearance: &Appearance,
    ) -> ColorU {
        let color = appearance
            .theme()
            .sub_text_color(appearance.theme().surface_1())
            .into_solid();

        // If rendered in the alt screen, the footer is rendered with the inferred background color
        // of the alt screen output grid (if there is one). In such cases, we have to ensure that
        // the text within the footer is high-contrast enough to be legible, since the background
        // color can essentially be anything.
        if let Some(bg) = self.inferred_alt_screen_bg() {
            return Self::contrast_adjusted_color(
                bg,
                color,
                MinimumAllowedContrast::Text,
                appearance,
            );
        }
        color
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
