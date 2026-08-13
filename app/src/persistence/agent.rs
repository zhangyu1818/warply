use diesel::associations::HasTable;
use diesel::{SqliteConnection, prelude::*, result::Error};
use std::collections::HashMap;

use super::model::{AgentConversation, AgentConversationData};
use crate::persistence::model::AgentConversationRecord;
use crate::persistence::schema::{self, agent_conversations};

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = agent_conversations)]
struct NewAgentConversation {
    conversation_id: String,
    conversation_data: String,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum UpsertConversationError {
    #[error("Failed to serialize conversation data: {0:?}")]
    Serialization(#[from] serde_json::Error),
    #[error("Failed to upsert conversation to sqlite: {0:?}")]
    DB(#[from] diesel::result::Error),
}

pub(super) fn upsert_agent_conversation(
    conn: &mut SqliteConnection,
    conversation_id_param: &str,
    conversation_data_param: AgentConversationData,
) -> Result<(), UpsertConversationError> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use schema::agent_conversations::dsl::*;
    const MAX_PERSISTED_CONVERSATION_COUNT: i64 = 100;

    let serialized_conversation_data = serde_json::to_string(&conversation_data_param)?;

    conn.transaction::<_, Error, _>(|conn| {
        // Upsert the conversation level metadata
        let new_conversation = NewAgentConversation {
            conversation_id: conversation_id_param.to_owned(),
            conversation_data: serialized_conversation_data,
        };

        diesel::insert_into(agent_conversations::table())
            .values(&new_conversation)
            .on_conflict(conversation_id)
            .do_update()
            .set(&new_conversation)
            .execute(conn)?;

        // Prune old conversations if we exceed MAX_PERSISTED_CONVERSATION_COUNT conversations
        let conversation_count: i64 = agent_conversations::table().count().get_result(conn)?;
        if conversation_count > MAX_PERSISTED_CONVERSATION_COUNT {
            // Remove the oldest conversations, keeping only the most recent MAX_PERSISTED_CONVERSATION_COUNT
            let conversations_to_remove: Vec<String> = agent_conversations::table()
                .order(last_modified_at.asc())
                .limit(conversation_count - MAX_PERSISTED_CONVERSATION_COUNT)
                .select(conversation_id)
                .load(conn)?;

            delete_agent_conversations(conn, conversations_to_remove)?;
        }

        Ok(())
    })?;

    Ok(())
}

pub(super) fn read_agent_conversations(
    conn: &mut SqliteConnection,
) -> Result<Vec<AgentConversation>, diesel::result::Error> {
    use schema::agent_conversations::dsl::*;

    let conversations_by_id = HashMap::<String, AgentConversation>::from_iter(
        agent_conversations
            .select(AgentConversationRecord::as_select())
            .load(conn)?
            .into_iter()
            .map(|conversation| {
                (
                    conversation.conversation_id.clone(),
                    AgentConversation { conversation },
                )
            }),
    );

    Ok(conversations_by_id.into_values().collect())
}

pub(crate) fn read_agent_conversation_by_id(
    conn: &mut SqliteConnection,
    conversation_id_str: &str,
) -> Result<Option<AgentConversation>, diesel::result::Error> {
    use schema::agent_conversations::dsl as convo_dsl;
    let maybe_record: Option<AgentConversationRecord> = convo_dsl::agent_conversations
        .filter(convo_dsl::conversation_id.eq(conversation_id_str.to_owned()))
        .select(AgentConversationRecord::as_select())
        .first(conn)
        .optional()?;

    let Some(conversation_record) = maybe_record else {
        return Ok(None);
    };

    Ok(Some(AgentConversation {
        conversation: conversation_record,
    }))
}

pub(super) fn delete_agent_conversations(
    conn: &mut SqliteConnection,
    conversation_ids: Vec<String>,
) -> Result<(), diesel::result::Error> {
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use schema::agent_conversations::dsl::*;
    conn.transaction::<_, Error, _>(|conn| {
        diesel::delete(
            agent_conversations::table().filter(conversation_id.eq_any(&conversation_ids)),
        )
        .execute(conn)?;

        Ok(())
    })?;

    Ok(())
}
