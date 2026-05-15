use warpui::{App, ModelHandle};

use crate::cloud_object::model::actions::ObjectActions;
use crate::cloud_object::CloudObjectMetadata;
use crate::cloud_object::CloudObjectPermissions;
use crate::cloud_object::CloudObjectStatuses;
use crate::cloud_object::CloudObjectSyncStatus;
use crate::cloud_object::Owner;
use crate::drive::folders::CloudFolderModel;
use crate::drive::DriveIndexVariant;
use crate::http_api::HttpApiProvider;
use crate::identity::LocalIdentityProvider;
use crate::system::SystemStats;

use crate::cloud_object::update_manager::UpdateManager;
use crate::workflows::workflow::Workflow;
use crate::workflows::SavedWorkflowModel;

use super::*;

fn create_cloud_model(
    app: &mut App,
    objects: Vec<Box<dyn CloudObject>>,
) -> ModelHandle<CloudModel> {
    // Make sure to register the CloudModel singleton - some CloudObject methods
    // find it and other dependencies via the AppContext.
    app.add_singleton_model(|_ctx| CloudModel::new(None, objects))
}

fn initialize_app(app: &mut App, cached_objects: Vec<Box<dyn CloudObject>>) {
    // Add the necessary singleton models to the App
    app.add_singleton_model(|_| SystemStats::new());
    app.add_singleton_model(|_| HttpApiProvider::new_for_test());
    app.add_singleton_model(|_| LocalIdentityProvider::new_for_test());
    app.add_singleton_model(|_ctx| CloudModel::new(None, cached_objects));
    app.add_singleton_model(|ctx| UpdateManager::new(None, ctx));
    app.add_singleton_model(|_| ObjectActions::new(Vec::new()));
}

fn mock_permissions() -> CloudObjectPermissions {
    CloudObjectPermissions {
        owner: Owner::mock_current_user(),
    }
}

fn mock_cloud_folder(id: SyncId, name: String, folder_id: Option<SyncId>) -> CloudFolder {
    CloudFolder::new(
        id,
        CloudFolderModel {
            name,
            is_open: true,
        },
        CloudObjectMetadata {
            pending_changes_statuses: CloudObjectStatuses {
                content_sync_status: CloudObjectSyncStatus::NoLocalChanges,
            },
            folder_id,
            revision: Default::default(),
            metadata_last_updated_ts: Default::default(),
            current_editor_uid: Default::default(),
            trashed_ts: Default::default(),
            is_welcome_object: false,
            creator_uid: None,
            last_editor_uid: None,
            last_task_run_ts: None,
        },
        mock_permissions(),
    )
}

fn mock_saved_workflow(id: SyncId, title: String, folder_id: Option<SyncId>) -> SavedWorkflow {
    SavedWorkflow::new(
        id,
        SavedWorkflowModel::new(Workflow::new(title, "test")),
        CloudObjectMetadata {
            pending_changes_statuses: CloudObjectStatuses {
                content_sync_status: CloudObjectSyncStatus::NoLocalChanges,
            },
            folder_id,
            revision: Default::default(),
            metadata_last_updated_ts: Default::default(),
            current_editor_uid: Default::default(),
            trashed_ts: Default::default(),
            is_welcome_object: false,
            creator_uid: None,
            last_editor_uid: None,
            last_task_run_ts: None,
        },
        mock_permissions(),
    )
}

fn mock_trashed_cloud_folder(id: SyncId, name: String, folder_id: Option<SyncId>) -> CloudFolder {
    let mut folder = mock_cloud_folder(id, name, folder_id);
    folder.metadata.trashed_ts = Some(ServerTimestamp::from_unix_timestamp_micros(10).unwrap());
    folder
}

fn folder_from_model(model: &CloudModel, id: SyncId) -> &CloudFolder {
    model.get_folder_by_uid(&id.uid()).expect("is a folder")
}

