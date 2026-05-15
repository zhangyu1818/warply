use agent_client_protocol::schema::{ContentBlock, ContentChunk, SessionUpdate};

use super::events::AcpEvent;
use super::AcpToolCall;

pub fn map_session_update(update: SessionUpdate) -> Option<AcpEvent> {
    match update {
        SessionUpdate::UserMessageChunk(chunk) => {
            text_from_chunk(chunk).map(|text| AcpEvent::UserTextDelta { text })
        }
        SessionUpdate::AgentMessageChunk(chunk) => {
            text_from_chunk(chunk).map(|text| AcpEvent::AssistantTextDelta { text })
        }
        SessionUpdate::AgentThoughtChunk(chunk) => {
            text_from_chunk(chunk).map(|text| AcpEvent::AssistantThoughtDelta { text })
        }
        SessionUpdate::ToolCall(tool_call) => Some(AcpEvent::ToolCallStarted {
            tool_call: AcpToolCall::from_acp(tool_call),
        }),
        SessionUpdate::ToolCallUpdate(update) => Some(AcpEvent::ToolCallUpdated { update }),
        SessionUpdate::Plan(plan) => Some(AcpEvent::PlanUpdated { plan }),
        SessionUpdate::AvailableCommandsUpdate(update) => {
            Some(AcpEvent::AvailableCommandsUpdated {
                commands: update.available_commands,
            })
        }
        SessionUpdate::CurrentModeUpdate(update) => Some(AcpEvent::CurrentModeUpdated { update }),
        SessionUpdate::ConfigOptionUpdate(update) => {
            Some(AcpEvent::ConfigOptionsUpdated { update })
        }
        SessionUpdate::SessionInfoUpdate(update) => Some(AcpEvent::SessionInfoUpdated { update }),
        _ => None,
    }
}

fn text_from_chunk(chunk: ContentChunk) -> Option<String> {
    match chunk.content {
        ContentBlock::Text(text) => Some(text.text),
        _ => None,
    }
}

pub fn session_update_label(update: &SessionUpdate) -> &'static str {
    match update {
        SessionUpdate::UserMessageChunk(_) => "user_message_chunk",
        SessionUpdate::AgentMessageChunk(_) => "agent_message_chunk",
        SessionUpdate::AgentThoughtChunk(_) => "agent_thought_chunk",
        SessionUpdate::ToolCall(_) => "tool_call",
        SessionUpdate::ToolCallUpdate(_) => "tool_call_update",
        SessionUpdate::Plan(_) => "plan",
        SessionUpdate::AvailableCommandsUpdate(_) => "available_commands_update",
        SessionUpdate::CurrentModeUpdate(_) => "current_mode_update",
        SessionUpdate::ConfigOptionUpdate(_) => "config_option_update",
        SessionUpdate::SessionInfoUpdate(_) => "session_info_update",
        _ => "unknown",
    }
}
