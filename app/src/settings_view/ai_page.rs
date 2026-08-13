use crate::ai::acp::config_options::{AcpConfigOption, probe_config_options};
use crate::ai::acp::registry::AcpRegistryModel;
use crate::appearance::{Appearance, AppearanceEvent};
use crate::editor::{EditorView, Event as EditorEvent, SingleLineEditorOptions, TextColors};
use crate::settings::{
    AISettings, AISettingsChangedEvent, LongRunningCommandSubmissionMode, PromptSubmissionMode,
    TerminalSuggestionEffort,
};
use crate::terminal::local_shell::LocalShellState;
use crate::util::bindings::BindingGroup;
use crate::view_components::{Dropdown, DropdownItem};
use agent_client_protocol::schema::SessionConfigOptionCategory;
use settings::{Setting, ToggleableSetting};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use strum::IntoEnumIterator;
use warp_core::channel::{Channel, ChannelState};
use warp_core::ui::theme::color::internal_colors;
use warpui::elements::{
    Align, Container, CrossAxisAlignment, Fill, Flex, ParentElement, Shrinkable, Text,
};
use warpui::fonts::{Properties, Weight};
use warpui::ui_components::{
    components::{Coords, UiComponent, UiComponentStyles},
    switch::SwitchStateHandle,
};
use warpui::{
    Action, AppContext, Element, Entity, SingletonEntity, TypedActionView, View, ViewContext,
    ViewHandle, id, keymap::FixedBinding,
};

use super::settings_page::{
    HEADER_PADDING, MatchData, PageType, SettingsPageMeta, SettingsPageViewHandle, SettingsWidget,
    ToggleState, build_sub_header, build_toggle_element, render_body_item_label,
    render_dropdown_item, render_dropdown_item_label, render_separator,
};
use super::{SettingsAction, SettingsSection, flags};

const CONTENT_FONT_SIZE: f32 = 12.;
const AI_SETTINGS_DROPDOWN_WIDTH: f32 = 250.;

fn ai_text_colors(appearance: &Appearance) -> TextColors {
    TextColors {
        default_color: appearance.theme().active_ui_text_color(),
        disabled_color: appearance.theme().disabled_ui_text_color(),
        hint_color: appearance.theme().disabled_ui_text_color(),
    }
}

pub fn init_actions_from_parent_view<T: Action + Clone>(
    app: &mut AppContext,
    context: &warpui::keymap::ContextPredicate,
    builder: fn(SettingsAction) -> T,
) {
    let ai_context = context.clone() & id!(flags::IS_ANY_AI_ENABLED);
    let prompt_mode_bindings: Vec<FixedBinding> = PromptSubmissionMode::iter()
        .map(|mode| {
            let context_flag = match mode {
                PromptSubmissionMode::Interrupt => flags::PROMPT_SUBMISSION_INTERRUPT,
                PromptSubmissionMode::Queue => flags::PROMPT_SUBMISSION_QUEUE,
            };
            FixedBinding::empty(
                mode.command_palette_description(),
                builder(SettingsAction::AI(
                    AISettingsPageAction::SetPromptSubmissionMode(mode),
                )),
                ai_context.clone() & !id!(context_flag),
            )
            .with_group(BindingGroup::Ai.as_str())
        })
        .collect();
    app.register_fixed_bindings(prompt_mode_bindings);

    let lrc_mode_bindings: Vec<FixedBinding> = LongRunningCommandSubmissionMode::iter()
        .map(|mode| {
            let context_flag = match mode {
                LongRunningCommandSubmissionMode::SendImmediately => {
                    flags::LRC_SUBMISSION_SEND_IMMEDIATELY
                }
                LongRunningCommandSubmissionMode::QueueUntilCommandCompletes => {
                    flags::LRC_SUBMISSION_QUEUE_UNTIL_COMMAND_COMPLETES
                }
            };
            FixedBinding::empty(
                mode.command_palette_description(),
                builder(SettingsAction::AI(
                    AISettingsPageAction::SetLongRunningCommandSubmissionMode(mode),
                )),
                ai_context.clone() & id!(flags::PROMPT_SUBMISSION_INTERRUPT) & !id!(context_flag),
            )
            .with_group(BindingGroup::Ai.as_str())
        })
        .collect();
    app.register_fixed_bindings(lrc_mode_bindings);
}

