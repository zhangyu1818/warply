use std::path::PathBuf;
use std::rc::Rc;

use crate::ai::blocklist::inline_action::code_diff_view::DiffSessionType;
use ai::diff_validation::DiffType;
use warp_core::safe_error;
use warp_files::{FileModel, FileModelEvent};
use warp_util::file::FileId;
use warp_util::file::FileSaveError;
use warp_util::standardized_path::StandardizedPath;
use warpui::SingletonEntity;
use warpui::elements::ChildView;
use warpui::{AppContext, Element, Entity, TypedActionView, View, ViewContext, ViewHandle};

use super::DiffResult;
use super::diff_viewer::DiffViewer;
use super::diff_viewer::DisplayMode;
use super::editor::NavBarBehavior;
use super::editor::scroll::{ScrollPosition, ScrollTrigger};
use super::editor::view::{CodeEditorEvent, CodeEditorView};
use crate::editor::InteractionState;

pub enum InlineDiffViewEvent {
    DiffStatusUpdated,
    FileLoaded,
    FileSaved,
    FailedToSave { error: Rc<FileSaveError> },
    DiffAccepted { diff: Rc<DiffResult> },
    UserEdited,
}

/// An inline diff viewer with optional file-backed save support.
///
/// When a backing file is registered (via [`Self::register_file`]), this view supports the full
/// accept/save/revert lifecycle through `FileModel`. Without a registered file, it behaves
/// as a read-only diff viewer for restored conversations.
pub struct InlineDiffView {
    editor: ViewHandle<CodeEditorView>,
    diff_type: Option<DiffType>,
    file_path: Option<StandardizedPath>,
    /// Whether the user has edited the diff content.
    was_edited: bool,
    /// `FileModel` file ID for the backing file. Set via [`Self::register_file`].
    ///
    /// When `Some`:
    /// - The editor is editable (interaction state follows the `DisplayMode` rules).
    /// - Accept, save, and revert operations write through `FileModel`.
    ///
    /// When `None` (restored conversations, or before registration):
    /// - The editor is selection-only (never editable).
    /// - Accept, save, and revert are no-ops.
    backing_file_id: Option<FileId>,
    session_type: Option<DiffSessionType>,
}

impl InlineDiffView {
    pub fn new(
        editor: ViewHandle<CodeEditorView>,
        diff_type: Option<DiffType>,
        display_mode: Option<DisplayMode>,
        file_path: Option<StandardizedPath>,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        ctx.subscribe_to_view(&editor, |me, _view, event, ctx| match event {
            CodeEditorEvent::DiffUpdated => {
                ctx.emit(InlineDiffViewEvent::DiffStatusUpdated);
            }
            CodeEditorEvent::UnifiedDiffComputed(diff) => {
                ctx.emit(InlineDiffViewEvent::DiffAccepted { diff: diff.clone() });
            }
            CodeEditorEvent::ContentChanged { origin } => {
                if origin.from_user() && !me.was_edited {
                    me.was_edited = true;
                    ctx.emit(InlineDiffViewEvent::UserEdited);
                }
            }
            _ => {}
        });

        let model = Self {
            editor,
            diff_type,
            file_path,
            was_edited: false,
            backing_file_id: None,
            session_type: None,
        };

        model.apply_diffs_if_any(ctx);
        if let Some(display_mode) = display_mode {
            model.set_display_mode(display_mode, ctx);
        }

        model
    }

    /// Register a file with `FileModel` for save support.
    ///
    /// The `session_type` determines whether the file is local or remote.
    /// For `Local`, the file is registered by path on the local filesystem.
    /// For `Remote`, the file is registered against the remote backend so
    /// that `save()` / `delete()` dispatch over the wire via
    /// `RemoteServerClient`.
    ///
    /// This must be called after construction for save support.
    pub fn register_file(&mut self, session_type: &DiffSessionType, ctx: &mut ViewContext<Self>) {
        let Some(file_path) = &self.file_path else {
            return;
        };

        let file_model = FileModel::handle(ctx);
        let file_id = match session_type {
            DiffSessionType::Local => {
                let Some(local_path) = file_path.to_local_path() else {
                    safe_error!(
                        safe: (
                            "Failed to convert StandardizedPath to local path; diff will be \
                            read-only"
                        ),
                        full: (
                            "Failed to convert StandardizedPath to local path: {file_path}; diff \
                            will be read-only"
                        )
                    );
                    return;
                };
                file_model.update(ctx, |file_model, ctx| {
                    file_model.register_file_path(&local_path, false, ctx)
                })
            }
            DiffSessionType::Remote(host_id) => {
                let host_id = host_id.clone();
                let remote_path = file_path.clone();
                file_model.update(ctx, |file_model, _ctx| {
                    file_model.register_remote_file(host_id, remote_path)
                })
            }
        };

        self.session_type = Some(session_type.clone());
        self.finish_file_registration(file_id, ctx);
    }

