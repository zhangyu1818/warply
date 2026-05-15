use crate::cloud_object::{
    CloudModelType, CloudObjectLocation, GenericCloudObject, ObjectIdType, ObjectType, Owner,
    Revision, RevisionAndLastEditor, ServerTimestamp, Space,
};
use crate::drive::folders::{CloudFolder, CloudFolderModel};
use crate::drive::{CloudObjectTypeAndId, DriveIndexVariant};
use crate::env_vars::SavedEnvVarCollection;
use crate::object_ids::{HashableId, ObjectUid, SyncId, ToServerId};
use crate::persistence::ModelEvent;
use crate::workflows::workflow_enum::SavedWorkflowEnum;
use crate::workflows::SavedWorkflow;

use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::SyncSender;

use warpui::{AppContext, Entity, ModelContext, SingletonEntity};

use crate::cloud_object::CloudObject;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudModelEvent {
    ObjectMoved {
        type_and_id: CloudObjectTypeAndId,
        from_folder: Option<SyncId>,
        to_folder: Option<SyncId>,
    },
    ObjectUpdated {
        type_and_id: CloudObjectTypeAndId,
    },
    ObjectTrashed {
        type_and_id: CloudObjectTypeAndId,
    },
    ObjectUntrashed {
        type_and_id: CloudObjectTypeAndId,
    },
    ObjectCreated {
        type_and_id: CloudObjectTypeAndId,
    },
    /// An object was permanently deleted.
    ObjectDeleted {
        type_and_id: CloudObjectTypeAndId,
        /// The parent folder of this object, since it's no longer in the model.
        folder_id: Option<SyncId>,
    },
    /// An object identified by `id` was force expanded.
    ObjectForceExpanded {
        id: String,
    },
    InitialLoadCompleted,
}

enum FolderOpenState {
    Open,
    Closed,
    Reversed,
}

pub struct CloudModel {
    objects_by_id: HashMap<ObjectUid, Box<dyn CloudObject>>,
    model_event_sender: Option<SyncSender<ModelEvent>>,
}

impl CloudModel {
    pub fn new(
        model_event_sender: Option<SyncSender<ModelEvent>>,
        cached_objects: Vec<Box<dyn CloudObject>>,
    ) -> Self {
        let objects_by_id = cached_objects
            .into_iter()
            .map(|object| (object.uid().to_owned(), object))
            .collect::<HashMap<ObjectUid, Box<dyn CloudObject>>>();

        Self {
            objects_by_id,
            model_event_sender,
        }
    }

    /// Determines whether the given object id can be moved to the given location.
    pub fn can_move_object_to_location(
        &self,
        hashed_id: &str,
        new_location: CloudObjectLocation,
        app: &AppContext,
    ) -> bool {
        if let Some(object) = self.objects_by_id.get(hashed_id) {
            if let CloudObjectLocation::Space(space) = new_location {
                if !object.can_move_to_space(space, app) {
                    return false;
                }
            }

            if let CloudObjectLocation::Folder(target_folder_id) = new_location {
                let folder_to_move: Option<&CloudFolder> = object.into();
                if let Some(folder_to_move) = folder_to_move {
                    // We do not want to move a folder into itself.
                    if folder_to_move.id == target_folder_id {
                        return false;
                    }

                    // Since we are trying to move a folder into a folder, we want to ensure that the
                    // target folder is not a child of the folder we are trying to move.
                    let mut target_folder_parent_folder_id = self
                        .get_folder(&target_folder_id)
                        .and_then(|folder| folder.metadata().folder_id);
                    while let Some(parent_id) = target_folder_parent_folder_id {
                        if parent_id == folder_to_move.id {
                            return false;
                        }
                        target_folder_parent_folder_id = self
                            .get_folder(&parent_id)
                            .and_then(|folder| folder.metadata().folder_id);
                    }
                }
                if let Some(target_folder) = self.get_folder(&target_folder_id) {
                    // TODO: @ianhodge We do not yet support moving directly into a folder from another space
                    if target_folder.permissions.owner != object.permissions().owner {
                        return false;
                    }
                }
            }

            return true;
        }
        false
    }

