//! This module contains functions for loading and merging conversation data from local storage.

use std::collections::HashMap;
use std::future::Future;

use crate::ai::agent::conversation::{AIConversation, AIConversationId};
use crate::ai::agent::task::Task;
use crate::persistence::model::{AgentConversation, AgentConversationData};
use futures::FutureExt;
use itertools::Itertools as _;
use persistence::model::AgentConversationRecord;
use warpui::AppContext;

#[cfg(feature = "local_fs")]
use crate::persistence::agent::read_agent_conversation_by_id;

use super::{AIConversationMetadata, BlocklistAIHistoryModel, MAX_HISTORICAL_CONVERSATIONS};

pub enum RestoredConversationData {
    Conversation(Box<AIConversation>),
}

/// Converts an `AgentConversation` from the database to an `AIConversation`.
/// This utility function extracts the conversion logic that was originally embedded
/// in the terminal view restoration process.
#[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
pub fn convert_persisted_conversation_to_ai_conversation(
    persisted_conversation: AgentConversation,
) -> Option<AIConversation> {
    convert_persisted_conversation_to_ai_conversation_with_metadata(persisted_conversation)
}

/// Enhanced version of the conversion function with additional metadata.
/// This version supports the full feature set needed by terminal view restoration.
pub fn convert_persisted_conversation_to_ai_conversation_with_metadata(
    persisted_conversation: AgentConversation,
) -> Option<AIConversation> {
    let AgentConversation {
        tasks,
        conversation:
            AgentConversationRecord {
                conversation_id,
                conversation_data,
                ..
            },
    } = persisted_conversation;

    let conversation_id = match AIConversationId::try_from(conversation_id) {
        Ok(id) => id,
        Err(e) => {
            log::warn!("Failed to convert conversation ID: {e:?}");
            return None;
        }
    };

    let conversation_data = serde_json::from_str::<AgentConversationData>(&conversation_data).ok();

    match AIConversation::new_restored(conversation_id, tasks, conversation_data) {
        Ok(conversation) => Some(conversation),
        Err(e) => {
            log::warn!("Failed to convert persisted conversation to AIConversation: {e:?}");
            None
        }
    }
}

/// Boxes a future with the right type for the platform.
/// On WASM, futures must not implement Send.
fn box_future<F>(f: F) -> warpui::r#async::BoxFuture<'static, Option<RestoredConversationData>>
where
    F: Future<Output = Option<RestoredConversationData>> + warpui::r#async::Spawnable,
{
    cfg_if::cfg_if! {
        if #[cfg(target_family = "wasm")] {
            f.boxed_local()
        } else {
            f.boxed()
        }
    }
}

impl BlocklistAIHistoryModel {
    pub fn load_conversation_data(
        &self,
        conversation_id: AIConversationId,
        _ctx: &AppContext,
    ) -> warpui::r#async::BoxFuture<'static, Option<RestoredConversationData>> {
        if let Some(conversation) = self.conversations_by_id.get(&conversation_id) {
            return box_future(futures::future::ready(Some(
                RestoredConversationData::Conversation(Box::new(conversation.clone())),
            )));
        }

        let Some(metadata) = self
            .all_conversations_metadata
            .get(&conversation_id)
            .cloned()
        else {
            log::warn!("No metadata found for conversation {conversation_id}");
            return box_future(futures::future::ready(None));
        };

