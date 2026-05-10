use super::{LocalObjectItem, LocalObjectItemId};
use crate::{
    ai::mcp::CloudMCPServer,
    appearance::Appearance,
    cloud_object::CloudObjectMetadata,
    drive::{CloudObjectTypeAndId, DriveObjectType},
    themes::theme::Fill,
};
use warpui::{elements::MouseStateHandle, AppContext, Element};

#[derive(Clone)]
pub struct LocalObjectMCPServer {
    id: CloudObjectTypeAndId,
    mcp_server: CloudMCPServer,
}

impl LocalObjectMCPServer {
    pub fn new(id: CloudObjectTypeAndId, mcp_server: CloudMCPServer) -> Self {
        Self { id, mcp_server }
    }
}

impl LocalObjectItem for LocalObjectMCPServer {
    fn display_name(&self) -> Option<String> {
        Some(self.mcp_server.model().string_model.name.clone())
    }
    fn metadata(&self) -> Option<&CloudObjectMetadata> {
        Some(&self.mcp_server.metadata)
    }

    fn object_type(&self) -> Option<DriveObjectType> {
        Some(DriveObjectType::MCPServer)
    }

    fn secondary_icon(&self, _color: Option<Fill>) -> Option<Box<dyn Element>> {
        None
    }

    fn preview(&self, _appearance: &Appearance) -> Option<Box<dyn Element>> {
        // TODO
        None
    }

    fn local_object_id(&self) -> LocalObjectItemId {
        LocalObjectItemId::Object(self.id)
    }

    fn sync_status_icon(
        &self,
        hover_state: MouseStateHandle,
        appearance: &Appearance,
    ) -> Option<Box<dyn Element>> {
        self.mcp_server
            .metadata
            .pending_changes_statuses
            .render_icon(hover_state, appearance)
    }

    fn action_summary(&self, _app: &AppContext) -> Option<String> {
        None
    }

    fn clone_box(&self) -> Box<dyn LocalObjectItem> {
        Box::new(self.clone())
    }
}
