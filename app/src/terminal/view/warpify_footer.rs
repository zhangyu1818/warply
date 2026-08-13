use std::sync::Arc;

use parking_lot::FairMutex;
use pathfinder_color::ColorU;
use warp_core::ui::{
    appearance::Appearance,
    color::contrast::{MinimumAllowedContrast, high_enough_contrast, pick_best_foreground_color},
    theme::{Fill as ThemeFill, color::internal_colors},
};
use warpui::{
    AppContext, Element, Entity, ModelHandle, SingletonEntity, TypedActionView, View, ViewContext,
    ViewHandle,
    elements::{
        ChildView, Container, CrossAxisAlignment, Empty, Expanded, Flex, MainAxisSize,
        ParentElement,
    },
};

use crate::{
    settings::InputModeSettings,
    terminal::{
        TerminalModel,
        model_events::{ModelEvent, ModelEventDispatcher},
    },
    ui_components::{blended_colors, icons::Icon},
    view_components::action_button::{
        ActionButton, ActionButtonTheme, ButtonSize, KeystrokeSource, TooltipAlignment,
    },
};

use super::{RichContentInsertionPosition, TerminalAction, TerminalView};
use crate::terminal::view::block_banner::WarpificationMode;

impl TerminalView {
    pub(super) fn register_subscriptions_for_warpify_footer(
        &mut self,
        ctx: &mut ViewContext<Self>,
    ) {
        ctx.subscribe_to_view(&self.warpify_footer, |me, _, event, ctx| {
            me.handle_warpify_footer_event(event, ctx);
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
                me.maybe_show_warpify_footer_in_blocklist(ctx);
            }
        });
    }

    fn handle_warpify_footer_event(
        &mut self,
        event: &WarpifyFooterEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            WarpifyFooterEvent::Dismiss => {
                self.hide_warpify_footer_in_blocklist(ctx);
                ctx.notify();
            }
            WarpifyFooterEvent::Warpify { mode } => {
                self.hide_warpify_footer_in_blocklist(ctx);
                match mode {
                    WarpificationMode::Ssh { .. } => {
                        self.handle_action(&TerminalAction::WarpifySSHSession, ctx);
                    }
                    WarpificationMode::Subshell { .. } => {
                        self.handle_action(&TerminalAction::TriggerSubshellBootstrap, ctx);
                    }
                }
            }
        }
    }

    pub(super) fn should_render_warpify_footer(
        &self,
        _model: &TerminalModel,
        app: &AppContext,
    ) -> bool {
        self.warpify_footer.as_ref(app).mode(app).is_some()
    }

    pub(super) fn maybe_show_warpify_footer_in_blocklist(&mut self, ctx: &mut ViewContext<Self>) {
        self.hide_warpify_footer_in_blocklist(ctx);
        let (should_render_footer, is_alt_screen_active) = {
            let model = self.model.lock();
            (
                self.should_render_warpify_footer(&model, ctx),
                model.is_alt_screen_active(),
            )
        };
        if is_alt_screen_active || !should_render_footer {
            return;
        }

        let should_insert_after_block = !InputModeSettings::as_ref(ctx).is_pinned_to_top();

        self.insert_rich_content(
            None,
            self.warpify_footer.clone(),
            None,
            RichContentInsertionPosition::Append {
                insert_below_long_running_block: should_insert_after_block,
            },
            ctx,
        );
    }

    pub(super) fn hide_warpify_footer_in_blocklist(&mut self, ctx: &mut ViewContext<Self>) {
        let mut model = self.model.lock();
        let block_list = model.block_list_mut();
        block_list.remove_rich_content(self.warpify_footer.id());
        ctx.notify();
    }
}

pub(super) struct WarpifyFooter {
    terminal_model: Arc<FairMutex<TerminalModel>>,
    warpify_button: ViewHandle<ActionButton>,
    dismiss_button: ViewHandle<ActionButton>,
    mode: Option<WarpificationMode>,
}