pub struct AISettingsPageView {
    page: PageType<Self>,
    acp_agent_backend_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    acp_config_options: Vec<AcpConfigOption>,
    acp_config_option_dropdowns: Vec<ViewHandle<Dropdown<AISettingsPageAction>>>,
    acp_config_options_status: Option<String>,
    terminal_suggestions_endpoint_editor: ViewHandle<EditorView>,
    terminal_suggestions_api_key_editor: ViewHandle<EditorView>,
    terminal_suggestions_model_editor: ViewHandle<EditorView>,
    terminal_suggestions_effort_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    default_prompt_submission_mode_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    lrc_submission_mode_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
}

fn acp_config_option_selected_value(
    option: &AcpConfigOption,
    default_config_options: &HashMap<String, String>,
) -> Option<String> {
    default_config_options
        .get(&option.id)
        .cloned()
        .or_else(|| Some(option.current_value.clone()))
        .or_else(|| option.values.first().map(|value| value.id.clone()))
}

fn acp_config_option_dropdown_items(
    option: &AcpConfigOption,
) -> Vec<DropdownItem<AISettingsPageAction>> {
    option
        .values
        .iter()
        .map(|value| {
            DropdownItem::new(
                value.name.clone(),
                AISettingsPageAction::SetAcpDefaultConfigOption {
                    config_id: option.id.clone(),
                    value_id: value.id.clone(),
                },
            )
        })
        .collect()
}