    /// Given a hashed object-id, returns the object's CloudObjectLocation
    /// (either a folder or top level space)
    pub fn object_location(
        &self,
        hashed_id: &str,
        app: &AppContext,
    ) -> Option<CloudObjectLocation> {
        self.objects_by_id
            .get(hashed_id)
            .map(|object| object.location(self, app))
    }

    pub fn set_latest_revision_and_editor(
        &mut self,
        uid: &str,
        revision_and_editor: RevisionAndLastEditor,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(object) = self.objects_by_id.get_mut(uid) {
            object.metadata_mut().revision = Some(revision_and_editor.revision);
            object.metadata_mut().last_editor_uid = revision_and_editor.last_editor_uid;
        }
        ctx.notify();
    }

    pub fn get_by_uid(&self, uid: &ObjectUid) -> Option<&dyn CloudObject> {
        self.objects_by_id.get(uid).map(|o| o.as_ref())
    }

    pub fn get_mut_by_uid(&mut self, uid: &ObjectUid) -> Option<&mut Box<dyn CloudObject>> {
        self.objects_by_id.get_mut(uid)
    }

    pub fn cloud_objects(&self) -> impl Iterator<Item = &Box<dyn CloudObject>> {
        self.objects_by_id.values()
    }

    pub fn cloud_objects_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn CloudObject>> {
        self.objects_by_id.values_mut()
    }

    pub fn create_object(
        &mut self,
        id: SyncId,
        object: impl CloudObject + 'static,
        ctx: &mut ModelContext<CloudModel>,
    ) {
        ctx.emit(CloudModelEvent::ObjectCreated {
            type_and_id: object.cloud_object_type_and_id(),
        });
        self.create_object_internal(id, object);
        ctx.notify();
    }

    fn create_object_internal(&mut self, id: SyncId, object: impl CloudObject + 'static) {
        self.objects_by_id.insert(id.uid(), Box::new(object));
    }

    pub fn delete_objects_by_id(
        &mut self,
        uids: Vec<ObjectUid>,
        ctx: &mut ModelContext<Self>,
    ) -> (Vec<(SyncId, ObjectIdType)>, i32) {
        let mut count = 0;
        let mut sync_ids_and_types: Vec<(SyncId, ObjectIdType)> = Vec::new();
        for uid in uids {
            if let Some(object) = self.objects_by_id.remove(&uid) {
                let cloud_object_type_and_id = object.cloud_object_type_and_id();
                sync_ids_and_types.push((
                    cloud_object_type_and_id.sync_id(),
                    cloud_object_type_and_id.object_id_type(),
                ));

                ctx.emit(CloudModelEvent::ObjectDeleted {
                    type_and_id: object.cloud_object_type_and_id(),
                    folder_id: object.metadata().folder_id,
                });
                count += 1;
            }
        }
        ctx.notify();
        (sync_ids_and_types, count)
    }

    /// Remove an object and all its descendants from `CloudModel` recursively.
    pub fn delete_object_and_descendants(
        &mut self,
        uid: ObjectUid,
        ctx: &mut ModelContext<Self>,
    ) -> Vec<(SyncId, ObjectIdType)> {
        let mut accumulator = Vec::new();
        self.delete_object_and_descendants_internal(uid, &mut accumulator, ctx);
        accumulator
    }

    fn delete_object_and_descendants_internal(
        &mut self,
        uid: ObjectUid,
        accumulator: &mut Vec<(SyncId, ObjectIdType)>,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(object) = self.objects_by_id.remove(&uid) {
            accumulator.push((
                object.sync_id(),
                object.cloud_object_type_and_id().object_id_type(),
            ));
            ctx.emit(CloudModelEvent::ObjectDeleted {
                type_and_id: object.cloud_object_type_and_id(),
                folder_id: object.metadata().folder_id,
            });
            if object.object_type() == ObjectType::Folder {
                let contents = self
                    .objects_by_id
                    .iter()
                    .filter_map(|(child_uid, child)| {
                        if child
                            .metadata()
                            .folder_id
                            .is_some_and(|parent| parent.uid() == uid)
                        {
                            Some(child_uid.clone())
                        } else {
                            None
                        }
                    })
                    // Collect into a temporary Vec so that we can can call this mutable method
                    // recursively.
                    .collect_vec();
                for child in contents {
                    self.delete_object_and_descendants_internal(child, accumulator, ctx);
                }
            }
        }
    }

