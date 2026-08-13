use warpui::{AppContext, Element};

use crate::{
    appearance::Appearance,
    cloud_object::CloudObjectMetadata,
    drive::{
        CloudObjectTypeAndId, DriveObjectType, cloud_object_styling::local_object_icon_color,
        folders::CloudFolder,
    },
    themes::theme::Fill,
    ui_components::icons::Icon,
};

use super::{LocalObjectItem, LocalObjectItemId};

#[derive(Clone)]
pub struct LocalObjectFolder {
    id: CloudObjectTypeAndId,
    folder: CloudFolder,
}

impl LocalObjectFolder {
    pub fn new(id: CloudObjectTypeAndId, folder: CloudFolder) -> Self {
        Self { id, folder }
    }
}

impl LocalObjectItem for LocalObjectFolder {
    fn display_name(&self) -> Option<String> {
        if self.folder.model().name.is_empty() {
            None
        } else {
            Some(self.folder.model().name.clone())
        }
    }

    fn metadata(&self) -> Option<&CloudObjectMetadata> {
        Some(&self.folder.metadata)
    }

    fn object_type(&self) -> Option<DriveObjectType> {
        Some(DriveObjectType::Folder)
    }

    fn icon(&self, appearance: &Appearance, color: Option<Fill>) -> Option<Box<dyn Element>> {
        let icon_fill =
            color.unwrap_or(local_object_icon_color(appearance, DriveObjectType::Folder).into());
        Some(
            Icon::from(DriveObjectType::Folder)
                .to_warpui_icon(icon_fill)
                .finish(),
        )
    }

    fn secondary_icon(&self, _color: Option<Fill>) -> Option<Box<dyn Element>> {
        None
    }

    fn is_folder_open(&self) -> Option<bool> {
        Some(self.folder.model().is_open)
    }

    fn preview(&self, _: &Appearance) -> Option<Box<dyn Element>> {
        None
    }

    fn local_object_id(&self) -> LocalObjectItemId {
        LocalObjectItemId::Object(self.id)
    }

    fn action_summary(&self, _app: &AppContext) -> Option<String> {
        None
    }

    fn clone_box(&self) -> Box<dyn LocalObjectItem> {
        Box::new(self.clone())
    }
}