fn acp_config_option_description(option: &AcpConfigOption) -> Option<Cow<'_, str>> {
    if let Some(description) = option.description.as_deref() {
        return Some(Cow::Borrowed(description));
    }

    match option.category {
        Some(SessionConfigOptionCategory::Model) => {
            Some(Cow::Borrowed("Default model for new sessions."))
        }
        Some(SessionConfigOptionCategory::Mode) => {
            Some(Cow::Borrowed("Default approval mode for new sessions."))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::acp::config_options::AcpConfigOptionValue;

    #[test]
    fn acp_config_option_dropdown_items_do_not_mark_overrides_in_labels() {
        let option = AcpConfigOption {
            id: "model".to_string(),
            name: "Model".to_string(),
            description: None,
            category: None,
            current_value: "default".to_string(),
            values: vec![
                AcpConfigOptionValue {
                    id: "default".to_string(),
                    name: "Default".to_string(),
                },
                AcpConfigOptionValue {
                    id: "other".to_string(),
                    name: "Other".to_string(),
                },
            ],
        };

        let items = acp_config_option_dropdown_items(&option);

        assert_eq!(
            items
                .iter()
                .map(|item| item.display_text.as_str())
                .collect::<Vec<_>>(),
            vec!["Default", "Other"]
        );
    }
}

impl AISettingsPageView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let acp_agent_backend_dropdown = Self::create_acp_agent_backend_dropdown(ctx);
        let terminal_suggestions_endpoint_editor = Self::create_ai_text_editor(
            ctx,
            AISettings::as_ref(ctx)
                .terminal_suggestions_endpoint
                .to_string(),
            "https://api.openai.com/v1",
            false,
        );
        ctx.subscribe_to_view(
            &terminal_suggestions_endpoint_editor,
            |_, editor, event, ctx| {
                if matches!(event, EditorEvent::Blurred | EditorEvent::Enter) {
                    let value = editor.as_ref(ctx).buffer_text(ctx);
                    AISettings::handle(ctx).update(ctx, |settings, ctx| {
                        if let Err(err) =
                            settings.terminal_suggestions_endpoint.set_value(value, ctx)
                        {
                            log::warn!("Failed to set Terminal Suggestions endpoint: {err:?}");
                        }
                    });
                }
                if matches!(event, EditorEvent::Escape) {
                    ctx.emit(AISettingsPageEvent::FocusModal);
                }
            },
        );

        let terminal_suggestions_api_key_editor = Self::create_ai_text_editor(
            ctx,
            AISettings::as_ref(ctx)
                .terminal_suggestions_api_key
                .to_string(),
            "sk-...",
            true,
        );
        ctx.subscribe_to_view(
            &terminal_suggestions_api_key_editor,
            |_, editor, event, ctx| {
                if matches!(event, EditorEvent::Blurred | EditorEvent::Enter) {
                    let value = editor.as_ref(ctx).buffer_text(ctx);
                    AISettings::handle(ctx).update(ctx, |settings, ctx| {
                        if let Err(err) =
                            settings.terminal_suggestions_api_key.set_value(value, ctx)
                        {
                            log::warn!("Failed to set Terminal Suggestions API key: {err:?}");
                        }
                    });
                }
                if matches!(event, EditorEvent::Escape) {
                    ctx.emit(AISettingsPageEvent::FocusModal);
                }
            },
        );

        let terminal_suggestions_model_editor = Self::create_ai_text_editor(
            ctx,
            AISettings::as_ref(ctx)
                .terminal_suggestions_model
                .to_string(),
            "gpt-5.5",
            false,
        );
        ctx.subscribe_to_view(
            &terminal_suggestions_model_editor,
            |_, editor, event, ctx| {
                if matches!(event, EditorEvent::Blurred | EditorEvent::Enter) {
                    let value = editor.as_ref(ctx).buffer_text(ctx);
                    AISettings::handle(ctx).update(ctx, |settings, ctx| {
                        if let Err(err) = settings.terminal_suggestions_model.set_value(value, ctx)
                        {
                            log::warn!("Failed to set Terminal Suggestions model: {err:?}");
                        }
                    });
                }
                if matches!(event, EditorEvent::Escape) {
                    ctx.emit(AISettingsPageEvent::FocusModal);
                }
            },
        );

        let terminal_suggestions_effort_dropdown =
            Self::create_terminal_suggestions_effort_dropdown(ctx);
        let default_prompt_submission_mode_dropdown =
            Self::create_default_prompt_submission_mode_dropdown(ctx);
        let lrc_submission_mode_dropdown = Self::create_lrc_submission_mode_dropdown(ctx);

        let terminal_suggestions_editors = [
            terminal_suggestions_endpoint_editor.clone(),
            terminal_suggestions_api_key_editor.clone(),
            terminal_suggestions_model_editor.clone(),
        ];
        ctx.subscribe_to_model(&Appearance::handle(ctx), move |_, _, event, ctx| {
            if matches!(event, AppearanceEvent::ThemeChanged) {
                let text_colors = ai_text_colors(Appearance::as_ref(ctx));
                for editor in &terminal_suggestions_editors {
                    let colors = text_colors.clone();
                    editor.update(ctx, move |editor, ctx| {
                        editor.set_text_colors(colors, ctx);
                    });
                }
            }
        });

        ctx.subscribe_to_model(&AcpRegistryModel::handle(ctx), |me, _, _, ctx| {
            me.refresh_acp_agent_backend_dropdown(ctx);
            me.refresh_acp_config_options(ctx);
            ctx.notify();
        });
        ctx.subscribe_to_model(&AISettings::handle(ctx), |me, _, event, ctx| {
            match event {
                AISettingsChangedEvent::PromptSubmissionMode { .. } => {
                    let current_mode = AISettings::as_ref(ctx).default_prompt_submission_mode;
                    me.default_prompt_submission_mode_dropdown
                        .update(ctx, |dropdown, ctx| {
                            dropdown.set_selected_by_action(
                                AISettingsPageAction::SetPromptSubmissionMode(current_mode),
                                ctx,
                            );
                        });
                }
                AISettingsChangedEvent::LongRunningCommandSubmissionMode { .. } => {
                    let current_mode = AISettings::as_ref(ctx).long_running_command_submission_mode;
                    me.lrc_submission_mode_dropdown
                        .update(ctx, |dropdown, ctx| {
                            dropdown.set_selected_by_action(
                                AISettingsPageAction::SetLongRunningCommandSubmissionMode(
                                    current_mode,
                                ),
                                ctx,
                            );
                        });
                }
                _ => {}
            }
            ctx.notify();
        });

        let mut view = Self {
            page: Self::build_page(ctx),
            acp_agent_backend_dropdown,
            acp_config_options: Vec::new(),
            acp_config_option_dropdowns: Vec::new(),
            acp_config_options_status: None,
            terminal_suggestions_endpoint_editor,
            terminal_suggestions_api_key_editor,
            terminal_suggestions_model_editor,
            terminal_suggestions_effort_dropdown,
            default_prompt_submission_mode_dropdown,
            lrc_submission_mode_dropdown,
        };
        view.refresh_acp_config_options(ctx);
        view
    }

    fn create_ai_text_editor(
        ctx: &mut ViewContext<Self>,
        initial_value: String,
        placeholder: &'static str,
        is_password: bool,
    ) -> ViewHandle<EditorView> {
        ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::as_ref(ctx);
            let options = SingleLineEditorOptions {
                is_password,
                text: crate::editor::TextOptions {
                    font_size_override: Some(appearance.ui_font_size()),
                    font_family_override: Some(appearance.monospace_font_family()),
                    text_colors_override: Some(ai_text_colors(appearance)),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(placeholder, ctx);
            editor.set_buffer_text(&initial_value, ctx);
            editor
        })
    }

    fn create_acp_agent_backend_dropdown(
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let current = AISettings::as_ref(ctx).acp_agent_backend.to_string();
        ctx.add_typed_action_view(move |ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_items(Self::acp_agent_backend_dropdown_items(ctx), ctx);
            dropdown.set_selected_by_action(
                AISettingsPageAction::SetAcpAgentBackend(current.clone()),
                ctx,
            );
            dropdown
        })
    }

    fn acp_agent_backend_dropdown_items(
        ctx: &mut ViewContext<Dropdown<AISettingsPageAction>>,
    ) -> Vec<DropdownItem<AISettingsPageAction>> {
        let registry = AcpRegistryModel::as_ref(ctx).registry();
        registry
            .selectable_agents()
            .into_iter()
            .map(|agent| {
                DropdownItem::new(
                    agent.name.clone(),
                    AISettingsPageAction::SetAcpAgentBackend(agent.id.clone()),
                )
            })
            .collect()
    }

    fn refresh_acp_agent_backend_dropdown(&mut self, ctx: &mut ViewContext<Self>) {
        let current = AISettings::as_ref(ctx).acp_agent_backend.to_string();
        self.acp_agent_backend_dropdown
            .update(ctx, |dropdown, ctx| {
                dropdown.set_items(Self::acp_agent_backend_dropdown_items(ctx), ctx);
                dropdown.set_selected_by_action(
                    AISettingsPageAction::SetAcpAgentBackend(current.clone()),
                    ctx,
                );
            });
    }

    fn create_acp_config_option_dropdown(
        ctx: &mut ViewContext<Self>,
        option: &AcpConfigOption,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let option = option.clone();
        let selected_value = acp_config_option_selected_value(
            &option,
            &AISettings::as_ref(ctx).acp_default_config_options,
        );

        ctx.add_typed_action_view(move |ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_items(acp_config_option_dropdown_items(&option), ctx);

            if let Some(selected_value) = selected_value.clone() {
                dropdown.set_selected_by_action(
                    AISettingsPageAction::SetAcpDefaultConfigOption {
                        config_id: option.id.clone(),
                        value_id: selected_value,
                    },
                    ctx,
                );
            }

            dropdown
        })
    }

    fn refresh_acp_config_option_dropdowns(&mut self, ctx: &mut ViewContext<Self>) {
        self.acp_config_option_dropdowns = self
            .acp_config_options
            .iter()
            .map(|option| Self::create_acp_config_option_dropdown(ctx, option))
            .collect();
    }

    fn refresh_acp_config_options(&mut self, ctx: &mut ViewContext<Self>) {
        let backend_id = AISettings::as_ref(ctx).acp_agent_backend.to_string();
        self.acp_config_options.clear();
        self.acp_config_option_dropdowns.clear();

        if cfg!(test) || ChannelState::channel() == Channel::Integration {
            self.acp_config_options_status = None;
            return;
        }

        let Some(launch) = AcpRegistryModel::as_ref(ctx)
            .registry()
            .launch_for_agent(&backend_id)
        else {
            self.acp_config_options_status = Some(format!(
                "No ACP registry launch configuration for {backend_id}."
            ));
            ctx.notify();
            return;
        };
        let display_name = launch.display_name.clone();
        let install_command = launch.install_command.clone();
        self.acp_config_options_status = Some(format!("Loading {} options...", display_name));
        ctx.notify();

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let adapter_path_env = LocalShellState::handle(ctx).update(ctx, |shell_state, ctx| {
            shell_state.get_interactive_path_env_var(ctx)
        });
        let _ = ctx.spawn(
            async move {
                let adapter_path_env = adapter_path_env.await;
                probe_config_options(launch, cwd, adapter_path_env).await
            },
            move |me, result, ctx| {
                if AISettings::as_ref(ctx).acp_agent_backend.as_str() != backend_id {
                    return;
                }

                match result {
                    Ok(options) => {
                        me.acp_config_options = options;
                        me.acp_config_options_status = if me.acp_config_options.is_empty() {
                            Some("No ACP config options detected.".to_string())
                        } else {
                            None
                        };
                        me.refresh_acp_config_option_dropdowns(ctx);
                    }
                    Err(err) => {
                        me.acp_config_options.clear();
                        me.acp_config_option_dropdowns.clear();
                        me.acp_config_options_status = Some(format!(
                            "{} options unavailable: {}. Install with {}.",
                            display_name, err, install_command
                        ));
                    }
                }
                ctx.notify();
            },
        );
    }

    fn create_terminal_suggestions_effort_dropdown(
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let current = *AISettings::as_ref(ctx).terminal_suggestions_effort;
        ctx.add_typed_action_view(move |ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.add_items(
                TerminalSuggestionEffort::iter()
                    .map(|effort| {
                        DropdownItem::new(
                            effort.display_name(),
                            AISettingsPageAction::SetTerminalSuggestionsEffort(effort),
                        )
                    })
                    .collect(),
                ctx,
            );
            dropdown.set_selected_by_action(
                AISettingsPageAction::SetTerminalSuggestionsEffort(current),
                ctx,
            );
            dropdown
        })
    }

    fn create_default_prompt_submission_mode_dropdown(
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let current = AISettings::as_ref(ctx).default_prompt_submission_mode;
        ctx.add_typed_action_view(move |ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.add_items(
                PromptSubmissionMode::iter()
                    .map(|mode| {
                        DropdownItem::new(
                            mode.display_name(),
                            AISettingsPageAction::SetPromptSubmissionMode(mode),
                        )
                    })
                    .collect(),
                ctx,
            );
            dropdown.set_selected_by_action(
                AISettingsPageAction::SetPromptSubmissionMode(current),
                ctx,
            );
            dropdown
        })
    }

    fn create_lrc_submission_mode_dropdown(
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let current = AISettings::as_ref(ctx).long_running_command_submission_mode;
        ctx.add_typed_action_view(move |ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.add_items(
                LongRunningCommandSubmissionMode::iter()
                    .map(|mode| {
                        DropdownItem::new(
                            mode.display_name(),
                            AISettingsPageAction::SetLongRunningCommandSubmissionMode(mode),
                        )
                    })
                    .collect(),
                ctx,
            );
            dropdown.set_selected_by_action(
                AISettingsPageAction::SetLongRunningCommandSubmissionMode(current),
                ctx,
            );
            dropdown
        })
    }

    fn build_page(_ctx: &mut ViewContext<Self>) -> PageType<Self> {
        PageType::new_uncategorized(vec![Box::new(AIWidget::default())], None)
    }
}

