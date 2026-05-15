use crate::ai::execution_profiles::{AIExecutionProfile, ActionPermission};
use crate::editor::EditorView;
use crate::settings::AISettings;
use crate::ui_components::icons::Icon;
use crate::view_components::{Dropdown, SubmittableTextInput};
use crate::Appearance;
use pathfinder_geometry::vector::vec2f;
use warpui::elements::Hoverable;
use warpui::elements::MouseStateHandle;
use warpui::elements::{
    ChildAnchor, ChildView, ConstrainedBox, Container, Flex, OffsetPositioning, ParentAnchor,
    ParentElement, ParentOffsetBounds, Shrinkable, Stack, Text,
};
use warpui::fonts::{Properties, Weight};
use warpui::ui_components::components::UiComponent;
use warpui::{Element, SingletonEntity, ViewHandle};

use super::ExecutionProfileEditorView;
use super::ExecutionProfileEditorViewAction;

use crate::settings_view::{render_input_list, render_separator, InputListItem};

pub const DISABLED_AI_OPTION_TOOLTIP_MESSAGE: &str = "Enable AI to customize this option.";
pub fn render_header_section(
    appearance: &Appearance,
    profile_name_editor: &ViewHandle<EditorView>,
    is_default_profile: bool,
) -> Box<dyn Element> {
    let mut column = Flex::column()
        .with_child(render_header_title(appearance))
        .with_child(render_header_name_label(appearance))
        .with_child(
            Container::new(
                appearance
                    .ui_builder()
                    .text_input(profile_name_editor.clone())
                    .build()
                    .finish(),
            )
            .with_margin_top(8.)
            .with_margin_bottom(8.)
            .finish(),
        );

    if is_default_profile {
        column.add_child(render_info_section(
            "Default profile name cannot be changed.",
            None,
            appearance,
        ));
    }

    Container::new(column.finish())
        .with_margin_bottom(24.)
        .finish()
}

fn render_header_title(appearance: &Appearance) -> Box<dyn Element> {
    Text::new_inline("Edit Profile", appearance.ui_font_family(), 16.)
        .with_style(Properties::default().weight(Weight::Bold))
        .with_color(appearance.theme().active_ui_text_color().into())
        .finish()
}

fn render_header_name_label(appearance: &Appearance) -> Box<dyn Element> {
    Container::new(
        Text::new("Name", appearance.ui_font_family(), 13.)
            .with_color(appearance.theme().active_ui_text_color().into())
            .finish(),
    )
    .with_margin_top(16.)
    .finish()
}

pub fn render_section_label(label: &str, appearance: &Appearance) -> Box<dyn Element> {
    Container::new(
        Text::new(label.to_string(), appearance.ui_font_family(), 12.)
            .with_color(appearance.theme().disabled_ui_text_color().into())
            .finish(),
    )
    .with_margin_top(12.)
    .with_margin_bottom(20.)
    .finish()
}

fn render_info_section(
    text: &str,
    _subtext: Option<&str>,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let description_color = appearance.theme().disabled_ui_text_color();
    let alert_icon = Container::new(
        ConstrainedBox::new(
            Icon::AlertCircle
                .to_warpui_icon(
                    appearance
                        .theme()
                        .sub_text_color(appearance.theme().surface_2()),
                )
                .finish(),
        )
        .with_width(14.)
        .with_height(14.)
        .finish(),
    )
    .with_margin_right(4.)
    .finish();
    let text = Text::new(
        text.to_string(),
        appearance.ui_font_family(),
        appearance.ui_font_size(),
    )
    .with_color(description_color.into())
    .finish();
    let description = Flex::row()
        .with_children([alert_icon, Shrinkable::new(1.0, text).finish()])
        .finish();
    Container::new(description).with_margin_bottom(12.).finish()
}

fn render_permission_row<T: Clone + 'static + std::fmt::Debug + Send + Sync>(
    appearance: &Appearance,
    icon: Icon,
    label: &str,
    dropdown: &ViewHandle<Dropdown<T>>,
    info_text: &str,
    show_disabled_tooltip: bool,
    tooltip_mouse_state: MouseStateHandle,
) -> Box<dyn Element> {
    let icon_elem = Container::new(
        ConstrainedBox::new(
            icon.to_warpui_icon(appearance.theme().active_ui_text_color())
                .finish(),
        )
        .with_width(16.)
        .with_height(16.)
        .finish(),
    )
    .with_margin_right(8.)
    .finish();
    let label_elem = Text::new(label.to_string(), appearance.ui_font_family(), 13.)
        .with_color(appearance.theme().active_ui_text_color().into())
        .finish();
    let icon_label_row = Flex::row()
        .with_child(icon_elem)
        .with_child(label_elem)
        .finish();
    let dropdown_element = ChildView::new(dropdown).finish();
    let dropdown_row = if show_disabled_tooltip {
        wrap_disabled_ai_option_tooltip(dropdown_element, tooltip_mouse_state, appearance)
    } else {
        dropdown_element
    };
    let info_section = Container::new(render_info_section(info_text, None, appearance))
        .with_margin_bottom(12.)
        .finish();
    Flex::column()
        .with_child(icon_label_row)
        .with_child(dropdown_row)
        .with_child(info_section)
        .finish()
}

