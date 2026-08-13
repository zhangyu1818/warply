use crate::ai::blocklist::BlocklistAIPermissions;
use crate::ai::execution_profiles::{
    AIExecutionProfile, ActionPermission, WriteToPtyPermission,
    profiles::{AIExecutionProfilesModel, AIExecutionProfilesModelEvent, ClientProfileId},
};
use crate::ai::paths::host_native_absolute_path;
use crate::editor::InteractionState;
use crate::editor::{EditorView, Event as EditorEvent, SingleLineEditorOptions};
use crate::pane_group::focus_state::PaneFocusHandle;
use crate::settings::{AISettings, AISettingsChangedEvent, AgentModeCommandExecutionPredicate};
use crate::ui_components::icons::Icon;
use crate::view_components::{
    Dropdown, DropdownItem, SubmittableTextInput, SubmittableTextInputEvent,
    action_button::{ActionButton, DangerSecondaryTheme},
};
use crate::{
    Appearance,
    pane_group::{BackingView, PaneConfiguration, PaneEvent, pane::view},
};
use regex::Regex;

use std::path::{Path, PathBuf};
use warpui::{
    AppContext, Element, Entity, ModelHandle, SingletonEntity, TypedActionView, View, ViewContext,
    ViewHandle,
    elements::{
        Align, ChildView, ClippedScrollStateHandle, ClippedScrollable, Container, Flex,
        MouseStateHandle, ParentElement, ScrollbarWidth,
    },
};

#[derive(Default)]
struct TooltipMouseStateHandles {
    apply_code_diffs_tooltip_mouse_state: MouseStateHandle,
    read_files_tooltip_mouse_state: MouseStateHandle,
    execute_commands_tooltip_mouse_state: MouseStateHandle,
    write_to_pty_tooltip_mouse_state: MouseStateHandle,
    computer_use_tooltip_mouse_state: MouseStateHandle,
    ask_user_question_tooltip_mouse_state: MouseStateHandle,
    command_allowlist_editor_tooltip_mouse_state: MouseStateHandle,
    directory_allowlist_editor_tooltip_mouse_state: MouseStateHandle,
}

pub mod manager;
pub use manager::*;

pub const HEADER_TEXT: &str = "Profile Editor";

