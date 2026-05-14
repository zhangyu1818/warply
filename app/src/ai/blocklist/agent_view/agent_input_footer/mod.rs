pub(super) mod chips;
pub mod editor;
pub mod toolbar_item;

use crate::{
    ai::{
        acp::{model::AcpAgentModel, AcpSessionState},
        blocklist::history_model::{BlocklistAIHistoryEvent, BlocklistAIHistoryModel},
    },
    appearance::Appearance,
    completer::SessionContext,
    context_chips::{
        self,
        display_chip::{render_udi_chip, DisplayChip, DisplayChipConfig, UdiChipConfig},
        prompt_type::PromptType,
        ContextChipKind,
    },
    features::FeatureFlag,
    terminal::{
        cli_agent_sessions::{
            CLIAgentInputState, CLIAgentSessionsModel, CLIAgentSessionsModelEvent,
        },
        session_settings::{SessionSettings, SessionSettingsChangedEvent, ToolbarChipSelection},
        view::init::OPEN_CLI_AGENT_RICH_INPUT_KEYBINDING,
        CLIAgent, TerminalModel,
    },
    ui_components::icons::Icon,
    view_components::{
        action_button::{
            ActionButton, ActionButtonTheme, ButtonSize, KeystrokeSource, TooltipAlignment,
        },
        DismissibleToast,
    },
    workspace::{view::TOGGLE_PROJECT_EXPLORER_BINDING_NAME, ToastStack},
};
use std::sync::Arc;
use toolbar_item::AgentToolbarItemKind;

use ai::document::{AIDocumentId, AIDocumentVersion};
use parking_lot::FairMutex;
use pathfinder_color::ColorU;
use pathfinder_geometry::vector::Vector2F;

use warp_core::ui::{
    color::{blend::Blend, contrast::MinimumAllowedContrast, ContrastingColor},
    theme::{color::internal_colors, Fill},
};
use warpui::{
    elements::{
        ChildView, ConstrainedBox, Container, CrossAxisAlignment, DispatchEventResult, Element,
        EventHandler, Flex, MainAxisAlignment, MainAxisSize, ParentElement, Wrap, WrapFill,
        WrapFillEntireRun,
    },
    AppContext, Entity, EntityId, ModelHandle, SingletonEntity, TypedActionView, View, ViewContext,
    ViewHandle,
};

/// Footer control bar at the bottom of the agent input.
///
/// Renders in two modes:
/// - **Agent View mode** (default): local controls and context chips.
/// - **CLI agent mode**: agent icon, image, mic, file explorer, view changes, rich input.
///
/// The mode is determined by reading `CLIAgentSessionsModel` at render time.
/// A single `ViewHandle<AgentInputFooter>` is shared between `Input` and
/// `UseAgentToolbar`, rendering the appropriate mode in each context.
pub struct AgentInputFooter {
    terminal_view_id: EntityId,
    file_button: ViewHandle<ActionButton>,
    left_display_chips: Vec<ViewHandle<DisplayChip>>,
    right_display_chips: Vec<ViewHandle<DisplayChip>>,
    // Separate set of display chips for the CLI agent footer.
    // Needed because the CLI footer chip selection can include chips not present in the agent view selection.
    cli_display_chips: Vec<ViewHandle<DisplayChip>>,
    display_chip_config: DisplayChipConfig,

    terminal_model: Arc<FairMutex<TerminalModel>>,

    // CLI agent-specific buttons (rendered when a CLI agent session is active).
    file_explorer_button: ViewHandle<ActionButton>,
    rich_input_button: ViewHandle<ActionButton>,
}

