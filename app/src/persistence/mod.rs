#![cfg_attr(not(feature = "local_fs"), allow(dead_code))]

cfg_if::cfg_if! {
    if #[cfg(feature = "local_fs")] {
        pub mod agent;
        mod block_list;
        mod sqlite;
        pub mod commands;
    }
}

pub use persistence::model;
#[cfg_attr(not(feature = "local_fs"), expect(unused_imports))]
pub use persistence::schema;

#[cfg(feature = "integration_tests")]
pub mod testing;

use instant::Instant;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::ai::persisted_workspace::EnablementState;
use ai::project_context::model::ProjectRulePath;
use chrono::{DateTime, Local};
use lsp::supported_servers::LSPServerType;
use uuid::Uuid;
use warp_core::command::ExitCode;
use warp_multi_agent_api as api;
use warpui::{AppContext, Entity, SingletonEntity};

use crate::ai::blocklist::PersistedAIInput;
use crate::ai::mcp::TemplatableMCPServerInstallation;
use crate::app_state::AppState;
use crate::cloud_object::model::actions::ObjectAction;
use crate::cloud_object::model::generic_string_model::CloudStringObject;

use crate::cloud_object::{
    CloudObject, CloudObjectMetadata, ObjectIdType, RevisionAndLastEditor, ServerTimestamp,
};
use crate::drive::folders::CloudFolder;
use crate::object_ids::SyncId;
use crate::suggestions::ignored_suggestions_model::SuggestionType;
use crate::terminal::history::PersistedCommand;
use crate::terminal::model::block::{SerializedAgentViewVisibility, SerializedBlock};
use crate::terminal::model::session::SessionId;
use crate::workflows::CloudWorkflow;

use ai::workspace::WorkspaceMetadata as CodeWorkspaceMetadata;

use self::model::{AgentConversation, AgentConversationData, Project};

#[cfg(any(feature = "local_fs", feature = "integration_tests"))]
pub use sqlite::database_file_path;
#[cfg(any(feature = "local_fs", feature = "integration_tests"))]
pub use sqlite::establish_ro_connection;

/// Initializes the persistence "subsystem".
///
/// Returns the previously-persisted data, if any, and handles for
/// writing updated data to persist, if the persistence subsystem is
/// available.
#[cfg_attr(not(feature = "local_fs"), allow(unused_variables))]
pub fn initialize(ctx: &mut AppContext) -> (Option<PersistedData>, Option<WriterHandles>) {
    cfg_if::cfg_if! {
        if #[cfg(feature = "local_fs")] {
            sqlite::initialize(ctx)
        } else {
            (None, None)
        }
    }
}

/// Holds interfaces to the writer thread.
pub struct WriterHandles {
    pub handle: JoinHandle<()>,
    pub sender: SyncSender<ModelEvent>,
}

/// Model for interacting with the writer thread.
pub struct PersistenceWriter {
    thread_handle: Option<JoinHandle<()>>,
    model_event_sender: Option<SyncSender<ModelEvent>>,
}

impl PersistenceWriter {
    pub fn new(handle: Option<WriterHandles>) -> Self {
        let (thread_handle, model_event_sender) = match handle {
            Some(handle) => (Some(handle.handle), Some(handle.sender)),
            None => (None, None),
        };
        Self {
            thread_handle,
            model_event_sender,
        }
    }

    /// Sending half for sending model updates to the persistence writer thread.
    pub fn sender(&self) -> Option<SyncSender<ModelEvent>> {
        self.model_event_sender.clone()
    }

    /// Synchronously terminate the SQLite writer thread.
    pub fn terminate(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let start = Instant::now();
            let Some(sender) = self.sender() else {
                log::error!("Model event sender should exist if thread handle is set");
                return;
            };
            if let Err(err) = sender.send(ModelEvent::Terminate) {
                log::error!("Could not terminate SQLite writer thread: {err}");
            }
            if handle.join().is_err() {
                log::error!("SQLite writer thread panicked");
            }
            log::info!("Shut down SQLite writer in {:?}", start.elapsed());
        }
    }
}

impl Drop for PersistenceWriter {
    fn drop(&mut self) {
        self.terminate();
    }
}

impl Entity for PersistenceWriter {
    type Event = ();
}

impl SingletonEntity for PersistenceWriter {}

pub struct PersistedData {
    /// Session restoration data
    pub app_state: AppState,

    /// Shareable objects.
    pub cloud_objects: Vec<Box<dyn CloudObject>>,
    pub command_history: Vec<PersistedCommand>,
    pub object_actions: Vec<ObjectAction>,
    pub ai_queries: Vec<PersistedAIInput>,
    pub code_workspaces: Vec<CodeWorkspaceMetadata>,
    pub workspace_language_servers: HashMap<PathBuf, HashMap<LSPServerType, EnablementState>>,
    pub multi_agent_conversations: Vec<AgentConversation>,
    pub projects: Vec<Project>,
    pub project_rules: Vec<ProjectRulePath>,
    pub ignored_suggestions: Vec<(String, SuggestionType)>,
    pub mcp_server_installations: HashMap<Uuid, TemplatableMCPServerInstallation>,
    pub mcp_servers_to_restore: Vec<Uuid>,
}