#[derive(Debug, Clone)]
pub enum ExecutionProfileEditorViewEvent {
    Pane(PaneEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionProfileEditorViewAction {
    Save,
    Close,

    SetApplyCodeDiffs {
        permission: ActionPermission,
    },
    SetReadFiles {
        permission: ActionPermission,
    },

    SetExecuteCommands {
        permission: ActionPermission,
    },
    SetWriteToPty {
        permission: WriteToPtyPermission,
    },
    SetComputerUse {
        permission: super::ComputerUsePermission,
    },
    SetAskUserQuestion {
        permission: super::AskUserQuestionPermission,
    },
    AddToCommandAllowlist {
        predicate: AgentModeCommandExecutionPredicate,
    },
    RemoveFromCommandAllowlist {
        predicate: AgentModeCommandExecutionPredicate,
    },
    AddToCommandDenylist {
        predicate: AgentModeCommandExecutionPredicate,
    },
    RemoveFromCommandDenylist {
        predicate: AgentModeCommandExecutionPredicate,
    },
    AddToDirectoryAllowlist {
        path: PathBuf,
    },
    RemoveFromDirectoryAllowlist {
        path: PathBuf,
    },
    DeleteProfile,
}

pub struct ExecutionProfileEditorView {
    profile_id: ClientProfileId,
    pane_configuration: ModelHandle<PaneConfiguration>,
    focus_handle: Option<PaneFocusHandle>,
    clipped_scroll_state: ClippedScrollStateHandle,
    apply_code_diffs_dropdown: ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
    read_files_dropdown: ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
    execute_commands_dropdown: ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
    write_to_pty_dropdown: ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
    computer_use_dropdown: ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
    ask_user_question_dropdown: ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
    command_allowlist_editor: ViewHandle<SubmittableTextInput>,
    command_denylist_editor: ViewHandle<SubmittableTextInput>,
    directory_allowlist_editor: ViewHandle<SubmittableTextInput>,
    command_allowlist_mouse_state_handles: Vec<MouseStateHandle>,
    command_denylist_mouse_state_handles: Vec<MouseStateHandle>,
    directory_allowlist_mouse_state_handles: Vec<MouseStateHandle>,
    profile_name_editor: ViewHandle<EditorView>,
    delete_button: ViewHandle<ActionButton>,
    tooltip_mouse_state_handles: TooltipMouseStateHandles,
}

impl ExecutionProfileEditorView {
    pub fn new(profile_id: ClientProfileId, ctx: &mut ViewContext<Self>) -> Self {
        let pane_configuration = ctx.add_model(|_ctx| PaneConfiguration::new(HEADER_TEXT));

        let apply_code_diffs_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        "Agent decides",
                        ExecutionProfileEditorViewAction::SetApplyCodeDiffs {
                            permission: ActionPermission::AgentDecides,
                        },
                    ),
                    DropdownItem::new(
                        "Always allow",
                        ExecutionProfileEditorViewAction::SetApplyCodeDiffs {
                            permission: ActionPermission::AlwaysAllow,
                        },
                    ),
                    DropdownItem::new(
                        "Always ask",
                        ExecutionProfileEditorViewAction::SetApplyCodeDiffs {
                            permission: ActionPermission::AlwaysAsk,
                        },
                    ),
                ],
                ctx,
            );
            dropdown
        });

        let read_files_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        "Agent decides",
                        ExecutionProfileEditorViewAction::SetReadFiles {
                            permission: ActionPermission::AgentDecides,
                        },
                    ),
                    DropdownItem::new(
                        "Always allow",
                        ExecutionProfileEditorViewAction::SetReadFiles {
                            permission: ActionPermission::AlwaysAllow,
                        },
                    ),
                    DropdownItem::new(
                        "Always ask",
                        ExecutionProfileEditorViewAction::SetReadFiles {
                            permission: ActionPermission::AlwaysAsk,
                        },
                    ),
                ],
                ctx,
            );
            dropdown
        });

        let execute_commands_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        "Agent decides",
                        ExecutionProfileEditorViewAction::SetExecuteCommands {
                            permission: ActionPermission::AgentDecides,
                        },
                    ),
                    DropdownItem::new(
                        "Always allow",
                        ExecutionProfileEditorViewAction::SetExecuteCommands {
                            permission: ActionPermission::AlwaysAllow,
                        },
                    ),
                    DropdownItem::new(
                        "Always ask",
                        ExecutionProfileEditorViewAction::SetExecuteCommands {
                            permission: ActionPermission::AlwaysAsk,
                        },
                    ),
                ],
                ctx,
            );
            dropdown
        });

        let write_to_pty_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        "Always allow",
                        ExecutionProfileEditorViewAction::SetWriteToPty {
                            permission: WriteToPtyPermission::AlwaysAllow,
                        },
                    ),
                    DropdownItem::new(
                        "Always ask",
                        ExecutionProfileEditorViewAction::SetWriteToPty {
                            permission: WriteToPtyPermission::AlwaysAsk,
                        },
                    ),
                    DropdownItem::new(
                        "Ask on first write",
                        ExecutionProfileEditorViewAction::SetWriteToPty {
                            permission: WriteToPtyPermission::AskOnFirstWrite,
                        },
                    ),
                ],
                ctx,
            );
            dropdown
        });

        let computer_use_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        "Never",
                        ExecutionProfileEditorViewAction::SetComputerUse {
                            permission: super::ComputerUsePermission::Never,
                        },
                    ),
                    DropdownItem::new(
                        "Always ask",
                        ExecutionProfileEditorViewAction::SetComputerUse {
                            permission: super::ComputerUsePermission::AlwaysAsk,
                        },
                    ),
                    DropdownItem::new(
                        "Always allow",
                        ExecutionProfileEditorViewAction::SetComputerUse {
                            permission: super::ComputerUsePermission::AlwaysAllow,
                        },
                    ),
                ],
                ctx,
            );
            dropdown
        });

        let ask_user_question_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        "Never ask",
                        ExecutionProfileEditorViewAction::SetAskUserQuestion {
                            permission: super::AskUserQuestionPermission::Never,
                        },
                    ),
                    DropdownItem::new(
                        "Ask unless auto-approve",
                        ExecutionProfileEditorViewAction::SetAskUserQuestion {
                            permission: super::AskUserQuestionPermission::AskExceptInAutoApprove,
                        },
                    ),
                    DropdownItem::new(
                        "Always ask",
                        ExecutionProfileEditorViewAction::SetAskUserQuestion {
                            permission: super::AskUserQuestionPermission::AlwaysAsk,
                        },
                    ),
                ],
                ctx,
            );
            dropdown
        });

        let permissions = BlocklistAIPermissions::as_ref(ctx);
        let profile_data = permissions.permissions_profile_for_id(ctx, profile_id);

        let command_allowlist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input =
                SubmittableTextInput::new(ctx).validate_on_edit(|s| Regex::new(s).is_ok());
            input.set_placeholder_text("e.g. ls .*", ctx);
            input
        });

        let command_allowlist_mouse_state_handles = profile_data
            .command_allowlist
            .iter()
            .map(|_| Default::default())
            .collect();

        let command_denylist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input =
                SubmittableTextInput::new(ctx).validate_on_edit(|s| Regex::new(s).is_ok());
            input.set_placeholder_text("e.g. rm .*", ctx);
            input
        });

        let command_denylist_mouse_state_handles = profile_data
            .command_denylist
            .iter()
            .map(|_| Default::default())
            .collect();

        let directory_allowlist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input = SubmittableTextInput::new(ctx).validate_on_submit(|s| {
                let expanded = host_native_absolute_path(s, &None, &None);
                Path::new(&expanded).is_dir()
            });
            input.set_placeholder_text("e.g. ~/code-repos/repo", ctx);
            input
        });

        let directory_allowlist_mouse_state_handles = profile_data
            .directory_allowlist
            .iter()
            .map(|_| Default::default())
            .collect();

        let profile_name_editor = ctx.add_view(|ctx| {
            let mut editor = EditorView::single_line(
                SingleLineEditorOptions {
                    max_buffer_len: Some(super::PROFILE_NAME_MAX_LENGTH),
                    ..Default::default()
                },
                ctx,
            );
            editor.set_placeholder_text("e.g. \"YOLO code\"", ctx);
            editor
        });

        let font_family = Appearance::as_ref(ctx).ui_font_family();

        profile_name_editor.update(ctx, |editor, ctx| {
            editor.set_font_size(12., ctx);
            editor.set_font_family(font_family, ctx);
        });

        Self::update_profile_name_editor(&profile_name_editor, &profile_data, ctx);

        let delete_button = ctx.add_typed_action_view(|_| {
            ActionButton::new("Delete profile", DangerSecondaryTheme)
                .with_icon(Icon::Trash)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(ExecutionProfileEditorViewAction::DeleteProfile);
                })
        });

        let mut view = Self {
            profile_id,
            pane_configuration,
            focus_handle: None,
            clipped_scroll_state: Default::default(),
            apply_code_diffs_dropdown,
            read_files_dropdown,
            execute_commands_dropdown,
            write_to_pty_dropdown,
            computer_use_dropdown,
            ask_user_question_dropdown,
            command_allowlist_editor,
            command_denylist_editor,
            directory_allowlist_editor,
            command_allowlist_mouse_state_handles,
            command_denylist_mouse_state_handles,
            directory_allowlist_mouse_state_handles,
            profile_name_editor,
            delete_button,
            tooltip_mouse_state_handles: Default::default(),
        };

        ctx.subscribe_to_view(&view.profile_name_editor, |view, _, event, ctx| {
            if let EditorEvent::Edited(_) = event {
                view.save_profile_name_if_valid(ctx);
            }
        });

        ctx.subscribe_to_view(&view.command_allowlist_editor, |view, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let predicate = match AgentModeCommandExecutionPredicate::new_regex(s) {
                    Ok(regex) => regex,
                    Err(e) => {
                        log::warn!(
                            "Failed to convert string to regex for cmd execution allowlist: {e}"
                        );
                        return;
                    }
                };
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.add_to_command_allowlist(view.profile_id, &predicate, ctx);
                });
                ctx.notify();
            }
        });

        ctx.subscribe_to_view(&view.command_denylist_editor, |view, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let predicate = match AgentModeCommandExecutionPredicate::new_regex(s) {
                    Ok(regex) => regex,
                    Err(e) => {
                        log::warn!(
                            "Failed to convert string to regex for cmd execution denylist: {e}"
                        );
                        return;
                    }
                };
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.add_to_command_denylist(view.profile_id, &predicate, ctx);
                });
                ctx.notify();
            }
        });

        ctx.subscribe_to_view(&view.directory_allowlist_editor, |view, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let expanded = host_native_absolute_path(s, &None, &None);
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.add_to_directory_allowlist(
                        view.profile_id,
                        &PathBuf::from(expanded),
                        ctx,
                    );
                });
                ctx.notify();
            }
        });

        ctx.subscribe_to_model(
            &AIExecutionProfilesModel::handle(ctx),
            |me, _, event, ctx| {
                if matches!(event, AIExecutionProfilesModelEvent::ProfileUpdated(profile_id) if *profile_id == me.profile_id) {
                    me.refresh_profile_state(ctx);
                    me.update_mouse_state_handles(ctx);
                }
            },
        );

        ctx.subscribe_to_model(&AISettings::handle(ctx), |me, _, event, ctx| {
            if let AISettingsChangedEvent::IsAnyAIEnabled { .. } = event {
                Self::update_all_editor_interaction_states(me, ctx);
                ctx.notify();
            }
        });

        Self::update_all_editor_interaction_states(&view, ctx);

        view.refresh_profile_state(ctx);

        view.update_mouse_state_handles(ctx);

        view
    }

    pub fn profile_id(&self) -> ClientProfileId {
        self.profile_id
    }

    fn update_mouse_state_handles(&mut self, ctx: &mut ViewContext<Self>) {
        let app = ctx;
        let permissions = BlocklistAIPermissions::as_ref(app);
        let current_permissions = permissions.permissions_profile_for_id(app, self.profile_id);

        self.command_allowlist_mouse_state_handles = current_permissions
            .command_allowlist
            .iter()
            .map(|_| Default::default())
            .collect();

        self.command_denylist_mouse_state_handles = current_permissions
            .command_denylist
            .iter()
            .map(|_| Default::default())
            .collect();

        self.directory_allowlist_mouse_state_handles = current_permissions
            .directory_allowlist
            .iter()
            .map(|_| Default::default())
            .collect();
    }

    fn refresh_profile_state(&mut self, ctx: &mut ViewContext<Self>) {
        let permissions = BlocklistAIPermissions::as_ref(ctx);
        let current_permissions = permissions.permissions_profile_for_id(ctx, self.profile_id);
        let ai_settings = AISettings::as_ref(ctx);

        let apply_code_diffs_disabled = !ai_settings.is_code_diffs_permissions_editable(ctx);
        let read_files_disabled = !ai_settings.is_read_files_permissions_editable(ctx);
        let execute_commands_disabled = !ai_settings.is_execute_commands_permissions_editable(ctx);
        let write_to_pty_disabled = !ai_settings.is_write_to_pty_permissions_editable(ctx);
        let computer_use_disabled = !ai_settings.is_computer_use_permissions_editable(ctx);
        let ask_user_question_disabled =
            !ai_settings.is_ask_user_question_permissions_editable(ctx);
        Self::refresh_execution_profile_dropdown_menu(
            &self.apply_code_diffs_dropdown,
            current_permissions.apply_code_diffs,
            apply_code_diffs_disabled,
            ctx,
        );
        Self::refresh_execution_profile_dropdown_menu(
            &self.read_files_dropdown,
            current_permissions.read_files,
            read_files_disabled,
            ctx,
        );
        Self::refresh_execution_profile_dropdown_menu(
            &self.execute_commands_dropdown,
            current_permissions.execute_commands,
            execute_commands_disabled,
            ctx,
        );
        Self::refresh_write_to_pty_dropdown_menu(
            &self.write_to_pty_dropdown,
            current_permissions.write_to_pty,
            write_to_pty_disabled,
            ctx,
        );
        Self::refresh_computer_use_dropdown_menu(
            &self.computer_use_dropdown,
            current_permissions.computer_use,
            computer_use_disabled,
            ctx,
        );
        Self::refresh_ask_user_question_dropdown_menu(
            &self.ask_user_question_dropdown,
            current_permissions.ask_user_question,
            ask_user_question_disabled,
            ctx,
        );
        Self::update_profile_name_editor(&self.profile_name_editor, &current_permissions, ctx);
    }

    fn refresh_execution_profile_dropdown_menu(
        menu: &ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
        current_permission: ActionPermission,
        disabled: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            if !disabled {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            let active = match current_permission {
                ActionPermission::AgentDecides => 0,
                ActionPermission::AlwaysAllow => 1,
                ActionPermission::AlwaysAsk => 2,
            };

            menu.set_selected_by_index(active, ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    fn refresh_write_to_pty_dropdown_menu(
        menu: &ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
        current_permission: WriteToPtyPermission,
        disabled: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            if !disabled {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            let active = match current_permission {
                WriteToPtyPermission::AlwaysAllow => 0,
                WriteToPtyPermission::AlwaysAsk => 1,
                WriteToPtyPermission::AskOnFirstWrite => 2,
            };

            menu.set_selected_by_index(active, ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    fn refresh_computer_use_dropdown_menu(
        menu: &ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
        current_permission: super::ComputerUsePermission,
        disabled: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            if !disabled {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            let active = match current_permission {
                super::ComputerUsePermission::Never => 0,
                super::ComputerUsePermission::AlwaysAsk => 1,
                super::ComputerUsePermission::AlwaysAllow => 2,
            };

            menu.set_selected_by_index(active, ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    fn refresh_ask_user_question_dropdown_menu(
        menu: &ViewHandle<Dropdown<ExecutionProfileEditorViewAction>>,
        current_permission: super::AskUserQuestionPermission,
        disabled: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            if !disabled {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            let active = match current_permission {
                super::AskUserQuestionPermission::Never => 0,
                super::AskUserQuestionPermission::AskExceptInAutoApprove => 1,
                super::AskUserQuestionPermission::AlwaysAsk => 2,
            };

            menu.set_selected_by_index(active, ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    pub fn pane_configuration(&self) -> ModelHandle<PaneConfiguration> {
        self.pane_configuration.clone()
    }

    pub fn focus(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.focus_self();
    }

    fn save_profile_name_if_valid(&self, ctx: &mut ViewContext<Self>) {
        let new_name = self.profile_name_editor.read(ctx, |editor, ctx| {
            editor.buffer_text(ctx).trim().to_string()
        });

        if new_name.is_empty() {
            return;
        }

        let current_name = BlocklistAIPermissions::as_ref(ctx)
            .permissions_profile_for_id(ctx, self.profile_id)
            .name;

        if current_name != new_name {
            AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                profiles_model.set_profile_name(self.profile_id, &new_name, ctx);
            });
        }
    }

    fn update_profile_name_editor(
        profile_name_editor: &ViewHandle<EditorView>,
        profile_data: &AIExecutionProfile,
        ctx: &mut ViewContext<Self>,
    ) {
        profile_name_editor.update(ctx, |editor, ctx| {
            let display_name = if profile_data.is_default_profile {
                "Default".to_string()
            } else {
                profile_data.name.clone()
            };

            // Only update the buffer text if it's different from what's currently displayed
            // This preserves the cursor position when the text hasn't changed
            let current_text = editor.buffer_text(ctx);
            if current_text != display_name {
                editor.set_buffer_text(&display_name, ctx);
            }

            if profile_data.is_default_profile {
                editor.set_interaction_state(InteractionState::Disabled, ctx);
            }
        });
    }

    fn update_all_editor_interaction_states(view: &Self, ctx: &mut ViewContext<Self>) {
        let is_any_ai_enabled = AISettings::as_ref(ctx).is_any_ai_enabled(ctx);

        Self::update_editor_interaction_state(
            view.command_denylist_editor.as_ref(ctx).editor().clone(),
            is_any_ai_enabled,
            ctx,
        );

        Self::update_editor_interaction_state(
            view.command_allowlist_editor.as_ref(ctx).editor().clone(),
            is_any_ai_enabled,
            ctx,
        );

        Self::update_editor_interaction_state(
            view.directory_allowlist_editor.as_ref(ctx).editor().clone(),
            is_any_ai_enabled,
            ctx,
        );
    }

    fn update_editor_interaction_state(
        editor: ViewHandle<EditorView>,
        is_editable: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        editor.update(ctx, |editor, ctx| {
            if !is_editable {
                editor.set_interaction_state(InteractionState::Disabled, ctx);
            } else {
                editor.set_interaction_state(InteractionState::Editable, ctx);
            }
        });
    }
}

mod ui_helpers;

impl View for ExecutionProfileEditorView {
    fn ui_name() -> &'static str {
        "ExecutionProfileEditorView"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        use ui_helpers::*;

        let permissions = BlocklistAIPermissions::as_ref(app);
        let profile_data = permissions.permissions_profile_for_id(app, self.profile_id);

        let mut column = Flex::column()
            .with_child(render_header_section(
                appearance,
                &self.profile_name_editor,
                profile_data.is_default_profile,
            ))
            .with_child(render_permissions_section(
                appearance,
                self,
                &profile_data,
                app,
            ));

        if !profile_data.is_default_profile {
            column.add_child(ChildView::new(&self.delete_button).finish());
        }

        let content = Container::new(column.finish())
            .with_uniform_padding(16.)
            .finish();

        ClippedScrollable::vertical(
            self.clipped_scroll_state.clone(),
            Align::new(content).top_center().finish(),
            ScrollbarWidth::Auto,
            appearance.theme().nonactive_ui_detail().into(),
            appearance.theme().active_ui_detail().into(),
            warpui::elements::Fill::None,
        )
        .finish()
    }
}

impl Entity for ExecutionProfileEditorView {
    type Event = ExecutionProfileEditorViewEvent;
}

impl TypedActionView for ExecutionProfileEditorView {
    type Action = ExecutionProfileEditorViewAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            ExecutionProfileEditorViewAction::Save => {
                // TODO: Implement save logic
                log::info!("Save profile");
            }
            ExecutionProfileEditorViewAction::Close => {
                ctx.emit(ExecutionProfileEditorViewEvent::Pane(PaneEvent::Close));
            }
            ExecutionProfileEditorViewAction::SetApplyCodeDiffs { permission } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.set_apply_code_diffs(self.profile_id, permission, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::SetReadFiles { permission } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.set_read_files(self.profile_id, permission, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::SetExecuteCommands { permission } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.set_execute_commands(self.profile_id, permission, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::SetWriteToPty { permission } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.set_write_to_pty(self.profile_id, permission, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::SetComputerUse { permission } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.set_computer_use(self.profile_id, permission, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::SetAskUserQuestion { permission } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.set_ask_user_question(self.profile_id, *permission, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::AddToCommandAllowlist { predicate } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.add_to_command_allowlist(self.profile_id, predicate, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::RemoveFromCommandAllowlist { predicate } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.remove_from_command_allowlist(self.profile_id, predicate, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::AddToCommandDenylist { predicate } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.add_to_command_denylist(self.profile_id, predicate, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::RemoveFromCommandDenylist { predicate } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.remove_from_command_denylist(self.profile_id, predicate, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::AddToDirectoryAllowlist { path } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.add_to_directory_allowlist(self.profile_id, path, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::RemoveFromDirectoryAllowlist { path } => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.remove_from_directory_allowlist(self.profile_id, path, ctx);
                });
                ctx.notify();
            }
            ExecutionProfileEditorViewAction::DeleteProfile => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    profiles_model.delete_profile(self.profile_id, ctx);
                });
                ctx.emit(ExecutionProfileEditorViewEvent::Pane(PaneEvent::Close));
            }
        }
    }
}

impl BackingView for ExecutionProfileEditorView {
    type PaneHeaderOverflowMenuAction = ExecutionProfileEditorViewAction;
    type CustomAction = ();
    type AssociatedData = ();

    fn handle_pane_header_overflow_menu_action(
        &mut self,
        _action: &Self::PaneHeaderOverflowMenuAction,
        _ctx: &mut warpui::ViewContext<Self>,
    ) {
        self.handle_action(_action, _ctx)
    }

    fn close(&mut self, ctx: &mut warpui::ViewContext<Self>) {
        self.save_profile_name_if_valid(ctx);
        ctx.emit(ExecutionProfileEditorViewEvent::Pane(PaneEvent::Close));
    }

    fn focus_contents(&mut self, ctx: &mut warpui::ViewContext<Self>) {
        self.focus(ctx);
    }

    fn render_header_content(
        &self,
        _ctx: &view::HeaderRenderContext,
        _app: &AppContext,
    ) -> view::HeaderContent {
        view::HeaderContent::Standard(view::StandardHeader {
            title: HEADER_TEXT.into(),
            title_secondary: None,
            title_style: None,
            title_clip_config: warpui::text_layout::ClipConfig::start(),
            title_max_width: None,
            left_of_title: None,
            right_of_title: None,
            left_of_overflow: None,
            options: view::StandardHeaderOptions {
                always_show_icons: true,
                ..Default::default()
            },
        })
    }

    fn set_focus_handle(&mut self, focus_handle: PaneFocusHandle, _ctx: &mut ViewContext<Self>) {
        self.focus_handle = Some(focus_handle);
    }
}