impl View for AISettingsPageView {
    fn ui_name() -> &'static str {
        "AISettingsPage"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        self.page.render(self, app)
    }
}

pub enum AISettingsPageEvent {
    FocusModal,
}

impl Entity for AISettingsPageView {
    type Event = AISettingsPageEvent;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AISettingsPageAction {
    SetAcpAgentBackend(String),
    SetAcpDefaultConfigOption { config_id: String, value_id: String },
    SetTerminalSuggestionsEffort(TerminalSuggestionEffort),
    SetPromptSubmissionMode(PromptSubmissionMode),
    SetLongRunningCommandSubmissionMode(LongRunningCommandSubmissionMode),
    ToggleTerminalNextCommand,
    ToggleTerminalPromptSuggestions,
    ToggleSubmitRichInputOnCtrlEnter,
}

impl TypedActionView for AISettingsPageView {
    type Action = AISettingsPageAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            AISettingsPageAction::SetAcpAgentBackend(backend) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings.acp_agent_backend.set_value(backend.clone(), ctx) {
                        log::warn!("Failed to set ACP agent backend: {err:?}");
                    }
                });
                self.refresh_acp_config_options(ctx);
                ctx.notify();
            }
            AISettingsPageAction::SetAcpDefaultConfigOption {
                config_id,
                value_id,
            } => {
                let is_agent_default = self
                    .acp_config_options
                    .iter()
                    .find(|option| option.id == *config_id)
                    .is_some_and(|option| option.current_value == *value_id);

                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut options = settings.acp_default_config_options.clone();
                    if is_agent_default {
                        options.remove(config_id);
                    } else {
                        options.insert(config_id.clone(), value_id.clone());
                    }
                    if let Err(err) = settings.acp_default_config_options.set_value(options, ctx) {
                        log::warn!("Failed to set ACP default config options: {err:?}");
                    }
                });
                self.refresh_acp_config_option_dropdowns(ctx);
                ctx.notify();
            }
            AISettingsPageAction::SetTerminalSuggestionsEffort(effort) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings.terminal_suggestions_effort.set_value(*effort, ctx) {
                        log::warn!("Failed to set Terminal Suggestions effort: {err:?}");
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::SetPromptSubmissionMode(mode) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings
                        .default_prompt_submission_mode
                        .set_value(*mode, ctx)
                    {
                        log::warn!("Failed to set default prompt submission mode: {err:?}");
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::SetLongRunningCommandSubmissionMode(mode) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings
                        .long_running_command_submission_mode
                        .set_value(*mode, ctx)
                    {
                        log::warn!("Failed to set long-running command submission mode: {err:?}");
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleTerminalNextCommand => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings
                        .terminal_next_command_enabled
                        .toggle_and_save_value(ctx)
                    {
                        log::warn!("Failed to toggle Next Command suggestions: {err:?}");
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleTerminalPromptSuggestions => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings
                        .terminal_prompt_suggestions_enabled
                        .toggle_and_save_value(ctx)
                    {
                        log::warn!("Failed to toggle Prompt Suggestions: {err:?}");
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleSubmitRichInputOnCtrlEnter => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings.submit_on_ctrl_enter.toggle_and_save_value(ctx) {
                        log::warn!("Failed to toggle Rich Input Ctrl+Enter submission: {err:?}");
                    }
                });
                ctx.notify();
            }
        }
    }
}

