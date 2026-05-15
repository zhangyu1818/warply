use serde::{Deserialize, Serialize};
use warp_core::context_flag::ContextFlag;
use warpui::AppContext;

pub mod categories;
use workflow::Workflow;

pub mod aliases;
pub mod command_parser;
pub mod export_workflow;
pub mod info_box;
pub mod local_workflows;
pub mod manager;
pub mod workflow;
pub mod workflow_enum;
pub mod workflow_view;

use crate::appearance::Appearance;
use crate::cloud_object::{CloudModelType, GenericCloudObject, ObjectType, SerializedModel};

use crate::drive::items::workflow::LocalObjectWorkflow;
use crate::drive::items::LocalObjectItem;
use crate::drive::CloudObjectTypeAndId;
use crate::notebooks::NotebookLocation;
use crate::object_ids::{ServerId, SyncId};
use crate::persistence::ModelEvent;
pub use categories::{CategoriesView, CategoriesViewEvent, WorkflowsViewAction};

pub fn init(app: &mut AppContext) {
    categories::init(app);
    self::workflow_view::init(app);
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum WorkflowSource {
    Global,
    Local,
    Project,
    Saved,
    Agent,
    Notebook {
        location: NotebookLocation,
    },

    /// A hardcoded workflow type that allows Warp to surface features as Workflows (e.g.
    /// a command to see our network log)
    App,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash, PartialOrd)]
pub enum WorkflowSelectionSource {
    LocalObject,
    CommandPalette,
    UniversalSearch,
    Voltron,
    Agent,
    Notebook,
    SlashMenu,
    UpArrowHistory,
    WorkflowView,
    AgentMode,
    Undefined,
    Alias,
}

#[derive(Debug, Clone, Copy)]
pub enum WorkflowViewMode {
    View,
    Edit,
    Create,
}

impl WorkflowViewMode {
    /// The editing mode supported for a workflow.
    ///
    /// Editing is disabled if the user does not have edit permissions.
    pub fn supported_edit_mode(_workflow_id: Option<SyncId>, _app: &AppContext) -> Self {
        Self::Edit
    }

    /// The viewing mode supported for this workflow.
    ///
    /// Viewing is disabled if the user is allowed to edit the workflow and in a context where
    /// running workflows is supported.
    pub fn supported_view_mode(_workflow_id: Option<SyncId>, _app: &AppContext) -> Self {
        if ContextFlag::RunWorkflow.is_enabled() {
            Self::Edit
        } else {
            Self::View
        }
    }

    fn is_editable(&self) -> bool {
        match self {
            Self::View => false,
            Self::Edit | Self::Create => true,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct WorkflowId(ServerId);
crate::server_id_traits! { WorkflowId, "Workflow" }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AIWorkflowOrigin {
    CommandSearch,
    AgentMode,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorkflowType {
    Local(Workflow),
    Saved(Box<SavedWorkflow>),
    AIGenerated {
        workflow: Workflow,
        origin: AIWorkflowOrigin,
    },
    Notebook(Workflow),
}

impl WorkflowType {
    pub fn as_workflow(&self) -> &Workflow {
        match self {
            WorkflowType::Local(workflow) => workflow,
            WorkflowType::AIGenerated { workflow, .. } => workflow,
            WorkflowType::Saved(workflow) => &workflow.model().data,
            WorkflowType::Notebook(workflow) => workflow,
        }
    }

    /// Returns the contained [`Workflow`], consuming `self`.
    pub fn take_workflow(self) -> Workflow {
        match self {
            WorkflowType::Local(workflow) => workflow,
            WorkflowType::AIGenerated { workflow, .. } => workflow,
            WorkflowType::Saved(workflow) => workflow.model().data.clone(),
            WorkflowType::Notebook(workflow) => workflow,
        }
    }

    pub fn object_id(&self) -> Option<CloudObjectTypeAndId> {
        match self {
            WorkflowType::Saved(workflow) => Some(CloudObjectTypeAndId::Workflow(workflow.id)),
            _ => None,
        }
    }

    pub fn sync_id(&self) -> Option<SyncId> {
        match self {
            WorkflowType::Saved(workflow) => Some(workflow.id),
            _ => None,
        }
    }

    /// We don't show env var selection for Agent Mode suggested commands.
    pub(super) fn should_show_env_var_selection(&self) -> bool {
        !matches!(self, WorkflowType::AIGenerated { .. },)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedWorkflowModel {
    pub data: Workflow,
}

impl SavedWorkflowModel {
    pub fn new(workflow: Workflow) -> Self {
        Self { data: workflow }
    }
}

pub type SavedWorkflow = GenericCloudObject<WorkflowId, SavedWorkflowModel>;

impl CloudModelType for SavedWorkflowModel {
    type CloudObjectType = SavedWorkflow;
    type IdType = WorkflowId;

    fn model_type_name(&self) -> &'static str {
        if self.data.is_agent_mode_workflow() {
            "Prompt"
        } else {
            "Workflow"
        }
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Workflow
    }

    fn cloud_object_type_and_id(&self, id: SyncId) -> CloudObjectTypeAndId {
        CloudObjectTypeAndId::Workflow(id)
    }

    fn display_name(&self) -> String {
        self.data.name().to_string()
    }

    fn set_display_name(&mut self, name: &str) {
        self.data.set_name(name);
    }

    fn upsert_event(&self, workflow: &SavedWorkflow) -> ModelEvent {
        ModelEvent::UpsertWorkflow {
            workflow: workflow.clone(),
        }
    }

    fn bulk_upsert_event(objects: &[SavedWorkflow]) -> ModelEvent {
        ModelEvent::UpsertWorkflows(objects.to_vec())
    }

    fn serialized(&self) -> SerializedModel {
        SerializedModel::new(
            serde_json::to_string(&self.data).expect("failed to serialize workflow"),
        )
    }

    fn renders_as_local_object(&self) -> bool {
        true
    }

    fn to_local_object_item(
        &self,
        id: SyncId,
        _appearance: &Appearance,
        workflow: &SavedWorkflow,
    ) -> Option<Box<dyn LocalObjectItem>> {
        Some(Box::new(LocalObjectWorkflow::new(
            self.cloud_object_type_and_id(id),
            workflow.clone(),
        )))
    }

    fn can_export(&self) -> bool {
        true
    }
}

impl PartialEq<Workflow> for SavedWorkflow {
    fn eq(&self, other: &Workflow) -> bool {
        self.model().data == *other
    }
}

impl PartialEq<SavedWorkflow> for SavedWorkflow {
    fn eq(&self, other: &SavedWorkflow) -> bool {
        self.model().data == other.model().data && self.id == other.id
    }
}

impl From<SavedWorkflow> for Workflow {
    fn from(saved_workflow: SavedWorkflow) -> Self {
        saved_workflow.model().data.clone()
    }
}

impl From<&SavedWorkflow> for Workflow {
    fn from(saved_workflow: &SavedWorkflow) -> Self {
        saved_workflow.model().data.to_owned()
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
