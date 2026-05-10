pub mod cloud_object_styling;
pub mod folders;
pub mod items;
pub mod workflows;

use std::fmt;

use crate::{
    cloud_object::{GenericStringObjectFormat, ObjectIdType, ObjectType},
    object_ids::{HashedSqliteId, ObjectUid, ServerId, SyncId},
    ui_components::icons::Icon,
};

#[derive(Copy, Clone, Debug)]
pub enum DriveObjectType {
    Workflow,
    AgentModeWorkflow,
    AIFact,
    AIFactCollection,
    Folder,
    EnvVarCollection,
    MCPServer,
    MCPServerCollection,
}

#[derive(Copy, Clone, PartialEq)]
pub enum DriveIndexVariant {
    MainIndex,
    Trash,
}

impl From<DriveObjectType> for Icon {
    fn from(cloud_object_type: DriveObjectType) -> Icon {
        match cloud_object_type {
            DriveObjectType::Workflow => Icon::Workflow,
            DriveObjectType::AgentModeWorkflow => Icon::Prompt,
            DriveObjectType::AIFact => Icon::BookOpen,
            DriveObjectType::AIFactCollection => Icon::BookOpen,
            DriveObjectType::Folder => Icon::Folder,
            DriveObjectType::EnvVarCollection => Icon::EnvVarCollection,
            DriveObjectType::MCPServer => Icon::Dataflow,
            DriveObjectType::MCPServerCollection => Icon::Dataflow,
        }
    }
}

impl fmt::Display for DriveObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DriveObjectType::Workflow => write!(f, "workflow"),
            DriveObjectType::Folder => write!(f, "folder"),
            DriveObjectType::EnvVarCollection => write!(f, "env var collection"),
            DriveObjectType::AgentModeWorkflow => write!(f, "prompt"),
            DriveObjectType::AIFact => write!(f, "ai fact"),
            DriveObjectType::AIFactCollection => write!(f, "ai fact collection"),
            DriveObjectType::MCPServer => write!(f, "mcp server"),
            DriveObjectType::MCPServerCollection => write!(f, "mcp server collection"),
        }
    }
}

/// Enum to use to pass down type and id between actions to avoid multiplying actions whenever we
/// need to pass the object id etc.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CloudObjectTypeAndId {
    Workflow(SyncId),
    Folder(SyncId),
    GenericStringObject {
        object_type: GenericStringObjectFormat,
        id: SyncId,
    },
}

impl CloudObjectTypeAndId {
    pub fn from_id_and_type(id: SyncId, object_type: ObjectType) -> Self {
        match object_type {
            ObjectType::Workflow => Self::Workflow(id),
            ObjectType::Folder => Self::Folder(id),
            ObjectType::GenericStringObject(format) => Self::GenericStringObject {
                object_type: format,
                id,
            },
        }
    }

    pub fn uid(self) -> ObjectUid {
        match self {
            Self::Workflow(id) => id.uid(),
            Self::Folder(id) => id.uid(),
            Self::GenericStringObject { id, .. } => id.uid(),
        }
    }

    pub fn sync_id(self) -> SyncId {
        match self {
            Self::Workflow(id) | Self::Folder(id) | Self::GenericStringObject { id, .. } => id,
        }
    }

    pub fn sqlite_uid_hash(self) -> HashedSqliteId {
        match self {
            CloudObjectTypeAndId::Workflow(id) => id.sqlite_uid_hash(ObjectIdType::Workflow),
            CloudObjectTypeAndId::Folder(id) => id.sqlite_uid_hash(ObjectIdType::Folder),
            CloudObjectTypeAndId::GenericStringObject { object_type: _, id } => {
                id.sqlite_uid_hash(ObjectIdType::GenericStringObject)
            }
        }
    }

    pub fn object_id_type(&self) -> ObjectIdType {
        match self {
            CloudObjectTypeAndId::Workflow(_) => ObjectIdType::Workflow,
            CloudObjectTypeAndId::GenericStringObject { .. } => ObjectIdType::GenericStringObject,
            CloudObjectTypeAndId::Folder(_) => ObjectIdType::Folder,
        }
    }

    pub fn object_type(&self) -> ObjectType {
        match self {
            CloudObjectTypeAndId::Workflow(_) => ObjectType::Workflow,
            CloudObjectTypeAndId::Folder(_) => ObjectType::Folder,
            CloudObjectTypeAndId::GenericStringObject { object_type, .. } => {
                ObjectType::GenericStringObject(*object_type)
            }
        }
    }

    pub fn as_folder_id(self) -> Option<SyncId> {
        match self {
            CloudObjectTypeAndId::Workflow(_) => None,
            CloudObjectTypeAndId::GenericStringObject { .. } => None,
            CloudObjectTypeAndId::Folder(f) => Some(f),
        }
    }

    pub fn as_generic_string_object_id(self) -> Option<SyncId> {
        match self {
            CloudObjectTypeAndId::GenericStringObject { object_type: _, id } => Some(id),
            _ => None,
        }
    }

    pub fn has_server_id(self) -> bool {
        matches!(
            self,
            CloudObjectTypeAndId::Workflow(SyncId::ServerId(_))
                | CloudObjectTypeAndId::Folder(SyncId::ServerId(_))
                | CloudObjectTypeAndId::GenericStringObject {
                    id: SyncId::ServerId(_),
                    ..
                }
        )
    }

    pub fn server_id(self) -> Option<ServerId> {
        match self {
            CloudObjectTypeAndId::Workflow(SyncId::ServerId(workflow_id)) => Some(workflow_id),
            CloudObjectTypeAndId::Folder(SyncId::ServerId(folder_id)) => Some(folder_id),
            CloudObjectTypeAndId::GenericStringObject {
                id: SyncId::ServerId(json_object_id),
                ..
            } => Some(json_object_id),
            _ => None,
        }
    }

    pub fn drive_row_position_id(self) -> String {
        format!("LocalObjectRow_{}", self.uid())
    }

    pub fn from_generic_string_object(object_type: GenericStringObjectFormat, id: SyncId) -> Self {
        Self::GenericStringObject { object_type, id }
    }
}
