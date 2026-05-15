mod assertion;

pub use assertion::*;
use itertools::Itertools;
use std::future::Future;
use std::pin::Pin;
use warpui::{App, SingletonEntity};

use crate::{
    cloud_object::update_manager::UpdateManager,
    cloud_object::{model::persistence::CloudModel, Space},
};

/// Clears the local object model of all non-welcome objects in the user's personal space.
/// Returns a future that resolves when the local object model is cleared.
pub fn clear_cloud_model(app: &mut App) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    let object_ids_to_delete = CloudModel::handle(app).read(app, |cloud_model, ctx| {
        cloud_model
            .active_non_welcome_cloud_objects_in_space(Space::Personal, ctx)
            .map(|object| object.cloud_object_type_and_id())
            .collect_vec()
    });

    for object_id in object_ids_to_delete {
        UpdateManager::handle(app).update(app, |update_manager, ctx| {
            update_manager.delete_object_by_user(object_id, ctx);
        });
    }

    Box::pin(async {})
}
