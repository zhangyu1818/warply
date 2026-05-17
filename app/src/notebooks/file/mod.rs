use std::{
    mem,
    path::{Path, PathBuf},
    sync::Arc,
};

use pathfinder_geometry::vector::vec2f;
use warp_util::path::user_friendly_path;
#[cfg(feature = "local_fs")]
use warpui::clipboard::ClipboardContent;
use warpui::{
    accessibility::{AccessibilityContent, WarpA11yRole},
    elements::{
        Align, Container, CrossAxisAlignment, DispatchEventResult, Empty, EventHandler, Flex,
        MainAxisAlignment, MainAxisSize, MouseStateHandle, ParentElement, SavePosition, Shrinkable,
        Text,
    },
    keymap::EditableBinding,
    presenter::ChildView,
    ui_components::{
        button::{ButtonVariant, TextAndIcon, TextAndIconAlignment},
        components::{UiComponent, UiComponentStyles},
    },
    AppContext, Element, Entity, ModelHandle, SingletonEntity, TypedActionView, View, ViewContext,
    ViewHandle,
};

use super::editor::view::{EditorViewEvent, RichTextEditorConfig, RichTextEditorView};
use crate::{
    appearance::Appearance,
    cmd_or_ctrl_shift,
    editor::InteractionState,
    menu::{MenuItem, MenuItemFields},
    notebooks::{
        editor::{model::NotebooksEditorModel, rich_text_styles},
        link::{NotebookLinks, SessionSource},
        post_process_notebook, styles,
    },
    pane_group::{
        focus_state::PaneFocusHandle,
        pane::view,
        pane::view::header::components::{
            render_pane_header_buttons, render_pane_header_title_text, render_three_column_header,
            CenteredHeaderEdgeWidth,
        },
        BackingView, PaneConfiguration, PaneEvent,
    },
    settings::FontSettings,
    terminal::model::session::Session,
    ui_components::icons::Icon,
    view_components::{MarkdownToggleEvent, MarkdownToggleView},
    workspace::ActiveSession,
};
#[cfg(feature = "local_fs")]
use crate::{
    code::editor_management::CodeSource,
    util::{
        file::external_editor::EditorSettings,
        openable_file_type::{
            resolve_file_target, resolve_file_target_to_open_in_warp, FileTarget,
        },
    },
};
use warp_core::ui::icons::ICON_DIMENSIONS;
#[cfg(feature = "local_fs")]
use warp_files::{FileModel, FileModelEvent};
#[cfg(feature = "local_fs")]
use warp_util::{file::FileId, path::LineAndColumnArg};

pub use crate::util::openable_file_type::is_markdown_file;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownDisplayMode {
    Rendered,
    Raw,
}

pub struct FileNotebookView {
    location: Option<FileLocation>,
    editor: ViewHandle<RichTextEditorView>,
    retry_button_mouse_state: MouseStateHandle,
    file_state: FileState,
    #[cfg(feature = "local_fs")]
    file_id: Option<FileId>,
    pane_configuration: ModelHandle<PaneConfiguration>,
    focus_handle: Option<PaneFocusHandle>,
    links: ModelHandle<NotebookLinks>,
    view_position_id: String,
    markdown_display_mode: MarkdownDisplayMode,
    display_mode_segmented_control: ViewHandle<MarkdownToggleView>,
    #[cfg(feature = "local_fs")]
    code_source: Option<CodeSource>,
}

#[derive(Debug, Clone)]
pub enum FileNotebookEvent {
    FileLoaded,
    Pane(PaneEvent),
    #[cfg(feature = "local_fs")]
    OpenFileWithTarget {
        path: PathBuf,
        target: FileTarget,
        line_col: Option<LineAndColumnArg>,
    },
}

impl From<PaneEvent> for FileNotebookEvent {
    fn from(event: PaneEvent) -> Self {
        FileNotebookEvent::Pane(event)
    }
}

