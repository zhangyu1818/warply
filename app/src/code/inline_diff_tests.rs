use std::fs;
use std::path::Path;

use ai::diff_validation::DiffDelta;
use line_ending::LineEnding;
use warp_core::ui::appearance::Appearance;
use warp_editor::content::buffer::InitialBufferState;
use warp_editor::render::element::VerticalExpansionBehavior;
use warpui::App;
use warpui::platform::WindowStyle;

use super::*;
use crate::LocalIdentityProvider;
use crate::cloud_object::model::persistence::CloudModel;
use crate::code::editor::view::CodeEditorRenderOptions;
use crate::notebooks::editor::keys::NotebookKeybindings;
use crate::settings_view::keybindings::KeybindingChangedNotifier;
use crate::test_util::settings::initialize_settings_for_tests;
use crate::vim_registers::VimRegisters;
use crate::workspace::ActiveSession;
use crate::workspace::sync_inputs::SyncedInputState;

fn add_diff(app: &mut App, path: &Path, base: &str, diff: DiffType) -> ViewHandle<InlineDiffView> {
    initialize_settings_for_tests(app);
    app.add_singleton_model(|_| Appearance::mock());
    app.add_singleton_model(|_| SyncedInputState::mock());
    app.add_singleton_model(|_| VimRegisters::new());
    app.add_singleton_model(|_| KeybindingChangedNotifier::mock());
    app.add_singleton_model(|_| LocalIdentityProvider::new_for_test());
    app.add_singleton_model(CloudModel::mock);
    app.add_singleton_model(|_| ActiveSession::default());
    app.add_singleton_model(NotebookKeybindings::new);
    app.add_singleton_model(FileModel::new);
    let (_, view) = app.add_window(WindowStyle::NotStealFocus, |ctx| {
        let editor = ctx.add_typed_action_view(|ctx| {
            let editor = CodeEditorView::new(
                None,
                None,
                CodeEditorRenderOptions::new(VerticalExpansionBehavior::GrowToMaxHeight)
                    .lazy_layout(),
                ctx,
            );
            editor.reset(InitialBufferState::plain_text(base), ctx);
            // Keep the fixture's LF content independent of the platform default for empty buffers.
            let buffer = editor.model.as_ref(ctx).buffer().clone();
            buffer.update(ctx, |buffer, _| {
                buffer.set_line_ending_mode(LineEnding::LF);
            });
            editor
        });
        let mut view = InlineDiffView::new(
            editor,
            Some(diff),
            None,
            Some(StandardizedPath::try_from_local(path).unwrap()),
            ctx,
        );
        view.register_file(&DiffSessionType::Local, ctx);
        view
    });
    view
}

/// Subscribes to the diff view's file-operation completion events and returns
/// a receiver that yields one item per `FileSaved` / `FailedToSave` event.
fn completion_receiver(
    app: &mut App,
    view: &ViewHandle<InlineDiffView>,
) -> async_channel::Receiver<()> {
    let (tx, rx) = async_channel::unbounded();
    app.update(|ctx| {
        ctx.subscribe_to_view(view, move |_, event, _| {
            if matches!(
                event,
                InlineDiffViewEvent::FileSaved | InlineDiffViewEvent::FailedToSave { .. }
            ) {
                let _ = tx.try_send(());
            }
        });
    });
    rx
}

async fn accept_and_wait(app: &mut App, view: &ViewHandle<InlineDiffView>) {
    let rx = completion_receiver(app, view);
    view.update(app, |view, ctx| view.accept_and_save_diff(ctx));
    rx.recv().await.unwrap();
}

/// Waits for the next `FileModel` operation to complete. Restore operations
/// dispatch on freshly registered file IDs (deleted and moved files), so the
/// view-level subscription does not observe them.
async fn restore(app: &mut App, view: &ViewHandle<InlineDiffView>) {
    let files = FileModel::handle(&*app);
    let (tx, rx) = async_channel::unbounded();
    app.update(|ctx| {
        ctx.subscribe_to_model(&files, move |_, event, _| {
            if matches!(
                event,
                FileModelEvent::FileSaved { .. } | FileModelEvent::FailedToSave { .. }
            ) {
                let _ = tx.try_send(());
            }
        });
    });
    view.update(app, |view, ctx| {
        view.restore_and_save_diff(ctx).expect("restore dispatches")
    });
    rx.recv().await.unwrap();
}

