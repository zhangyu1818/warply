use std::{borrow::Cow, fmt, str::FromStr};

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use derivative::Derivative;
use pathfinder_geometry::vector::vec2f;
use serde::{Deserialize, Serialize};
use warp_core::ui::{Icon, appearance::Appearance, theme::Fill};
use warpui_core::{
    Element,
    elements::{
        Align, ChildAnchor, ConstrainedBox, Hoverable, MouseStateHandle, OffsetPositioning,
        ParentAnchor, ParentElement, ParentOffsetBounds, Stack,
    },
    ui_components::components::UiComponent,
};

use crate::{identity::UserUid, ids::SyncId};

/// The type of object id each ObjectType corresponds to.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ObjectIdType {
    Workflow,
    Folder,
    GenericStringObject,
}

impl ObjectIdType {
    /// Returns the prefix for server IDs as we store them in sqlite. The prefix for these
    /// objects is in title case unlike how we store the object types, which is why two different
    /// APIs are needed.
    pub fn sqlite_prefix(&self) -> &'static str {
        match self {
            ObjectIdType::Workflow => "Workflow",
            ObjectIdType::Folder => "Folder",
            ObjectIdType::GenericStringObject => "GenericStringObject",
        }
    }
}

/// A type for communicating the type of cloud object to/from the server, absent of the object itself.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub enum ObjectType {
    Workflow,
    Folder,
    GenericStringObject(GenericStringObjectFormat),
}

impl ObjectType {
    /// Returns the serialized string for the object type, to be used for storing object_type in sqlite.
    pub fn sqlite_object_type_as_str(&self) -> Cow<'_, str> {
        match self {
            ObjectType::Workflow => "WORKFLOW".into(),
            ObjectType::Folder => "FOLDER".into(),
            ObjectType::GenericStringObject(format) => format.to_string().into(),
        }
    }
}

const WORKFLOW_OBJECT_STRING: &str = "workflow";
const PROMPT_OBJECT_STRING: &str = "prompt";
const FOLDER_OBJECT_STRING: &str = "folder";
const ENV_VAR_COLLECTION_STRING: &str = "env-vars";

impl FromStr for ObjectType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            WORKFLOW_OBJECT_STRING => Ok(Self::Workflow),
            PROMPT_OBJECT_STRING => Ok(Self::Workflow),
            FOLDER_OBJECT_STRING => Ok(Self::Folder),
            ENV_VAR_COLLECTION_STRING => Ok(Self::GenericStringObject(
                GenericStringObjectFormat::Json(JsonObjectType::EnvVarCollection),
            )),
            _ => Err(anyhow!("Unexpected object type")),
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Workflow => write!(f, "{WORKFLOW_OBJECT_STRING}"),
            ObjectType::Folder => write!(f, "{FOLDER_OBJECT_STRING}"),
            ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
                JsonObjectType::EnvVarCollection,
            )) => write!(f, "{ENV_VAR_COLLECTION_STRING}"),
            ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
                JsonObjectType::AIFact,
            )) => write!(f, "rule"),
            ObjectType::GenericStringObject(_) => write!(f, "string_object_placeholder"), // placeholder value
        }
    }
}

impl From<ObjectType> for ObjectIdType {
    fn from(value: ObjectType) -> Self {
        match value {
            ObjectType::Workflow => ObjectIdType::Workflow,
            ObjectType::Folder => ObjectIdType::Folder,
            ObjectType::GenericStringObject(_) => ObjectIdType::GenericStringObject,
        }
    }
}

/// The object type prefix for generic string objects.
pub const GENERIC_STRING_OBJECT_PREFIX: &str = "GENERIC_STRING_";

/// The object type prefix for json objects.
pub const JSON_OBJECT_PREFIX: &str = "JSON_";

/// The data format for the generic string object type.
/// Right now we only support json, but this is left
/// open to support markdown, yaml and other text based types.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum GenericStringObjectFormat {
    Json(JsonObjectType),
}