#[derive(Debug, Clone)]
pub enum FileNotebookAction {
    Focus,
    Close,
    FocusTerminalInput,
    ReloadFile,
    #[cfg(feature = "local_fs")]
    CopyFilePath,
    #[cfg(feature = "local_fs")]
    OpenInEditor,
    #[cfg(feature = "local_fs")]
    OpenAsCode,
    ToggleMarkdownDisplayMode(MarkdownDisplayMode),
}

#[derive(Debug, Clone)]
enum SourceFile {
    Local {
        local_path: PathBuf,
        session: Option<Arc<Session>>,
    },
    Static {
        title: String,
    },
}

impl SourceFile {
    fn local_path(&self) -> Option<&Path> {
        match self {
            SourceFile::Local { local_path, .. } => Some(local_path.as_path()),
            SourceFile::Static { .. } => None,
        }
    }

    fn display_name(&self) -> String {
        match self {
            SourceFile::Local { local_path, .. } => local_path.display().to_string(),
            SourceFile::Static { title } => title.clone(),
        }
    }
}

#[derive(Debug)]
enum FileState {
    NoFile,
    Loading(SourceFile),
    Error(SourceFile),
    Loaded(SourceFile),
}

impl FileState {
    fn local_path(&self) -> Option<&Path> {
        self.source().and_then(|source| source.local_path())
    }

    fn source(&self) -> Option<&SourceFile> {
        match self {
            FileState::NoFile => None,
            FileState::Loading(source) | FileState::Error(source) | FileState::Loaded(source) => {
                Some(source)
            }
        }
    }

    fn display_name(&self) -> Option<String> {
        self.source().map(SourceFile::display_name)
    }
}

pub fn init(app: &mut AppContext) {
    use warpui::keymap::macros::*;

    app.register_editable_bindings([
        EditableBinding::new(
            "notebookview:focus_terminal_input",
            "Focus Terminal Input from File",
            FileNotebookAction::FocusTerminalInput,
        )
        .with_context_predicate(id!("FileNotebookView"))
        .with_key_binding(cmd_or_ctrl_shift("l")),
        EditableBinding::new(
            "notebookview:reload_file",
            "Reload file",
            FileNotebookAction::ReloadFile,
        )
        .with_context_predicate(id!("FileNotebookView")),
    ])
}

