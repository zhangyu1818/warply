use crate::ai::agent::conversation::AIConversationId;
use crate::ai::artifacts::Artifact;
use crate::ai::blocklist::history_model::{AIConversationMetadata, BlocklistAIHistoryModel};
use crate::ai::conversation_navigation::ConversationNavigationData;
use chrono::{DateTime, Utc};

use super::{AgentRunDisplayStatus, ConversationMetadata};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AgentConversationEntryId {
    Conversation(AIConversationId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentConversationNavigationSubject {
    Entry(AgentConversationEntryId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentConversationEntry {
    pub id: AgentConversationEntryId,
    pub conversation_id: AIConversationId,
    pub display: AgentConversationDisplayData,
    pub capabilities: AgentConversationCapabilities,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentConversationDisplayData {
    pub title: String,
    pub initial_query: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub status: AgentRunDisplayStatus,
    pub run_time: Option<String>,
    pub working_directory: Option<String>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentConversationCapabilities {
    pub can_open: bool,
    pub can_delete: bool,
    pub can_fork_locally: bool,
    pub can_cancel: bool,
}

pub(super) fn entry_for_conversation(
    metadata: &ConversationMetadata,
    history_model: &BlocklistAIHistoryModel,
) -> AgentConversationEntry {
    let conversation_metadata = history_model.get_conversation_metadata(&metadata.nav_data.id);
    entry_for_conversation_parts(
        metadata.nav_data.clone(),
        conversation_metadata,
        history_model,
    )
}

fn entry_for_conversation_parts(
    nav_data: ConversationNavigationData,
    conversation_metadata: Option<&AIConversationMetadata>,
    history_model: &BlocklistAIHistoryModel,
) -> AgentConversationEntry {
    let conversation_id = nav_data.id;
    let conversation = history_model.conversation(&conversation_id);
    let status = conversation
        .map(|conversation| AgentRunDisplayStatus::from_conversation_status(conversation.status()))
        .unwrap_or(AgentRunDisplayStatus::ConversationSucceeded);
    let has_loaded_conversation = conversation.is_some();
    let has_local_persisted_data = conversation_metadata
        .is_some_and(|metadata| metadata.has_local_data)
        || has_loaded_conversation;
    let title = conversation
        .and_then(|conversation| conversation.title().clone())
        .unwrap_or_else(|| nav_data.title.clone());

    AgentConversationEntry {
        id: AgentConversationEntryId::Conversation(conversation_id),
        conversation_id,
        display: AgentConversationDisplayData {
            title,
            initial_query: nav_data.initial_query.clone(),
            created_at: nav_data.last_updated.into(),
            last_updated: nav_data.last_updated.into(),
            status: status.clone(),
            run_time: None,
            working_directory: nav_data
                .latest_working_directory
                .clone()
                .or_else(|| nav_data.initial_working_directory.clone()),
            artifacts: conversation
                .map(|conversation| conversation.artifacts().to_vec())
                .unwrap_or_default(),
        },
        capabilities: AgentConversationCapabilities {
            can_open: has_local_persisted_data,
            can_delete: has_local_persisted_data,
            can_fork_locally: has_local_persisted_data,
            can_cancel: status.is_cancellable(),
        },
    }
}