    /// Update an object's location (folder and owner). This is an implementation detail of
    /// `UpdateManager` to keep local state in sync with optimistic moves. It does not validate
    /// that the move is valid and MUST not be used elsewhere.
    ///
    /// If `new_space` is `None`, the space is unchanged.
    pub fn update_object_location(
        &mut self,
        uid: &ObjectUid,
        new_owner: Option<Owner>,
        new_folder: Option<SyncId>,
        ctx: &mut ModelContext<Self>,
    ) {
        // TODO(ben): This should use a container instead of a folder+owner pair.

        if let Some(object) = self.get_mut_by_uid(uid) {
            let old_folder = object.metadata().folder_id;
            let mut changed = false;

            if let Some(new_owner) = new_owner {
                if new_owner != object.permissions().owner {
                    object.permissions_mut().owner = new_owner;
                    changed = true;
                }
            }

            if new_folder != old_folder {
                object.metadata_mut().folder_id = new_folder;
                changed = true;
            }

            if changed {
                ctx.emit(CloudModelEvent::ObjectMoved {
                    type_and_id: object.cloud_object_type_and_id(),
                    from_folder: old_folder,
                    to_folder: new_folder,
                });
                ctx.notify();
            }
        }
    }

    pub fn update_object_metadata_last_updated_ts(
        &mut self,
        uid: &ObjectUid,
        new_ts: ServerTimestamp,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(object) = self.objects_by_id.get_mut(uid) {
            object.metadata_mut().metadata_last_updated_ts = Some(new_ts);

            if let Some(model_event_sender) = &self.model_event_sender {
                if let Err(e) = model_event_sender.send(ModelEvent::UpdateObjectMetadata {
                    id: object.hashed_sqlite_id(),
                    metadata: object.metadata().clone(),
                }) {
                    log::error!("Error saving to cache: {e:?}");
                }
            }
            ctx.notify();
        }
    }

    pub fn update_object_from_edit<K, M>(
        &mut self,
        model: M,
        object_id: SyncId,
        ctx: &mut ModelContext<Self>,
    ) where
        K: HashableId
            + ToServerId
            + std::fmt::Debug
            + Into<String>
            + Clone
            + Copy
            + Send
            + Sync
            + 'static,
        M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
    {
        if let Some(cloud_object) = self.get_object_of_type_mut(&object_id) {
            cloud_object.set_model(model);
            ctx.emit(CloudModelEvent::ObjectUpdated {
                type_and_id: cloud_object.cloud_object_type_and_id(),
            });
            ctx.notify();
        }
    }

    fn set_folder_open_state(
        &mut self,
        folder_id: SyncId,
        open_state: FolderOpenState,
        ctx: &mut ModelContext<Self>,
    ) {
        if let Some(folder) = self.get_folder_mut(&folder_id) {
            let is_open = match open_state {
                FolderOpenState::Open => true,
                FolderOpenState::Closed => false,
                FolderOpenState::Reversed => !folder.model.is_open,
            };

            folder.set_model(CloudFolderModel {
                is_open,
                name: folder.model.name.clone(),
            });

            let folder_clone = folder.clone();
            if let Some(model_event_sender) = &self.model_event_sender {
                if let Err(e) = model_event_sender.send(folder_clone.upsert_event()) {
                    log::error!("Error persisting folder: {e:?}");
                }
            }

            ctx.notify();
        }
    }

    pub fn open_folder(&mut self, folder_id: SyncId, ctx: &mut ModelContext<Self>) {
        self.set_folder_open_state(folder_id, FolderOpenState::Open, ctx)
    }

    pub fn close_folder(&mut self, folder_id: SyncId, ctx: &mut ModelContext<Self>) {
        self.set_folder_open_state(folder_id, FolderOpenState::Closed, ctx)
    }