    /// Common registration logic: subscribes to events and sets the
    /// backing file ID after a file has been registered with `FileModel`.
    fn finish_file_registration(&mut self, file_id: FileId, ctx: &mut ViewContext<Self>) {
        let file_model = FileModel::handle(ctx);

        let version = self.editor.as_ref(ctx).version(ctx);
        file_model.update(ctx, |file_model, _ctx| {
            file_model.set_version(file_id, version);
        });

        self.backing_file_id = Some(file_id);

        // Subscribe to FileModel events for this file.
        ctx.subscribe_to_model(&file_model, move |_me, _file_model, event, ctx| {
            if file_id == event.file_id() {
                match event {
                    FileModelEvent::FileSaved { .. } => {
                        ctx.emit(InlineDiffViewEvent::FileSaved);
                    }
                    FileModelEvent::FailedToSave { error, .. } => {
                        ctx.emit(InlineDiffViewEvent::FailedToSave {
                            error: error.clone(),
                        });
                    }
                    _ => {}
                }
            }
        });

        ctx.emit(InlineDiffViewEvent::FileLoaded);
    }

    fn apply_diffs_if_any(&self, ctx: &mut ViewContext<Self>) {
        let Some(diff) = self.diff_type.clone() else {
            return;
        };

        let deltas = match diff {
            DiffType::Create { delta } => vec![delta],
            DiffType::Update { mut deltas, .. } => {
                deltas.sort_by_key(|delta| delta.replacement_line_range.start);
                deltas
            }
            DiffType::Delete { delta } => vec![delta],
        };

        if deltas.is_empty() {
            return;
        }

        self.editor.update(ctx, |editor, ctx| {
            editor.apply_diffs(deltas, ctx);
            editor.toggle_diff_nav(None, ctx);
            editor.set_pending_scroll(ScrollTrigger::new(
                ScrollPosition::FocusedDiffHunk,
                editor.buffer_version(ctx),
            ));
        });
    }

    fn save_content(&self, ctx: &mut ViewContext<Self>) {
        let Some(file_id) = self.backing_file_id else {
            return;
        };
        let content = self.editor.as_ref(ctx).text(ctx).into_string();
        let version = self.editor.as_ref(ctx).version(ctx);

        if let Err(err) =
            FileModel::handle(ctx).update(ctx, |file_model, ctx| match &self.diff_type {
                Some(DiffType::Delete { .. }) => file_model.delete(file_id, version, ctx),
                Some(DiffType::Update {
                    rename: Some(to), ..
                }) if StandardizedPath::try_new(&to.to_string_lossy())
                    .ok()
                    .as_ref()
                    != self.file_path.as_ref() =>
                {
                    file_model.rename_and_save(file_id, to.clone(), content, version, ctx)
                }
                Some(DiffType::Update {
                    rename: Some(_), ..
                }) => file_model.save(file_id, content, version, ctx),
                Some(DiffType::Create { .. } | DiffType::Update { rename: None, .. }) | None => {
                    file_model.save(file_id, content, version, ctx)
                }
            })
        {
            ctx.emit(InlineDiffViewEvent::FailedToSave {
                error: Rc::new(err),
            });
        }
    }

