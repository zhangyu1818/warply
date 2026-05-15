use warpui::{AppContext, Element};

use crate::{
    appearance::Appearance,
    cloud_object::{CloudObjectMetadata, Space},
    themes::theme::Fill,
    ui_components::icons::Icon,
};

use super::{cloud_object_styling::local_object_icon_color, CloudObjectTypeAndId, DriveObjectType};

pub mod ai_fact;
pub mod env_var_collection;
pub mod folder;
pub mod space;
pub mod workflow;

pub trait LocalObjectItem {
    fn display_name(&self) -> Option<String>;
    fn metadata(&self) -> Option<&CloudObjectMetadata>;
    fn object_type(&self) -> Option<DriveObjectType>;
    fn secondary_icon(&self, color: Option<Fill>) -> Option<Box<dyn Element>>;
    fn preview(&self, appearance: &Appearance) -> Option<Box<dyn Element>>;
    fn local_object_id(&self) -> LocalObjectItemId;

    fn icon(&self, appearance: &Appearance, color: Option<Fill>) -> Option<Box<dyn Element>> {
        let object_type = self.object_type()?;
        let icon_fill = color.unwrap_or(local_object_icon_color(appearance, object_type).into());
        Some(Icon::from(object_type).to_warpui_icon(icon_fill).finish())
    }

    fn action_summary(&self, app: &AppContext) -> Option<String>;

    fn is_folder_open(&self) -> Option<bool> {
        None
    }

    fn clone_box(&self) -> Box<dyn LocalObjectItem>;
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum LocalObjectItemId {
    AIFactCollection,
    Object(CloudObjectTypeAndId),
    Space(Space),
    Trash,
}

impl Clone for Box<dyn LocalObjectItem> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