    pub fn toggle_folder_open(&mut self, folder_id: SyncId, ctx: &mut ModelContext<Self>) {
        self.set_folder_open_state(folder_id, FolderOpenState::Reversed, ctx)
    }

    /// Collapses all folders for a given location, including the folder provided
    /// (if location is a CloudObjectLocation::Folder).
    pub fn collapse_all_in_location(
        &mut self,
        location: CloudObjectLocation,
        index_variant: DriveIndexVariant,
        ctx: &mut ModelContext<Self>,
    ) {
        let mut folder_ids: Vec<SyncId> = Vec::new();
        self.collapse_all_in_location_helper(location, index_variant, &mut folder_ids, ctx);

        folder_ids.iter().for_each(|folder_id| {
            self.set_folder_open_state(*folder_id, FolderOpenState::Closed, ctx)
        });

        ctx.notify();
    }

    /// Helper function for collapse_all_in_location. Recursively traverses through descendents,
    /// adding IDs of any folders found to the folder_ids mutable vector reference.
    fn collapse_all_in_location_helper(
        &self,
        location: CloudObjectLocation,
        index_variant: DriveIndexVariant,
        folder_ids: &mut Vec<SyncId>,
        app: &AppContext,
    ) {
        if let CloudObjectLocation::Folder(folder_id) = location {
            folder_ids.push(folder_id);
        }

        match index_variant {
            DriveIndexVariant::MainIndex => self
                .active_cloud_objects_in_location_without_descendents(location, app)
                .for_each(|object| {
                    let folder: Option<&CloudFolder> = object.into();
                    if let Some(folder) = folder {
                        self.collapse_all_in_location_helper(
                            CloudObjectLocation::Folder(folder.id),
                            index_variant,
                            folder_ids,
                            app,
                        );
                    }
                }),
            DriveIndexVariant::Trash => {
                if let CloudObjectLocation::Space(space) = location {
                    self.directly_trashed_cloud_objects_in_space(space, app)
                        .for_each(|object| {
                            let folder: Option<&CloudFolder> = object.into();
                            if let Some(folder) = folder {
                                self.collapse_all_in_location_helper(
                                    CloudObjectLocation::Folder(folder.id),
                                    index_variant,
                                    folder_ids,
                                    app,
                                );
                            }
                        })
                } else {
                    self.indirectly_trashed_cloud_objects_in_location_without_descendents(
                        location, app,
                    )
                    .for_each(|object| {
                        let folder: Option<&CloudFolder> = object.into();
                        if let Some(folder) = folder {
                            self.collapse_all_in_location_helper(
                                CloudObjectLocation::Folder(folder.id),
                                index_variant,
                                folder_ids,
                                app,
                            );
                        }
                    })
                }
            }
        }
    }

    /// Force expands the object identified by `hash_id` and any of its ancestors. If an object is
    /// identified by `id`, [`CloudModelEvent::ObjectForceExpanded`] is emitted.
    pub fn force_expand_object_and_ancestors(&mut self, id: SyncId, ctx: &mut ModelContext<Self>) {
        let hashed_id = &id.uid();
        if !self.objects_by_id.contains_key(hashed_id) {
            return;
        }

        self.force_expand_object_and_ancestors_internal(id, ctx);
        ctx.emit(CloudModelEvent::ObjectForceExpanded {
            id: hashed_id.clone(),
        });
    }

    fn force_expand_object_and_ancestors_internal(
        &mut self,
        id: SyncId,
        ctx: &mut ModelContext<Self>,
    ) {
        let Some(object) = self.objects_by_id.get(&id.uid()) else {
            return;
        };

        let parent_folder_id = object.metadata().folder_id;
        let folder: Option<&CloudFolder> = object.into();

        if let Some(folder) = folder {
            self.set_folder_open_state(folder.id, FolderOpenState::Open, ctx);
        }

        if let Some(parent_folder_id) = parent_folder_id {
            self.force_expand_object_and_ancestors_internal(parent_folder_id, ctx);
        }
    }

