use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use dashmap::DashMap;
use futures::channel::oneshot;
use futures::io::{AsyncRead, AsyncWrite};
use warpui::r#async::{FutureExt as _, executor};

use crate::proto::{
    Abort, BufferEdit, ClientMessage, CloseBuffer, DeleteFile, DiffMode, DiffStateFileDelta,
    DiffStateMetadataUpdate, DiffStateSnapshot, DiscardFilesRequest, ErrorCode, FileStatusInfo,
    GetDiffState, GetDiffStateResponse, GitCommitChainMode, GitCommitChainRequest,
    GitCreatePrRequest, GitGenerateCommitMessageRequest, GitGetCommittedBranchFilesRequest,
    GitGetPrInfoRequest, GitPushRequest, Initialize, InitializeResponse,
    LoadRepoMetadataDirectoryResponse, NavigatedToDirectoryResponse, OpenBuffer,
    OpenBufferResponse, ReadFileContextRequest, ReadFileContextResponse, RunCommandRequest,
    RunCommandResponse, SaveBuffer, ServerMessage, SessionBootstrapped, TextEdit,
    UnsubscribeDiffState, WriteFile, client_message, server_message,
};

use crate::protocol::{self, ProtocolError, RequestId};

use warp_core::SessionId;
use warp_core::{safe_error, safe_warn};
use warp_util::standardized_path::StandardizedPath;
use warpui::r#async::TransportStream;

/// Default request timeout (2 minutes).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Errors from the `RemoteServerClient`.
#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Connection was dropped")]
    Disconnected,

    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Response channel closed before receiving a reply")]
    ResponseChannelClosed,

    #[error("Unexpected response from server")]
    UnexpectedResponse,

    #[error("Server error ({code:?}): {message}")]
    ServerError { code: ErrorCode, message: String },

    #[error("Request timed out after {0:?}")]
    Timeout(Duration),

    #[error("File operation failed: {0}")]
    FileOperationFailed(String),

    #[error("Discard files failed: {0}")]
    DiscardFailed(String),

    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),
}

/// Events received from the remote server, delivered through the event
/// channel returned by [`RemoteServerClient::new`].
///
/// The consumer (typically `RemoteServerManager`) drains this channel to
/// react to connection lifecycle changes and server-pushed data.
#[derive(Clone, Debug)]
pub enum ClientEvent {
    /// The reader task detected EOF or a fatal error. The connection is gone.
    /// This is always the last event sent on the channel.
    Disconnected,
    /// A full or lazy-loaded repo metadata snapshot was pushed by the server.
    RepoMetadataSnapshotReceived {
        update: repo_metadata::RepoMetadataUpdate,
    },
    /// An incremental repo metadata update was pushed by the server.
    RepoMetadataUpdated {
        update: repo_metadata::RepoMetadataUpdate,
    },
    DiffStateSnapshotReceived {
        repo_path: StandardizedPath,
        mode: DiffMode,
        snapshot: DiffStateSnapshot,
    },
    DiffStateMetadataUpdateReceived {
        repo_path: StandardizedPath,
        mode: DiffMode,
        update: DiffStateMetadataUpdate,
    },
    DiffStateFileDeltaReceived {
        repo_path: StandardizedPath,
        mode: DiffMode,
        delta: DiffStateFileDelta,
    },
    /// A server message could not be decoded and had no parseable request_id.
    MessageDecodingError,
    /// A buffer was updated on the server (file changed on disk).
    BufferUpdated {
        path: String,
        new_server_version: u64,
        expected_client_version: u64,
        edits: Vec<crate::proto::TextEdit>,
    },
    /// The file changed on disk while the client had unsaved edits.
    /// The server did NOT apply the change; the client should show a
    /// conflict resolution banner.
    BufferConflictDetected { path: String },
}
/// Client for communicating with a `remote_server` process over the remote server protocol.
///
/// Exposes async request/response APIs over generic I/O streams (child-process pipes,
/// SSH channels, or in-memory streams for testing).
///
/// Designed to be wrapped in `Arc` for sharing across threads. Construction
/// returns an event receiver that delivers push events and a final
/// `Disconnected` event when the connection drops.
///
/// This type does **not** own the child subprocess whose stdio backs it.
/// For transports that spawn a subprocess (e.g. SSH), the caller is
/// responsible for holding the `Child` for the lifetime of the session
/// so that `kill_on_drop` fires when teardown occurs. In Warp this is
/// the `RemoteServerManager`, which stores the child in
/// `RemoteSessionState` alongside the `Arc<RemoteServerClient>`. That
/// way the child's lifetime is gated by the manager's session map
/// rather than by `Arc` refcount -- cloning `Arc<RemoteServerClient>`
/// into other owners (e.g. the command executor) no longer keeps the
/// child alive.
pub struct RemoteServerClient {
    /// Channel for queuing ClientMessages to send to the remote server.
    outbound_tx: async_channel::Sender<ClientMessage>,