#[test]
fn test_collapse_all_in_location() {
    /*
       the folder structure looks like:

       test1
        ↳ test 4
         ↳ test 5
       test 2
        ↳ test 6
         ↳ test 7
       test 3

    */
    let folder_1_id: SyncId = SyncId::ServerId(1.into());
    let folder_2_id: SyncId = SyncId::ServerId(2.into());
    let folder_3_id: SyncId = SyncId::ServerId(3.into());
    let folder_4_id: SyncId = SyncId::ServerId(4.into());
    let folder_5_id: SyncId = SyncId::ServerId(5.into());
    let folder_6_id: SyncId = SyncId::ServerId(6.into());
    let folder_7_id: SyncId = SyncId::ServerId(7.into());

    let folders = vec![
        mock_cloud_folder(folder_1_id, "test1".to_string(), None),
        mock_cloud_folder(folder_2_id, "test2".to_string(), None),
        mock_cloud_folder(folder_3_id, "test3".to_string(), None),
        mock_cloud_folder(folder_4_id, "test4".to_string(), Some(folder_1_id)),
        mock_cloud_folder(folder_5_id, "test5".to_string(), Some(folder_4_id)),
        mock_cloud_folder(folder_6_id, "test6".to_string(), Some(folder_2_id)),
        mock_cloud_folder(folder_7_id, "test7".to_string(), Some(folder_6_id)),
    ]
    .into_iter()
    .map(|o| Box::new(o) as Box<dyn CloudObject>)
    .collect();

    App::test((), |mut app| async move {
        app.add_singleton_model(|_| LocalIdentityProvider::new_for_test());
        let cloud_model = create_cloud_model(&mut app, folders);

        cloud_model.update(&mut app, |model, ctx| {
            // first, collapse all folders in folder 1
            model.collapse_all_in_location(
                CloudObjectLocation::Folder(folder_1_id),
                DriveIndexVariant::MainIndex,
                ctx,
            );

            // folders 1, 4, and 5 should be collapsed
            let folder_1 = folder_from_model(model, folder_1_id);
            let folder_4 = folder_from_model(model, folder_4_id);
            let folder_5 = folder_from_model(model, folder_5_id);
            assert!(!folder_1.model.is_open);
            assert!(!folder_4.model.is_open);
            assert!(!folder_5.model.is_open);
            // but the others are still open
            let folder_2 = folder_from_model(model, folder_2_id);
            let folder_3 = folder_from_model(model, folder_3_id);
            let folder_6 = folder_from_model(model, folder_6_id);
            let folder_7 = folder_from_model(model, folder_7_id);
            assert!(folder_2.model.is_open);
            assert!(folder_3.model.is_open);
            assert!(folder_6.model.is_open);
            assert!(folder_7.model.is_open);

            model.collapse_all_in_location(
                CloudObjectLocation::Space(Default::default()),
                DriveIndexVariant::MainIndex,
                ctx,
            );
            // now all folders in this space are collapsed
            let folder_1 = folder_from_model(model, folder_1_id);
            let folder_2 = folder_from_model(model, folder_2_id);
            let folder_3 = folder_from_model(model, folder_3_id);
            let folder_4 = folder_from_model(model, folder_4_id);
            let folder_5 = folder_from_model(model, folder_5_id);
            let folder_6 = folder_from_model(model, folder_6_id);
            let folder_7 = folder_from_model(model, folder_7_id);
            assert!(!folder_1.model.is_open);
            assert!(!folder_2.model.is_open);
            assert!(!folder_3.model.is_open);
            assert!(!folder_4.model.is_open);
            assert!(!folder_5.model.is_open);
            assert!(!folder_6.model.is_open);
            assert!(!folder_7.model.is_open);
        });
    })
}