impl AgentInputFooter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        terminal_view_id: EntityId,
        terminal_model: Arc<FairMutex<TerminalModel>>,
        prompt: ModelHandle<PromptType>,
        display_chip_config: DisplayChipConfig,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        let button_size = ButtonSize::AgentInputButton;

        let file_button = ctx.add_typed_action_view(|_ctx| {
            ActionButton::new("", AgentInputButtonTheme)
                .with_icon(Icon::Plus)
                .with_tooltip("Attach file")
                .with_size(button_size)
                .with_tooltip_alignment(TooltipAlignment::Left)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(AgentInputFooterAction::SelectFile);
                })
        });

        // CLI agent-specific buttons (only rendered when a CLI agent session is active).
        let cli_button_size = ButtonSize::AgentInputButton;
        let file_explorer_button = ctx.add_typed_action_view(|ctx| {
            ActionButton::new("File explorer", AgentInputButtonTheme)
                .with_icon(Icon::FileCopy)
                .with_tooltip("Open file explorer")
                .with_size(cli_button_size)
                .with_tooltip_alignment(TooltipAlignment::Left)
                .with_keybinding(
                    KeystrokeSource::Binding(TOGGLE_PROJECT_EXPLORER_BINDING_NAME),
                    ctx,
                )
                .with_compact_keybinding(true)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(AgentInputFooterAction::ToggleFileExplorer);
                })
        });
        let rich_input_button = ctx.add_typed_action_view(|ctx| {
            ActionButton::new("Rich Input", AgentInputButtonTheme)
                .with_icon(Icon::TextInput)
                .with_tooltip("Open Rich Input")
                .with_size(cli_button_size)
                .with_tooltip_alignment(TooltipAlignment::Left)
                .with_keybinding(
                    KeystrokeSource::Binding(OPEN_CLI_AGENT_RICH_INPUT_KEYBINDING),
                    ctx,
                )
                .with_compact_keybinding(true)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(AgentInputFooterAction::ToggleRichInput);
                })
        });
        // Toggle rich input button label when CLI input session opens/closes.
        ctx.subscribe_to_model(
            &CLIAgentSessionsModel::handle(ctx),
            move |me, _, event, ctx| {
                if event.terminal_view_id() != terminal_view_id {
                    return;
                }

                let CLIAgentSessionsModelEvent::InputSessionChanged {
                    new_input_state, ..
                } = event
                else {
                    ctx.notify();
                    return;
                };
                let is_open = matches!(new_input_state, CLIAgentInputState::Open { .. });
                me.rich_input_button.update(ctx, |button, ctx| {
                    if is_open {
                        button.set_label("Hide Rich Input", ctx);
                        button.set_tooltip(Some("Hide Rich Input"), ctx);
                        button.set_keybinding(
                            Some(KeystrokeSource::Binding(
                                OPEN_CLI_AGENT_RICH_INPUT_KEYBINDING,
                            )),
                            ctx,
                        );
                    } else {
                        button.set_label("Rich Input", ctx);
                        button.set_tooltip(Some("Open Rich Input"), ctx);
                        button.set_keybinding(
                            Some(KeystrokeSource::Binding(
                                OPEN_CLI_AGENT_RICH_INPUT_KEYBINDING,
                            )),
                            ctx,
                        );
                    }
                });
                ctx.notify();
            },
        );

        ctx.subscribe_to_model(&AcpAgentModel::handle(ctx), |_, _, _, ctx| ctx.notify());

        let prompt_for_session_settings = prompt.clone();
        ctx.subscribe_to_model(
            &SessionSettings::handle(ctx),
            move |me, _, event, ctx| match event {
                SessionSettingsChangedEvent::AgentToolbarChipSelectionSetting { .. }
                | SessionSettingsChangedEvent::CLIAgentToolbarChipSelectionSetting { .. } => {
                    me.update_display_chips(&prompt_for_session_settings, ctx);
                    ctx.notify();
                }
                _ => {}
            },
        );

        ctx.subscribe_to_model(
            &BlocklistAIHistoryModel::handle(ctx),
            |me, _, event, ctx| {
                if event
                    .terminal_view_id()
                    .is_some_and(|id| id != me.terminal_view_id)
                {
                    return;
                }
                match event {
                    BlocklistAIHistoryEvent::StartedNewConversation { .. }
                    | BlocklistAIHistoryEvent::SetActiveConversation { .. }
                    | BlocklistAIHistoryEvent::ClearedActiveConversation { .. }
                    | BlocklistAIHistoryEvent::ClearedConversationsInTerminalView { .. }
                    | BlocklistAIHistoryEvent::RemoveConversation { .. }
                    | BlocklistAIHistoryEvent::UpdatedAutoexecuteOverride { .. } => {
                        ctx.notify();
                    }
                    BlocklistAIHistoryEvent::UpdatedTodoList { .. }
                    | BlocklistAIHistoryEvent::UpdatedConversationStatus { .. }
                    | BlocklistAIHistoryEvent::AppendedExchange { .. }
                    | BlocklistAIHistoryEvent::UpdatedStreamingExchange { .. } => {
                        ctx.notify();
                    }
                    _ => (),
                }
            },
        );

        ctx.observe(&prompt, |me, model, ctx| {
            me.update_display_chips(&model, ctx);
        });

        let mut me = Self {
            terminal_view_id,
            file_button,
            file_explorer_button,
            rich_input_button,
            terminal_model,
            left_display_chips: vec![],
            right_display_chips: vec![],
            cli_display_chips: vec![],
            display_chip_config,
        };
        me.update_display_chips(&prompt, ctx);
        me
    }

    pub fn set_current_repo_path(
        &mut self,
        repo_path: Option<std::path::PathBuf>,
        ctx: &mut ViewContext<Self>,
    ) {
        self.display_chip_config.current_repo_path = repo_path;
        // Chips will be rebuilt on the next GitRepoStatusEvent::MetadataChanged.
        // Notify to ensure any existing chips reflect the change.
        ctx.notify();
    }

    fn all_display_chips(&self) -> impl Iterator<Item = &ViewHandle<DisplayChip>> {
        self.left_display_chips
            .iter()
            .chain(self.right_display_chips.iter())
            .chain(self.cli_display_chips.iter())
    }

    pub fn update_session_context(
        &mut self,
        session_context: Option<SessionContext>,
        ctx: &mut ViewContext<Self>,
    ) {
        self.display_chip_config.session_context = session_context.clone();
        for chip_view in self.all_display_chips() {
            chip_view.update(ctx, |chip, chip_ctx| {
                chip.update_session_context(session_context.clone(), chip_ctx);
            });
        }
    }

    fn has_active_cli_agent_input_session(&self, app: &AppContext) -> bool {
        CLIAgentSessionsModel::as_ref(app).is_input_open(self.terminal_view_id)
    }

    fn cli_agent(&self, app: &AppContext) -> Option<CLIAgent> {
        CLIAgentSessionsModel::as_ref(app)
            .session(self.terminal_view_id)
            .map(|session| session.agent)
    }

    fn is_cli_agent_session_active(&self, app: &AppContext) -> bool {
        CLIAgentSessionsModel::as_ref(app)
            .session(self.terminal_view_id)
            .is_some()
    }

    fn select_cli_file(&mut self, ctx: &mut ViewContext<Self>) {
        let window_id = ctx.window_id();
        let view_id = ctx.view_id();
        let file_picker_config = warpui::platform::FilePickerConfiguration::new();

        ctx.open_file_picker(
            move |result, ctx| match result {
                Ok(paths) => {
                    if let Some(path) = paths.first() {
                        ctx.dispatch_typed_action_for_view(
                            window_id,
                            view_id,
                            &AgentInputFooterAction::InsertFilePath(path.clone()),
                        );
                    }
                }
                Err(err) => {
                    let window_id = ctx.window_id();
                    ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                        toast_stack.add_ephemeral_toast(
                            DismissibleToast::error(format!("{err}")),
                            window_id,
                            ctx,
                        );
                    });
                }
            },
            file_picker_config,
        );
    }

    fn cli_display_chip(
        &self,
        chip_kind: ContextChipKind,
        app: &AppContext,
    ) -> Option<Box<dyn Element>> {
        self.cli_display_chips
            .iter()
            .find(|chip| chip.as_ref(app).chip_kind() == &chip_kind)
            .filter(|chip| chip.as_ref(app).should_render(app))
            .map(|chip| ChildView::new(chip).finish())
    }

    fn render_cli_toolbar_item(
        &self,
        item: &AgentToolbarItemKind,
        app: &AppContext,
    ) -> Option<Box<dyn Element>> {
        if !item.available_in().is_available_for_cli() {
            return None;
        }

        match item {
            AgentToolbarItemKind::ContextChip(chip_kind) => {
                self.cli_display_chip(chip_kind.clone(), app)
            }
            AgentToolbarItemKind::FileExplorer => {
                Some(ChildView::new(&self.file_explorer_button).finish())
            }
            AgentToolbarItemKind::RichInput => FeatureFlag::CLIAgentRichInput
                .is_enabled()
                .then(|| ChildView::new(&self.rich_input_button).finish()),
            AgentToolbarItemKind::FileAttach => Some(ChildView::new(&self.file_button).finish()),
        }
    }

    fn render_cli_mode_footer(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        let cli_icon_size = ButtonSize::AgentInputButton.icon_size(appearance, app);

        // Extract everything we need from the terminal model up front and drop
        // the lock before calling into helpers like `should_use_manual_mode`
        // and `render_cli_toolbar_item`, which may re-lock the same model and
        // would deadlock since the lock is non-reentrant.
        let background_color = {
            let terminal_model = self.terminal_model.lock();
            if terminal_model.is_alt_screen_active() {
                terminal_model
                    .alt_screen()
                    .inferred_bg_color()
                    .unwrap_or_else(|| appearance.theme().surface_1().into_solid())
            } else {
                appearance.theme().surface_1().into_solid()
            }
        };

        let session_settings = SessionSettings::as_ref(app);
        let left_items = session_settings
            .cli_agent_footer_chip_selection
            .left_items();
        let right_items = session_settings
            .cli_agent_footer_chip_selection
            .right_items();

        let mut left_buttons = Wrap::row()
            .with_main_axis_size(MainAxisSize::Min)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_main_axis_alignment(MainAxisAlignment::Start)
            .with_run_spacing(4.)
            .with_spacing(4.);

        // CLI agent brand icon is always rendered (not configurable).
        if let Some(agent) = self.cli_agent(app) {
            if let Some(icon) = agent.icon() {
                let icon_color = agent
                    .brand_color()
                    .map(|c| c.on_background(background_color, MinimumAllowedContrast::NonText))
                    .unwrap_or_else(|| appearance.theme().foreground().into_solid());
                left_buttons.add_child(
                    Container::new(
                        ConstrainedBox::new(icon.to_warpui_icon(Fill::Solid(icon_color)).finish())
                            .with_width(cli_icon_size)
                            .with_height(cli_icon_size)
                            .finish(),
                    )
                    .with_padding_right(8.)
                    .finish(),
                );
            }
        }

        for item in &left_items {
            if let Some(element) = self.render_cli_toolbar_item(item, app) {
                left_buttons.add_child(element);
            }
        }

        let mut right_buttons = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_main_axis_size(MainAxisSize::Min)
            .with_spacing(4.);

        for item in &right_items {
            if let Some(element) = self.render_cli_toolbar_item(item, app) {
                right_buttons.add_child(element);
            }
        }

        let content = Wrap::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(WrapFillEntireRun::new(left_buttons.finish()).finish())
            .with_child(WrapFill::new(0., right_buttons.finish()).finish())
            .with_run_spacing(context_chips::spacing::UDI_ROW_RUN_SPACING)
            .finish();
        let content = EventHandler::new(content)
            .on_right_mouse_down(|ctx, _, position| {
                ctx.dispatch_typed_action(AgentInputFooterAction::ShowContextMenu { position });
                DispatchEventResult::StopPropagation
            })
            .finish();

        Container::new(content).with_vertical_padding(4.).finish()
    }

    pub fn has_open_chip_menu(&self, app: &AppContext) -> bool {
        let has_open_display_chip = self
            .all_display_chips()
            .any(|chip| chip.as_ref(app).display_chip_kind().has_open_menu());

        has_open_display_chip
    }

    fn current_acp_mode_label(&self, app: &AppContext) -> Option<String> {
        let conversation_id = BlocklistAIHistoryModel::as_ref(app)
            .active_conversation(self.terminal_view_id)?
            .id();
        let session_state = AcpAgentModel::as_ref(app).session_state(conversation_id)?;
        acp_current_mode_label(session_state)
    }

    fn render_acp_current_mode_chip(&self, app: &AppContext) -> Option<Box<dyn Element>> {
        let label = self.current_acp_mode_label(app)?;
        let appearance = Appearance::as_ref(app);
        let color = internal_colors::neutral_6(appearance.theme());
        Some(render_udi_chip(
            UdiChipConfig::new_with_icon(Icon::Sliders, color, label).for_agent_view(),
            appearance,
        ))
    }

    fn render_toolbar_item(
        &self,
        item: &AgentToolbarItemKind,
        app: &AppContext,
    ) -> Option<Box<dyn Element>> {
        if !item.is_local_agent_view_control() {
            return None;
        }

        if !item.available_in().is_available_for_agent_view() {
            return None;
        }
        match item {
            AgentToolbarItemKind::ContextChip(chip_kind) => {
                let chips = match SessionSettings::as_ref(app)
                    .agent_footer_chip_selection
                    .left_chips()
                    .contains(chip_kind)
                {
                    true => &self.left_display_chips,
                    false => &self.right_display_chips,
                };
                chips
                    .iter()
                    .find(|chip| chip.as_ref(app).chip_kind() == chip_kind)
                    .filter(|chip| chip.as_ref(app).should_render(app))
                    .map(|chip| ChildView::new(chip).finish())
            }
            AgentToolbarItemKind::FileAttach => Some(ChildView::new(&self.file_button).finish()),
            AgentToolbarItemKind::FileExplorer | AgentToolbarItemKind::RichInput => None,
        }
    }

    #[cfg(test)]
    pub fn displayed_chip_kinds(
        &self,
        app: &AppContext,
    ) -> (
        Vec<crate::context_chips::ContextChipKind>,
        Vec<crate::context_chips::ContextChipKind>,
    ) {
        let collect_chip_kinds = |chips: &[ViewHandle<DisplayChip>]| {
            chips
                .iter()
                .map(|chip| chip.as_ref(app).chip_kind().clone())
                .collect()
        };

        (
            collect_chip_kinds(&self.left_display_chips),
            collect_chip_kinds(&self.right_display_chips),
        )
    }

    #[cfg(test)]
    pub fn cli_display_chip_kinds(
        &self,
        app: &AppContext,
    ) -> Vec<crate::context_chips::ContextChipKind> {
        self.cli_display_chips
            .iter()
            .map(|chip| chip.as_ref(app).chip_kind().clone())
            .collect()
    }
}