impl SettingsPageMeta for AISettingsPageView {
    fn section() -> SettingsSection {
        SettingsSection::AI
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        true
    }

    fn on_page_selected(&mut self, _: bool, _ctx: &mut ViewContext<Self>) {}

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id)
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

impl From<ViewHandle<AISettingsPageView>> for SettingsPageViewHandle {
    fn from(view_handle: ViewHandle<AISettingsPageView>) -> Self {
        SettingsPageViewHandle::AI(view_handle)
    }
}

fn render_ai_setting_toggle(
    label: impl Into<String>,
    action: AISettingsPageAction,
    is_setting_enabled: bool,
    is_setting_toggleable: bool,
    switch_state: SwitchStateHandle,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    build_toggle_element(
        render_body_item_label::<AISettingsPageAction>(
            label.into(),
            Some(styles::header_font_color(is_setting_toggleable, app)),
            None,
            ToggleState::Enabled,
            appearance,
        ),
        render_ai_feature_switch(
            switch_state,
            is_setting_enabled,
            is_setting_toggleable,
            action,
            app,
        ),
        appearance,
        None,
    )
}

#[derive(Default)]
struct AIWidget {
    next_command_toggle: SwitchStateHandle,
    prompt_suggestions_toggle: SwitchStateHandle,
    submit_on_ctrl_enter_toggle: SwitchStateHandle,
}