impl FileNotebookView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let window_id = ctx.window_id();
        let links = ctx.add_model(|ctx| NotebookLinks::new(SessionSource::Active(window_id), ctx));
        let view_position_id = format!("file_notebook_view_{}", ctx.view_id());

        let editor_model = ctx.add_model(|ctx| {
            let styles = rich_text_styles(Appearance::as_ref(ctx), FontSettings::as_ref(ctx));
            NotebooksEditorModel::new(styles, window_id, ctx)
        });
        let editor_model_for_view = editor_model.clone();
        let links_for_view = links.clone();
        let editor_position_id = view_position_id.clone();
        let editor = ctx.add_typed_action_view(move |ctx| {
            let mut view = RichTextEditorView::new(
                editor_position_id,
                editor_model_for_view,
                links_for_view,
                RichTextEditorConfig {
                    max_width: Some(styles::notebook_editor_max_width()),
                    ..Default::default()
                },
                ctx,
            );
            view.set_interaction_state(InteractionState::Selectable, ctx);
            view
        });

        ctx.subscribe_to_view(&editor, Self::handle_editor_event);

        let pane_configuration = ctx.add_model(|_ctx| PaneConfiguration::new(""));

        ctx.observe(
            &ActiveSession::handle(ctx),
            Self::handle_active_session_change,
        );

        let display_mode_segmented_control = ctx.add_typed_action_view(|ctx| {
            MarkdownToggleView::new(MarkdownDisplayMode::Rendered, ctx)
        });

        ctx.subscribe_to_view(&display_mode_segmented_control, |view, _, event, ctx| {
            let MarkdownToggleEvent::ModeSelected(mode) = event;
            view.handle_action(&FileNotebookAction::ToggleMarkdownDisplayMode(*mode), ctx);
        });

        Self {
            location: None,
            editor,
            retry_button_mouse_state: Default::default(),
            file_state: FileState::NoFile,
            #[cfg(feature = "local_fs")]
            file_id: None,
            pane_configuration,
            focus_handle: None,
            links,
            view_position_id,
            markdown_display_mode: MarkdownDisplayMode::Rendered,
            display_mode_segmented_control,
            #[cfg(feature = "local_fs")]
            code_source: None,
        }
    }

    #[cfg(feature = "local_fs")]
    pub fn set_code_source(&mut self, source: Option<CodeSource>) {
        self.code_source = source;
    }

    pub fn title(&self) -> String {
        self.location
            .as_ref()
            .map(|location| location.name.clone())
            .or_else(|| self.file_state.display_name())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    pub fn focus(&self, ctx: &mut ViewContext<Self>) {
        if let Some(a11y_content) = self.accessibility_contents(ctx) {
            ctx.emit_a11y_content(a11y_content);
        }
        ctx.focus(&self.editor);
    }

    pub fn set_content(&mut self, content: &str, ctx: &mut ViewContext<Self>) {
        self.editor.update(ctx, |editor, ctx| {
            editor.reset_with_markdown(content, ctx);
        });
    }

    fn set_context(&mut self, path: &Path, session: Arc<Session>, ctx: &mut ViewContext<Self>) {
        self.location = Some(FileLocation::new(path, session.home_dir()));
        let title = self.title();
        self.pane_configuration.update(ctx, |pane_config, ctx| {
            pane_config.set_title(title, ctx);
        });
        if let Some(parent) = path.parent() {
            self.links.update(ctx, |links, ctx| {
                links.set_session_source(
                    SessionSource::Target {
                        session,
                        base_directory: parent.to_path_buf(),
                    },
                    ctx,
                )
            })
        }

        ctx.notify();
    }

    pub fn open_local(
        &mut self,
        path: impl Into<PathBuf>,
        session: Option<Arc<Session>>,
        ctx: &mut ViewContext<Self>,
    ) {
        let local_path = path.into();

        if let Some(session) = &session {
            self.set_context(&local_path, session.clone(), ctx);
        } else {
            self.pane_configuration.update(ctx, |pane_config, ctx| {
                pane_config.set_title(local_path.display().to_string(), ctx);
            });
        }

        self.file_state = FileState::Loading(SourceFile::Local {
            local_path: local_path.clone(),
            session: session.clone(),
        });

        #[cfg(feature = "local_fs")]
        {
            if let Some(prev_id) = self.file_id.take() {
                FileModel::handle(ctx).update(ctx, |m, ctx| {
                    m.cancel(prev_id);
                    m.unsubscribe(prev_id, ctx)
                });
            }

            let file_model = FileModel::handle(ctx);
            let file_id = file_model.update(ctx, |m, ctx| m.open(&local_path, true, ctx));
            let session_for_callback = session.clone();
            self.file_id = Some(file_id);

            ctx.subscribe_to_model(
                &file_model,
                move |me, file_model: ModelHandle<FileModel>, event: &FileModelEvent, ctx| {
                    if event.file_id() != file_id {
                        return;
                    }
                    match event {
                        FileModelEvent::FileLoaded { content, .. } => {
                            let cleaned = post_process_notebook(content);
                            me.set_content(&cleaned, ctx);

                            if let Some(canonical_path) = file_model.as_ref(ctx).file_path(file_id)
                            {
                                me.file_state = FileState::Loaded(SourceFile::Local {
                                    local_path: canonical_path,
                                    session: session_for_callback.clone(),
                                });
                            }

                            me.pane_configuration.update(ctx, |pane_config, ctx| {
                                pane_config.refresh_pane_header_overflow_menu_items(ctx);
                            });
                            ctx.notify();
                            ctx.emit(FileNotebookEvent::FileLoaded);
                        }
                        FileModelEvent::FailedToLoad { error, .. } => {
                            log::warn!("Unable to read local notebook file: {error:?}");
                            me.file_state =
                                match mem::replace(&mut me.file_state, FileState::NoFile) {
                                    FileState::NoFile => FileState::NoFile,
                                    FileState::Loading(source)
                                    | FileState::Loaded(source)
                                    | FileState::Error(source) => FileState::Error(source),
                                };
                            ctx.notify();
                        }
                        FileModelEvent::FileUpdated { content, .. } => {
                            let cleaned = post_process_notebook(content);
                            me.set_content(&cleaned, ctx);
                        }
                        FileModelEvent::FileSaved { .. } | FileModelEvent::FailedToSave { .. } => {}
                    }
                },
            );
        }
    }

    pub fn open_static(
        &mut self,
        title: impl Into<String>,
        content: &str,
        ctx: &mut ViewContext<Self>,
    ) {
        #[cfg(feature = "local_fs")]
        {
            if let Some(prev_id) = self.file_id.take() {
                FileModel::handle(ctx).update(ctx, |m, ctx| m.unsubscribe(prev_id, ctx));
            }
        }
        self.set_content(content, ctx);
        let title = title.into();
        self.pane_configuration.update(ctx, |pane_config, ctx| {
            pane_config.set_title(title.clone(), ctx);
            pane_config.refresh_pane_header_overflow_menu_items(ctx);
        });
        self.file_state = FileState::Loaded(SourceFile::Static { title });
    }

    fn reload_file(&mut self, ctx: &mut ViewContext<Self>) {
        let (local_path, session) = match mem::replace(&mut self.file_state, FileState::NoFile) {
            FileState::NoFile => return,
            FileState::Loading(source) | FileState::Error(source) | FileState::Loaded(source) => {
                match source {
                    SourceFile::Local {
                        local_path,
                        session,
                    } => (local_path, session),
                    SourceFile::Static { .. } => return,
                }
            }
        };
        self.open_local(local_path, session, ctx);
    }

    #[cfg(feature = "local_fs")]
    fn open_as_code(&mut self, ctx: &mut ViewContext<Self>) {
        if let Some(path) = self.local_path() {
            ctx.emit(FileNotebookEvent::Pane(PaneEvent::ReplaceWithCodePane {
                path,
                source: self.code_source.clone(),
            }));
        }
    }

    pub fn local_path(&self) -> Option<PathBuf> {
        self.file_state.local_path().map(Path::to_path_buf)
    }

    pub fn pane_configuration(&self) -> ModelHandle<PaneConfiguration> {
        self.pane_configuration.clone()
    }

    pub fn links(&self) -> ModelHandle<NotebookLinks> {
        self.links.clone()
    }

    #[cfg(feature = "local_fs")]
    fn is_markdown_file(&self) -> bool {
        self.file_state
            .local_path()
            .map(is_markdown_file)
            .unwrap_or(false)
    }

    #[cfg(not(feature = "local_fs"))]
    fn is_markdown_file(&self) -> bool {
        false
    }

    fn handle_editor_event(
        &mut self,
        _handle: ViewHandle<RichTextEditorView>,
        event: &EditorViewEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            EditorViewEvent::Focused => ctx.emit(FileNotebookEvent::Pane(PaneEvent::FocusSelf)),
            #[cfg(feature = "local_fs")]
            EditorViewEvent::OpenFile {
                path,
                line_and_column_num,
                force_open_in_warp,
            } => {
                let settings = EditorSettings::as_ref(ctx);
                let target = if *force_open_in_warp {
                    resolve_file_target_to_open_in_warp(path, settings, None)
                } else {
                    resolve_file_target(path, settings, None)
                };
                ctx.emit(FileNotebookEvent::OpenFileWithTarget {
                    path: path.clone(),
                    target,
                    line_col: *line_and_column_num,
                });
            }
            EditorViewEvent::Edited
            | EditorViewEvent::CmdEnter
            | EditorViewEvent::OpenedFindBar
            | EditorViewEvent::TextSelectionChanged
            | EditorViewEvent::EscapePressed => {}
        }
    }

    fn handle_active_session_change(
        &mut self,
        handle: ModelHandle<ActiveSession>,
        ctx: &mut ViewContext<Self>,
    ) {
        if self.location.is_some() {
            return;
        }
        let Some(path) = self.local_path() else {
            return;
        };
        if let Some(active_session) = handle.as_ref(ctx).session(ctx.window_id()) {
            if active_session.is_local() {
                self.set_context(&path, active_session, ctx);
                ctx.unsubscribe_to_model(&handle);
            }
        }
    }

    fn render_title(
        &self,
        appearance: &Appearance,
        font_settings: &FontSettings,
    ) -> Box<dyn Element> {
        let title = Text::new_inline(
            self.title(),
            appearance.ui_font_family(),
            styles::title_font_size(font_settings),
        )
        .with_color(styles::title_text_fill(appearance).into())
        .with_style(styles::TITLE_FONT_PROPERTIES)
        .finish();

        let details = self.location.as_ref().map(|location| {
            appearance
                .ui_builder()
                .span(location.breadcrumbs.clone())
                .with_style(UiComponentStyles {
                    font_color: Some(styles::title_text_fill(appearance).into_solid()),
                    ..Default::default()
                })
                .build()
                .finish()
        });

        styles::wrap_title(title, details)
    }

    fn state_style(&self, appearance: &Appearance) -> UiComponentStyles {
        UiComponentStyles {
            font_color: Some(
                appearance
                    .theme()
                    .sub_text_color(appearance.theme().background())
                    .into_solid(),
            ),
            ..Default::default()
        }
    }

    fn render_error(&self, source: &SourceFile, appearance: &Appearance) -> Box<dyn Element> {
        let error_text_color = appearance
            .theme()
            .sub_text_color(appearance.theme().background());
        let error = Flex::column()
            .with_main_axis_alignment(MainAxisAlignment::Center)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                appearance
                    .ui_builder()
                    .paragraph(format!("Could not read {}", source.display_name()))
                    .with_style(self.state_style(appearance))
                    .build()
                    .finish(),
            )
            .with_child(
                Container::new(
                    appearance
                        .ui_builder()
                        .button(ButtonVariant::Basic, self.retry_button_mouse_state.clone())
                        .with_text_and_icon_label(
                            TextAndIcon::new(
                                TextAndIconAlignment::TextFirst,
                                "Try again".to_string(),
                                Icon::Refresh.to_warpui_icon(error_text_color),
                                MainAxisSize::Min,
                                MainAxisAlignment::Center,
                                vec2f(16., 16.),
                            )
                            .with_inner_padding(4.),
                        )
                        .build()
                        .on_click(|ctx, _, _| {
                            ctx.dispatch_typed_action(FileNotebookAction::ReloadFile)
                        })
                        .finish(),
                )
                .with_margin_top(8.)
                .finish(),
            );

        Align::new(error.finish()).finish()
    }

    fn render_loading(&self, source: &SourceFile, appearance: &Appearance) -> Box<dyn Element> {
        Align::new(
            appearance
                .ui_builder()
                .paragraph(format!("Loading {}...", source.display_name()))
                .with_style(self.state_style(appearance))
                .build()
                .finish(),
        )
        .finish()
    }

    fn render_no_file(&self, appearance: &Appearance) -> Box<dyn Element> {
        Align::new(
            appearance
                .ui_builder()
                .paragraph("Missing source file".to_string())
                .with_style(self.state_style(appearance))
                .build()
                .finish(),
        )
        .finish()
    }

    fn render_body(&self, appearance: &Appearance) -> Box<dyn Element> {
        let body = match &self.file_state {
            FileState::NoFile => self.render_no_file(appearance),
            FileState::Loading(source) => self.render_loading(source, appearance),
            FileState::Error(source) => self.render_error(source, appearance),
            FileState::Loaded(_) => ChildView::new(&self.editor).finish(),
        };
        styles::wrap_body(body)
    }
}

