use crate::{
    cloud_object::update_manager::UpdateManager,
    cloud_object::{
        model::persistence::CloudModel, CloudObjectEventEntrypoint, CloudObjectLocation, Space,
    },
    object_ids::ClientId,
    util::bindings::keybinding_name_to_display_string,
    workflows::workflow::Workflow,
    workspaces::user_workspaces::UserWorkspaces,
};
use warpui::{async_assert, async_assert_eq, integration::TestStep, SingletonEntity};

pub fn create_a_personal_workflow() -> TestStep {
    TestStep::new("Create a personal workflow")
        .with_action(move |app, _, _| {
            UpdateManager::handle(app).update(app, |update_manager, ctx| {
                update_manager.create_workflow(
                    Workflow::new("My first workflow", "ls"),
                    UserWorkspaces::as_ref(ctx).current_user_owner(ctx),
                    None,
                    ClientId::default(),
                    CloudObjectEventEntrypoint::Unknown,
                    true,
                    ctx,
                )
            })
        })
        .add_assertion(move |app, _| {
            CloudModel::handle(app).read(app, |cloud_model, ctx| {
                async_assert!(
                    cloud_model
                        .active_cloud_objects_in_location_without_descendents(
                            CloudObjectLocation::Space(Space::Personal),
                            ctx,
                        )
                        .count()
                        > 0,
                    "local objects exist"
                )
            })
        })
}

pub fn assert_binding_display_string(
    binding: &'static str,
    display_string: Option<&'static str>,
) -> TestStep {
    TestStep::new("Assert a binding's display string").add_named_assertion(
        format!("Binding {binding} should have display string {display_string:?}"),
        move |app, _| {
            app.update(|ctx| {
                async_assert_eq!(
                    keybinding_name_to_display_string(binding, ctx).as_deref(),
                    display_string
                )
            })
        },
    )
}