pub fn render_permissions_section(
    appearance: &Appearance,
    view: &ExecutionProfileEditorView,
    profile_data: &AIExecutionProfile,
    app: &warpui::AppContext,
) -> Box<dyn Element> {
    let ai_settings = AISettings::as_ref(app);
    let mut column = Flex::column().with_children([
        render_separator(appearance),
        render_section_label("PERMISSIONS", appearance),
        render_permission_row(
            appearance,
            Icon::Code2,
            "Apply code diffs",
            &view.apply_code_diffs_dropdown,
            profile_data.apply_code_diffs.description(),
            !ai_settings.is_code_diffs_permissions_editable(app),
            view.tooltip_mouse_state_handles
                .apply_code_diffs_tooltip_mouse_state
                .clone(),
        ),
        render_permission_row(
            appearance,
            Icon::Notebook,
            "Read files",
            &view.read_files_dropdown,
            profile_data.read_files.description(),
            !ai_settings.is_read_files_permissions_editable(app),
            view.tooltip_mouse_state_handles
                .read_files_tooltip_mouse_state
                .clone(),
        ),
    ]);

    if profile_data.read_files == ActionPermission::AlwaysAsk
        || profile_data.read_files == ActionPermission::AgentDecides
    {
        column.add_child(render_directory_allowlist_section(
            view,
            profile_data,
            appearance,
            app,
        ));
    }

    column.add_child(render_permission_row(
        appearance,
        Icon::Terminal,
        "Execute commands",
        &view.execute_commands_dropdown,
        profile_data.execute_commands.description(),
        !ai_settings.is_execute_commands_permissions_editable(app),
        view.tooltip_mouse_state_handles
            .execute_commands_tooltip_mouse_state
            .clone(),
    ));

    match profile_data.execute_commands {
        ActionPermission::AlwaysAllow => {
            column.add_child(render_command_denylist_section(
                view,
                profile_data,
                appearance,
                app,
            ));
        }
        ActionPermission::AlwaysAsk => {
            column.add_child(render_command_allowlist_section(
                view,
                profile_data,
                appearance,
                app,
            ));
        }
        ActionPermission::AgentDecides => {
            column.add_children([
                render_command_allowlist_section(view, profile_data, appearance, app),
                render_command_denylist_section(view, profile_data, appearance, app),
            ]);
        }
    }

    column.add_child(render_permission_row(
        appearance,
        Icon::Workflow,
        "Interact with running commands",
        &view.write_to_pty_dropdown,
        profile_data.write_to_pty.description(),
        !ai_settings.is_write_to_pty_permissions_editable(app),
        view.tooltip_mouse_state_handles
            .write_to_pty_tooltip_mouse_state
            .clone(),
    ));

    column.add_child(render_permission_row(
        appearance,
        Icon::Laptop,
        "Computer use",
        &view.computer_use_dropdown,
        profile_data.computer_use.description(),
        !ai_settings.is_computer_use_permissions_editable(app),
        view.tooltip_mouse_state_handles
            .computer_use_tooltip_mouse_state
            .clone(),
    ));

    column.add_child(render_permission_row(
        appearance,
        Icon::MessageText,
        "Ask questions",
        &view.ask_user_question_dropdown,
        profile_data.ask_user_question.description(),
        !ai_settings.is_ask_user_question_permissions_editable(app),
        view.tooltip_mouse_state_handles
            .ask_user_question_tooltip_mouse_state
            .clone(),
    ));

    Container::new(column.finish())
        .with_margin_bottom(24.)
        .finish()
}

fn create_section_header(
    label: &str,
    description: &str,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let label_elem = Text::new(label.to_string(), appearance.ui_font_family(), 13.)
        .with_color(appearance.theme().active_ui_text_color().into())
        .finish();

    let desc_elem = Text::new(description.to_string(), appearance.ui_font_family(), 11.)
        .with_color(
            appearance
                .theme()
                .sub_text_color(appearance.theme().surface_1())
                .into(),
        )
        .finish();

    Container::new(
        Flex::column()
            .with_child(label_elem)
            .with_child(desc_elem)
            .finish(),
    )
    .with_margin_bottom(4.)
    .finish()
}