    /// Maps `request_id` → oneshot sender for the correlated response from the remote server.
    pending_requests: Arc<DashMap<RequestId, oneshot::Sender<Result<ServerMessage, ClientError>>>>,

    /// Set to `true` by the reader task when the connection is lost. Checked by
    /// `send_request` after inserting into `pending_requests` to avoid hanging
    /// on a dead connection.
    disconnected: Arc<AtomicBool>,
}

impl fmt::Debug for RemoteServerClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteServerClient").finish_non_exhaustive()
    }
}
impl RemoteServerClient {
    /// Creates a client from a child process's stdin, stdout, and stderr.
    ///
    /// The caller retains ownership of the `Child` itself. Typically the
    /// caller spawns the `Command` with `kill_on_drop(true)` and stashes
    /// the returned `Child` somewhere whose lifetime matches the
    /// session's (in Warp, on the `RemoteServerManager`'s
    /// `RemoteSessionState`). Dropping the `Child` there triggers
    /// SIGKILL on the subprocess, regardless of how many
    /// `Arc<RemoteServerClient>` clones are still alive.
    ///
    /// Internally forwards stderr lines to local logging via
    /// [`spawn_stderr_forwarder`], then delegates to [`Self::new`] for the
    /// protocol reader/writer setup.
    ///
    /// Returns the client and an event receiver that delivers push events
    /// and a final `Disconnected` event when the connection drops.
    pub fn from_child_streams(
        stdin: async_process::ChildStdin,
        stdout: async_process::ChildStdout,
        stderr: async_process::ChildStderr,
        executor: &executor::Background,
    ) -> (Self, async_channel::Receiver<ClientEvent>) {
        spawn_stderr_forwarder(stderr, executor);
        Self::new(stdout, stdin, executor)
    }
}

impl RemoteServerClient {
    /// Creates a new client, spawning background reader and writer tasks on the
    /// provided executor.
    ///
    /// Returns the client and an event receiver that delivers push events
    /// and a final `Disconnected` event when the connection drops.
    pub fn new(
        reader: impl AsyncRead + TransportStream,
        writer: impl AsyncWrite + TransportStream,
        executor: &executor::Background,
    ) -> (Self, async_channel::Receiver<ClientEvent>) {
        let pending_requests: Arc<
            DashMap<RequestId, oneshot::Sender<Result<ServerMessage, ClientError>>>,
        > = Arc::new(DashMap::new());
        let (outbound_tx, outbound_rx) = async_channel::unbounded::<ClientMessage>();
        let (event_tx, event_rx) = async_channel::unbounded::<ClientEvent>();
        let disconnected = Arc::new(AtomicBool::new(false));

        executor
            .spawn(Self::writer_task(
                writer,
                outbound_rx,
                Arc::clone(&pending_requests),
            ))
            .detach();
        executor
            .spawn(Self::reader_task(
                reader,
                Arc::clone(&pending_requests),
                event_tx,
                Arc::clone(&disconnected),
            ))
            .detach();

        (
            Self {
                outbound_tx,
                pending_requests,
                disconnected,
            },
            event_rx,
        )
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected.load(Ordering::Acquire)
    }