#[derive(Clone, Debug)]
pub struct BlockCompleted {
    pub pane_id: Vec<u8>,
    /// Indicates if the block was created locally (e.g. not in a remote session)
    pub is_local: bool,
    pub block: Arc<SerializedBlock>,
}

#[derive(Debug)]
pub struct StartedCommandMetadata {
    pub command: String,
    pub start_ts: Option<DateTime<Local>>,
    pub pwd: Option<String>,
    pub shell: Option<String>,
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub session_id: Option<SessionId>,
    pub git_branch: Option<String>,
    pub cloud_workflow_id: Option<SyncId>,
    pub workflow_command: Option<String>,
    pub is_agent_executed: bool,
}

#[derive(Debug)]
pub struct FinishedCommandMetadata {
    pub exit_code: ExitCode,
    pub start_ts: DateTime<Local>,
    pub completed_ts: DateTime<Local>,
    pub session_id: SessionId,
}

#[derive(Debug)]
pub enum ModelEvent {
    SaveBlock(BlockCompleted),
    DeleteBlocks(Vec<u8>),
    Snapshot(AppState),
    UpsertWorkflows(Vec<CloudWorkflow>),
    UpsertFolders(Vec<CloudFolder>),
    MarkObjectAsSynced {
        hashed_sqlite_id: String,
        revision_and_editor: RevisionAndLastEditor,
        metadata_ts: Option<ServerTimestamp>,
    },
    IncrementRetryCount(String),
    UpsertGenericStringObject {
        object: Box<dyn CloudStringObject>,
    },
    UpsertGenericStringObjects(Vec<Box<dyn CloudStringObject>>),
    UpsertWorkflow {
        workflow: CloudWorkflow,
    },
    UpsertFolder {
        folder: CloudFolder,
    },
    DeleteObjects {
        ids: Vec<(SyncId, ObjectIdType)>,
    },
    UpdateObjectMetadata {
        id: String,
        metadata: CloudObjectMetadata,
    },
    InsertCommand {
        metadata: StartedCommandMetadata,
    },
    UpdateFinishedCommand {
        metadata: FinishedCommandMetadata,
    },
    InsertObjectAction {
        object_action: ObjectAction,
    },
    SyncObjectActions {
        actions_to_sync: Vec<ObjectAction>,
    },
    /// Close the SQLite writer thread when the app is about to quit.
    Terminate,
    UpsertAIQuery {
        query: Arc<PersistedAIInput>,
    },
    /// Delete the AI query and related data for a given conversation.
    DeleteAIConversation {
        conversation_id: String,
    },
    UpdateMultiAgentConversation {
        conversation_id: String,
        updated_tasks: Vec<api::Task>,
        conversation_data: AgentConversationData,
    },
    DeleteMultiAgentConversations {
        conversation_ids: Vec<String>,
    },

    UpsertCodeWorkspaceMetadata {
        index_metadata: Box<CodeWorkspaceMetadata>,
    },
    DeleteCodeWorkspaceMetadata {
        repo_path: PathBuf,
    },
    UpsertProject {
        project: Project,
    },
    DeleteProject {
        path: String,
    },
    UpsertMCPServerEnvironmentVariables {
        mcp_server_uuid: Vec<u8>,
        environment_variables: String,
    },
    UpsertProjectRules {
        project_rule_paths: Vec<ProjectRulePath>,
    },
    DeleteProjectRules {
        path: Vec<PathBuf>,
    },
    AddIgnoredSuggestion {
        suggestion: String,
        suggestion_type: SuggestionType,
    },
    RemoveIgnoredSuggestion {
        suggestion: String,
        suggestion_type: SuggestionType,
    },
    UpsertMCPServerInstallation {
        mcp_server_installation: TemplatableMCPServerInstallation,
    },
    DeleteMCPServerInstallations {
        installation_uuids: Vec<Uuid>,
    },
    DeleteMCPServerInstallationsByTemplateUuid {
        template_uuid: Uuid,
    },
    UpdateMCPInstallationRunning {
        installation_uuid: Uuid,
        running: bool,
    },
    UpsertWorkspaceLanguageServer {
        workspace_path: PathBuf,
        lsp_type: LSPServerType,
        enabled: EnablementState,
    },
    UpdateBlockAgentViewVisibility {
        block_id: String,
        agent_view_visibility: SerializedAgentViewVisibility,
    },
    SaveAIDocumentContent {
        document_id: String,
        content: String,
        version: i32,
        title: String,
    },
}