fn acp_current_mode_label(session_state: &AcpSessionState) -> Option<String> {
    let mode_id = session_state
        .config
        .current_mode
        .as_ref()?
        .current_mode_id
        .0
        .as_ref()
        .trim();
    (!mode_id.is_empty()).then(|| mode_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_client_protocol::schema::CurrentModeUpdate;

    #[test]
    fn acp_current_mode_label_uses_current_mode_update() {
        let mut state = AcpSessionState::default();

        assert_eq!(acp_current_mode_label(&state), None);

        state.config = state
            .config
            .clone()
            .with_current_mode(CurrentModeUpdate::new("plan"));
        assert_eq!(acp_current_mode_label(&state).as_deref(), Some("plan"));

        state.config = state
            .config
            .clone()
            .with_current_mode(CurrentModeUpdate::new("   "));
        assert_eq!(acp_current_mode_label(&state), None);
    }
}

impl View for AgentInputFooter {
    fn ui_name() -> &'static str {
        "AgentViewFooter"
    }

    fn render(&self, app: &warpui::AppContext) -> Box<dyn warpui::Element> {
        // When a CLI agent session is active, render the CLI agent toolbar instead.
        if self.is_cli_agent_session_active(app) {
            return self.render_cli_mode_footer(app);
        }

        let session_settings = SessionSettings::as_ref(app);
        let left_items = session_settings.agent_footer_chip_selection.left_items();
        let right_items = session_settings.agent_footer_chip_selection.right_items();

        let mut left_buttons = Wrap::row()
            .with_main_axis_size(MainAxisSize::Min)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_main_axis_alignment(MainAxisAlignment::Start)
            .with_run_spacing(4.)
            .with_spacing(4.);

        if let Some(mode_chip) = self.render_acp_current_mode_chip(app) {
            left_buttons.add_child(mode_chip);
        }

        for item in &left_items {
            if let Some(element) = self.render_toolbar_item(item, app) {
                left_buttons.add_child(element);
            }
        }

        let mut right_buttons = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_main_axis_size(MainAxisSize::Min)
            .with_spacing(4.);

        for item in &right_items {
            if let Some(element) = self.render_toolbar_item(item, app) {
                right_buttons.add_child(element);
            }
        }

        let content = Wrap::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(WrapFill::new(0., left_buttons.finish()).finish())
            .with_child(WrapFill::new(0., right_buttons.finish()).finish())
            .with_run_spacing(context_chips::spacing::UDI_ROW_RUN_SPACING)
            .finish();
        let content = EventHandler::new(content)
            .on_right_mouse_down(|ctx, _, position| {
                ctx.dispatch_typed_action(AgentInputFooterAction::ShowContextMenu { position });
                DispatchEventResult::StopPropagation
            })
            .finish();

        Container::new(content)
            .with_padding_bottom(8.0)
            .with_padding_right(16.)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum AgentInputFooterAction {
    SelectFile,
    InsertFilePath(String),
    ToggleCodeReview,
    ToggleFileExplorer,
    ToggleRichInput,
    ShowContextMenu { position: Vector2F },
}

impl TypedActionView for AgentInputFooter {
    type Action = AgentInputFooterAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut warpui::ViewContext<Self>) {
        match action {
            AgentInputFooterAction::SelectFile => {
                // Fork based on CLI agent session: in CLI mode, open a file
                // picker and insert/write the path; in normal mode, use the
                // standard AI file attachment flow.
                if self.is_cli_agent_session_active(ctx) {
                    self.select_cli_file(ctx);
                } else {
                    ctx.emit(AgentInputFooterEvent::SelectFile);
                }
            }
            AgentInputFooterAction::InsertFilePath(path) => {
                if let Some(_agent) = self.cli_agent(ctx) {}
                let path_with_space = format!("{path} ");
                if self.has_active_cli_agent_input_session(ctx) {
                    ctx.emit(AgentInputFooterEvent::InsertIntoCLIRichInput(
                        path_with_space,
                    ));
                } else {
                    ctx.emit(AgentInputFooterEvent::WriteToPty(path_with_space));
                }
            }
            AgentInputFooterAction::ToggleCodeReview => {
                if let Some(agent) = self.cli_agent(ctx) {
                    ctx.emit(AgentInputFooterEvent::ToggleCodeReviewPane(agent));
                }
            }
            AgentInputFooterAction::ToggleFileExplorer => {
                if let Some(agent) = self.cli_agent(ctx) {
                    ctx.emit(AgentInputFooterEvent::ToggleFileExplorer(agent));
                }
            }
            AgentInputFooterAction::ToggleRichInput => {
                if self.has_active_cli_agent_input_session(ctx) {
                    ctx.emit(AgentInputFooterEvent::HideRichInput);
                } else {
                    ctx.emit(AgentInputFooterEvent::OpenRichInput);
                }
            }
            AgentInputFooterAction::ShowContextMenu { position } => {
                ctx.emit(AgentInputFooterEvent::ShowContextMenu {
                    position: *position,
                });
            }
        }
    }
}

pub enum AgentInputFooterEvent {
    SelectFile,
    WriteToPty(String),
    /// Insert text into the CLI agent rich input.
    InsertIntoCLIRichInput(String),
    ToggleCodeReviewPane(CLIAgent),
    ToggleFileExplorer(CLIAgent),
    OpenRichInput,
    HideRichInput,
    ToggledChipMenu {
        open: bool,
    },
    TryExecuteChipCommand(String),
    OpenCodeReview,
    OpenAIDocument {
        document_id: AIDocumentId,
        document_version: AIDocumentVersion,
    },
    ShowContextMenu {
        position: Vector2F,
    },
}

impl Entity for AgentInputFooter {
    type Event = AgentInputFooterEvent;
}

pub(crate) struct AgentInputButtonTheme;

impl ActionButtonTheme for AgentInputButtonTheme {
    fn background(&self, hovered: bool, appearance: &Appearance) -> Option<Fill> {
        // Solid surface fills keep the button readable even when its parent
        // isn't `theme.background()` (for example, over an alt-screen CLI agent).
        let theme = appearance.theme();
        Some(if hovered {
            theme.surface_2()
        } else {
            theme.surface_1()
        })
    }

    fn text_color(
        &self,
        _hovered: bool,
        background: Option<Fill>,
        appearance: &Appearance,
    ) -> ColorU {
        // If a caller overrides `background()` with a translucent fill, blend
        // it over `surface_1` so text contrast is computed against the actual
        // rendered color rather than the raw overlay.
        let base_bg = appearance.theme().surface_1();
        let effective_bg = background
            .map(|overlay| base_bg.blend(&overlay))
            .unwrap_or(base_bg);

        appearance.theme().sub_text_color(effective_bg).into_solid()
    }

    fn border(&self, appearance: &Appearance) -> Option<ColorU> {
        Some(internal_colors::neutral_3(appearance.theme()))
    }

    fn should_opt_out_of_contrast_adjustment(&self) -> bool {
        true
    }

    fn font_properties(&self) -> Option<warpui::fonts::Properties> {
        None
    }
}