#[test]
fn test_collapse_all_in_trash() {
    /*
       the folder structure looks like:

       test1 -- trashed by user
        ↳ test 4
         ↳ test 5 -- trashed by user
       test 2 -- trashed by user
        ↳ test 6
         ↳ test 7
       test 3 -- trashed by user

       the structure in the trash index looks like:

       test1 -- trashed by user
        ↳ test 4
       test 5 -- trashed by user
       test 2 -- trashed by user
        ↳ test 6
         ↳ test 7
       test 3 -- trashed by user

    */
    let folder_1_id: SyncId = SyncId::ServerId(1.into());
    let folder_2_id: SyncId = SyncId::ServerId(2.into());
    let folder_3_id: SyncId = SyncId::ServerId(3.into());
    let folder_4_id: SyncId = SyncId::ServerId(4.into());
    let folder_5_id: SyncId = SyncId::ServerId(5.into());
    let folder_6_id: SyncId = SyncId::ServerId(6.into());
    let folder_7_id: SyncId = SyncId::ServerId(7.into());

    let folders = vec![
        mock_trashed_cloud_folder(folder_1_id, "test1".to_string(), None),
        mock_trashed_cloud_folder(folder_2_id, "test2".to_string(), None),
        mock_trashed_cloud_folder(folder_3_id, "test3".to_string(), None),
        mock_cloud_folder(folder_4_id, "test4".to_string(), Some(folder_1_id)),
        mock_trashed_cloud_folder(folder_5_id, "test5".to_string(), Some(folder_4_id)),
        mock_cloud_folder(folder_6_id, "test6".to_string(), Some(folder_2_id)),
        mock_cloud_folder(folder_7_id, "test7".to_string(), Some(folder_6_id)),
    ]
    .into_iter()
    .map(|o| Box::new(o) as Box<dyn CloudObject>)
    .collect();

    App::test((), |mut app| async move {
        app.add_singleton_model(|_| LocalIdentityProvider::new_for_test());
        let cloud_model = create_cloud_model(&mut app, folders);

        cloud_model.update(&mut app, |model, ctx| {
            // first, collapse all folders in folder 1
            model.collapse_all_in_location(
                CloudObjectLocation::Folder(folder_1_id),
                DriveIndexVariant::Trash,
                ctx,
            );

            // folders 1, 4 should be collapsed
            let folder_1 = folder_from_model(model, folder_1_id);
            let folder_4 = folder_from_model(model, folder_4_id);
            assert!(!folder_1.model.is_open);
            assert!(!folder_4.model.is_open);
            // but the others, including folder 5, are still open
            let folder_2 = folder_from_model(model, folder_2_id);
            let folder_3 = folder_from_model(model, folder_3_id);
            let folder_5 = folder_from_model(model, folder_5_id);
            let folder_6 = folder_from_model(model, folder_6_id);
            let folder_7 = folder_from_model(model, folder_7_id);
            assert!(folder_2.model.is_open);
            assert!(folder_3.model.is_open);
            assert!(folder_5.model.is_open);
            assert!(folder_6.model.is_open);
            assert!(folder_7.model.is_open);

            model.collapse_all_in_location(
                CloudObjectLocation::Space(Default::default()),
                DriveIndexVariant::Trash,
                ctx,
            );
            // now all folders in this space are collapsed
            let folder_1 = folder_from_model(model, folder_1_id);
            let folder_2 = folder_from_model(model, folder_2_id);
            let folder_3 = folder_from_model(model, folder_3_id);
            let folder_4 = folder_from_model(model, folder_4_id);
            let folder_5 = folder_from_model(model, folder_5_id);
            let folder_6 = folder_from_model(model, folder_6_id);
            let folder_7 = folder_from_model(model, folder_7_id);
            assert!(!folder_1.model.is_open);
            assert!(!folder_2.model.is_open);
            assert!(!folder_3.model.is_open);
            assert!(!folder_4.model.is_open);
            assert!(!folder_5.model.is_open);
            assert!(!folder_6.model.is_open);
            assert!(!folder_7.model.is_open);
        });
    })
}

#[test]
fn test_breadcrumbs() {
    let folder_1_id: SyncId = SyncId::ServerId(1.into());
    let folder_2_id: SyncId = SyncId::ServerId(2.into());
    let folder_3_id: SyncId = SyncId::ServerId(3.into());

    let folders = vec![
        mock_cloud_folder(folder_1_id, "test1".to_string(), None),
        mock_cloud_folder(folder_2_id, "test2".to_string(), Some(folder_1_id)),
        mock_cloud_folder(folder_3_id, "test3".to_string(), Some(folder_2_id)),
    ]
    .into_iter()
    .map(|f| Box::new(f) as Box<dyn CloudObject>)
    .collect::<Vec<_>>();

    App::test((), |mut app| async move {
        initialize_app(&mut app, folders.clone());

        CloudModel::handle(&app).read(&app, |_, ctx| {
            assert_eq!("Personal".to_string(), folders[0].breadcrumbs(ctx));
            assert_eq!("Personal / test1".to_string(), folders[1].breadcrumbs(ctx));
            assert_eq!(
                "Personal / test1 / test2".to_string(),
                folders[2].breadcrumbs(ctx)
            );
        });
    });
}

/// Helper: compute active UIDs using the naive (non-memoized) is_trashed approach.
fn naive_active_object_uids(model: &CloudModel) -> HashSet<String> {
    model
        .as_cloud_objects()
        .filter(|obj| !obj.is_trashed(model))
        .map(|obj| obj.uid())
        .collect()
}