impl Entity for FileNotebookView {
    type Event = FileNotebookEvent;
}

impl View for FileNotebookView {
    fn ui_name() -> &'static str {
        "FileNotebookView"
    }

    fn accessibility_contents(&self, _ctx: &AppContext) -> Option<AccessibilityContent> {
        Some(AccessibilityContent::new_without_help(
            format!("{} notebook", self.title()),
            WarpA11yRole::TextRole,
        ))
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        let font_settings = FontSettings::as_ref(app);

        let column = Flex::column().with_children([
            self.render_title(appearance, font_settings),
            Shrinkable::new(1., self.render_body(appearance)).finish(),
        ]);

        SavePosition::new(
            EventHandler::new(Align::new(column.finish()).top_left().finish())
                .on_left_mouse_down(|ctx, _, _| {
                    ctx.dispatch_typed_action(FileNotebookAction::Focus);
                    DispatchEventResult::StopPropagation
                })
                .finish(),
            &self.view_position_id,
        )
        .finish()
    }
}

impl TypedActionView for FileNotebookView {
    type Action = FileNotebookAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            FileNotebookAction::Focus => ctx.focus_self(),
            FileNotebookAction::Close => ctx.emit(FileNotebookEvent::Pane(PaneEvent::Close)),
            FileNotebookAction::FocusTerminalInput => {
                ctx.emit(FileNotebookEvent::Pane(PaneEvent::FocusActiveSession))
            }
            FileNotebookAction::ReloadFile => self.reload_file(ctx),
            #[cfg(feature = "local_fs")]
            FileNotebookAction::CopyFilePath => {
                if let Some(path) = self.local_path() {
                    ctx.clipboard()
                        .write(ClipboardContent::plain_text(path.display().to_string()));
                }
            }
            #[cfg(feature = "local_fs")]
            FileNotebookAction::OpenInEditor => {
                if let Some(path) = self.local_path() {
                    let settings = EditorSettings::as_ref(ctx);
                    let target = resolve_file_target(&path, settings, None);
                    ctx.emit(FileNotebookEvent::OpenFileWithTarget {
                        path,
                        target,
                        line_col: None,
                    });
                }
            }
            #[cfg(feature = "local_fs")]
            FileNotebookAction::OpenAsCode => self.open_as_code(ctx),
            FileNotebookAction::ToggleMarkdownDisplayMode(mode) => {
                self.markdown_display_mode = *mode;
                self.display_mode_segmented_control
                    .update(ctx, |control, ctx| control.set_selected_mode(*mode, ctx));

                match mode {
                    MarkdownDisplayMode::Rendered => {}
                    MarkdownDisplayMode::Raw => {
                        #[cfg(feature = "local_fs")]
                        self.open_as_code(ctx);
                    }
                }
            }
        }
    }
}

