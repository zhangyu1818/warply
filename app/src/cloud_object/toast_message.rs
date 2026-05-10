use warpui::AppContext;

use crate::cloud_object::update_manager::{InitiatedBy, ObjectOperation};

use super::CloudObject;

pub struct CloudObjectToastMessage;

impl CloudObjectToastMessage {
    pub fn toast_message(
        object: &dyn CloudObject,
        operation: &ObjectOperation,
        app: &AppContext,
    ) -> Option<String> {
        let object_name = object.model_type_name().to_owned();
        match (object.object_type(), operation) {
            (
                _,
                ObjectOperation::Create {
                    initiated_by: InitiatedBy::User,
                },
            ) => {
                let containing_object_name = object.containing_object_name(app);
                Some(format!("{object_name} saved to {containing_object_name}"))
            }
            (_, ObjectOperation::Update) => Some(format!("{object_name} updated")),
            (_, ObjectOperation::Trash) => Some(format!("{object_name} trashed")),
            (_, ObjectOperation::Untrash) => Some(format!("{object_name} restored")),
            _ => None,
        }
    }

    pub fn toast_deletion_confirm_message(
        num_objects: i32,
        operation: &ObjectOperation,
    ) -> Option<String> {
        let count_objects_message = match num_objects {
            1 => "1 object".to_string(),
            n => {
                format!("{n} objects")
            }
        };
        match operation {
            ObjectOperation::Delete {
                initiated_by: InitiatedBy::User,
            } => Some(format!("{count_objects_message} deleted forever")),
            _ => None,
        }
    }
}