// Temporarily suppress clippy warnings about the `ToString` impl until we
// move `ObjectType` away from using `std::fmt::Display` for serialization.
#[allow(clippy::to_string_trait_impl)]
impl ToString for GenericStringObjectFormat {
    fn to_string(&self) -> String {
        match self {
            GenericStringObjectFormat::Json(json_object_type) => format!(
                "{}{}{}",
                GENERIC_STRING_OBJECT_PREFIX,
                JSON_OBJECT_PREFIX,
                json_object_type.as_str()
            ),
        }
    }
}

/// An object sub-type for objects that implement the JsonModel trait.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum JsonObjectType {
    EnvVarCollection,
    WorkflowEnum,
    AIFact,
    MCPServer,
    TemplatableMCPServer,
}

impl JsonObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            JsonObjectType::EnvVarCollection => "ENVVARCOLLECTION",
            JsonObjectType::WorkflowEnum => "WORKFLOWENUM",
            JsonObjectType::AIFact => "AIFACT",
            JsonObjectType::MCPServer => "MCPSERVER",
            JsonObjectType::TemplatableMCPServer => "TEMPLATABLEMCPSERVER",
        }
    }
}

impl TryFrom<&str> for JsonObjectType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "ENVVARCOLLECTION" => Ok(JsonObjectType::EnvVarCollection),
            "WORKFLOWENUM" => Ok(JsonObjectType::WorkflowEnum),
            "AIFACT" => Ok(JsonObjectType::AIFact),
            "MCPSERVER" => Ok(JsonObjectType::MCPServer),
            "TEMPLATABLEMCPSERVER" => Ok(JsonObjectType::TemplatableMCPServer),
            _ => Err(anyhow!("could not convert unknown json object type")),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct ServerTimestamp(DateTime<Utc>);

impl ServerTimestamp {
    pub fn new(time: DateTime<Utc>) -> Self {
        Self(time)
    }

    pub fn from_unix_timestamp_micros(ms_since_epoch: i64) -> Result<Self> {
        let date_time = DateTime::from_timestamp_micros(ms_since_epoch)
            .ok_or_else(|| anyhow!("Unable to convert microseconds into NaiveDateTime"))?;
        Ok(ServerTimestamp::new(date_time))
    }

    pub fn timestamp_micros(&self) -> i64 {
        self.0.timestamp_micros()
    }

    pub fn utc(&self) -> DateTime<Utc> {
        self.0
    }
}

impl From<DateTime<Utc>> for ServerTimestamp {
    fn from(value: DateTime<Utc>) -> Self {
        ServerTimestamp::new(value)
    }
}

/// The revision timestamp at which an object was edited. This is used by the server
/// to determine if an edit to an object was at the latest revision. Edits at older
/// revisions are rejected by the server.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct Revision(ServerTimestamp);

impl Revision {
    pub fn from_unix_timestamp_micros(ms_since_epoch: i64) -> Result<Self> {
        let ts = ServerTimestamp::from_unix_timestamp_micros(ms_since_epoch)?;
        Ok(Self(ts))
    }

    pub fn timestamp_micros(&self) -> i64 {
        self.0.timestamp_micros()
    }

    pub fn utc(&self) -> DateTime<Utc> {
        self.0.utc()
    }

    /// Returns the inner `ServerTimestamp`.
    pub fn timestamp(&self) -> ServerTimestamp {
        self.0
    }

    #[cfg(any(test, feature = "test-util"))]
    pub fn now() -> Self {
        Self(ServerTimestamp::new(Utc::now()))
    }
}

impl From<Revision> for ServerTimestamp {
    fn from(revision: Revision) -> Self {
        revision.0
    }
}

impl From<ServerTimestamp> for Revision {
    fn from(time: ServerTimestamp) -> Self {
        Revision(time)
    }
}

#[cfg(any(test, feature = "test-util"))]
impl From<DateTime<Utc>> for Revision {
    fn from(time: DateTime<Utc>) -> Self {
        Self(ServerTimestamp::new(time))
    }
}

