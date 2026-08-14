use settings::Setting;
use warp_core::features::FeatureFlag;
use warpui::elements::{Flex, ParentElement};
use warpui::ui_components::components::UiComponent;
use warpui::ui_components::switch::SwitchStateHandle;
use warpui::{Element, Entity, SingletonEntity, TypedActionView, View, ViewContext, ViewHandle};

use crate::appearance::Appearance;
use crate::settings::log_setting_result;
use crate::settings_view::settings_page::{ToggleState, render_body_item, render_dropdown_item};
use crate::util::file::external_editor::settings::{
    EditorChoice, EditorLayout, OpenConversationPreference,
};
use crate::util::file::external_editor::{EditorSettings, SUPPORTED_EDITORS};
use crate::view_components::{Dropdown, DropdownItem};

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalEditorAction {
    SetEditor(EditorChoice),
    SetCodePanelsEditor(EditorChoice),
    SetLayout(EditorLayout),
    SetConversationLayout(OpenConversationPreference),
    TogglePreferMarkdownViewer,
    ToggleTabbedEditorView,
}

pub struct ExternalEditorView {
    editor_dropdown: ViewHandle<Dropdown<ExternalEditorAction>>,
    code_panels_editor_dropdown: ViewHandle<Dropdown<ExternalEditorAction>>,
    layout_dropdown: ViewHandle<Dropdown<ExternalEditorAction>>,
    conversation_layout_dropdown: ViewHandle<Dropdown<ExternalEditorAction>>,
    tabbed_editor_view_mouse_state: SwitchStateHandle,
    prefer_markdown_viewer_switch: SwitchStateHandle,
}