#[test]
fn accept_deletes_file() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("delete.txt");
        fs::write(&path, "original\n").unwrap();
        let view = add_diff(&mut app, &path, "original\n", DiffType::deletion(1));

        accept_and_wait(&mut app, &view).await;

        assert!(!path.exists());
    });
}

#[test]
fn accept_move_to_same_normalized_path_writes_file() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("source.txt");
        let target = dir.path().join("./source.txt");
        let view = add_diff(
            &mut app,
            &path,
            "",
            DiffType::update(
                vec![DiffDelta {
                    replacement_line_range: 1..1,
                    insertion: "updated\n".to_owned(),
                }],
                Some(target.to_str().unwrap().to_owned()),
            ),
        );

        accept_and_wait(&mut app, &view).await;

        assert_eq!(fs::read_to_string(&path).unwrap(), "updated\n");
    });
}

#[test]
fn accept_creates_file() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new.txt");
        let view = add_diff(
            &mut app,
            &path,
            "",
            DiffType::creation("created\n".to_owned()),
        );

        accept_and_wait(&mut app, &view).await;

        assert_eq!(fs::read_to_string(&path).unwrap(), "created\n");
    });
}

#[test]
fn restore_deleted_file_recreates_original_content() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deleted.txt");
        fs::write(&path, "original\n").unwrap();
        let view = add_diff(&mut app, &path, "original\n", DiffType::deletion(1));
        accept_and_wait(&mut app, &view).await;

        restore(&mut app, &view).await;

        assert_eq!(fs::read_to_string(&path).unwrap(), "original\n");
    });
}

#[test]
fn restore_moved_file_restores_original_path_and_content() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("old.txt");
        let destination = dir.path().join("new.txt");
        fs::write(&path, "original\n").unwrap();
        let view = add_diff(
            &mut app,
            &path,
            "original\n",
            DiffType::update(
                vec![DiffDelta {
                    replacement_line_range: 1..2,
                    insertion: "updated\n".to_owned(),
                }],
                Some(destination.to_str().unwrap().to_owned()),
            ),
        );
        accept_and_wait(&mut app, &view).await;

        restore(&mut app, &view).await;

        assert_eq!(fs::read_to_string(&path).unwrap(), "original\n");
        assert!(!destination.exists());
    });
}

#[test]
fn accept_deletes_initially_empty_file() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.txt");
        fs::write(&path, "").unwrap();
        let view = add_diff(&mut app, &path, "", DiffType::deletion(0));

        accept_and_wait(&mut app, &view).await;

        assert!(!path.exists());
    });
}

#[test]
fn accept_update_to_empty_keeps_file() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.txt");
        fs::write(&path, "original\n").unwrap();
        let view = add_diff(
            &mut app,
            &path,
            "original\n",
            DiffType::update(
                vec![DiffDelta {
                    replacement_line_range: 1..2,
                    insertion: String::new(),
                }],
                None,
            ),
        );

        accept_and_wait(&mut app, &view).await;

        assert_eq!(fs::read_to_string(&path).unwrap(), "");
    });
}

#[test]
fn accept_moves_file_with_updated_content() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("old.txt");
        let destination = dir.path().join("nested/new.txt");
        fs::write(&path, "original\n").unwrap();
        let view = add_diff(
            &mut app,
            &path,
            "original\n",
            DiffType::update(
                vec![DiffDelta {
                    replacement_line_range: 1..2,
                    insertion: "updated\n".to_owned(),
                }],
                Some(destination.to_str().unwrap().to_owned()),
            ),
        );

        accept_and_wait(&mut app, &view).await;

        assert!(!path.exists());
        assert_eq!(fs::read_to_string(&destination).unwrap(), "updated\n");
    });
}

#[test]
fn accept_delete_failure_reports_failed_save_event() {
    App::test((), |mut app| async move {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.txt");
        let view = add_diff(&mut app, &path, "", DiffType::deletion(0));

        accept_and_wait(&mut app, &view).await;

        assert!(!path.exists());
    });
}