impl BackingView for FileNotebookView {
    type PaneHeaderOverflowMenuAction = FileNotebookAction;
    type CustomAction = ();
    type AssociatedData = ();

    fn handle_pane_header_overflow_menu_action(
        &mut self,
        action: &Self::PaneHeaderOverflowMenuAction,
        ctx: &mut ViewContext<Self>,
    ) {
        self.handle_action(action, ctx);
    }

    fn pane_header_overflow_menu_items(
        &self,
        _ctx: &AppContext,
    ) -> Vec<MenuItem<FileNotebookAction>> {
        let mut actions = vec![];
        if let Some(SourceFile::Local { .. }) = self.file_state.source() {
            actions.push(
                MenuItemFields::new("Refresh file")
                    .with_on_select_action(FileNotebookAction::ReloadFile)
                    .into_item(),
            );

            #[cfg(feature = "local_fs")]
            {
                actions.push(
                    MenuItemFields::new("Open in editor")
                        .with_on_select_action(FileNotebookAction::OpenInEditor)
                        .into_item(),
                );
                actions.extend([
                    MenuItem::Separator,
                    MenuItemFields::new("Copy file path")
                        .with_on_select_action(FileNotebookAction::CopyFilePath)
                        .into_item(),
                ]);
            }
        }
        actions
    }

