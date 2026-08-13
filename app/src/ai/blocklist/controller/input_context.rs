use std::{collections::HashMap, sync::Arc};

use chrono::Local;
use lazy_static::lazy_static;
use regex::Regex;
use warpui::{AppContext, SingletonEntity};

use crate::{
    ai::{
        agent::{
            AIAgentAttachment, AIAgentContext, DocumentContentAttachmentSource,
            conversation::AIConversationId,
        },
        block_context::BlockContext,
        blocklist::BlocklistAIContextModel,
        document::ai_document_model::{AIDocumentId, AIDocumentModel},
    },
    terminal::{
        TerminalView,
        model::{block::BlockId, session::active_session::ActiveSession},
    },
};

lazy_static! {
    // Regex to match <block:[block_id]> patterns
    pub static ref BLOCK_CONTEXT_ATTACHMENT_REGEX: Regex = Regex::new(r"<block:([^>]+)>")
        .expect("Block context attachment regex should be parsed");
    // Regex to match <change:filename:line_start-line_end> patterns
    pub static ref DIFF_HUNK_ATTACHMENT_REGEX: Regex = Regex::new(r"<change:([^>]+)>")
        .expect("Diff hunk attachment regex should be parsed");
    pub static ref PLAN_CONTEXT_ATTACHMENT_REGEX: Regex = Regex::new(r"<plan:([^>]+)>")
        .expect("Plan context attachment regex should be parsed");
}

// Returns the context to be attached to the AIAgentInput sent in a request.
// If `is_user_query` is true, includes selected blocks, text, and images from the context model.
// Always includes base context like current time and execution environment.
pub(super) fn input_context_for_request(
    is_user_query: bool,
    context_model: &BlocklistAIContextModel,
    active_session: &ActiveSession,
    _conversation_id: Option<AIConversationId>,
    additional_context: Vec<AIAgentContext>,
    app: &AppContext,
) -> Arc<[AIAgentContext]> {
    let mut context = context_model.pending_context(app, is_user_query);

    context.push(AIAgentContext::CurrentTime {
        current_time: Local::now(),
    });

    if let Some(env) = active_session.ai_execution_environment(app) {
        context.push(AIAgentContext::ExecutionEnvironment(env));
    }

    context.extend(additional_context);

    context.into()
}

/// Parses context reference strings like <block:123> from the user query and returns
/// a map of reference strings to AIAgentAttachment objects.
///
/// This searches across ALL TerminalModels, not just the active session, to find
/// the requested blocks.
pub(super) fn parse_context_attachments(
    query: &str,
    context_model: &BlocklistAIContextModel,
    ctx: &AppContext,
) -> HashMap<String, AIAgentAttachment> {
    let mut referenced_attachments = HashMap::new();

    // Parse block attachments
    for capture in BLOCK_CONTEXT_ATTACHMENT_REGEX.captures_iter(query) {
        if let (Some(full_match), Some(block_id_match)) = (capture.get(0), capture.get(1)) {
            let reference_string = full_match.as_str().to_string();
            let block_id_str = block_id_match.as_str();

            let block_id = BlockId::from(block_id_str.to_string());

            // Search across ALL TerminalModels to find the block
            if let Some(attachment) = find_block_attachment_in_all_terminals(&block_id, ctx) {
                referenced_attachments.insert(reference_string, attachment);
            }
        }
    }

    // Parse diff hunk attachments
    for capture in DIFF_HUNK_ATTACHMENT_REGEX.captures_iter(query) {
        if let (Some(full_match), Some(diff_hunk_match)) = (capture.get(0), capture.get(1)) {
            let reference_string = full_match.as_str().to_string();
            let diff_hunk_key = diff_hunk_match.as_str();

            // Check if we have a stored diff hunk attachment for this key
            if let Some(attachment) = context_model.get_diff_hunk_attachment(diff_hunk_key) {
                referenced_attachments.insert(reference_string, attachment.clone());
            }
        }
    }

    for capture in PLAN_CONTEXT_ATTACHMENT_REGEX.captures_iter(query) {
        if let (Some(full_match), Some(document_id_match)) = (capture.get(0), capture.get(1)) {
            let Ok(document_id) = AIDocumentId::try_from(document_id_match.as_str()) else {
                continue;
            };
            if let Some(content) =
                AIDocumentModel::as_ref(ctx).get_document_content(&document_id, ctx)
            {
                let document_id_str = document_id.to_string();
                referenced_attachments.insert(
                    full_match.as_str().to_string(),
                    AIAgentAttachment::DocumentContent {
                        document_id: document_id_str,
                        content,
                        source: DocumentContentAttachmentSource::UserAttached,
                        line_range: None,
                    },
                );
            }
        }
    }

    // Add pending file attachments as FilePathReference.
    // Duplicate basenames get a (1), (2), ... suffix to avoid collisions,
    // matching the pattern in build_file_attachment_map.
    for file in context_model.pending_files().iter() {
        let attachment = AIAgentAttachment::FilePathReference {
            file_id: uuid::Uuid::new_v4().to_string(),
            file_name: file.file_name.clone(),
            file_path: file.file_path.to_string_lossy().to_string(),
        };
        let mut key = file.file_name.clone();
        if referenced_attachments.contains_key(&key) {
            let mut suffix = 1;
            loop {
                key = format!("{} ({suffix})", file.file_name);
                if !referenced_attachments.contains_key(&key) {
                    break;
                }
                suffix += 1;
            }
        }
        referenced_attachments.insert(key, attachment);
    }

    // Add pending AI document as attachment if present
    if let Some(document_id) = context_model.pending_document_id() {
        if let Some(content) = AIDocumentModel::as_ref(ctx).get_document_content(&document_id, ctx)
        {
            let document_id_str = document_id.to_string();
            let attachment = AIAgentAttachment::DocumentContent {
                document_id: document_id_str.clone(),
                content,
                source: DocumentContentAttachmentSource::PlanEdited,
                line_range: None,
            };
            // Use the document ID as the reference key
            referenced_attachments.insert(document_id_str, attachment);
        }
    }

    referenced_attachments
}