    /// Sends an `Initialize` request and awaits the `InitializeResponse`.
    pub async fn initialize(&self) -> Result<InitializeResponse, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::Initialize(Initialize {})),
        };

        let response = self.send_request(request_id, msg).await?;

        match response.message {
            Some(server_message::Message::InitializeResponse(resp)) => Ok(resp),
            other => {
                safe_error!(
                    safe: ("Remote server unexpected response for Initialize"),
                    full: ("Remote server unexpected response for Initialize: response={other:?}")
                );
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Sends a `SessionBootstrapped` notification (fire-and-forget) so the
    /// server can create a `LocalCommandExecutor` for the session.
    pub fn notify_session_bootstrapped(
        &self,
        session_id: SessionId,
        shell_type: &str,
        shell_path: Option<&str>,
    ) {
        let msg = ClientMessage {
            request_id: String::new(),
            message: Some(client_message::Message::SessionBootstrapped(
                SessionBootstrapped {
                    session_id: session_id.as_u64(),
                    shell_type: shell_type.to_owned(),
                    shell_path: shell_path.map(ToOwned::to_owned),
                },
            )),
        };
        self.send_notification(msg);
    }

    /// Sends a `NavigatedToDirectory` request and awaits the response.
    pub async fn navigate_to_directory(
        &self,
        path: String,
    ) -> Result<NavigatedToDirectoryResponse, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::NavigatedToDirectory(
                crate::proto::NavigatedToDirectory { path },
            )),
        };

        let response = self.send_request(request_id, msg).await?;

        match response.message {
            Some(server_message::Message::NavigatedToDirectoryResponse(resp)) => Ok(resp),
            other => {
                safe_error!(
                    safe: ("Remote server unexpected response for NavigatedToDirectory"),
                    full: ("Remote server unexpected response for NavigatedToDirectory: response={other:?}")
                );
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Sends a `LoadRepoMetadataDirectory` request and awaits the response.
    pub async fn load_repo_metadata_directory(
        &self,
        repo_path: String,
        dir_path: String,
    ) -> Result<LoadRepoMetadataDirectoryResponse, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::LoadRepoMetadataDirectory(
                crate::proto::LoadRepoMetadataDirectory {
                    repo_path,
                    dir_path,
                },
            )),
        };

        let response = self.send_request(request_id, msg).await?;

        match response.message {
            Some(server_message::Message::LoadRepoMetadataDirectoryResponse(resp)) => Ok(resp),
            other => {
                safe_error!(
                    safe: ("Remote server unexpected response for LoadRepoMetadataDirectory"),
                    full: ("Remote server unexpected response for LoadRepoMetadataDirectory: response={other:?}")
                );
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Writes content to a file on the remote host.
    /// Creates parent directories if they don't exist.
    pub async fn write_file(&self, path: String, content: String) -> Result<(), ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::WriteFile(WriteFile {
                path,
                content,
            })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::WriteFileResponse(resp)) => match resp.result {
                Some(crate::proto::write_file_response::Result::Success(_)) | None => Ok(()),
                Some(crate::proto::write_file_response::Result::Error(e)) => {
                    Err(ClientError::FileOperationFailed(e.message))
                }
            },
            other => {
                safe_error!(
                    safe: ("Remote server unexpected response for WriteFile"),
                    full: ("Remote server unexpected response for WriteFile: response={other:?}")
                );
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Batch-reads one or more files from the remote host with full context
    /// (line ranges, binary/image support, metadata, size limits).
    ///
    /// Per-file failures are reported in `ReadFileContextResponse::failed_files`
    /// rather than as a top-level error. The method only returns `Err` for
    /// transport-level failures (disconnect, timeout, etc.).
    pub async fn read_file_context(
        &self,
        request: ReadFileContextRequest,
    ) -> Result<ReadFileContextResponse, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::ReadFileContext(request)),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::ReadFileContextResponse(resp)) => Ok(resp),
            other => {
                safe_error!(
                    safe: ("Remote server unexpected response for ReadFileContext"),
                    full: ("Remote server unexpected response for ReadFileContext: response={other:?}")
                );
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Opens a buffer on the remote host for bidirectional syncing.
    ///
    /// When `force_reload` is true, the server discards any in-memory buffer
    /// state and re-reads the file from disk. Used to resolve conflicts.
    pub async fn open_buffer(
        &self,
        path: String,
        force_reload: bool,
    ) -> Result<OpenBufferResponse, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::OpenBuffer(OpenBuffer {
                path,
                force_reload,
            })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::OpenBufferResponse(resp)) => Ok(resp),
            other => {
                log::error!("Unexpected response variant for OpenBuffer: {other:?}");
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Sends a buffer edit notification (fire-and-forget) to the remote host.
    pub fn send_buffer_edit(
        &self,
        path: String,
        expected_server_version: u64,
        new_client_version: u64,
        edits: Vec<TextEdit>,
    ) {
        let msg = ClientMessage {
            request_id: String::new(), // notification — no response expected
            message: Some(client_message::Message::BufferEdit(BufferEdit {
                path,
                expected_server_version,
                new_client_version,
                edits,
            })),
        };
        self.send_notification(msg);
    }

    /// Saves a buffer on the remote host to disk.
    pub async fn save_buffer(&self, path: String) -> Result<(), ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::SaveBuffer(SaveBuffer { path })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::SaveBufferResponse(resp)) => match resp.result {
                Some(crate::proto::save_buffer_response::Result::Success(_)) | None => Ok(()),
                Some(crate::proto::save_buffer_response::Result::Error(e)) => {
                    Err(ClientError::FileOperationFailed(e.message))
                }
            },
            other => {
                log::error!("Unexpected response variant for SaveBuffer: {other:?}");
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Tells the remote host to close a buffer (stop watching).
    pub fn close_buffer(&self, path: String) {
        let msg = ClientMessage {
            request_id: String::new(),
            message: Some(client_message::Message::CloseBuffer(CloseBuffer { path })),
        };
        self.send_notification(msg);
    }

    /// Deletes a file on the remote host.
    pub async fn delete_file(&self, path: String) -> Result<(), ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::DeleteFile(DeleteFile { path })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::DeleteFileResponse(resp)) => match resp.result {
                Some(crate::proto::delete_file_response::Result::Success(_)) | None => Ok(()),
                Some(crate::proto::delete_file_response::Result::Error(e)) => {
                    Err(ClientError::FileOperationFailed(e.message))
                }
            },
            other => {
                safe_error!(
                    safe: ("Remote server unexpected response for DeleteFile"),
                    full: ("Remote server unexpected response for DeleteFile: response={other:?}")
                );
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Converts a server push message (empty request_id) into a domain event.
    fn push_message_to_event(msg: ServerMessage) -> Option<ClientEvent> {
        match msg.message? {
            server_message::Message::RepoMetadataSnapshot(snapshot) => {
                let update = crate::repo_metadata_proto::proto_snapshot_to_update(&snapshot)?;
                Some(ClientEvent::RepoMetadataSnapshotReceived { update })
            }
            server_message::Message::RepoMetadataUpdate(push) => {
                let update = crate::repo_metadata_proto::proto_to_repo_metadata_update(&push)?;
                Some(ClientEvent::RepoMetadataUpdated { update })
            }
            server_message::Message::DiffStateSnapshot(snapshot) => {
                let repo_path = StandardizedPath::try_new(&snapshot.repo_path).ok()?;
                let mode = snapshot.mode.clone()?;
                Some(ClientEvent::DiffStateSnapshotReceived {
                    repo_path,
                    mode,
                    snapshot,
                })
            }
            server_message::Message::DiffStateMetadataUpdate(update) => {
                let repo_path = StandardizedPath::try_new(&update.repo_path).ok()?;
                let mode = update.mode.clone()?;
                Some(ClientEvent::DiffStateMetadataUpdateReceived {
                    repo_path,
                    mode,
                    update,
                })
            }
            server_message::Message::DiffStateFileDelta(delta) => {
                let repo_path = StandardizedPath::try_new(&delta.repo_path).ok()?;
                let mode = delta.mode.clone()?;
                Some(ClientEvent::DiffStateFileDeltaReceived {
                    repo_path,
                    mode,
                    delta,
                })
            }
            server_message::Message::BufferUpdated(push) => Some(ClientEvent::BufferUpdated {
                path: push.path,
                new_server_version: push.new_server_version,
                expected_client_version: push.expected_client_version,
                edits: push.edits,
            }),
            server_message::Message::BufferConflictDetected(push) => {
                Some(ClientEvent::BufferConflictDetected { path: push.path })
            }
            other => {
                safe_warn!(
                    safe: ("Unhandled push message variant"),
                    full: ("Unhandled push message variant: {other:?}")
                );
                None
            }
        }
    }

    pub async fn get_diff_state(
        &self,
        repo_path: &StandardizedPath,
        mode: DiffMode,
    ) -> Result<GetDiffStateResponse, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::GetDiffState(GetDiffState {
                repo_path: repo_path.to_string(),
                mode: Some(mode),
            })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::GetDiffStateResponse(response)) => Ok(response),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    pub fn unsubscribe_diff_state(&self, repo_path: &StandardizedPath, mode: DiffMode) {
        self.send_notification(ClientMessage {
            request_id: String::new(),
            message: Some(client_message::Message::UnsubscribeDiffState(
                UnsubscribeDiffState {
                    repo_path: repo_path.to_string(),
                    mode: Some(mode),
                },
            )),
        });
    }

    pub async fn discard_files(
        &self,
        repo_path: &StandardizedPath,
        files: Vec<FileStatusInfo>,
        should_stash: bool,
        branch_name: Option<String>,
        mode: DiffMode,
    ) -> Result<(), ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::DiscardFiles(DiscardFilesRequest {
                repo_path: repo_path.to_string(),
                files,
                should_stash,
                branch_name,
                mode: Some(mode),
            })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::DiscardFilesResponse(response)) => {
                match response.result {
                    Some(crate::proto::discard_files_response::Result::Success(_)) => Ok(()),
                    Some(crate::proto::discard_files_response::Result::Error(error)) => {
                        Err(ClientError::DiscardFailed(error.message))
                    }
                    None => Err(ClientError::UnexpectedResponse),
                }
            }
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    pub async fn git_commit_chain(
        &self,
        repo_path: &StandardizedPath,
        mode: GitCommitChainMode,
        message: String,
        include_unstaged: bool,
        branch: String,
        autogenerate_pr_content: bool,
    ) -> Result<(crate::proto::GitOpDelta, Option<crate::proto::PrInfo>), ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::GitCommitChain(
                GitCommitChainRequest {
                    repo_path: repo_path.to_string(),
                    mode: mode as i32,
                    message,
                    include_unstaged,
                    branch,
                    autogenerate_pr_content,
                },
            )),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::GitCommitChainResponse(response)) => {
                match response.result {
                    Some(crate::proto::git_commit_chain_response::Result::Success(success)) => {
                        Ok((success.delta.unwrap_or_default(), success.pr_info))
                    }
                    Some(crate::proto::git_commit_chain_response::Result::Error(error)) => {
                        Err(ClientError::GitOperationFailed(error.message))
                    }
                    None => Err(ClientError::UnexpectedResponse),
                }
            }
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    pub async fn git_push(
        &self,
        repo_path: &StandardizedPath,
        branch: String,
    ) -> Result<crate::proto::GitOpDelta, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::GitPush(GitPushRequest {
                repo_path: repo_path.to_string(),
                branch,
            })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::GitPushResponse(response)) => match response.result {
                Some(crate::proto::git_push_response::Result::Success(delta)) => Ok(delta),
                Some(crate::proto::git_push_response::Result::Error(error)) => {
                    Err(ClientError::GitOperationFailed(error.message))
                }
                None => Err(ClientError::UnexpectedResponse),
            },
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    pub async fn git_create_pr(
        &self,
        repo_path: &StandardizedPath,
        branch: String,
        autogenerate_content: bool,
    ) -> Result<crate::proto::PrInfo, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::GitCreatePr(GitCreatePrRequest {
                repo_path: repo_path.to_string(),
                branch,
                autogenerate_content,
            })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::GitCreatePrResponse(response)) => match response.result {
                Some(crate::proto::git_create_pr_response::Result::Success(pr)) => Ok(pr),
                Some(crate::proto::git_create_pr_response::Result::Error(error)) => {
                    Err(ClientError::GitOperationFailed(error.message))
                }
                None => Err(ClientError::UnexpectedResponse),
            },
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    pub async fn git_get_pr_info(
        &self,
        repo_path: &StandardizedPath,
    ) -> Result<Option<crate::proto::PrInfo>, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::GitGetPrInfo(GitGetPrInfoRequest {
                repo_path: repo_path.to_string(),
            })),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::GitGetPrInfoResponse(response)) => {
                match response.result {
                    Some(crate::proto::git_get_pr_info_response::Result::Success(success)) => {
                        Ok(success.pr_info)
                    }
                    Some(crate::proto::git_get_pr_info_response::Result::Error(error)) => {
                        Err(ClientError::GitOperationFailed(error.message))
                    }
                    None => Err(ClientError::UnexpectedResponse),
                }
            }
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    pub async fn git_generate_commit_message(
        &self,
        repo_path: &StandardizedPath,
        include_unstaged: bool,
        branch_name: String,
    ) -> Result<String, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::GitGenerateCommitMessage(
                GitGenerateCommitMessageRequest {
                    repo_path: repo_path.to_string(),
                    include_unstaged,
                    branch_name,
                },
            )),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::GitGenerateCommitMessageResponse(response)) => {
                match response.result {
                    Some(crate::proto::git_generate_commit_message_response::Result::Message(
                        message,
                    )) => Ok(message),
                    Some(crate::proto::git_generate_commit_message_response::Result::Error(
                        error,
                    )) => Err(ClientError::GitOperationFailed(error.message)),
                    None => Err(ClientError::UnexpectedResponse),
                }
            }
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    pub async fn git_get_committed_branch_files(
        &self,
        repo_path: &StandardizedPath,
    ) -> Result<Vec<crate::proto::FileChangeEntry>, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::GitGetCommittedBranchFiles(
                GitGetCommittedBranchFilesRequest {
                    repo_path: repo_path.to_string(),
                },
            )),
        };
        let response = self.send_request(request_id, msg).await?;
        match response.message {
            Some(server_message::Message::GitGetCommittedBranchFilesResponse(response)) => {
                match response.result {
                    Some(
                        crate::proto::git_get_committed_branch_files_response::Result::Success(
                            success,
                        ),
                    ) => Ok(success.files),
                    Some(crate::proto::git_get_committed_branch_files_response::Result::Error(
                        error,
                    )) => Err(ClientError::GitOperationFailed(error.message)),
                    None => Err(ClientError::UnexpectedResponse),
                }
            }
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Sends a `RunCommand` request
    pub async fn run_command(
        &self,
        session_id: SessionId,
        command: String,
        working_directory: Option<String>,
        environment_variables: HashMap<String, String>,
    ) -> Result<RunCommandResponse, ClientError> {
        let request_id = RequestId::new();
        let msg = ClientMessage {
            request_id: request_id.to_string(),
            message: Some(client_message::Message::RunCommand(RunCommandRequest {
                command,
                working_directory,
                environment_variables,
                session_id: session_id.as_u64(),
            })),
        };

        let response = self.send_request(request_id, msg).await?;

        match response.message {
            Some(server_message::Message::RunCommandResponse(resp)) => Ok(resp),
            other => {
                safe_error!(
                    safe: ("Remote server unexpected response for RunCommand"),
                    full: ("Remote server unexpected response for RunCommand: response={other:?}")
                );
                Err(ClientError::UnexpectedResponse)
            }
        }
    }

    /// Generic request/response correlation.
    ///
    /// Registers a oneshot channel keyed by `request_id`, sends the message
    /// through the outbound channel, and awaits the correlated response.
    /// Times out after `REQUEST_TIMEOUT` and sends an `Abort` to the server.
    async fn send_request(
        &self,
        request_id: RequestId,
        msg: ClientMessage,
    ) -> Result<ServerMessage, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(request_id.clone(), tx);

        // Check if the reader task has already marked the connection as dead.
        // The DashMap lock from `insert` above synchronizes with the lock from
        // `clear` in `reader_task`, so if `clear` ran before our insert the
        // flag is guaranteed to be visible here.
        if self.disconnected.load(Ordering::Acquire) {
            self.pending_requests.clear();
            return Err(ClientError::Disconnected);
        }

        if self.outbound_tx.send(msg).await.is_err() {
            self.pending_requests.remove(&request_id);
            return Err(ClientError::Disconnected);
        }

        let result = match rx.with_timeout(REQUEST_TIMEOUT).await {
            Ok(Ok(inner)) => inner,
            Ok(Err(_)) => return Err(ClientError::ResponseChannelClosed),
            Err(_) => {
                // Timed out — clean up and send abort.
                self.pending_requests.remove(&request_id);
                self.send_abort(&request_id);
                return Err(ClientError::Timeout(REQUEST_TIMEOUT));
            }
        };

        // Unwrap the inner Result (reader task may send Err for decode failures).
        let response = result?;

        // Convert server-reported ErrorResponse into ClientError so callers
        // only need to match on success variants.
        if let Some(server_message::Message::Error(ref e)) = response.message {
            return Err(ClientError::ServerError {
                code: e.code(),
                message: e.message.clone(),
            });
        }

        Ok(response)
    }

    /// Sends an `Abort` notification for the given request ID.
    fn send_abort(&self, request_id_to_abort: &RequestId) {
        let msg = ClientMessage {
            request_id: RequestId::new().to_string(),
            message: Some(client_message::Message::Abort(Abort {
                request_id_to_abort: request_id_to_abort.to_string(),
            })),
        };
        self.send_notification(msg);
    }

    /// Sends a message without registering a pending request (fire-and-forget).
    fn send_notification(&self, msg: ClientMessage) {
        // Use try_send to avoid blocking; if the channel is full or closed,
        // the notification is best-effort.
        if let Err(e) = self.outbound_tx.try_send(msg) {
            log::debug!("Failed to send notification (best-effort): {e}");
        }
    }

    /// Background task that writes `ClientMessage`s to the underlying stream.
    async fn writer_task(
        writer: impl AsyncWrite + TransportStream,
        outbound_rx: async_channel::Receiver<ClientMessage>,
        pending_requests: Arc<
            DashMap<RequestId, oneshot::Sender<Result<ServerMessage, ClientError>>>,
        >,
    ) {
        let mut writer = futures::io::BufWriter::new(writer);
        while let Ok(msg) = outbound_rx.recv().await {
            if let Err(e) = protocol::write_client_message(&mut writer, &msg).await {
                let request_id = RequestId::from(msg.request_id);
                if !e.is_write_recoverable() {
                    log::error!("Writer task fatal error: request_id={request_id} error={e}");
                    pending_requests.clear();
                    break;
                }
                log::warn!("Remote server writer task error: request_id={request_id} error={e}");
                // Drop the sender so the caller receives ResponseChannelClosed.
                pending_requests.remove(&request_id);
            }
        }
    }

    /// Background task that reads `ServerMessage`s and resolves pending
    /// requests by `request_id`, or converts push messages to events.
    ///
    /// Sends `ClientEvent::Disconnected` as the final event when the
    /// connection is lost.
    async fn reader_task(
        reader: impl AsyncRead + TransportStream,
        pending_requests: Arc<
            DashMap<RequestId, oneshot::Sender<Result<ServerMessage, ClientError>>>,
        >,
        event_tx: async_channel::Sender<ClientEvent>,
        disconnected: Arc<AtomicBool>,
    ) {
        let mut reader = futures::io::BufReader::new(reader);
        loop {
            match protocol::read_server_message(&mut reader).await {
                Ok(msg) => {
                    let request_id = RequestId::from(msg.request_id.clone());
                    if request_id.is_empty() {
                        // Push message — convert to a domain event and forward.
                        if let Some(event) = Self::push_message_to_event(msg) {
                            if event_tx.send(event).await.is_err() {
                                log::warn!("Event channel closed, dropping push message");
                            }
                        }
                    } else if let Some((_, tx)) = pending_requests.remove(&request_id) {
                        // Ignore send failure — the caller may have dropped the receiver.
                        let _ = tx.send(Ok(msg));
                    } else {
                        log::warn!("Received unexpected response with request_id={request_id}");
                    }
                }
                Err(ProtocolError::Decode(ref err, Some(ref request_id))) => {
                    if let Some((_, tx)) = pending_requests.remove(request_id) {
                        log::warn!(
                            "Reader task: malformed response \
                             (request_id={request_id}): {err}"
                        );
                        let _ = tx.send(Err(ClientError::Protocol(ProtocolError::Decode(
                            err.clone(),
                            Some(request_id.clone()),
                        ))));
                    } else {
                        log::warn!(
                            "Reader task: malformed response for \
                             unknown request (request_id={request_id}): {err}"
                        );
                    }
                }
                Err(ProtocolError::Decode(ref err, None)) => {
                    log::warn!(
                        "Reader task: skipping malformed response \
                         (no parseable request_id): {err}"
                    );
                    let _ = event_tx.send(ClientEvent::MessageDecodingError).await;
                }
                Err(e) if e.is_read_recoverable() => {
                    log::warn!("Reader task: skipping message: {e}");
                }
                Err(e) => {
                    match e {
                        ProtocolError::UnexpectedEof => {
                            log::info!("Reader task: server disconnected (EOF)");
                        }
                        _ => log::error!("Reader task fatal error: {e}"),
                    }
                    break;
                }
            }
        }

        // Mark the connection as dead so that any new `send_request` calls
        // fail immediately rather than hanging forever. This prevents a race
        // where `pending_requests.clear()` runs before `send_request` has
        // inserted its oneshot entry.
        disconnected.store(true, Ordering::Release);

        // Notify all pending requests that the connection is gone.
        pending_requests.clear();

        // Signal disconnection as the final event.
        let _ = event_tx.send(ClientEvent::Disconnected).await;
    }
}

/// Spawns a background task that reads lines from the server's stderr and
/// forwards them to the client's logging.
pub fn spawn_stderr_forwarder(
    stderr: impl AsyncRead + TransportStream,
    executor: &executor::Background,
) {
    use futures::StreamExt;
    use futures::io::AsyncBufReadExt;

    executor
        .spawn(async move {
            let reader = futures::io::BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Some(Ok(line)) = lines.next().await {
                log::info!("[remote_server] {line}");
            }
        })
        .detach();
}

#[cfg(test)]
#[path = "../client_tests.rs"]
mod tests;