    fn close(&mut self, ctx: &mut ViewContext<Self>) {
        #[cfg(feature = "local_fs")]
        {
            if let Some(prev_id) = self.file_id.take() {
                FileModel::handle(ctx).update(ctx, |m, ctx| m.unsubscribe(prev_id, ctx));
            }
        }
        ctx.emit(FileNotebookEvent::Pane(PaneEvent::Close));
    }

    fn focus_contents(&mut self, ctx: &mut ViewContext<Self>) {
        self.focus(ctx);
    }

    fn render_header_content(
        &self,
        ctx: &view::HeaderRenderContext,
        app: &AppContext,
    ) -> view::HeaderContent {
        let title = self.pane_configuration.as_ref(app).title().to_owned();

        if self.is_markdown_file() {
            let appearance = Appearance::as_ref(app);
            let is_pane_dragging = ctx.draggable_state.is_dragging();

            let mut right_row = Flex::row()
                .with_main_axis_alignment(MainAxisAlignment::End)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_main_axis_size(MainAxisSize::Min);

            right_row.add_child(ChildView::new(&self.display_mode_segmented_control).finish());

            let show_close_button = self
                .focus_handle
                .as_ref()
                .is_some_and(|h| h.is_in_split_pane(app));

            right_row.add_child(render_pane_header_buttons::<FileNotebookAction, ()>(
                ctx,
                appearance,
                show_close_button,
                None,
                None,
            ));

            let button_count = show_close_button as u32 + ctx.has_overflow_items as u32;
            let buttons_width = button_count as f32 * ICON_DIMENSIONS;

            let title_text = render_pane_header_title_text(
                title,
                appearance,
                warpui::text_layout::ClipConfig::start(),
            );

            view::HeaderContent::Custom {
                element: render_three_column_header(
                    Empty::new().finish(),
                    title_text,
                    right_row.finish(),
                    CenteredHeaderEdgeWidth {
                        min: buttons_width,
                        max: 220.0,
                    },
                    ctx.header_left_inset,
                    is_pane_dragging,
                ),
                has_custom_draggable_behavior: false,
            }
        } else {
            view::HeaderContent::Standard(view::StandardHeader {
                title,
                title_secondary: None,
                title_style: None,
                title_clip_config: warpui::text_layout::ClipConfig::start(),
                title_max_width: None,
                left_of_title: None,
                right_of_title: None,
                left_of_overflow: None,
                options: Default::default(),
            })
        }
    }

    fn set_focus_handle(&mut self, focus_handle: PaneFocusHandle, _ctx: &mut ViewContext<Self>) {
        self.focus_handle = Some(focus_handle);
    }
}

struct FileLocation {
    breadcrumbs: String,
    name: String,
}

impl FileLocation {
    fn new(path: &Path, home_directory: Option<&str>) -> Self {
        let breadcrumbs = match path.parent() {
            Some(directory) => {
                user_friendly_path(directory.to_string_lossy().as_ref(), home_directory)
                    .into_owned()
            }
            None => String::new(),
        };
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unnamed".to_string());

        Self { breadcrumbs, name }
    }
}