    pub fn delete_object(&mut self, id: SyncId, ctx: &mut ModelContext<Self>) {
        // TODO: for now we are simply hard deleting the object from memory.
        // Persisted content cleanup is handled separately by the local delete path.
        if let Some(object) = self.objects_by_id.remove(&id.uid()) {
            ctx.emit(CloudModelEvent::ObjectDeleted {
                type_and_id: object.cloud_object_type_and_id(),
                folder_id: object.metadata().folder_id,
            });
        }
        ctx.notify();
    }

    /// Number of local objects with pending content changes.
    pub fn num_unsaved_objects(&self) -> usize {
        self.objects_by_id
            .values()
            .filter(|object| object.metadata().has_pending_content_changes())
            .count()
    }

    /// Number of local objects with pending content changes that require a quit warning.
    pub fn num_unsaved_objects_to_warn_about_before_quitting(&self) -> usize {
        self.objects_by_id
            .values()
            .filter(|object| {
                object.warn_if_unsaved_at_quit() && object.metadata().has_pending_content_changes()
            })
            .count()
    }

    pub fn has_objects(&self) -> bool {
        !self.objects_by_id.is_empty()
    }

    pub fn has_non_welcome_objects(&self) -> bool {
        self.objects_by_id
            .iter()
            .any(|(_, object)| !object.metadata().is_welcome_object)
    }

    pub fn get_folder_by_uid(&self, uid: &str) -> Option<&CloudFolder> {
        self.objects_by_id.get(uid).and_then(|object| object.into())
    }

    pub fn get_folder(&self, folder_id: &SyncId) -> Option<&CloudFolder> {
        self.objects_by_id
            .get(&folder_id.uid())
            .and_then(|object| object.into())
    }

    pub fn get_folder_mut(&mut self, folder_id: &SyncId) -> Option<&mut CloudFolder> {
        self.objects_by_id
            .get_mut(&folder_id.uid())
            .and_then(|object| object.into())
    }

    pub fn get_all_exportable_object_ids(&self) -> Vec<CloudObjectTypeAndId> {
        self.objects_by_id
            .values()
            .filter(|object| object.can_export())
            .map(|object| object.cloud_object_type_and_id())
            .collect()
    }

    #[allow(unused)]
    /// Returns only active (not trashed) folders in the local object model.
    pub fn get_all_active_folders(&self) -> impl Iterator<Item = &CloudFolder> {
        self.objects_by_id
            .values()
            .filter(|object| !object.is_trashed(self))
            .filter_map(|object| object.into())
    }

    /// Returns all folders (trashed or not) in the local object model.
    pub fn get_all_active_and_inactive_folders(&self) -> impl Iterator<Item = &CloudFolder> {
        self.objects_by_id
            .values()
            .filter_map(|object| object.into())
    }

    pub fn get_workflow(&self, workflow_id: &SyncId) -> Option<&SavedWorkflow> {
        self.objects_by_id
            .get(&workflow_id.uid())
            .and_then(|object| object.into())
    }

    pub fn get_workflow_by_uid(&self, uid: &str) -> Option<&SavedWorkflow> {
        self.objects_by_id.get(uid).and_then(|object| object.into())
    }

    pub fn get_workflow_enum(&self, enum_id: &SyncId) -> Option<&SavedWorkflowEnum> {
        self.objects_by_id
            .get(&enum_id.uid())
            .and_then(|object| object.into())
    }

    pub fn get_workflow_enum_mut(&mut self, enum_id: &SyncId) -> Option<&mut SavedWorkflowEnum> {
        self.objects_by_id
            .get_mut(&enum_id.uid())
            .and_then(|object| object.into())
    }

    pub fn get_workflow_mut(&mut self, workflow_id: &SyncId) -> Option<&mut SavedWorkflow> {
        self.objects_by_id
            .get_mut(&workflow_id.uid())
            .and_then(|object| object.into())
    }

    /// Returns only active (not trashed) workflows in the local object model.
    pub fn get_all_active_workflows(&self) -> impl Iterator<Item = &SavedWorkflow> {
        self.objects_by_id
            .values()
            .filter(|object| !object.is_trashed(self))
            .filter_map(|object| object.into())
    }

    /// Returns all workflows (trashed or not) in the local object model.
    pub fn get_all_active_and_inactive_workflows(&self) -> impl Iterator<Item = &SavedWorkflow> {
        self.objects_by_id
            .values()
            .filter_map(|object| object.into())
    }

