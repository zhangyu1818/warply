use crate::ai::agent::conversation::AIConversationId;
use crate::ai::blocklist::history_model::{AIConversationMetadata, BlocklistAIHistoryModel};
use crate::ai::conversation_navigation::ConversationNavigationData;
use chrono::{DateTime, Utc};

use super::AgentRunDisplayStatus;

#[derive(Clone, Debug, PartialEq)]
pub struct AgentConversationEntry {
    pub conversation_id: AIConversationId,
    pub display: AgentConversationDisplayData,
    pub is_available_locally: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentConversationDisplayData {
    pub title: String,
    pub last_updated: DateTime<Utc>,
    pub status: AgentRunDisplayStatus,
    pub working_directory: Option<String>,
}

pub(super) fn entry_for_conversation(
    nav_data: &ConversationNavigationData,
    history_model: &BlocklistAIHistoryModel,
) -> AgentConversationEntry {
    let conversation_metadata = history_model.get_conversation_metadata(&nav_data.id);
    entry_for_conversation_parts(nav_data, conversation_metadata, history_model)
}

fn entry_for_conversation_parts(
    nav_data: &ConversationNavigationData,
    conversation_metadata: Option<&AIConversationMetadata>,
    history_model: &BlocklistAIHistoryModel,
) -> AgentConversationEntry {
    let conversation_id = nav_data.id;
    let conversation = history_model.conversation(&conversation_id);
    let status = conversation
        .map(|conversation| AgentRunDisplayStatus::from_conversation_status(conversation.status()))
        .unwrap_or(AgentRunDisplayStatus::ConversationSucceeded);
    let has_loaded_conversation = conversation.is_some();
    let is_available_locally = conversation_metadata
        .is_some_and(|metadata| metadata.has_local_data)
        || has_loaded_conversation;
    let title = conversation
        .and_then(|conversation| conversation.title().clone())
        .unwrap_or_else(|| nav_data.title.clone());

    AgentConversationEntry {
        conversation_id,
        display: AgentConversationDisplayData {
            title,
            last_updated: nav_data.last_updated.into(),
            status,
            working_directory: nav_data
                .latest_working_directory
                .clone()
                .or_else(|| nav_data.initial_working_directory.clone()),
        },
        is_available_locally,
    }
}
