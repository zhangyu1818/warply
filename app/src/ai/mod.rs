//! This module houses AI functionality throughout Warp.
pub mod acp;
pub(crate) mod active_agent_views_model;
pub(crate) mod agent;
pub(crate) mod agent_conversations_model;
pub(crate) mod agent_tips;
pub(crate) mod ai_document_view;
pub mod artifacts;
pub(crate) mod ask;
pub(crate) mod block_context;
pub(crate) mod blocklist;
pub mod control_code_parser;
pub(crate) mod conversation_details_action_buttons;
pub(crate) mod conversation_details_panel;
pub(crate) mod conversation_navigation;
pub(crate) mod conversation_status_ui;
pub(crate) mod conversation_utils;
pub(crate) mod document;
pub(crate) mod execution_context;
pub(crate) mod get_relevant_files;
pub(crate) mod llms;
pub(crate) mod persisted_workspace;
pub(crate) mod predict;
pub(crate) mod restored_conversations;
pub(crate) mod skills;
pub mod terminal_suggestions;
pub use agent_tips::*;
use warpui::AppContext;
pub mod execution_profiles;
pub mod facts;
pub(crate) mod loading;
pub mod mcp;
pub mod outline;

pub(crate) use ai::paths;
pub(crate) use ask::AskAIType;

pub fn init(app: &mut AppContext) {
    blocklist::keyboard_navigable_buttons::init(app);
    blocklist::block::number_shortcut_buttons::init(app);
    blocklist::toggleable_items::init(app);
    blocklist::suggested_agent_mode_workflow_modal::init(app);
    blocklist::suggested_rule_modal::init(app);
    ai_document_view::init(app);
    conversation_details_panel::init(app);
}
