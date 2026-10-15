use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use ai::project_context::model::ProjectContextModel;
use lsp::LspManagerModel;
use prost::Message;
use remote_server::proto::{
    ClientMessage, DeleteFile, RenameFile, ServerMessage, client_message, delete_file_response,
    rename_file_response, server_message,
};
use remote_server::protocol::RequestId;
use warp_files::FileModel;
use warpui::{App, ModelHandle};

use super::ServerModel;
use crate::code::global_buffer_model::GlobalBufferModel;
use crate::test_util::terminal::initialize_app_for_terminal_view;

fn add_server(
    app: &mut App,
) -> (
    ModelHandle<ServerModel>,
    async_channel::Receiver<ServerMessage>,
) {
    initialize_app_for_terminal_view(app);
    app.add_singleton_model(FileModel::new);
    app.add_singleton_model(|_| ProjectContextModel::default());
    app.add_singleton_model(|_| LspManagerModel::new());
    app.add_singleton_model(GlobalBufferModel::new);
    let server = app.add_model(ServerModel::new);
    let conn_id = uuid::Uuid::new_v4();
    let (tx, rx) = async_channel::unbounded();
    server.update(app, |server, ctx| {
        server.register_connection(conn_id, tx, ctx)
    });
    (server, rx)
}

#[test]
#[cfg(unix)]
fn remote_delete_removes_link_without_touching_referent() {
    App::test((), |mut app| async move {
        let (server, responses) = add_server(&mut app);
        let dir = tempfile::tempdir().unwrap();
        let referent = dir.path().join("referent.txt");
        let link = dir.path().join("link.txt");
        fs::write(&referent, "original\n").unwrap();
        std::os::unix::fs::symlink(&referent, &link).unwrap();
        dispatch(
            &mut app,
            &server,
            client_message::Message::DeleteFile(DeleteFile {
                path: link.to_str().unwrap().to_owned(),
            }),
        );
        let response = responses.recv().await.unwrap();
        delete_file_result(&response).unwrap();
        assert!(fs::symlink_metadata(&link).is_err());
        assert_eq!(fs::read_to_string(&referent).unwrap(), "original\n");
    });
}

#[test]
#[cfg(unix)]
fn remote_move_moves_link_entry_not_referent() {
    App::test((), |mut app| async move {
        let (server, responses) = add_server(&mut app);
        let dir = tempfile::tempdir().unwrap();
        let referent = dir.path().join("referent.txt");
        let link = dir.path().join("link.txt");
        let destination = dir.path().join("moved-link.txt");
        fs::write(&referent, "original\n").unwrap();
        std::os::unix::fs::symlink(&referent, &link).unwrap();

        move_file(&mut app, &server, &link, destination.clone());
        rename_file_result(&responses.recv().await.unwrap()).unwrap();

        assert!(fs::symlink_metadata(&link).is_err());
        assert_eq!(fs::read_link(&destination).unwrap(), referent);
        assert_eq!(fs::read_to_string(&referent).unwrap(), "updated\n");
    });
}

#[test]
fn remote_hard_link_collision_fails_before_writing() {
    App::test((), |mut app| async move {
        let (server, responses) = add_server(&mut app);
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let destination = dir.path().join("alias.txt");
        fs::write(&source, "original\n").unwrap();
        fs::hard_link(&source, &destination).unwrap();

        move_file(&mut app, &server, &source, destination.clone());
        assert!(rename_file_result(&responses.recv().await.unwrap()).is_err());
        assert_eq!(fs::read_to_string(&source).unwrap(), "original\n");
        assert_eq!(fs::read_to_string(&destination).unwrap(), "original\n");
    });
}

fn dispatch(app: &mut App, server: &ModelHandle<ServerModel>, request: client_message::Message) {
    let message = ClientMessage {
        request_id: RequestId::new().to_string(),
        message: Some(request),
    };
    let message = ClientMessage::decode(message.encode_to_vec().as_slice()).unwrap();
    server.update(app, |server, ctx| {
        let conn_id = *server.connection_senders.keys().next().unwrap();
        server.handle_message(conn_id, message, ctx);
    });
}

