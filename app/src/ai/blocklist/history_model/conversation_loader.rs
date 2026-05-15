//! This module contains functions for loading and merging conversation data from local storage.

use std::collections::HashMap;
use std::future::Future;

use crate::ai::agent::conversation::{AIConversation, AIConversationId};
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

    let conversation_data = match serde_json::from_str::<AgentConversationData>(&conversation_data)
    {
        Ok(data) => data,
        Err(e) => {
            log::warn!("Failed to deserialize persisted conversation data: {e:?}");
            return None;
        }
    };

    match AIConversation::new_restored(conversation_id, conversation_data) {
        Ok(conversation) => Some(conversation),
        Err(e) => {
            log::warn!("Failed to convert persisted conversation to AIConversation: {e:?}");
            None
        }
    }
}

fn box_future<F>(f: F) -> warpui::r#async::BoxFuture<'static, Option<RestoredConversationData>>
where
    F: Future<Output = Option<RestoredConversationData>> + warpui::r#async::Spawnable,
{
    f.boxed()
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

                let conversation_data = match serde_json::from_str::<AgentConversationData>(
                    &agent_conv.conversation.conversation_data,
                ) {
                    Ok(data) => data,
                    Err(e) => {
                        log::warn!("Failed to deserialize conversation data: {e:?}");
                        return None;
                    }
                };
                let conversation =
                    match AIConversation::new_restored(conversation_id, conversation_data) {
                        Ok(conversation) => conversation,
                        Err(e) => {
                            log::warn!(
                                "Failed to record conversation with ID {conversation_id}: {e:?}"
                            );
                            return None;
                        }
                    };
                let mut metadata = AIConversationMetadata::from(&conversation);
                if metadata.initial_query.is_empty() {
                    return None;
                }
                metadata.last_modified_at = agent_conv.conversation.last_modified_at;

                Some((conversation_id, metadata))
            })
            .collect();

        self.all_conversations_metadata = collected;
    }
}
