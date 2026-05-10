use agent_client_protocol::schema::{
    AvailableCommand, ConfigOptionUpdate, CurrentModeUpdate, Plan, SessionInfoUpdate,
    ToolCallUpdate,
};

use super::{AcpPermissionRequest, AcpTerminalTrace, AcpToolCall};

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum AcpEvent {
    AdapterMissing {
        command: String,
        install_command: String,
    },
    SessionStarted,
    UserTextDelta {
        text: String,
    },
    AssistantTextDelta {
        text: String,
    },
    AssistantThoughtDelta {
        text: String,
    },
    ToolCallStarted {
        tool_call: AcpToolCall,
    },
    ToolCallUpdated {
        update: ToolCallUpdate,
    },
    TerminalUpdated {
        terminal_id: String,
        trace: AcpTerminalTrace,
    },
    PlanUpdated {
        plan: Plan,
    },
    AvailableCommandsUpdated {
        commands: Vec<AvailableCommand>,
    },
    CurrentModeUpdated {
        update: CurrentModeUpdate,
    },
    ConfigOptionsUpdated {
        update: ConfigOptionUpdate,
    },
    SessionInfoUpdated {
        update: SessionInfoUpdate,
    },
    PermissionRequested {
        request: AcpPermissionRequest,
    },
    Cancelled,
    Completed,
    Failed {
        message: String,
    },
}
