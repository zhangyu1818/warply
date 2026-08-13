use warp_util::file::FileSaveError;

use std::collections::HashMap;
use std::path::PathBuf;

use futures::channel::oneshot;
use itertools::Itertools;
use warpui::{Entity, EntityId, ModelContext, SingletonEntity as _, ViewHandle};

use crate::ai::{
    agent::{
        AIAgentAction, AIAgentActionId, AIAgentActionResultType, AIAgentActionType,
        RequestFileEditsResult, UpdatedFileContext,
    },
    blocklist::{
        inline_action::code_diff_view::{CodeDiffView, CodeDiffViewEvent},
        BlocklistAIPermissions,
    },
};
use crate::BlocklistAIHistoryModel;

use super::{ActionExecution, AnyActionExecution, ExecuteActionInput};

pub struct RequestFileEditsExecutor {
    diff_views: HashMap<AIAgentActionId, ViewHandle<CodeDiffView>>,
    terminal_view_id: EntityId,
}

impl RequestFileEditsExecutor {
    pub fn new(terminal_view_id: EntityId) -> Self {
        Self {
            diff_views: HashMap::new(),
            terminal_view_id,
        }
    }

    pub(super) fn should_autoexecute(
        &self,
        input: ExecuteActionInput,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        let ExecuteActionInput {
            action:
                AIAgentAction {
                    action: AIAgentActionType::RequestFileEdits { file_edits, .. },
                    ..
                },
            conversation_id,
        } = input
        else {
            return false;
        };

        let paths: Vec<PathBuf> = file_edits
            .iter()
            .filter_map(|edit| edit.file().map(PathBuf::from))
            .collect();

        // Don't allow autoexecution if the diff was generated passively.
        let Some(latest_exchange) = BlocklistAIHistoryModel::as_ref(ctx)
            .conversation(&conversation_id)
            .and_then(|c| c.latest_exchange())
        else {
            return false;
        };
        if latest_exchange.has_passive_request() {
            return false;
        }

        BlocklistAIPermissions::as_ref(ctx)
            .can_write_files(&conversation_id, &paths, Some(self.terminal_view_id), ctx)
            .is_allowed()
    }

    /// Registers a diff view to handle a RequestFileEdits action.
    /// Note this MUST be called before `execute` or `preprocess_action` is invoked in
    /// order for the necessary state to be set to handle the action.
    pub fn register_requested_edits(
        &mut self,
        action_id: &AIAgentActionId,
        view: &ViewHandle<CodeDiffView>,
    ) {
        self.diff_views.insert(action_id.clone(), view.clone());
    }

    pub(super) fn execute(
        &mut self,
        input: ExecuteActionInput,
        ctx: &mut ModelContext<Self>,
    ) -> impl Into<AnyActionExecution> + use<> {
        let ExecuteActionInput {
            action:
                AIAgentAction {
                    id,
                    action: AIAgentActionType::RequestFileEdits { .. },
                    ..
                },
            ..
        } = input
        else {
            return ActionExecution::InvalidAction;
        };

        let Some(diff_view) = self.diff_views.get(id) else {
            log::warn!("Tried to execute a RequestFileEdits action without a diff view");
            return ActionExecution::NotReady;
        };

        let (result_tx, result_rx) = oneshot::channel();
        let mut result_tx = Some(result_tx);

        ctx.subscribe_to_view(diff_view, move |_me, event, _ctx| match event {
            CodeDiffViewEvent::Rejected => {
                let Some(result_tx) = result_tx.take() else {
                    return;
                };
                let _ = result_tx.send(RequestFileEditsResult::Cancelled);
            }
            CodeDiffViewEvent::SavedAcceptedDiffs {
                diff,
                updated_files,
                file_contents,
                deleted_files,
                save_errors,
            } => {
                let Some(result_tx) = result_tx.take() else {
                    return;
                };

                // If saving any file failed, report it as an error to the LLM. Other files may
                // have saved successfully, but we're ignoring this edge case for now.
                if !save_errors.is_empty() {
                    let error = save_errors
                        .iter()
                        .filter_map(|err| match err.as_ref() {
                            FileSaveError::IOError { error, path } => {
                                Some(format!("Failed to save file {path:?}: {error}"))
                            }
                            _ => None,
                        })
                        .join("\n");

                    let _ = result_tx.send(RequestFileEditsResult::DiffApplicationFailed { error });
                    return;
                }

                // Build a map of file path → content from the editor buffers.
                // This avoids re-reading files from disk or the remote server.
                let content_map: HashMap<String, String> = file_contents.iter().cloned().collect();

                let mut file_edited_map = HashMap::new();
                for (file_location, was_edited) in updated_files.iter() {
                    file_edited_map.insert(file_location.name.clone(), *was_edited);
                }

                let _ = result_tx.send(RequestFileEditsResult::Success {
                    diff: diff.unified_diff.clone(),
                    updated_files: updated_files
                        .iter()
                        .map(|(file_location, was_edited)| {
                            let content = content_map
                                .get(&file_location.name)
                                .cloned()
                                .unwrap_or_default();
                            let line_count = content.lines().count();
                            UpdatedFileContext {
                                was_edited_by_user: *was_edited,
                                file_context: crate::ai::agent::FileContext {
                                    file_name: file_location.name.clone(),
                                    content: crate::ai::agent::AnyFileContent::StringContent(
                                        content,
                                    ),
                                    line_range: None,
                                    last_modified: None,
                                    line_count,
                                },
                            }
                        })
                        .collect(),
                    deleted_files: deleted_files.clone(),
                    lines_added: diff.lines_added,
                    lines_removed: diff.lines_removed,
                });
            }
            _ => (),
        });
        diff_view.update(ctx, |diff_view, ctx| {
            diff_view.accept_and_save(ctx);
        });

        ActionExecution::new_async(result_rx, |result, _ctx| match result {
            Ok(result) => AIAgentActionResultType::RequestFileEdits(result),
            Err(oneshot::Canceled) => {
                AIAgentActionResultType::RequestFileEdits(RequestFileEditsResult::Cancelled)
            }
        })
    }
}

impl Entity for RequestFileEditsExecutor {
    type Event = ();
}