    /// Returns all workflows (trashed or not) in the local object model.
    pub fn get_all_active_and_inactive_workflows_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut SavedWorkflow> {
        self.objects_by_id
            .values_mut()
            .filter_map(|object| object.into())
    }

    /// Returns all active (not trashed) workflows in the space.
    pub fn active_workflows_in_space<'a>(
        &'a self,
        space: Space,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a SavedWorkflow> + 'a {
        self.active_cloud_objects_in_space(space, app)
            .filter_map(|object| object.into())
    }

    /// Returns all active (not trashed) and non-welcome workflows (ie. non starter workflows) in the space.
    pub fn active_non_welcome_workflows_in_space<'a>(
        &'a self,
        space: Space,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a SavedWorkflow> + 'a {
        self.active_non_welcome_cloud_objects_in_space(space, app)
            .filter_map(|object| object.into())
    }

    /// Returns all active (not trashed) and non-welcome env var collections in the space.
    pub fn active_non_welcome_env_var_collections_in_space<'a>(
        &'a self,
        space: Space,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a SavedEnvVarCollection> + 'a {
        self.active_non_welcome_cloud_objects_in_space(space, app)
            .filter_map(|object| object.into())
    }

    /// Returns all workflow enums with a given owner.
    pub fn workflow_enums_with_owner<'a>(
        &'a self,
        owner: Owner,
        _: &'a AppContext,
    ) -> impl Iterator<Item = &'a SavedWorkflowEnum> + 'a {
        self.objects_by_id
            .values()
            .filter(move |object| !object.is_trashed(self) && object.permissions().owner == owner)
            .filter_map(|object| object.into())
    }

    pub fn get_object_of_type<K, M>(&self, object_id: &SyncId) -> Option<&GenericCloudObject<K, M>>
    where
        K: HashableId + ToServerId + std::fmt::Debug + Into<String> + Clone + 'static,
        M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
    {
        self.objects_by_id
            .get(&object_id.uid())
            .and_then(|object| object.into())
    }

    pub fn get_object_of_type_mut<K, M>(
        &mut self,
        object_id: &SyncId,
    ) -> Option<&mut GenericCloudObject<K, M>>
    where
        K: HashableId + ToServerId + std::fmt::Debug + Into<String> + Clone + 'static,
        M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
    {
        self.objects_by_id
            .get_mut(&object_id.uid())
            .and_then(|object| object.into())
    }

    pub fn get_all_objects_of_type<K, M>(&self) -> impl Iterator<Item = &GenericCloudObject<K, M>>
    where
        K: HashableId + ToServerId + std::fmt::Debug + Into<String> + Clone + 'static,
        M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
    {
        self.objects_by_id
            .values()
            .filter_map(|object| object.into())
    }

    pub fn get_env_var_collection(
        &self,
        env_var_collection_id: &SyncId,
    ) -> Option<&SavedEnvVarCollection> {
        self.objects_by_id
            .get(&env_var_collection_id.uid())
            .and_then(|object| object.into())
    }

    pub fn get_env_var_collection_by_uid(&self, uid: &str) -> Option<&SavedEnvVarCollection> {
        self.objects_by_id.get(uid).and_then(|object| object.into())
    }

    /// Returns only active (not trashed) EVCs in the local object model.
    pub fn get_all_active_env_var_collections(
        &self,
    ) -> impl Iterator<Item = &SavedEnvVarCollection> {
        self.objects_by_id
            .values()
            .filter(|object| !object.is_trashed(self))
            .filter_map(|object| object.into())
    }

    pub fn current_revision(&self, id: &SyncId) -> Option<&Revision> {
        self.objects_by_id
            .get(&id.uid())
            .and_then(|warp_cloud_object| warp_cloud_object.metadata().revision.as_ref())
    }

    #[cfg(test)]
    pub fn as_cloud_objects(&self) -> impl Iterator<Item = &'_ Box<dyn CloudObject>> {
        self.objects_by_id.values()
    }

    #[cfg(test)]
    pub fn add_object(&mut self, id: SyncId, object: impl CloudObject + 'static) {
        self.objects_by_id.insert(id.uid(), Box::new(object));
    }

    /// Pre-computes the set of UIDs for all active (non-trashed) objects using memoization.
    /// This is O(N) amortized instead of O(N × D) for the naive approach, because each
    /// object's trashed status is computed at most once and cached.
    pub fn active_object_uids(&self) -> HashSet<ObjectUid> {
        let mut cache = HashMap::new();
        let mut visiting = HashSet::new();
        let mut active = HashSet::new();
        for uid in self.objects_by_id.keys() {
            if !self.is_trashed_memoized(uid, &mut cache, &mut visiting) {
                active.insert(uid.clone());
            }
        }
        active
    }

    /// Memoized version of `is_trashed` that caches results to avoid redundant ancestor traversals.
    fn is_trashed_memoized(
        &self,
        uid: &str,
        cache: &mut HashMap<String, bool>,
        visiting: &mut HashSet<String>,
    ) -> bool {
        if let Some(&cached) = cache.get(uid) {
            return cached;
        }

        // Cycle detection: if we're already visiting this UID in the current traversal, treat as trashed.
        if visiting.contains(uid) {
            return true;
        }

        let result = match self.objects_by_id.get(uid) {
            Some(object) => {
                if object.metadata().trashed_ts.is_some() {
                    true
                } else {
                    match object.metadata().folder_id.map(|parent_id| parent_id.uid()) {
                        Some(parent_uid) => {
                            visiting.insert(uid.to_owned());
                            let r = self.is_trashed_memoized(&parent_uid, cache, visiting);
                            visiting.remove(uid);
                            r
                        }
                        None => false,
                    }
                }
            }
            None => true,
        };

        cache.insert(uid.to_owned(), result);
        result
    }

    /// Given a CloudObjectLocation (either a folder or a space), returns an iterator of active local objects
    /// that live directly in this location (its children). I.e. this function does NOT look into nested folders in order
    /// to return those children.
    pub fn active_cloud_objects_in_location_without_descendents<'a>(
        &'a self,
        location: CloudObjectLocation,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a dyn CloudObject> + 'a {
        self.objects_by_id
            .values()
            .filter(move |object| {
                !object.is_trashed(self) && object.location(self, app) == location
            })
            .map(|object| object.as_ref())
    }

    /// Given a CloudObjectLocation (either a folder or a space), returns an iterator of trashed local objects
    /// that live directly in this location (its children). I.e. this function does NOT look into nested folders in order
    /// to return those children.
    pub fn trashed_cloud_objects_in_location_without_descendents<'a>(
        &'a self,
        location: CloudObjectLocation,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a dyn CloudObject> + 'a {
        self.objects_by_id
            .values()
            .filter(move |object| object.is_trashed(self) && object.location(self, app) == location)
            .map(|object| object.as_ref())
    }

    pub fn trashed_cloud_object_types_in_location_with_descendants(
        &self,
        location: CloudObjectLocation,
        app: &AppContext,
    ) -> Vec<ObjectType> {
        let mut trashed_objects: Vec<ObjectType> = Vec::new();
        self.trashed_cloud_object_types_in_location_with_descendants_helper(
            location,
            &mut trashed_objects,
            app,
        );
        trashed_objects
    }

    /// Helper function for trashed_cloud_objects_in_location_with_descendants.
    /// Recursively traverses through descendants, adding object types of any trashed
    /// objects found to the trashed_objects mutable vector reference.
    fn trashed_cloud_object_types_in_location_with_descendants_helper(
        &self,
        location: CloudObjectLocation,
        trashed_objects: &mut Vec<ObjectType>,
        app: &AppContext,
    ) {
        // Fetch direct descendants of the location
        self.trashed_cloud_objects_in_location_without_descendents(location, app)
            .for_each(|object| {
                trashed_objects.push(object.object_type());
                let folder: Option<&CloudFolder> = object.into();
                // If any of the direct descendants are folders, recursively traverse through them
                if let Some(folder) = folder {
                    self.trashed_cloud_object_types_in_location_with_descendants_helper(
                        CloudObjectLocation::Folder(folder.id),
                        trashed_objects,
                        app,
                    );
                }
            });
    }

    /// Given a CloudObjectLocation (either a folder or a space), returns an iterator of local objects
    /// that live directly in this location (its children) are in the trash but have not been explicitly
    /// trashed by a user. I.e. this function does NOT look into nested folders in order to return those children.
    pub fn indirectly_trashed_cloud_objects_in_location_without_descendents<'a>(
        &'a self,
        location: CloudObjectLocation,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a dyn CloudObject> {
        self.objects_by_id
            .values()
            .filter(move |object| {
                object.is_trashed(self)
                    && object.location(self, app) == location
                    && object.metadata().trashed_ts.is_none()
            })
            .map(|object| object.as_ref())
    }

    /// Returns all active local objects in the space.
    pub fn active_cloud_objects_in_space<'a>(
        &'a self,
        space: Space,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a dyn CloudObject> + 'a {
        self.objects_by_id
            .values()
            .filter(move |object| object.is_in_space(space, app) && !object.is_trashed(self))
            .map(|object| object.as_ref())
    }

    /// Returns all active non-welcome local objects in the space.
    pub fn active_non_welcome_cloud_objects_in_space<'a>(
        &'a self,
        space: Space,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a dyn CloudObject> + 'a {
        self.objects_by_id
            .values()
            .filter(move |object| {
                object.is_in_space(space, app)
                    && !object.is_trashed(self)
                    && !object.is_welcome_object()
            })
            .map(|object| object.as_ref())
    }

    // Returns all objects, trashed or otherwise, in the space.
    pub fn all_cloud_objects_in_space<'a>(
        &'a self,
        space: Space,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a dyn CloudObject> + 'a {
        self.objects_by_id
            .values()
            .filter(move |object| object.is_in_space(space, app))
            .map(|object| object.as_ref())
    }

    /// Returns all trashed local objects in the space.
    pub fn trashed_cloud_objects_in_space<'a>(
        &'a self,
        space: Space,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a dyn CloudObject> + 'a {
        self.objects_by_id
            .values()
            .filter(move |object| object.is_in_space(space, app) && object.is_trashed(self))
            .map(|object| object.as_ref())
    }

    /// Returns all local objects in the space that have been explicitly trashed by a user.
    pub fn directly_trashed_cloud_objects_in_space<'a>(
        &'a self,
        space: Space,
        app: &'a AppContext,
    ) -> impl Iterator<Item = &'a dyn CloudObject> {
        self.objects_by_id
            .values()
            .filter(move |object| {
                object.is_in_space(space, app) && object.metadata().trashed_ts.is_some()
            })
            .map(|object| object.as_ref())
    }

    /// Returns a map of how many active (not trashed) objects reside within specified spaces.
    pub fn num_active_cloud_objects_per_space<'a, I>(
        &self,
        spaces: I,
        app: &AppContext,
    ) -> HashMap<Space, usize>
    where
        I: Iterator<Item = &'a Space>,
    {
        spaces
            .map(|space| {
                (
                    *space,
                    self.active_cloud_objects_in_space(*space, app).count(),
                )
            })
            .collect::<HashMap<_, _>>()
    }

    /// Returns a map of how many trashed objects reside within specified spaces.
    pub fn num_trashed_cloud_objects_per_space<'a, I>(
        &self,
        spaces: I,
        app: &AppContext,
    ) -> HashMap<Space, usize>
    where
        I: Iterator<Item = &'a Space>,
    {
        spaces
            .map(|space| {
                (
                    *space,
                    self.trashed_cloud_objects_in_space(*space, app).count(),
                )
            })
            .collect::<HashMap<_, _>>()
    }

    #[cfg(test)]
    pub fn mock(_ctx: &mut ModelContext<Self>) -> Self {
        Self::new(None, Vec::new())
    }

    pub fn reset(&mut self) {
        self.objects_by_id = HashMap::new();
    }
}

impl Entity for CloudModel {
    type Event = CloudModelEvent;
}

/// Mark CloudModel as global application state.
impl SingletonEntity for CloudModel {}

#[cfg(test)]
#[path = "model_test.rs"]
mod tests;
