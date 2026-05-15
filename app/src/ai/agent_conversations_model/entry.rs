use crate::ai::agent::conversation::AIConversationId;
use crate::ai::artifacts::Artifact;
use crate::ai::blocklist::history_model::{AIConversationMetadata, BlocklistAIHistoryModel};
use crate::ai::conversation_navigation::ConversationNavigationData;
use crate::identity::LocalIdentityProvider;
use chrono::{DateTime, Utc};
use warpui::{AppContext, SingletonEntity};

use super::{
    artifacts_match_filter, AgentRunDisplayStatus, ArtifactFilter, ConversationListFilters,
    ConversationMetadata, CreatedOnFilter, CreatorFilter, OwnerFilter, SourceFilter, StatusFilter,
};

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
    pub identity: AgentConversationIdentity,
    pub provenance: AgentConversationProvenance,
    pub display: AgentConversationDisplayData,
    pub backing: AgentConversationBackingData,
    pub capabilities: AgentConversationCapabilities,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentConversationIdentity {
    pub local_conversation_id: Option<AIConversationId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentConversationDisplayData {
    pub title: String,
    pub initial_query: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub status: AgentRunDisplayStatus,
    pub creator: AgentConversationCreator,
    pub run_time: Option<String>,
    pub working_directory: Option<String>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentConversationCreator {
    pub name: Option<String>,
    pub uid: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentConversationProvenance {
    LocalInteractive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentConversationBackingData {
    pub has_loaded_conversation: bool,
    pub has_local_persisted_data: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentConversationCapabilities {
    pub can_open: bool,
    pub can_copy_link: bool,
    pub can_share: bool,
    pub can_delete: bool,
    pub can_fork_locally: bool,
    pub can_cancel: bool,
}

impl AgentConversationEntry {
    pub(super) fn matches_filters(
        &self,
        filters: &ConversationListFilters,
        app: &AppContext,
    ) -> bool {
        self.matches_owner_and_creator(&filters.owners, &filters.creator, app)
            && self.matches_status(&filters.status)
            && self.matches_source(&filters.source)
            && self.matches_created_on(&filters.created_on)
            && self.matches_artifact(&filters.artifact)
    }

    fn matches_owner_and_creator(
        &self,
        owner_filter: &OwnerFilter,
        creator_filter: &CreatorFilter,
        _app: &AppContext,
    ) -> bool {
        if matches!(owner_filter, OwnerFilter::PersonalOnly) {
            return true;
        }

        match creator_filter {
            CreatorFilter::All => true,
            CreatorFilter::Specific { name, .. } => {
                self.display.creator.name.as_ref() == Some(name)
            }
        }
    }

    fn matches_status(&self, status_filter: &StatusFilter) -> bool {
        match status_filter {
            StatusFilter::All => true,
            StatusFilter::Working | StatusFilter::Done | StatusFilter::Failed => {
                self.display.status.status_filter() == *status_filter
            }
        }
    }

    fn matches_source(&self, source_filter: &SourceFilter) -> bool {
        matches!(source_filter, SourceFilter::All)
    }

    fn matches_created_on(&self, created_on_filter: &CreatedOnFilter) -> bool {
        let now = Utc::now();
        let created_cutoff = match created_on_filter {
            CreatedOnFilter::All => None,
            CreatedOnFilter::Last24Hours => Some(now - chrono::Duration::hours(24)),
            CreatedOnFilter::Past3Days => Some(now - chrono::Duration::days(3)),
            CreatedOnFilter::LastWeek => Some(now - chrono::Duration::days(7)),
        };
        match created_cutoff {
            Some(cutoff) => self.display.created_at >= cutoff,
            None => true,
        }
    }

    fn matches_artifact(&self, artifact_filter: &ArtifactFilter) -> bool {
        artifacts_match_filter(&self.display.artifacts, artifact_filter)
    }
}

pub(super) fn entry_for_conversation(
    metadata: &ConversationMetadata,
    history_model: &BlocklistAIHistoryModel,
    app: &AppContext,
) -> AgentConversationEntry {
    let conversation_metadata = history_model.get_conversation_metadata(&metadata.nav_data.id);
    entry_for_conversation_parts(
        metadata.nav_data.clone(),
        conversation_metadata,
        history_model,
        app,
    )
}

fn entry_for_conversation_parts(
    nav_data: ConversationNavigationData,
    conversation_metadata: Option<&AIConversationMetadata>,
    history_model: &BlocklistAIHistoryModel,
    app: &AppContext,
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
    let provenance = AgentConversationProvenance::LocalInteractive;
    let title = conversation
        .and_then(|conversation| conversation.title().clone())
        .unwrap_or_else(|| nav_data.title.clone());

    AgentConversationEntry {
        id: AgentConversationEntryId::Conversation(conversation_id),
        identity: AgentConversationIdentity {
            local_conversation_id: Some(conversation_id),
        },
        provenance,
        display: AgentConversationDisplayData {
            title,
            initial_query: nav_data.initial_query.clone(),
            created_at: nav_data.last_updated.into(),
            last_updated: nav_data.last_updated.into(),
            status: status.clone(),
            creator: AgentConversationCreator {
                name: Some(
                    LocalIdentityProvider::as_ref(app)
                        .get()
                        .username_for_display(),
                ),
                uid: Some(
                    LocalIdentityProvider::as_ref(app)
                        .get()
                        .user_id()
                        .to_string(),
                ),
            },
            run_time: None,
            working_directory: nav_data
                .latest_working_directory
                .clone()
                .or_else(|| nav_data.initial_working_directory.clone()),
            artifacts: conversation
                .map(|conversation| conversation.artifacts().to_vec())
                .unwrap_or_default(),
        },
        backing: AgentConversationBackingData {
            has_loaded_conversation,
            has_local_persisted_data,
        },
        capabilities: AgentConversationCapabilities {
            can_open: has_local_persisted_data,
            can_copy_link: false,
            can_share: false,
            can_delete: has_local_persisted_data,
            can_fork_locally: has_local_persisted_data,
            can_cancel: status.is_cancellable(),
        },
    }
}
