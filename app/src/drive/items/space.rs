use warpui::{elements::MouseStateHandle, Element};

use crate::{
    appearance::Appearance,
    cloud_object::{CloudObjectMetadata, Space},
    drive::DriveObjectType,
    themes::theme::Fill,
};

use super::{LocalObjectItem, LocalObjectItemId};

#[derive(Clone)]
pub struct LocalObjectSpace {
    space: Space,
}

impl LocalObjectSpace {
    #[allow(dead_code)]
    pub fn new(space: Space) -> Self {
        Self { space }
    }
}

impl LocalObjectItem for LocalObjectSpace {
    fn display_name(&self) -> Option<String> {
        None
    }

    fn metadata(&self) -> Option<&CloudObjectMetadata> {
        None
    }

    fn object_type(&self) -> Option<DriveObjectType> {
        None
    }

    fn secondary_icon(&self, _color: Option<Fill>) -> Option<Box<dyn Element>> {
        None
    }

    fn preview(&self, _appearance: &Appearance) -> Option<Box<dyn Element>> {
        None
    }

    fn local_object_id(&self) -> LocalObjectItemId {
        LocalObjectItemId::Space(self.space)
    }

    fn sync_status_icon(
        &self,
        _hover_state: MouseStateHandle,
        _appearance: &Appearance,
    ) -> Option<Box<dyn Element>> {
        None
    }

    fn clone_box(&self) -> Box<dyn LocalObjectItem> {
        Box::new(self.clone())
    }

    fn action_summary(&self, _app: &warpui::AppContext) -> Option<String> {
        None
    }
}