#[test]
fn active_object_uids_matches_naive_with_no_trashed_objects() {
    let folder_id = SyncId::ServerId(1.into());
    let objects: Vec<Box<dyn CloudObject>> = vec![
        Box::new(mock_cloud_folder(folder_id, "Folder".into(), None)),
        Box::new(mock_saved_workflow(
            SyncId::ServerId(2.into()),
            "Workflow".into(),
            Some(folder_id),
        )),
    ];

    App::test((), |mut app| async move {
        let cloud_model = create_cloud_model(&mut app, objects);
        cloud_model.read(&app, |model, _| {
            assert_eq!(model.active_object_uids(), naive_active_object_uids(model));
            assert_eq!(model.active_object_uids().len(), 2);
        });
    });
}

#[test]
fn active_object_uids_matches_naive_with_directly_trashed_object() {
    let trashed_folder_id = SyncId::ServerId(1.into());
    let active_workflow_id = SyncId::ServerId(2.into());
    let objects: Vec<Box<dyn CloudObject>> = vec![
        Box::new(mock_trashed_cloud_folder(
            trashed_folder_id,
            "Trashed Folder".into(),
            None,
        )),
        Box::new(mock_saved_workflow(
            active_workflow_id,
            "Active Workflow".into(),
            None,
        )),
    ];

    App::test((), |mut app| async move {
        let cloud_model = create_cloud_model(&mut app, objects);
        cloud_model.read(&app, |model, _| {
            let active = model.active_object_uids();
            assert_eq!(active, naive_active_object_uids(model));
            assert_eq!(active.len(), 1);
            assert!(active.contains(&active_workflow_id.uid()));
            assert!(!active.contains(&trashed_folder_id.uid()));
        });
    });
}

#[test]
fn active_object_uids_matches_naive_with_indirectly_trashed_children() {
    let trashed_folder_id = SyncId::ServerId(1.into());
    let child_workflow_id = SyncId::ServerId(2.into());
    let active_workflow_id = SyncId::ServerId(3.into());
    let objects: Vec<Box<dyn CloudObject>> = vec![
        Box::new(mock_trashed_cloud_folder(
            trashed_folder_id,
            "Trashed Folder".into(),
            None,
        )),
        Box::new(mock_saved_workflow(
            child_workflow_id,
            "Child in Trashed Folder".into(),
            Some(trashed_folder_id),
        )),
        Box::new(mock_saved_workflow(
            active_workflow_id,
            "Top-level Notebook".into(),
            None,
        )),
    ];

    App::test((), |mut app| async move {
        let cloud_model = create_cloud_model(&mut app, objects);
        cloud_model.read(&app, |model, _| {
            let active = model.active_object_uids();
            assert_eq!(active, naive_active_object_uids(model));
            assert_eq!(active.len(), 1);
            assert!(active.contains(&active_workflow_id.uid()));
        });
    });
}

#[test]
fn active_object_uids_matches_naive_with_nested_trashed_folder() {
    let folder_a_id = SyncId::ServerId(1.into());
    let folder_b_id = SyncId::ServerId(2.into());
    let workflow_id = SyncId::ServerId(3.into());
    let active_workflow_id = SyncId::ServerId(4.into());
    let objects: Vec<Box<dyn CloudObject>> = vec![
        Box::new(mock_trashed_cloud_folder(
            folder_a_id,
            "Folder A (trashed)".into(),
            None,
        )),
        Box::new(mock_cloud_folder(
            folder_b_id,
            "Folder B".into(),
            Some(folder_a_id),
        )),
        Box::new(mock_saved_workflow(
            workflow_id,
            "Deeply nested".into(),
            Some(folder_b_id),
        )),
        Box::new(mock_saved_workflow(
            active_workflow_id,
            "Active".into(),
            None,
        )),
    ];

    App::test((), |mut app| async move {
        let cloud_model = create_cloud_model(&mut app, objects);
        cloud_model.read(&app, |model, _| {
            let active = model.active_object_uids();
            assert_eq!(active, naive_active_object_uids(model));
            assert_eq!(active.len(), 1);
            assert!(active.contains(&active_workflow_id.uid()));
        });
    });
}

#[test]
fn active_object_uids_matches_naive_with_empty_model() {
    App::test((), |mut app| async move {
        let cloud_model = create_cloud_model(&mut app, vec![]);
        cloud_model.read(&app, |model, _| {
            let active = model.active_object_uids();
            assert_eq!(active, naive_active_object_uids(model));
            assert!(active.is_empty());
        });
    });
}
