pub mod data_source;
mod search_item;

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ConversationContextItem {
    pub title: String,
    pub conversation_id: String,
    pub last_updated: DateTime<Utc>,
}