impl AIWidget {
    fn render_dropdown(
        label: &'static str,
        secondary_text: &'static str,
        dropdown: &ViewHandle<Dropdown<AISettingsPageAction>>,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        render_dropdown_item(
            appearance,
            label,
            Some(secondary_text),
            None,
            None,
            dropdown,
        )
    }

    fn render_text_input(
        label: &'static str,
        secondary_text: &'static str,
        editor: ViewHandle<EditorView>,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        const INPUT_WIDTH: f32 = 360.;

        let label = Align::new(render_dropdown_item_label(
            label.to_string(),
            Some(secondary_text.to_string()),
            None,
            appearance,
        ))
        .left()
        .finish();

        let input = appearance
            .ui_builder()
            .text_input(editor)
            .with_style(UiComponentStyles {
                width: Some(INPUT_WIDTH),
                padding: Some(Coords {
                    top: 10.,
                    bottom: 10.,
                    left: 16.,
                    right: 16.,
                }),
                background: Some(appearance.theme().surface_2().into()),
                ..Default::default()
            })
            .build()
            .finish();

        Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                Shrinkable::new(1., Container::new(label).with_padding_right(16.).finish())
                    .finish(),
            )
            .with_child(input)
            .finish()
    }

    fn render_acp_default_config_options(
        view: &AISettingsPageView,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        let mut column = Flex::column().with_spacing(12.);

        for (option, dropdown) in view
            .acp_config_options
            .iter()
            .zip(view.acp_config_option_dropdowns.iter())
        {
            let description = acp_config_option_description(option);
            column.add_child(render_dropdown_item(
                appearance,
                &option.name,
                description.as_deref(),
                None,
                None,
                dropdown,
            ));
        }

        if let Some(status) = &view.acp_config_options_status {
            let label = Align::new(render_dropdown_item_label(
                "Agent options".to_string(),
                Some(status.clone()),
                None,
                appearance,
            ))
            .left()
            .finish();

            column.add_child(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(
                        Shrinkable::new(1., Container::new(label).with_padding_right(16.).finish())
                            .finish(),
                    )
                    .finish(),
            );
        }

        column.finish()
    }

    fn render_section_header(
        title: &'static str,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        Container::new(
            Text::new_inline(title, appearance.ui_font_family(), CONTENT_FONT_SIZE)
                .with_style(Properties::default().weight(Weight::Semibold))
                .with_color(styles::header_font_color(true, app).into())
                .finish(),
        )
        .with_margin_bottom(2.)
        .finish()
    }

    fn render_toggle(
        label: &'static str,
        description: &'static str,
        action: AISettingsPageAction,
        enabled: bool,
        toggle: SwitchStateHandle,
        app: &AppContext,
    ) -> Box<dyn Element> {
        Flex::column()
            .with_child(render_ai_setting_toggle(
                label, action, enabled, true, toggle, app,
            ))
            .with_child(render_ai_setting_description(description, true, app))
            .finish()
    }
}

