use serde::{Deserialize, Serialize};

use crate::workflows::WorkflowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionEntrypoint {
    Keyboard,
    Button,
    Menu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "block_type")]
pub enum BlockInfo {
    EmbeddedWorkflow { workflow_id: Option<WorkflowId> },
    CodeBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode {
    Command,
    Text,
}