    /// Restores an accepted operation. Deletion undo retains historical
    /// content-only restoration.
    pub fn restore_and_save_diff(&mut self, ctx: &mut ViewContext<Self>) -> Result<(), String> {
        let file_id = self
            .backing_file_id
            .ok_or_else(|| "Missing backing file".to_owned())?;
        let version = self.editor.as_ref(ctx).version(ctx);
        let file_model = FileModel::handle(ctx);
        match &self.diff_type {
            Some(DiffType::Create { .. }) => file_model
                .update(ctx, |model, ctx| model.delete(file_id, version, ctx))
                .map_err(|error| error.to_string()),
            diff => {
                let base_content = self
                    .base_content(ctx)
                    .ok_or_else(|| "Missing base content".to_owned())?;
                match diff {
                    Some(DiffType::Delete { .. })
                        if matches!(self.session_type, Some(DiffSessionType::Local)) =>
                    {
                        // Local IDs cache canonical paths, so reuse could write a deleted
                        // symlink's referent. Remote IDs already preserve the requested path.
                        let original_path =
                            self.file_path.as_ref().expect("registered file has a path");
                        file_model
                            .update(ctx, |model, ctx| {
                                let restored_id = model.register_file_path(
                                    &PathBuf::from(original_path.as_str()),
                                    false,
                                    ctx,
                                );
                                let result = model.save(restored_id, base_content, version, ctx);
                                model.unsubscribe(restored_id, ctx);
                                result
                            })
                            .map_err(|error| error.to_string())
                    }
                    Some(DiffType::Update {
                        rename: Some(to), ..
                    }) if StandardizedPath::try_new(&to.to_string_lossy())
                        .ok()
                        .as_ref()
                        != self.file_path.as_ref() =>
                    {
                        let target_id = file_model.update(ctx, |model, ctx| {
                            match self
                                .session_type
                                .as_ref()
                                .expect("registered file has a session")
                            {
                                DiffSessionType::Local => {
                                    Ok::<_, String>(model.register_file_path(to, false, ctx))
                                }
                                DiffSessionType::Remote(host_id) => {
                                    let path = StandardizedPath::try_new(&to.to_string_lossy())
                                        .map_err(|error| error.to_string())?;
                                    Ok(model.register_remote_file(host_id.clone(), path))
                                }
                            }
                        })?;
                        let original_path =
                            self.file_path.as_ref().expect("registered file has a path");
                        file_model
                            .update(ctx, |model, ctx| {
                                let result = model.rename_and_save(
                                    target_id,
                                    PathBuf::from(original_path.as_str()),
                                    base_content,
                                    version,
                                    ctx,
                                );
                                model.unsubscribe(target_id, ctx);
                                result
                            })
                            .map_err(|error| error.to_string())
                    }
                    Some(DiffType::Delete { .. } | DiffType::Update { .. }) | None => file_model
                        .update(ctx, |model, ctx| {
                            model.save(file_id, base_content, version, ctx)
                        })
                        .map_err(|error| error.to_string()),
                    Some(DiffType::Create { .. }) => unreachable!(),
                }
            }
        }
    }
}

impl InlineDiffView {
    pub fn file_path(&self) -> Option<&StandardizedPath> {
        self.file_path.as_ref()
    }

    pub fn base_content(&self, app: &AppContext) -> Option<String> {
        self.editor
            .as_ref(app)
            .model
            .as_ref(app)
            .diff()
            .as_ref(app)
            .base()
            .map(|base| base.to_string())
    }

    pub fn file_name(&self) -> Option<String> {
        self.file_path()
            .map(|p| p.file_name().unwrap_or_default().to_owned())
    }
}

impl DiffViewer for InlineDiffView {
    fn editor(&self) -> &ViewHandle<CodeEditorView> {
        &self.editor
    }

    fn diff(&self) -> Option<&DiffType> {
        self.diff_type.as_ref()
    }

    fn was_edited(&self) -> bool {
        self.was_edited
    }

    fn set_display_mode(&self, mode: DisplayMode, ctx: &mut ViewContext<Self>) {
        let is_delete = matches!(self.diff(), Some(DiffType::Delete { .. }));
        let interaction_state = if self.backing_file_id.is_some() {
            mode.interaction_state(is_delete)
        } else {
            InteractionState::Selectable
        };
        self.editor().update(ctx, |editor, ctx| {
            editor.set_scroll_wheel_behavior(mode.scroll_wheel_behavior());
            editor.set_vertical_expansion_behavior(mode.vertical_expansion_behavior(), ctx);
            editor.set_vertical_scrollbar_appearance(mode.scrollbar_appearance());
            editor.set_horizontal_scrollbar_appearance(mode.scrollbar_appearance());
            editor.set_interaction_state(interaction_state, ctx);
            editor.set_show_nav_bar(mode.show_nav_bar());
            editor.set_nav_bar_behavior(NavBarBehavior::NotClosable, ctx);
        });
    }
    fn accept_and_save_diff(&self, ctx: &mut ViewContext<Self>) {
        if self.backing_file_id.is_none() {
            return;
        }

        // Compute the unified diff (result arrives via CodeEditorEvent::UnifiedDiffComputed).
        if let Some(file_path) = &self.file_path {
            let file_name = file_path.to_string();
            self.editor.update(ctx, |editor, ctx| {
                editor.retrieve_unified_diff(file_name, ctx)
            });
        }
        // Persist the diff's file operation to disk.
        self.save_content(ctx);
    }
}

impl Entity for InlineDiffView {
    type Event = InlineDiffViewEvent;
}

impl View for InlineDiffView {
    fn ui_name() -> &'static str {
        "InlineDiffView"
    }

    fn render(&self, _app: &AppContext) -> Box<dyn Element> {
        ChildView::new(&self.editor).finish()
    }
}

impl TypedActionView for InlineDiffView {
    type Action = ();
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "inline_diff_tests.rs"]
mod tests;