impl SettingsWidget for AIWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "ai acp codex claude natural language input openai compatible endpoint api key model reasoning next command prompt suggestions terminal suggestions third party cli agent rich input ctrl enter submit newline queue interrupt submission auto-queue response long-running lrc"
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let settings = AISettings::as_ref(app);

        Flex::column()
            .with_spacing(14.)
            .with_child(
                build_sub_header(appearance, "AI", Some(styles::header_font_color(true, app)))
                    .with_padding_bottom(HEADER_PADDING)
                    .finish(),
            )
            .with_child(Self::render_section_header("ACP Agent", appearance, app))
            .with_child(Self::render_dropdown(
                "Agent backend",
                "Choose the ACP adapter for agent conversations.",
                &view.acp_agent_backend_dropdown,
                appearance,
            ))
            .with_child(Self::render_acp_default_config_options(
                view, appearance, app,
            ))
            .with_child(Self::render_dropdown(
                "Default prompt submission mode",
                "What happens when you submit a new prompt while the agent is still responding. You can override this per conversation using the auto-queue toggle.",
                &view.default_prompt_submission_mode_dropdown,
                appearance,
            ))
            .with_children((settings.default_prompt_submission_mode == PromptSubmissionMode::Interrupt).then(|| {
                Self::render_dropdown(
                    "Default long-running command submission mode",
                    "What happens when you submit a prompt while an agent is driving an agent-requested long-running command. Queued prompts are sent when the command finishes.",
                    &view.lrc_submission_mode_dropdown,
                    appearance,
                )
            }))
            .with_child(render_separator(appearance))
            .with_child(Self::render_section_header(
                "Terminal Suggestions",
                appearance,
                app,
            ))
            .with_child(Self::render_text_input(
                "Endpoint",
                "Base URL used for Next Command and Prompt Suggestions.",
                view.terminal_suggestions_endpoint_editor.clone(),
                appearance,
            ))
            .with_child(Self::render_text_input(
                "API key",
                "Bearer token for the suggestions endpoint.",
                view.terminal_suggestions_api_key_editor.clone(),
                appearance,
            ))
            .with_child(Self::render_text_input(
                "Model",
                "Model used to generate terminal suggestions.",
                view.terminal_suggestions_model_editor.clone(),
                appearance,
            ))
            .with_child(Self::render_dropdown(
                "Reasoning effort",
                "Reasoning level to request when the model supports it.",
                &view.terminal_suggestions_effort_dropdown,
                appearance,
            ))
            .with_child(render_separator(appearance))
            .with_child(Self::render_toggle(
                "Next Command",
                "Suggests the next shell command from the current terminal context.",
                AISettingsPageAction::ToggleTerminalNextCommand,
                *settings.terminal_next_command_enabled,
                self.next_command_toggle.clone(),
                app,
            ))
            .with_child(Self::render_toggle(
                "Prompt Suggestions",
                "Suggests follow-up prompts from the current terminal context.",
                AISettingsPageAction::ToggleTerminalPromptSuggestions,
                *settings.terminal_prompt_suggestions_enabled,
                self.prompt_suggestions_toggle.clone(),
                app,
            ))
            .with_child(render_separator(appearance))
            .with_child(Self::render_section_header(
                "Third-party CLI Agent",
                appearance,
                app,
            ))
            .with_child(Self::render_toggle(
                "Submit Rich Input with Ctrl+Enter",
                "When enabled, the Rich Input editor submits on Ctrl+Enter instead of Enter. Enter inserts a newline.",
                AISettingsPageAction::ToggleSubmitRichInputOnCtrlEnter,
                *settings.submit_on_ctrl_enter,
                self.submit_on_ctrl_enter_toggle.clone(),
                app,
            ))
            .finish()
    }
}

