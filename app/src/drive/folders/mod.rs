use super::items::folder::LocalObjectFolder;
use super::items::LocalObjectItem;
use super::CloudObjectTypeAndId;
use crate::{
    appearance::Appearance,
    cloud_object::{CloudModelType, GenericCloudObject, ObjectType, SerializedModel, Space},
    object_ids::SyncId,
    persistence::ModelEvent,
};

pub use local_object_model::ids::FolderId;

/// The model for a `CloudFolder`.
#[derive(Clone, Debug, PartialEq)]
pub struct CloudFolderModel {
    pub name: String,
    pub is_open: bool,
    pub is_warp_pack: bool,
}

impl CloudFolderModel {
    pub fn new(name: &str, is_warp_pack: bool) -> Self {
        Self {
            name: name.to_owned(),
            is_open: false,
            is_warp_pack,
        }
    }
}

pub type CloudFolder = GenericCloudObject<FolderId, CloudFolderModel>;

impl CloudModelType for CloudFolderModel {
    type CloudObjectType = CloudFolder;
    type IdType = FolderId;

    fn model_type_name(&self) -> &'static str {
        "Folder"
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Folder
    }

    fn cloud_object_type_and_id(&self, id: SyncId) -> CloudObjectTypeAndId {
        CloudObjectTypeAndId::Folder(id)
    }

    fn display_name(&self) -> String {
        self.name.clone()
    }

    fn upsert_event(&self, folder: &CloudFolder) -> ModelEvent {
        ModelEvent::UpsertFolder {
            folder: folder.clone(),
        }
    }

    fn bulk_upsert_event(objects: &[CloudFolder]) -> ModelEvent {
        ModelEvent::UpsertFolders(objects.to_vec())
    }

    fn serialized(&self) -> SerializedModel {
        SerializedModel::new(self.name.to_owned())
    }

    fn can_move_to_space(&self, current_space: Space, new_space: Space) -> bool {
        // We don't currently support moving folders across spaces.
        current_space == new_space
    }

    fn supports_linking(&self) -> bool {
        true
    }

    fn renders_as_local_object(&self) -> bool {
        true
    }

    fn to_local_object_item(
        &self,
        id: SyncId,
        _appearance: &Appearance,
        folder: &CloudFolder,
    ) -> Option<Box<dyn LocalObjectItem>> {
        Some(Box::new(LocalObjectFolder::new(
            self.cloud_object_type_and_id(id),
            folder.clone(),
        )))
    }
}
