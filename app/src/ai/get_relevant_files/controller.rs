use ai::index::locations::CodeContextLocation;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use warpui::{AppContext, Entity, ModelContext, SingletonEntity};

use crate::ai::{
    agent::AIAgentActionId,
    outline::{OutlineStatus, RepoOutlines},
};

#[derive(Debug)]
pub enum GetRelevantFilesControllerEvent {
    Success {
        action_id: AIAgentActionId,
        fragments: Arc<HashSet<CodeContextLocation>>,
    },
    Error {
        action_id: AIAgentActionId,
    },
}

impl GetRelevantFilesControllerEvent {
    pub fn action_id(&self) -> &AIAgentActionId {
        match self {
            GetRelevantFilesControllerEvent::Success { action_id, .. } => action_id,
            GetRelevantFilesControllerEvent::Error { action_id } => action_id,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetRelevantFilesError {
    #[error("Repo outline is still being computed.")]
    Pending,
    #[error("Failed to create outline.")]
    CreateFailed,
    #[error("Failed to create outline.")]
    Missing,
}

/// Controller for GetRelevantFiles action. This is scoped per terminal session.
#[derive(Default)]
pub struct GetRelevantFilesController;

impl GetRelevantFilesController {
    pub fn new(_ctx: &mut ModelContext<Self>) -> Self {
        Self::default()
    }

    /// Start a new search query based on the repo outline.
    pub fn send_request(
        &mut self,
        directory: &Path,
        _query: String,
        partial_path_segments: Option<&Vec<String>>,
        action_id: AIAgentActionId,
        ctx: &mut ModelContext<Self>,
    ) -> Result<(), GetRelevantFilesError> {
        self.cancel_request_for_action(&action_id, ctx);

        match RepoOutlines::as_ref(ctx).get_outline(directory) {
            Some((OutlineStatus::Complete(outline), base_path)) => {
                let file_outlines = outline.to_file_symbols(partial_path_segments);
                ctx.emit(GetRelevantFilesControllerEvent::Success {
                    action_id,
                    fragments: Arc::new(
                        file_outlines
                            .into_iter()
                            .filter_map(|file| {
                                let file_path = base_path.join(file.path);
                                file_path
                                    .exists()
                                    .then_some(CodeContextLocation::WholeFile(file_path))
                            })
                            .collect(),
                    ),
                });
                Ok(())
            }
            Some((OutlineStatus::Pending, _)) => Err(GetRelevantFilesError::Pending),
            Some((OutlineStatus::Failed, _)) => Err(GetRelevantFilesError::CreateFailed),
            None => Err(GetRelevantFilesError::Missing),
        }
    }

    /// Returns the path to the root directory for a codebase search where pwd is `directory`.
    pub fn root_directory_for_search(&self, directory: &Path, app: &AppContext) -> Option<PathBuf> {
        RepoOutlines::as_ref(app)
            .get_outline(directory)
            .map(|(_, root)| root)
    }

    pub fn cancel_request_for_action(
        &mut self,
        _action_id: &AIAgentActionId,
        _ctx: &mut ModelContext<Self>,
    ) {
    }
}

impl Entity for GetRelevantFilesController {
    type Event = GetRelevantFilesControllerEvent;
}