impl ExternalEditorView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let editor = *EditorSettings::as_ref(ctx).open_file_editor;
        let code_panels_editor = *EditorSettings::as_ref(ctx).open_code_panels_file_editor;
        let layout = *EditorSettings::as_ref(ctx).open_file_layout;
        let conversation_layout = *EditorSettings::as_ref(ctx).open_conversation_layout_preference;

        let editor_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            Self::init_editor_dropdown(
                &editor,
                &mut dropdown,
                ExternalEditorAction::SetEditor,
                ctx,
            );
            dropdown
        });
        let code_panels_editor_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            Self::init_editor_dropdown(
                &code_panels_editor,
                &mut dropdown,
                ExternalEditorAction::SetCodePanelsEditor,
                ctx,
            );
            dropdown
        });
        let layout_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            Self::init_layout_dropdown(&layout, &mut dropdown, ctx);
            dropdown
        });
        let conversation_layout_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            Self::init_conversation_layout_dropdown(&conversation_layout, &mut dropdown, ctx);
            dropdown
        });

        ctx.subscribe_to_model(&EditorSettings::handle(ctx), |me, _, _, ctx| {
            let editor = *EditorSettings::as_ref(ctx).open_file_editor;
            let code_panels_editor = *EditorSettings::as_ref(ctx).open_code_panels_file_editor;
            let layout = *EditorSettings::as_ref(ctx).open_file_layout;
            let conversation_layout =
                *EditorSettings::as_ref(ctx).open_conversation_layout_preference;
            me.editor_dropdown.update(ctx, |dropdown, ctx| {
                Self::init_editor_dropdown(&editor, dropdown, ExternalEditorAction::SetEditor, ctx);
            });
            me.code_panels_editor_dropdown.update(ctx, |dropdown, ctx| {
                Self::init_editor_dropdown(
                    &code_panels_editor,
                    dropdown,
                    ExternalEditorAction::SetCodePanelsEditor,
                    ctx,
                );
            });
            me.layout_dropdown.update(ctx, |dropdown, ctx| {
                Self::init_layout_dropdown(&layout, dropdown, ctx);
            });
            me.conversation_layout_dropdown
                .update(ctx, |dropdown, ctx| {
                    Self::init_conversation_layout_dropdown(&conversation_layout, dropdown, ctx);
                });
            ctx.notify();
        });

        Self {
            editor_dropdown,
            code_panels_editor_dropdown,
            layout_dropdown,
            conversation_layout_dropdown,
            tabbed_editor_view_mouse_state: Default::default(),
            prefer_markdown_viewer_switch: Default::default(),
        }
    }

    fn init_editor_dropdown(
        editor: &EditorChoice,
        dropdown: &mut Dropdown<ExternalEditorAction>,
        mut make_action: impl FnMut(EditorChoice) -> ExternalEditorAction,
        ctx: &mut ViewContext<Dropdown<ExternalEditorAction>>,
    ) {
        let default_name = "Default App";
        let mut items = vec![DropdownItem::new(
            default_name,
            make_action(EditorChoice::SystemDefault),
        )];
        items.push(DropdownItem::new("Warp", make_action(EditorChoice::Warp)));
        items.push(DropdownItem::new(
            "$EDITOR",
            make_action(EditorChoice::EnvEditor),
        ));
        for editor in SUPPORTED_EDITORS {
            if editor.is_installed(ctx) {
                items.push(DropdownItem::new(
                    editor.to_string(),
                    make_action(EditorChoice::ExternalEditor(*editor)),
                ));
            }
        }

        dropdown.set_items(items, ctx);
        match editor {
            EditorChoice::ExternalEditor(editor) => {
                dropdown.set_selected_by_name(editor.to_string(), ctx)
            }
            EditorChoice::Warp => dropdown.set_selected_by_name("Warp", ctx),
            EditorChoice::EnvEditor => dropdown.set_selected_by_name("$EDITOR", ctx),
            EditorChoice::SystemDefault => dropdown.set_selected_by_name(default_name, ctx),
        }
    }

    fn init_layout_dropdown(
        layout: &EditorLayout,
        dropdown: &mut Dropdown<ExternalEditorAction>,
        ctx: &mut ViewContext<Dropdown<ExternalEditorAction>>,
    ) {
        dropdown.set_items(
            vec![
                DropdownItem::new(
                    "Split Pane",
                    ExternalEditorAction::SetLayout(EditorLayout::SplitPane),
                ),
                DropdownItem::new(
                    "New Tab",
                    ExternalEditorAction::SetLayout(EditorLayout::NewTab),
                ),
            ],
            ctx,
        );
        dropdown.set_selected_by_name(
            match layout {
                EditorLayout::SplitPane => "Split Pane",
                EditorLayout::NewTab => "New Tab",
            },
            ctx,
        );
    }

    fn init_conversation_layout_dropdown(
        layout: &OpenConversationPreference,
        dropdown: &mut Dropdown<ExternalEditorAction>,
        ctx: &mut ViewContext<Dropdown<ExternalEditorAction>>,
    ) {
        dropdown.set_items(
            vec![
                DropdownItem::new(
                    "New Tab",
                    ExternalEditorAction::SetConversationLayout(OpenConversationPreference::NewTab),
                ),
                DropdownItem::new(
                    "Split Pane",
                    ExternalEditorAction::SetConversationLayout(
                        OpenConversationPreference::SplitPane,
                    ),
                ),
            ],
            ctx,
        );
        dropdown.set_selected_by_name(
            match layout {
                OpenConversationPreference::NewTab => "New Tab",
                OpenConversationPreference::SplitPane => "Split Pane",
            },
            ctx,
        );
    }

    fn set_editor(&mut self, editor: &EditorChoice, ctx: &mut ViewContext<Self>) {
        EditorSettings::handle(ctx).update(ctx, |settings, ctx| {
            log_setting_result(
                settings.open_file_editor.set_value(*editor, ctx),
                "open_file_editor",
            );
        });
    }

    fn set_code_panels_editor(&mut self, editor: &EditorChoice, ctx: &mut ViewContext<Self>) {
        EditorSettings::handle(ctx).update(ctx, |settings, ctx| {
            log_setting_result(
                settings
                    .open_code_panels_file_editor
                    .set_value(*editor, ctx),
                "open_code_panels_file_editor",
            );
        });
    }

    fn set_layout(&mut self, layout: &EditorLayout, ctx: &mut ViewContext<Self>) {
        EditorSettings::handle(ctx).update(ctx, |settings, ctx| {
            log_setting_result(
                settings.open_file_layout.set_value(*layout, ctx),
                "open_file_layout",
            );
        });
    }

    fn set_conversation_layout(
        &mut self,
        layout: &OpenConversationPreference,
        ctx: &mut ViewContext<Self>,
    ) {
        EditorSettings::handle(ctx).update(ctx, |settings, ctx| {
            log_setting_result(
                settings
                    .open_conversation_layout_preference
                    .set_value(*layout, ctx),
                "open_conversation_layout_preference",
            );
        });
    }

    fn toggle_prefer_markdown_viewer(&mut self, ctx: &mut ViewContext<Self>) {
        EditorSettings::handle(ctx).update(ctx, |settings, ctx| {
            log_setting_result(
                settings
                    .prefer_markdown_viewer
                    .set_value(!*settings.prefer_markdown_viewer, ctx),
                "prefer_markdown_viewer",
            );
        });
    }

    fn toggle_tabbed_editor_view(&mut self, ctx: &mut ViewContext<Self>) {
        EditorSettings::handle(ctx).update(ctx, |settings, ctx| {
            log_setting_result(
                settings
                    .prefer_tabbed_editor_view
                    .set_value(!*settings.prefer_tabbed_editor_view, ctx),
                "prefer_tabbed_editor_view",
            );
        });
    }
}