fn render_ai_setting_description(
    description: impl Into<Cow<'static, str>>,
    is_setting_toggleable: bool,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    appearance
        .ui_builder()
        .paragraph(description)
        .with_style(UiComponentStyles {
            font_size: Some(appearance.ui_font_size()),
            font_color: Some(styles::description_font_color(is_setting_toggleable, app).into()),
            margin: Some(
                Coords::default()
                    .top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
                    .bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                    .right(styles::TOGGLE_WIDTH_MARGIN),
            ),
            ..Default::default()
        })
        .build()
        .finish()
}

fn render_ai_feature_switch(
    state_handle: SwitchStateHandle,
    is_setting_enabled: bool,
    is_setting_toggleable: bool,
    toggle_action: AISettingsPageAction,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    appearance
        .ui_builder()
        .switch(state_handle)
        .check(is_setting_enabled)
        .with_disabled(!is_setting_toggleable)
        .with_disabled_styles(UiComponentStyles {
            background: Some(Fill::Solid(internal_colors::neutral_4(appearance.theme()))),
            foreground: Some(Fill::Solid(internal_colors::neutral_5(appearance.theme()))),
            ..Default::default()
        })
        .build()
        .on_click(move |ctx, _, _| {
            if !is_setting_toggleable {
                return;
            }
            ctx.dispatch_typed_action(toggle_action.clone());
        })
        .finish()
}

mod styles {
    use warp_core::ui::{appearance::Appearance, theme::Fill};
    use warpui::{AppContext, SingletonEntity};

    pub const DESCRIPTION_NEGATIVE_MARGIN_OFFSET: f32 = -12.;
    pub const DESCRIPTION_MARGIN_BOTTOM: f32 = 12.;
    pub const TOGGLE_WIDTH_MARGIN: f32 = 48.;

    pub fn header_font_color(is_enabled_setting: bool, app: &AppContext) -> Fill {
        let appearance = Appearance::as_ref(app);
        if is_enabled_setting {
            appearance
                .theme()
                .main_text_color(appearance.theme().surface_2())
        } else {
            appearance.theme().disabled_ui_text_color()
        }
    }

    pub fn description_font_color(is_enabled_setting: bool, app: &AppContext) -> Fill {
        let appearance = Appearance::as_ref(app);
        if is_enabled_setting {
            appearance
                .theme()
                .sub_text_color(appearance.theme().surface_1())
        } else {
            appearance.theme().disabled_ui_text_color()
        }
    }
}