        if metadata.has_local_data {
            let result = self
                .load_conversation_from_db(&conversation_id)
                .map(|c| RestoredConversationData::Conversation(Box::new(c)));
            box_future(futures::future::ready(result))
        } else {
            log::warn!("Cannot load conversation {conversation_id}: no local data");
            box_future(futures::future::ready(None))
        }
    }

    /// Loads a conversation from local DB and returns it.
    /// This is a private helper method. Use `get_load_conversation_data_future` instead.
    ///
    /// Note: This does NOT insert the conversation into memory. Callers are responsible
    /// for inserting the loaded conversation if needed.
    pub(super) fn load_conversation_from_db(
        &self,
        conversation_id: &AIConversationId,
    ) -> Option<AIConversation> {
        // First check if the conversation is in memory
        if let Some(conversation) = self.conversations_by_id.get(conversation_id) {
            return Some(conversation.clone());
        }

        // If not in memory, try to load from the database
        #[cfg(feature = "local_fs")]
        {
            let persisted_ai_conversation = self.db_connection.clone().and_then(|conn| {
                let mut conn = conn.lock().ok()?;

                let id_str = conversation_id.to_string();
                log::info!("Loading conversation {id_str} from db");
                match read_agent_conversation_by_id(&mut conn, &id_str) {
                    Ok(Some(conv)) => Some(conv),
                    Ok(None) => {
                        log::warn!("No AgentConversation found with id {id_str}");
                        None
                    }
                    Err(e) => {
                        log::warn!("Failed to read AgentConversation {id_str}: {e:?}");
                        None
                    }
                }
            });

            // Convert the persisted conversation to an AIConversation
            if let Some(persisted_conversation) = persisted_ai_conversation {
                if let Some(conversation) =
                    convert_persisted_conversation_to_ai_conversation(persisted_conversation)
                {
                    return Some(conversation);
                }
            }
        }

        None
    }

    /// Initializes historical conversations from restored agent conversations.
    pub(super) fn initialize_historical_conversations(
        &mut self,
        conversations: &[AgentConversation],
    ) {
        let conversations = conversations
            .iter()
            .sorted_by_key(|c| c.conversation.last_modified_at)
            .rev();

        let collected: HashMap<AIConversationId, AIConversationMetadata> = conversations
            .take(MAX_HISTORICAL_CONVERSATIONS)
            .filter_map(|agent_conv| {
                // Try to convert the conversation ID
                let conversation_id = match AIConversationId::try_from(
                    agent_conv.conversation.conversation_id.clone(),
                ) {
                    Ok(id) => id,
                    Err(e) => {
                        log::warn!("Failed to convert conversation ID: {e:?}");
                        return None;
                    }
                };

                if !agent_conv.is_restorable() {
                    return None;
                }

                // Skip conversations that contain AutoCodeDiff system queries but do not contain any UserQuery messages.
                // These are passive, auto-initiated diffs that were never interacted with (past accepting or rejecting the diff),
                // so we don't want to list them as historical conversations.
                let mut has_user_query = false;
                let mut has_autocodediff = false;
                for task in &agent_conv.tasks {
                    for message in &task.messages {
                        match &message.message {
                            Some(warp_multi_agent_api::message::Message::UserQuery(_)) => {
                                has_user_query = true;
                            }
                            Some(warp_multi_agent_api::message::Message::SystemQuery(sys)) => {
                                if let Some(warp_multi_agent_api::message::system_query::Type::AutoCodeDiff(_)) = &sys.r#type {
                                    has_autocodediff = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                if has_autocodediff && !has_user_query {
                    return None;
                }

                let root_task = agent_conv.tasks.iter().find(|task| {
                    task.dependencies.is_none()
                });

                let initial_query = root_task.map(|task| {
                    // find the first task with a user query
                    // (or in the case of a passive code diff, the summary of the diff)
                    task.messages.iter().find_map(|msg| {
                        match &msg.message {
                            Some(warp_multi_agent_api::message::Message::UserQuery(user_query)) => {
                                Some(user_query.query.clone())
                            }
                            Some(warp_multi_agent_api::message::Message::ToolCall(tool_call))  => {
                                let Some(tool) = &tool_call.tool else {
                                    return None;
                                };

                                if let warp_multi_agent_api::message::tool_call::Tool::ApplyFileDiffs(diff_suggestion) = tool {
                                    Some(diff_suggestion.summary.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }).unwrap_or_default()
                }).unwrap_or_default();

                if initial_query.is_empty() {
                    log::warn!(
                        "Failed to record conversation with ID {conversation_id} because it was missing an initial query"
                    );
                    return None;
                }

                // We derive the title from the description of the root task,
                // falling back to initial_query if the description is empty
                let title = root_task
                    .map(|task| task.description.clone())
                    .filter(|desc| !desc.is_empty())
                    .unwrap_or_else(|| initial_query.clone());

                // Extract working directory from the first UserQuery message in the tasks
                // TODO: search tasks in correct order once we've implemented task ordering.
                let initial_working_directory = agent_conv.tasks.iter().find_map(Task::api_task_initial_working_directory);
                let conversation_data = serde_json::from_str::<AgentConversationData>(
                    &agent_conv.conversation.conversation_data,
                )
                .ok();
                let artifacts = conversation_data
                    .as_ref()
                    .and_then(|data| data.artifacts_json.as_ref())
                    .and_then(|json| serde_json::from_str(json).ok())
                    .unwrap_or_default();

                Some((
                    conversation_id,
                    AIConversationMetadata {
                        id: conversation_id,
                        title,
                        initial_query,
                        last_modified_at: agent_conv.conversation.last_modified_at,
                        initial_working_directory,
                        has_local_data: true,
                        artifacts,
                    },
                ))
            })
            .collect();

        self.all_conversations_metadata = collected;
    }
}