impl Entity for ExternalEditorView {
    type Event = ();
}

impl View for ExternalEditorView {
    fn ui_name() -> &'static str {
        "ExternalEditorView"
    }

    fn render(&self, app: &warpui::AppContext) -> Box<dyn warpui::Element> {
        let appearance = Appearance::as_ref(app);
        let mut column = Flex::column()
            .with_child(render_dropdown_item(
                appearance,
                "Editor for opening file links",
                Some("Editor used when opening file links from the terminal."),
                None,
                None,
                &self.editor_dropdown,
            ))
            .with_child(render_dropdown_item(
                appearance,
                "Editor for code panels",
                Some("Editor used for files opened from Code Review, Project Explorer, and Global Search."),
                None,
                None,
                &self.code_panels_editor_dropdown,
            ))
            .with_child(render_dropdown_item(
                appearance,
                "File opening layout",
                Some("Choose whether files opened in Warp use a split pane or a new tab."),
                None,
                None,
                &self.layout_dropdown,
            ))
            .with_child(render_dropdown_item(
                appearance,
                "Conversation opening layout",
                Some("Choose whether existing agent conversations open in a new tab or a split pane."),
                None,
                None,
                &self.conversation_layout_dropdown,
            ));

        if FeatureFlag::TabbedEditorView.is_enabled() {
            column.add_child(render_body_item::<ExternalEditorAction>(
                "Group files into a single editor pane".to_string(),
                None,
                ToggleState::Enabled,
                appearance,
                appearance
                    .ui_builder()
                    .switch(self.tabbed_editor_view_mouse_state.clone())
                    .check(*EditorSettings::as_ref(app).prefer_tabbed_editor_view)
                    .build()
                    .on_click(|ctx, _, _| {
                        ctx.dispatch_typed_action(ExternalEditorAction::ToggleTabbedEditorView);
                    })
                    .finish(),
                Some("Group files opened in the same tab into one editor pane.".to_string()),
            ));
        }

        column.add_child(render_body_item::<ExternalEditorAction>(
            "Open Markdown files in Warp's Markdown Viewer by default".to_string(),
            None,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.prefer_markdown_viewer_switch.clone())
                .check(*EditorSettings::as_ref(app).prefer_markdown_viewer)
                .build()
                .on_click(|ctx, _, _| {
                    ctx.dispatch_typed_action(ExternalEditorAction::TogglePreferMarkdownViewer);
                })
                .finish(),
            None,
        ));

        column.finish()
    }
}

impl TypedActionView for ExternalEditorView {
    type Action = ExternalEditorAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            ExternalEditorAction::SetEditor(editor) => self.set_editor(editor, ctx),
            ExternalEditorAction::SetCodePanelsEditor(editor) => {
                self.set_code_panels_editor(editor, ctx)
            }
            ExternalEditorAction::SetLayout(layout) => self.set_layout(layout, ctx),
            ExternalEditorAction::SetConversationLayout(layout) => {
                self.set_conversation_layout(layout, ctx)
            }
            ExternalEditorAction::TogglePreferMarkdownViewer => {
                self.toggle_prefer_markdown_viewer(ctx)
            }
            ExternalEditorAction::ToggleTabbedEditorView => self.toggle_tabbed_editor_view(ctx),
        }
    }
}
