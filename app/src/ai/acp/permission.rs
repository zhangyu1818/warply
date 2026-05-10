use agent_client_protocol::schema::{
    PermissionOption, PermissionOptionKind, RequestPermissionRequest, ToolCallUpdate,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AcpPermissionRequest {
    pub request_id: String,
    pub tool_call_id: String,
    pub tool_call_update: ToolCallUpdate,
    pub options: Vec<AcpPermissionOption>,
    pub selected_option_id: Option<String>,
}

impl Eq for AcpPermissionRequest {}

impl AcpPermissionRequest {
    pub fn from_acp(request: RequestPermissionRequest) -> Self {
        let request_id = request.tool_call.tool_call_id.0.to_string();
        Self {
            request_id: request_id.clone(),
            tool_call_id: request_id,
            tool_call_update: request.tool_call,
            options: request
                .options
                .into_iter()
                .map(AcpPermissionOption::from_acp)
                .collect(),
            selected_option_id: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpPermissionOption {
    pub option_id: String,
    pub name: String,
    pub kind: PermissionOptionKind,
}

impl AcpPermissionOption {
    pub fn from_acp(option: PermissionOption) -> Self {
        Self {
            option_id: option.option_id.0.to_string(),
            name: option.name,
            kind: option.kind,
        }
    }
}

pub enum AcpPermissionSelection {
    Selected { option_id: String },
    Cancelled,
}