#[allow(clippy::too_many_arguments)]
fn render_list_section<T, F, D>(
    label: &str,
    description: &str,
    items: &[T],
    mouse_handles: &[MouseStateHandle],
    editor: Option<&ViewHandle<SubmittableTextInput>>,
    on_remove_action: F,
    display_fn: D,
    appearance: &Appearance,
    is_editable: bool,
    tooltip_mouse_state: MouseStateHandle,
) -> Box<dyn Element>
where
    T: Clone,
    F: Fn(T) -> ExecutionProfileEditorViewAction,
    D: Fn(&T) -> String,
{
    let input_items: Vec<InputListItem<ExecutionProfileEditorViewAction>> = items
        .iter()
        .cloned()
        .zip(mouse_handles.iter().cloned())
        .rev()
        .map(|(item, mouse_state_handle)| InputListItem {
            item: display_fn(&item),
            mouse_state_handle,
            on_remove_action: on_remove_action(item),
            is_disabled: !is_editable,
            tooltip_mouse_state: None,
        })
        .collect();

    let list = render_input_list(None, input_items, editor, appearance);
    let list_element = if !is_editable {
        wrap_disabled_ai_option_tooltip(list, tooltip_mouse_state, appearance)
    } else {
        list
    };

    let column = Flex::column()
        .with_child(create_section_header(label, description, appearance))
        .with_child(list_element);

    Container::new(column.finish())
        .with_margin_bottom(16.)
        .finish()
}

fn render_directory_allowlist_section(
    view: &ExecutionProfileEditorView,
    profile_data: &AIExecutionProfile,
    appearance: &Appearance,
    app: &warpui::AppContext,
) -> Box<dyn Element> {
    let ai_settings = AISettings::as_ref(app);
    let is_editable = ai_settings.is_directory_allowlist_editable(app);

    render_list_section(
        "Directory allowlist",
        "Give the agent file access to certain directories.",
        &profile_data.directory_allowlist,
        &view.directory_allowlist_mouse_state_handles,
        Some(&view.directory_allowlist_editor),
        |path| ExecutionProfileEditorViewAction::RemoveFromDirectoryAllowlist { path },
        |path| path.display().to_string(),
        appearance,
        is_editable,
        view.tooltip_mouse_state_handles
            .directory_allowlist_editor_tooltip_mouse_state
            .clone(),
    )
}
fn render_command_allowlist_section(
    view: &ExecutionProfileEditorView,
    profile_data: &AIExecutionProfile,
    appearance: &Appearance,
    app: &warpui::AppContext,
) -> Box<dyn Element> {
    let ai_settings = AISettings::as_ref(app);
    let is_editable = ai_settings.is_command_allowlist_editable(app);

    render_list_section(
        "Command allowlist",
        "Regular expressions to match commands the agent can automatically execute.",
        &profile_data.command_allowlist,
        &view.command_allowlist_mouse_state_handles,
        Some(&view.command_allowlist_editor),
        |predicate| ExecutionProfileEditorViewAction::RemoveFromCommandAllowlist { predicate },
        |item| item.to_string(),
        appearance,
        is_editable,
        view.tooltip_mouse_state_handles
            .command_allowlist_editor_tooltip_mouse_state
            .clone(),
    )
}

fn render_command_denylist_section(
    view: &ExecutionProfileEditorView,
    profile_data: &AIExecutionProfile,
    appearance: &Appearance,
    app: &warpui::AppContext,
) -> Box<dyn Element> {
    let ai_disabled = !AISettings::as_ref(app).is_any_ai_enabled(app);

    let input_items: Vec<InputListItem<ExecutionProfileEditorViewAction>> = profile_data
        .command_denylist
        .iter()
        .cloned()
        .zip(view.command_denylist_mouse_state_handles.iter().cloned())
        .rev()
        .map(|(predicate, mouse_state_handle)| InputListItem {
            item: predicate.to_string(),
            mouse_state_handle,
            on_remove_action: ExecutionProfileEditorViewAction::RemoveFromCommandDenylist {
                predicate,
            },
            is_disabled: ai_disabled,
            tooltip_mouse_state: None,
        })
        .collect();

    let list = render_input_list(
        None,
        input_items,
        Some(&view.command_denylist_editor),
        appearance,
    );

    let mut column = Flex::column().with_child(create_section_header(
        "Command denylist",
        "Regular expressions to match commands the agent should always ask permission to execute.",
        appearance,
    ));
    column = column.with_child(list);

    Container::new(column.finish())
        .with_margin_bottom(16.)
        .finish()
}

pub fn wrap_disabled_ai_option_tooltip(
    child: Box<dyn Element>,
    mouse_state: MouseStateHandle,
    appearance: &Appearance,
) -> Box<dyn Element> {
    Hoverable::new(mouse_state, |state| {
        let mut stack = Stack::new().with_child(child);
        if state.is_hovered() {
            let tooltip = appearance
                .ui_builder()
                .tool_tip(DISABLED_AI_OPTION_TOOLTIP_MESSAGE.to_string())
                .build()
                .finish();

            stack.add_positioned_child(
                tooltip,
                OffsetPositioning::offset_from_parent(
                    vec2f(0., -4.),
                    ParentOffsetBounds::Unbounded,
                    ParentAnchor::TopLeft,
                    ChildAnchor::BottomLeft,
                ),
            );
        }
        stack.finish()
    })
    .finish()
}