/// Searches for a block across all terminal models in the application.
/// Returns an AIAgentAttachment if the block is found.
fn find_block_attachment_in_all_terminals(
    block_id: &BlockId,
    ctx: &AppContext,
) -> Option<AIAgentAttachment> {
    // Iterate over all window IDs to search across all terminal views
    for window_id in ctx.window_ids() {
        // Try to get all terminal views for this window
        if let Some(terminal_views) = ctx.views_of_type::<TerminalView>(window_id) {
            for terminal_view_handle in terminal_views {
                let terminal_view = terminal_view_handle.as_ref(ctx);
                let terminal_model = terminal_view.model.lock();
                let block_list = terminal_model.block_list();

                if let Some(block) = block_list.block_with_id(block_id) {
                    // Create an AIAgentAttachment for the block
                    return Some(AIAgentAttachment::Block(BlockContext {
                        id: block.id().clone(),
                        index: block.index(),
                        command: block.command_to_string(),
                        output: block.output_to_string(),
                        exit_code: block.exit_code(),
                        is_auto_attached: false,
                        started_ts: block.start_ts().cloned(),
                        finished_ts: block.completed_ts().cloned(),
                        pwd: None,
                        shell: None,
                        username: None,
                        hostname: None,
                        git_branch: None,
                        os: None,
                        session_id: None,
                    }));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::FairMutex;
    use warpui::r#async::executor::Background;
    use warpui::{App, EntityId, ModelHandle};

    use super::*;
    use crate::ai::agent::conversation::AIConversationId;
    use crate::ai::blocklist::agent_view::{AgentViewController, EphemeralMessageModel};
    use crate::appearance::Appearance;
    use crate::cloud_object::model::persistence::CloudModel;
    use crate::terminal::color::{self, Colors};
    use crate::terminal::event_listener::ChannelEventListener;
    use crate::terminal::model::TerminalModel;
    use crate::terminal::model::test_utils::block_size;
    use crate::test_util::settings::initialize_settings_for_tests;

    fn build_test_context_model(app: &mut App) -> ModelHandle<BlocklistAIContextModel> {
        let terminal_model = Arc::new(FairMutex::new(TerminalModel::new_for_test(
            block_size(),
            color::List::from(&Colors::default()),
            ChannelEventListener::new_for_test(),
            Arc::new(Background::default()),
            false,
            None,
            false,
            false,
            None,
        )));
        let terminal_view_id = EntityId::new();
        let ephemeral_message_model = app.add_model(|_| EphemeralMessageModel::new());
        let agent_view_controller = app.add_model(|_| {
            AgentViewController::new(
                terminal_model.clone(),
                terminal_view_id,
                ephemeral_message_model,
            )
        });

        app.add_model(|_| {
            BlocklistAIContextModel::new_for_test(
                terminal_model,
                terminal_view_id,
                agent_view_controller,
            )
        })
    }

    #[test]
    fn parses_plan_reference_as_document_attachment() {
        App::test((), |mut app| async move {
            initialize_settings_for_tests(&mut app);
            app.add_singleton_model(|_| Appearance::mock());
            app.add_singleton_model(|_| CloudModel::new(None, Vec::new()));
            let document_model = app.add_singleton_model(|_| AIDocumentModel::new_for_test());
            let document_id = document_model.update(&mut app, |model, ctx| {
                model.create_document("Plan", "ship it", AIConversationId::new(), None, ctx)
            });
            let context_model = build_test_context_model(&mut app);
            let reference = format!("<plan:{document_id}>");

            let attachments = context_model.read(&app, |context_model, ctx| {
                parse_context_attachments(&format!("use {reference}"), context_model, ctx)
            });
            let attachment = attachments
                .get(&reference)
                .expect("plan reference should be attached");

            assert!(matches!(
                attachment,
                AIAgentAttachment::DocumentContent {
                    document_id: id,
                    content,
                    source,
                    line_range: None,
                } if id == &document_id.to_string()
                    && content == "ship it"
                    && source == &DocumentContentAttachmentSource::UserAttached
            ));
        });
    }
}