#[derive(Copy, Clone, Debug, Eq, Serialize, Deserialize, Derivative)]
#[derivative(PartialEq)]
pub enum Owner {
    User { user_uid: UserUid },
}

impl Owner {
    /// A mock [`Owner`] ID for testing.
    #[cfg(any(test, feature = "test-util"))]
    pub fn mock_current_user() -> Owner {
        use crate::identity::TEST_USER_UID;

        Owner::User {
            user_uid: UserUid::new(TEST_USER_UID),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NumInFlightRequests(pub usize);

#[derive(Clone, Debug)]
/// An enum representing what state a local cloud object's content changes can be in,
/// in relation to the server.
pub enum CloudObjectSyncStatus {
    /// The object's content hasn't changed from what we believe the server's representation
    /// to be.
    NoLocalChanges,
    /// The object's content has been modified locally, and is currently in the sync queue
    /// attempting to sync up with the server.
    InFlight(NumInFlightRequests),
    /// The object's content has been modified locally but has unresolved conflict with the server
    /// revision.
    InConflict,
    /// The object's content has been modified locally, but persisting the change on the server
    /// could not complete for some reason.
    Errored,
}

const SYNC_ICON_DIMENSIONS: f32 = 16.;

const SYNC_STATUS_TOOLTIP_INFLIGHT: &str = "Saving";
const SYNC_STATUS_TOOLTIP_ERROR: &str = "Failed to save";

#[derive(Debug, Clone, PartialEq)]
pub struct CloudObjectPermissions {
    pub owner: Owner,
}

impl CloudObjectPermissions {
    /// Mock permissions for a personal object.
    #[cfg(any(test, feature = "test-util"))]
    pub fn mock_personal() -> Self {
        Self {
            owner: Owner::mock_current_user(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CloudObjectMetadata {
    pub revision: Option<Revision>,
    pub metadata_last_updated_ts: Option<ServerTimestamp>,
    pub current_editor_uid: Option<String>,
    pub pending_changes_statuses: CloudObjectStatuses,
    pub trashed_ts: Option<ServerTimestamp>,
    pub folder_id: Option<SyncId>,
    pub is_welcome_object: bool,
    pub last_editor_uid: Option<String>,
    pub creator_uid: Option<String>,
    pub last_task_run_ts: Option<ServerTimestamp>,
}

impl CloudObjectMetadata {
    /// Creates a new set of metadata with reasonable defaults for a test:
    /// * Content and metadata timestamps set to now
    /// * No editor information
    /// * No parent folder
    /// * Not trashed
    #[cfg(any(test, feature = "test-util"))]
    pub fn mock() -> Self {
        Self {
            revision: Some(Revision::now()),
            current_editor_uid: None,
            metadata_last_updated_ts: Some(Utc::now().into()),
            pending_changes_statuses: CloudObjectStatuses::mock(),
            trashed_ts: None,
            folder_id: None,
            is_welcome_object: false,
            last_editor_uid: None,
            creator_uid: None,
            last_task_run_ts: None,
        }
    }

    pub fn has_pending_content_changes(&self) -> bool {
        !matches!(
            self.pending_changes_statuses.content_sync_status,
            CloudObjectSyncStatus::NoLocalChanges | CloudObjectSyncStatus::InConflict
        )
    }

    pub fn is_errored(&self) -> bool {
        matches!(
            self.pending_changes_statuses.content_sync_status,
            CloudObjectSyncStatus::Errored
        )
    }

    /// True iff there are unsynced online-only changes for the object.
    pub fn has_pending_online_only_change(&self) -> bool {
        self.pending_changes_statuses.has_pending_permissions_change
            || self.pending_changes_statuses.has_pending_metadata_change
            || self.pending_changes_statuses.pending_untrash
            || self.pending_changes_statuses.pending_delete
    }

    pub fn set_current_editor(&mut self, editor_uid: Option<String>) {
        self.current_editor_uid = editor_uid;
    }
}

/// A struct holding the different statuses of pending changes that a cloud object might have.
/// Note that content is handled differently than permissions/metadata:
///   * Content changes go through the sync queue, and thus can exist in more states
///   * Metadata/permissions changes are synchronous operations, and thus are only either
///     in flight or synced
#[derive(Clone, Debug)]
pub struct CloudObjectStatuses {
    pub content_sync_status: CloudObjectSyncStatus,
    /// True iff there are unsynced permission changes for the object.
    /// We intentionally don't persist this value in sqlite. And if true,
    /// we don't upsert any in-memory permission changes to sqlite.
    pub has_pending_permissions_change: bool,
    /// True iff there are unsynced metadata changes for the object.
    /// We intentionally don't persist this value in sqlite. And if true,
    /// we don't upsert trashed and folder changes to sqlite.
    pub has_pending_metadata_change: bool,

    /// True iff there is an unsynced untrash operation on the object.
    pub pending_untrash: bool,

    /// True iff there is an unsynced delete operation on the object.
    pub pending_delete: bool,
}

impl CloudObjectStatuses {
    /// Empty statuses with no in-flight changes, for use in tests.
    #[cfg(any(test, feature = "test-util"))]
    pub fn mock() -> Self {
        Self {
            content_sync_status: CloudObjectSyncStatus::NoLocalChanges,
            has_pending_permissions_change: false,
            has_pending_metadata_change: false,
            pending_untrash: false,
            pending_delete: false,
        }
    }

    pub fn render_icon(
        &self,
        hover_state: MouseStateHandle,
        appearance: &Appearance,
    ) -> Option<Box<dyn Element>> {
        let theme = appearance.theme();
        let has_in_flight_requests = match &self.content_sync_status {
            CloudObjectSyncStatus::InFlight(reqs) => reqs.0 > 0,
            _ => false,
        };

        let should_show_syncing_indicator = has_in_flight_requests
            || self.has_pending_metadata_change
            || self.has_pending_permissions_change
            || self.pending_untrash;
        let should_show_error_indicator = matches!(
            self.content_sync_status,
            CloudObjectSyncStatus::Errored | CloudObjectSyncStatus::InConflict
        );

        let icon_and_tooltip_text = if should_show_syncing_indicator {
            Some((
                Icon::Refresh.to_warpui_icon(theme.sub_text_color(theme.surface_2())),
                SYNC_STATUS_TOOLTIP_INFLIGHT,
            ))
        } else if should_show_error_indicator {
            Some((
                Icon::AlertTriangle.to_warpui_icon(Fill::Solid(theme.ui_error_color())),
                SYNC_STATUS_TOOLTIP_ERROR,
            ))
        } else {
            None
        };

        if let Some((icon, tooltip_text)) = icon_and_tooltip_text {
            return Some(
                Align::new(
                    Hoverable::new(hover_state, move |hover_state| {
                        let mut stack = Stack::new().with_child(
                            ConstrainedBox::new(icon.finish())
                                .with_height(SYNC_ICON_DIMENSIONS)
                                .with_width(SYNC_ICON_DIMENSIONS)
                                .finish(),
                        );

                        if hover_state.is_hovered() {
                            let tooltip = appearance
                                .ui_builder()
                                .tool_tip(tooltip_text.to_string())
                                .build()
                                .finish();

                            stack.add_positioned_overlay_child(
                                tooltip,
                                OffsetPositioning::offset_from_parent(
                                    vec2f(0., -24.),
                                    ParentOffsetBounds::Unbounded,
                                    ParentAnchor::Center,
                                    ChildAnchor::Center,
                                ),
                            );
                        }

                        stack.finish()
                    })
                    .finish(),
                )
                .finish(),
            );
        }

        None
    }
}

#[derive(Copy, Default, Clone, Debug, Eq, PartialEq)]
pub enum CloudObjectEventEntrypoint {
    #[default]
    Unknown,
}