impl WarpifyFooter {
    pub(crate) fn new(
        terminal_model: Arc<FairMutex<TerminalModel>>,
        model_event_dispatcher: &ModelHandle<ModelEventDispatcher>,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        let button_size = ButtonSize::XSmall;

        let warpify_button = ctx.add_typed_action_view(|_ctx| {
            ActionButton::new("Warpify subshell", WarpifyFooterButtonTheme::new(None))
                .with_icon(Icon::Warp)
                .with_size(button_size)
                .with_tooltip("Enable Warp shell integration in this session")
                .with_tooltip_alignment(TooltipAlignment::Left)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(WarpifyFooterAction::Warpify);
                })
        });

        let dismiss_button = ctx.add_typed_action_view(|_ctx| {
            ActionButton::new("Dismiss", WarpifyFooterButtonTheme::new(None))
                .with_size(button_size)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(WarpifyFooterAction::Dismiss);
                })
        });

        ctx.subscribe_to_model(model_event_dispatcher, |me, _, event, ctx| {
            if let ModelEvent::TerminalModeSwapped(..) = event {
                me.notify_and_notify_children(ctx);
            }
        });

        Self {
            terminal_model,
            warpify_button,
            dismiss_button,
            mode: None,
        }
    }

    pub(in crate::terminal) fn notify_and_notify_children(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.notify();
        self.warpify_button.update(ctx, |_, ctx| ctx.notify());
        self.dismiss_button.update(ctx, |_, ctx| ctx.notify());
    }

    pub(in crate::terminal) fn set_mode(
        &mut self,
        mode: WarpificationMode,
        ctx: &mut ViewContext<Self>,
    ) {
        let (label, binding_name) = match mode {
            WarpificationMode::Ssh { .. } => {
                ("Warpify SSH session", "terminal:warpify_ssh_session")
            }
            WarpificationMode::Subshell { .. } => ("Warpify subshell", "terminal:warpify_subshell"),
        };
        self.warpify_button.update(ctx, |button, ctx| {
            button.set_label(label, ctx);
            button.set_keybinding(Some(KeystrokeSource::Binding(binding_name)), ctx);
        });
        self.mode = Some(mode);
        ctx.notify();
    }

    pub(in crate::terminal) fn clear_mode(&mut self, ctx: &mut ViewContext<Self>) {
        self.mode = None;
        self.warpify_button.update(ctx, |button, ctx| {
            button.set_keybinding(None, ctx);
        });
        ctx.notify();
    }

    pub(in crate::terminal) fn mode(&self, _app: &AppContext) -> Option<WarpificationMode> {
        self.mode.clone()
    }
}

pub(super) enum WarpifyFooterEvent {
    Dismiss,
    Warpify { mode: WarpificationMode },
}

impl Entity for WarpifyFooter {
    type Event = WarpifyFooterEvent;
}

impl View for WarpifyFooter {
    fn ui_name() -> &'static str {
        "WarpifyFooter"
    }

    fn render(&self, _app: &AppContext) -> Box<dyn Element> {
        if self.mode.is_none() {
            return Empty::new().finish();
        }

        let terminal_model = self.terminal_model.lock();

        let button_row = Flex::row()
            .with_spacing(4.)
            .with_main_axis_size(MainAxisSize::Max)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(ChildView::new(&self.warpify_button).finish())
            .with_child(Expanded::new(1., Empty::new().finish()).finish())
            .with_child(ChildView::new(&self.dismiss_button).finish());

        let mut container = Container::new(button_row.finish())
            .with_horizontal_padding(*super::PADDING_LEFT)
            .with_vertical_padding(4.);

        if terminal_model.is_alt_screen_active() {
            if let Some(bg_color) = terminal_model.alt_screen().inferred_bg_color() {
                container = container.with_background(bg_color);
            }
        }

        container.finish()
    }
}

#[derive(Debug, Clone)]
pub(super) enum WarpifyFooterAction {
    Warpify,
    Dismiss,
}

impl TypedActionView for WarpifyFooter {
    type Action = WarpifyFooterAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            WarpifyFooterAction::Warpify => {
                if let Some(mode) = self.mode.clone() {
                    self.clear_mode(ctx);
                    ctx.emit(WarpifyFooterEvent::Warpify { mode });
                }
            }
            WarpifyFooterAction::Dismiss => {
                self.clear_mode(ctx);
                ctx.emit(WarpifyFooterEvent::Dismiss);
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct WarpifyFooterButtonTheme {
    terminal_model: Option<Arc<FairMutex<TerminalModel>>>,
}

impl WarpifyFooterButtonTheme {
    pub fn new(terminal_model: Option<Arc<FairMutex<TerminalModel>>>) -> Self {
        Self { terminal_model }
    }

    fn inferred_alt_screen_bg(&self) -> Option<ColorU> {
        let terminal_model = self.terminal_model.as_ref()?;
        let terminal_model = terminal_model.lock();
        terminal_model
            .is_alt_screen_active()
            .then(|| terminal_model.alt_screen().inferred_bg_color())
            .flatten()
    }

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

impl ActionButtonTheme for WarpifyFooterButtonTheme {
    fn background(&self, hovered: bool, appearance: &Appearance) -> Option<ThemeFill> {
        hovered.then(|| internal_colors::fg_overlay_2(appearance.theme()))
    }

    fn border(&self, appearance: &Appearance) -> Option<ColorU> {
        let color = blended_colors::neutral_3(appearance.theme());
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