fn move_file(app: &mut App, server: &ModelHandle<ServerModel>, from: &Path, to: PathBuf) {
    dispatch(
        app,
        server,
        client_message::Message::RenameFile(RenameFile {
            path: from.to_str().unwrap().to_owned(),
            new_path: to.to_str().unwrap().to_owned(),
            content: "updated\n".to_owned(),
        }),
    );
}

fn rename_file_result(msg: &ServerMessage) -> Result<(), String> {
    match &msg.message {
        Some(server_message::Message::RenameFileResponse(resp)) => match &resp.result {
            Some(rename_file_response::Result::Success(_)) | None => Ok(()),
            Some(rename_file_response::Result::Error(e)) => Err(e.message.clone()),
        },
        other => panic!("unexpected response for RenameFile: {other:?}"),
    }
}

fn delete_file_result(msg: &ServerMessage) -> Result<(), String> {
    match &msg.message {
        Some(server_message::Message::DeleteFileResponse(resp)) => match &resp.result {
            Some(delete_file_response::Result::Success(_)) | None => Ok(()),
            Some(delete_file_response::Result::Error(e)) => Err(e.message.clone()),
        },
        other => panic!("unexpected response for DeleteFile: {other:?}"),
    }
}

#[test]
#[cfg(unix)]
fn remote_move_replaces_destination_and_preserves_source_permissions() {
    App::test((), |mut app| async move {
        let (server, responses) = add_server(&mut app);
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.sh");
        let target = dir.path().join("target.sh");
        fs::write(&source, "original\n").unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(&target, "replace me\n").unwrap();
        move_file(&mut app, &server, &source, target.clone());
        rename_file_result(&responses.recv().await.unwrap()).unwrap();

        assert!(!source.exists());
        assert_eq!(fs::read_to_string(&target).unwrap(), "updated\n");
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o755
        );
    });
}

#[test]
fn remote_move_destination_setup_failure_leaves_source_unchanged() {
    App::test((), |mut app| async move {
        let (server, responses) = add_server(&mut app);
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let blocker = dir.path().join("not-a-directory");
        fs::write(&source, "original\n").unwrap();
        fs::write(&blocker, "blocker").unwrap();
        move_file(&mut app, &server, &source, blocker.join("target.txt"));
        let result = rename_file_result(&responses.recv().await.unwrap());

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&source).unwrap(), "original\n");
        assert_eq!(fs::read_to_string(&blocker).unwrap(), "blocker");
    });
}

#[test]
fn remote_move_native_rename_failure_reports_saved_source() {
    App::test((), |mut app| async move {
        let (server, responses) = add_server(&mut app);
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let target = dir.path().join("directory");
        fs::write(&source, "original\n").unwrap();
        fs::create_dir(&target).unwrap();
        move_file(&mut app, &server, &source, target.clone());
        let error = rename_file_result(&responses.recv().await.unwrap()).unwrap_err();

        assert!(error.to_string().contains("Saved content at"));
        assert_eq!(fs::read_to_string(&source).unwrap(), "updated\n");
        assert!(target.is_dir());
    });
}

#[test]
fn remote_same_path_move_writes_without_deleting_source() {
    App::test((), |mut app| async move {
        let (server, responses) = add_server(&mut app);
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.txt");
        move_file(&mut app, &server, &source, source.clone());
        rename_file_result(&responses.recv().await.unwrap()).unwrap();

        assert_eq!(fs::read_to_string(&source).unwrap(), "updated\n");
    });
}

#[test]
fn remote_delete_removes_initially_empty_file() {
    App::test((), |mut app| async move {
        let (server, responses) = add_server(&mut app);
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("empty.txt");
        fs::write(&source, "").unwrap();
        dispatch(
            &mut app,
            &server,
            client_message::Message::DeleteFile(DeleteFile {
                path: source.to_str().unwrap().to_owned(),
            }),
        );
        delete_file_result(&responses.recv().await.unwrap()).unwrap();

        assert!(!source.exists());
    });
}
