use self::{breadcrumbs::ContainingObject, model::persistence::CloudModel};
use crate::{
    appearance::Appearance,
    drive::{items::LocalObjectItem, CloudObjectTypeAndId},
    object_ids::{ClientId, HashableId, HashedSqliteId, ObjectUid, SyncId, ToServerId},
    persistence::ModelEvent,
    util::time_format::format_approx_duration_from_now_utc,
    workflows::WorkflowSource,
    workspaces::user_workspaces::UserWorkspaces,
};
use derivative::Derivative;
use std::{any::Any, collections::HashSet, fmt::Debug, sync::Arc};
use warpui::{AppContext, SingletonEntity};

pub mod breadcrumbs;
pub mod model;
pub mod toast_message;
pub mod update_manager;

pub use local_object_model::cloud_object::*;

/// A CloudObject represents a retained local object such as a workflow, folder, or notebook.
/// Local revision metadata is kept only for persisted object bookkeeping.
///
/// Note that this trait must be object-safe and non-generic.  The reason for this
/// is that (a) we need to be able to store instances of it as trait objects in
/// CloudModel and (b) we need to be able to support mixed collections of different
/// instances of it (e.g. in the map of id -> CloudObject in CloudModel).
///
/// There are two closely related types to this:
/// 1) GenericCloudObject: This is the concrete generic implementation of CloudObject that
///    holds onto a model of type CloudModelType and an id of type SyncId.
/// 2) CloudModelType: This is a trait that defines the model type for a CloudObject -
///    this is what implementors of new local object types typically have to implement.
///
/// These types are tightly coupled.  In an ideal world, rust would allow a mechanism
/// for us having a single interface that new model types could implement that could
/// be generic on id and model types, but as far as I (zach) can tell, that's not currently
/// possible.
///
/// The typical usage pattern for these types is to use dyn CloudObject whenever you
/// don't need access to a model or id, and to downcast to a GenericCloudObject whenever you do.
///
/// This implies that, for now, *all* CloudObjects must implement GenericCloudObject.
///
/// For more info on revisions: https://docs.google.com/document/d/1SGtX_5AiSJmUxXCRk5NzGTzrC_XrxQRsio-KZOec_ng/edit
pub trait CloudObject: Debug {
    /// Returns the name of this model type (e.g. Workflow, Folder, Notebook)
    fn model_type_name(&self) -> &'static str;

    /// Returns the  uid for this object.
    fn uid(&self) -> ObjectUid;

    /// Returns the [`SyncId`] that currently identifies this object.
    fn sync_id(&self) -> SyncId;

    /// Returns the id used to index into sqlite, this is the object's UID with its type
    /// prefixed, such as "Workflow-{UID}"
    fn hashed_sqlite_id(&self) -> HashedSqliteId;

    /// Returns the CloudObjectMetadata struct associated with this object.
    fn metadata(&self) -> &CloudObjectMetadata;

    /// Returns a mutable reference to the CloudObjectMetadata struct associated with this object.
    fn metadata_mut(&mut self) -> &mut CloudObjectMetadata;

    /// Returns the CloudObjectPermissions struct associated with this object.
    fn permissions(&self) -> &CloudObjectPermissions;

    /// Returnsa mutable reference to the CloudObjectPermissions struct associated with this object.
    fn permissions_mut(&mut self) -> &mut CloudObjectPermissions;

    /// Returns the ObjectType i.e. 'Workflow' or 'Notebook'
    fn object_type(&self) -> ObjectType;

    /// Returns the CloudObjectTypeAndId for this object.
    fn cloud_object_type_and_id(&self) -> CloudObjectTypeAndId;

    /// Returns whether this object can be moved to the given space.
    fn can_move_to_space(&self, _space: Space, _app: &AppContext) -> bool {
        true
    }

    // Whether to clear this object from the local SQLite DB on a unique key conflict.
    fn should_clear_on_unique_key_conflict(&self) -> bool {
        false
    }

    /// Whether to show a warning if this object is unsaved at quit time
    /// (which typically blocks the user from quitting)
    fn warn_if_unsaved_at_quit(&self) -> bool {
        true
    }

    /// Returns the "upsert" event for inserting / updating this object in the SQLite DB.
    fn upsert_event(&self) -> ModelEvent;

    // Returns the name of the object.
    fn display_name(&self) -> String;

    fn renders_as_local_object(&self) -> bool;

    /// Returns whether this model type should show update toasts in the UI.
    fn should_show_activity_toasts(&self) -> bool {
        true
    }

    fn to_local_object_item(&self, appearance: &Appearance) -> Option<Box<dyn LocalObjectItem>>;

    fn space(&self, app: &AppContext) -> Space {
        UserWorkspaces::as_ref(app).owner_to_space(self.permissions().owner, app)
    }

    /// Returns the name of the containing "object" for this object.
    /// This could be a folder, or in the case of top-level objects,
    /// the name of the space it belongs to.
    fn containing_object_name(&self, app: &AppContext) -> String {
        self.containing_objects_path(app)
            .into_iter()
            .next_back()
            .expect("Object should have at least one ancestor")
            .name
    }

    // Returns the path of all the containing "objects" for this object.
    // This could include folders or spaces.
    fn containing_objects_path(&self, app: &AppContext) -> Vec<ContainingObject> {
        let space = self.space(app);

        match self.metadata().folder_id {
            Some(folder_id) => {
                let cloud_model = CloudModel::as_ref(app);
                if let Some(folder) = cloud_model.get_folder_by_uid(&folder_id.uid()) {
                    let mut path = vec![];
                    let ancestors = folder.containing_objects_path(app);
                    path.extend(ancestors);
                    path.push(folder.into());
                    path
                } else {
                    // if for whatever reason the folder id is messed up,
                    // just default to showing the top-level space it wound up in
                    vec![space.into_containing_object(app)]
                }
            }
            None => vec![space.into_containing_object(app)],
        }
    }

    fn breadcrumbs(&self, app: &AppContext) -> String {
        self.containing_objects_path(app)
            .into_iter()
            .map(|object| object.name)
            .collect::<Vec<String>>()
            .join(" / ")
    }

    /// Returns whether this CloudObject is in the given space
    fn is_in_space(&self, space: Space, app: &AppContext) -> bool {
        self.space(app) == space
    }

    fn is_welcome_object(&self) -> bool {
        self.metadata().is_welcome_object
    }

    /// Returns the direct location of the object. If the object
    /// is not in a folder, this will be the object's space. Otherwise, it will
    /// be the folder the object is placed in directly, even if that folder is nested.
    fn location(&self, cloud_model: &CloudModel, app: &AppContext) -> CloudObjectLocation {
        if let Some(folder_id) = self.metadata().folder_id {
            if cloud_model.get_folder(&folder_id).is_some() {
                return CloudObjectLocation::Folder(folder_id);
            }
        }

        CloudObjectLocation::Space(self.space(app))
    }

    /// Return true is this object or any of its ancestors are trashed. Also returns true
    /// if a cycle is detected.
    fn is_trashed(&self, cloud_model: &CloudModel) -> bool {
        self.is_trashed_internal(cloud_model, &mut HashSet::new())
    }

    /// Helper function for is_trashed.
    fn is_trashed_internal(
        &self,
        cloud_model: &CloudModel,
        ancestors: &mut HashSet<String>,
    ) -> bool {
        // Base case: If the object is trashed, return true.
        if self.metadata().trashed_ts.is_some() {
            return true;
        }

        // Else: return true if the object's parent is trashed. Return false if the object has no parent.
        match self.metadata().folder_id.map(|parent_id| parent_id.uid()) {
            Some(hashed_parent_id) => {
                // We need to check for cycles to avoid causing a stack overflow. If a cycle is detected, return that the object is trashed.
                if ancestors.contains(&hashed_parent_id) {
                    return true;
                }
                ancestors.insert(hashed_parent_id.clone());

                match cloud_model.get_by_uid(&hashed_parent_id) {
                    Some(parent) => parent.is_trashed_internal(cloud_model, ancestors),
                    None => true,
                }
            }
            None => false,
        }
    }

    /// Whether or not this object can be exported.
    fn can_export(&self) -> bool;

    /// Returns this object as a ref to the Any type.  Needed for typecasts.
    fn as_any(&self) -> &dyn Any;

    /// Returns this object as a mut ref to Any type.  Needed for typecasts.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns the trait object as a concrete type reference by downcasting it.
    /// Returns None if the downcast fails.
    fn as_model_type<K, M>(cloud_object: &dyn CloudObject) -> Option<&GenericCloudObject<K, M>>
    where
        Self: Sized,
        K: HashableId + ToServerId + Debug + Into<String> + Clone + 'static,
        M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
    {
        cloud_object
            .as_any()
            .downcast_ref::<GenericCloudObject<K, M>>()
    }

    /// Returns the trait object as a concrete mutable type reference by downcasting it.
    /// Returns None if the downcast fails.
    fn as_model_type_mut<K, M>(
        cloud_object: &mut dyn CloudObject,
    ) -> Option<&mut GenericCloudObject<K, M>>
    where
        Self: Sized,
        K: HashableId + ToServerId + Debug + Into<String> + Clone + 'static,
        M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
    {
        cloud_object
            .as_any_mut()
            .downcast_mut::<GenericCloudObject<K, M>>()
    }

    /// Returns a cloned boxed version of this local object.
    /// Note that we can't force the CloudObject trait to derive from Cloned
    /// directly because that would make the trait not object safe.  This
    /// is a workaround.
    fn clone_box(&self) -> Box<dyn CloudObject>;
}

/// Defines a common trait for local object models to implement.
/// The "model" is the domain-specific piece of data for a retained local object,
/// e.g. it contains the notebook, workflow, or folder specific data, but has
/// no logic around metadata, permissions, or pending content status.
///
/// See the comments for CloudObject to understand the relationship between
/// this trait, CloudObject and GenericCloudObject.  They are tightly coupled.
///
/// When building new model types (e.g. for settings or launch configs) we should just
/// have to implement this trait, and not the entire CloudObject trait.
pub trait CloudModelType: Debug + Clone + Send + Sync {
    /// The associated CloudObject type for this model.
    type CloudObjectType: CloudObject + 'static;
    // TODO: @ianhodge - remove for sync ID refactor.
    type IdType: HashableId + ToServerId + Debug + Into<String> + Clone + 'static;

    /// Returns the name of this model type (e.g. Workflow, Folder, Notebook)
    fn model_type_name(&self) -> &'static str;

    /// Returns the CloudObjectTypeAndId for this object.
    fn cloud_object_type_and_id(&self, id: SyncId) -> CloudObjectTypeAndId;

    /// Returns the ObjectType for this model.
    fn object_type(&self) -> ObjectType;

    fn renders_as_local_object(&self) -> bool;

    /// Returns whether this model type should show update toasts in the UI.
    fn should_show_activity_toasts(&self) -> bool {
        true
    }

    /// Whether to show a warning if this model is unsaved at quit time
    /// (which typically blocks the user from quitting)
    fn warn_if_unsaved_at_quit(&self) -> bool {
        true
    }

    fn to_local_object_item(
        &self,
        id: SyncId,
        appearance: &Appearance,
        object: &Self::CloudObjectType,
    ) -> Option<Box<dyn LocalObjectItem>>;

    fn display_name(&self) -> String;

    /// Sets the display name. Setting the name
    /// is not currently supported by all object types, hence the default empty
    /// implementation.
    fn set_display_name(&mut self, _name: &str) {}

    /// Returns the upsert event for putting this model into the SQLite database.
    fn upsert_event(&self, object: &Self::CloudObjectType) -> ModelEvent;

    /// Returns a bulk upsert event for putting a list of this model into the SQLite database.
    fn bulk_upsert_event(objects: &[Self::CloudObjectType]) -> ModelEvent;

    /// Returns a serialized model.
    fn serialized(&self) -> SerializedModel;

    /// Returns whether this model type supports being moved to the given space.
    fn can_move_to_space(&self, _current_space: Space, _new_space: Space) -> bool {
        true
    }

    /// Returns whether this model type should clear on a unique key conflict.
    fn should_clear_on_unique_key_conflict(&self) -> bool {
        false
    }

    /// Whether this model type can be exported.
    fn can_export(&self) -> bool {
        false
    }
}

/// A generic implementation of retained local objects that can be used for any model and id types.
///
/// For instance, rather than directly implementing the CloudObject trait, CloudObjects can
/// implement GenericCloudObject<K, M> where K is their id type and M is their model type.
///
/// The advantage of using the generic model is you get common implementations
/// of CloudObject methods like ```versions``` for free.
///
/// See the comments for CloudObject to understand the relationship between
/// this trait, CloudObject and CloudModelType.  They are tightly coupled.
#[derive(Clone, Debug)]
pub struct GenericCloudObject<K, M>
where
    K: HashableId + ToServerId + Debug + Into<String> + Clone + 'static,
    M: CloudModelType<IdType = K> + 'static,
{
    pub id: SyncId,
    pub metadata: CloudObjectMetadata,
    pub permissions: CloudObjectPermissions,

    // Intentionally not public to prevent users of this class from holding
    // onto references to the model outside of this struct.
    //
    // This is an Arc in order to support clone-on-write semantics for the model.
    // By wrapping the model in an Arc, clones become cheap, and we can avoid
    // doing deep clones of the model whenever the containing object is cloned.
    //
    // Callers who want to update the model need to call set_model to update the
    // entire model atomically.
    model: Arc<M>,
}

impl<K, M> CloudObject for GenericCloudObject<K, M>
where
    K: HashableId + ToServerId + Debug + Into<String> + Clone + 'static,
    M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
{
    fn model_type_name(&self) -> &'static str {
        self.model.model_type_name()
    }

    fn uid(&self) -> ObjectUid {
        self.id.uid()
    }

    fn hashed_sqlite_id(&self) -> HashedSqliteId {
        self.id.sqlite_uid_hash(self.object_type().into())
    }

    fn sync_id(&self) -> SyncId {
        self.id
    }

    fn should_show_activity_toasts(&self) -> bool {
        self.model.should_show_activity_toasts()
    }

    fn warn_if_unsaved_at_quit(&self) -> bool {
        self.model.warn_if_unsaved_at_quit()
    }

    fn metadata(&self) -> &CloudObjectMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut CloudObjectMetadata {
        &mut self.metadata
    }

    fn permissions(&self) -> &CloudObjectPermissions {
        &self.permissions
    }

    fn permissions_mut(&mut self) -> &mut CloudObjectPermissions {
        &mut self.permissions
    }

    fn object_type(&self) -> ObjectType {
        self.model.object_type()
    }

    fn cloud_object_type_and_id(&self) -> CloudObjectTypeAndId {
        self.model.cloud_object_type_and_id(self.id)
    }

    fn should_clear_on_unique_key_conflict(&self) -> bool {
        self.model.should_clear_on_unique_key_conflict()
    }

    fn can_move_to_space(&self, space: Space, app: &AppContext) -> bool {
        self.model.can_move_to_space(self.space(app), space)
    }

    fn upsert_event(&self) -> ModelEvent {
        self.model.upsert_event(self)
    }

    fn display_name(&self) -> String {
        self.model.display_name()
    }

    fn renders_as_local_object(&self) -> bool {
        self.model.renders_as_local_object()
    }

    fn to_local_object_item(&self, appearance: &Appearance) -> Option<Box<dyn LocalObjectItem>> {
        self.model.to_local_object_item(self.id, appearance, self)
    }

    fn can_export(&self) -> bool {
        self.model.can_export()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn CloudObject> {
        Box::new(self.clone())
    }
}

impl<K, M> GenericCloudObject<K, M>
where
    K: HashableId + ToServerId + Debug + Into<String> + Clone + 'static,
    M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
{
    /// Gets a reference to the model held by the object.
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Returns a shared handle to the model.
    pub fn shared_model(&self) -> Arc<M> {
        self.model.clone()
    }

    /// Sets a new version of the model on the object, replacing the old version.
    pub fn set_model(&mut self, model: M) {
        self.model = model.into();
    }

    /// Returns a bulk upsert event for putting a list of this model into the SQLite database.
    pub fn bulk_upsert_event(objects: &[Self]) -> ModelEvent {
        M::bulk_upsert_event(objects)
    }

    /// Constructs a new instance of this model with the given id, model, metadata and permissions.
    pub fn new(
        id: SyncId,
        model: M,
        metadata: CloudObjectMetadata,
        permissions: CloudObjectPermissions,
    ) -> Self {
        Self {
            id,
            model: model.into(),
            metadata,
            permissions,
        }
    }

    /// Creates a new GenericCloudObject with the given model, owner, and initial folder id.
    /// This is for the local creation flow.
    pub fn new_local(
        model: M,
        owner: Owner,
        initial_folder_id: Option<SyncId>,
        client_id: ClientId,
    ) -> Self {
        Self {
            id: SyncId::ClientId(client_id),
            model: model.into(),
            metadata: CloudObjectMetadata {
                pending_changes_statuses: CloudObjectStatuses {
                    content_sync_status: CloudObjectSyncStatus::InFlight(NumInFlightRequests(1)),
                },
                folder_id: initial_folder_id,
                revision: Default::default(),
                metadata_last_updated_ts: Default::default(),
                current_editor_uid: Default::default(),
                trashed_ts: Default::default(),
                // Objects created from the client are never welcome objects.
                is_welcome_object: false,
                creator_uid: None,
                last_editor_uid: None,
                last_task_run_ts: None,
            },
            permissions: CloudObjectPermissions { owner },
        }
    }
}

impl<'a, K, M> From<&'a dyn CloudObject> for Option<&'a GenericCloudObject<K, M>>
where
    K: HashableId + ToServerId + Debug + Into<String> + Clone + 'static,
    M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
{
    fn from(value: &'a dyn CloudObject) -> Self {
        <GenericCloudObject<K, M> as CloudObject>::as_model_type(value)
    }
}

impl<'a, K, M> From<&'a Box<dyn CloudObject>> for Option<&'a GenericCloudObject<K, M>>
where
    K: HashableId + ToServerId + Debug + Into<String> + Clone + 'static,
    M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
{
    fn from(value: &'a Box<dyn CloudObject>) -> Self {
        <GenericCloudObject<K, M> as CloudObject>::as_model_type(value.as_ref())
    }
}

impl<'a, K, M> From<&'a mut Box<dyn CloudObject>> for Option<&'a mut GenericCloudObject<K, M>>
where
    K: HashableId + ToServerId + Debug + Into<String> + Clone + 'static,
    M: CloudModelType<IdType = K, CloudObjectType = GenericCloudObject<K, M>> + 'static,
{
    fn from(value: &'a mut Box<dyn CloudObject>) -> Self {
        <GenericCloudObject<K, M> as CloudObject>::as_model_type_mut(value.as_mut())
    }
}

impl Clone for Box<dyn CloudObject> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Represents a unique key for a generic string object.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct GenericStringObjectUniqueKey {
    /// The unique key. For settings-backed objects this is the storage key.
    pub key: String,

    /// Whether this key is unique for all generic string objects, or unique per user.
    pub unique_per: UniquePer,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum UniquePer {
    User,
}

impl From<&dyn CloudObject> for ObjectType {
    fn from(value: &dyn CloudObject) -> Self {
        value.object_type()
    }
}

impl From<&Box<dyn CloudObject>> for ObjectType {
    fn from(value: &Box<dyn CloudObject>) -> Self {
        <ObjectType as From<&dyn CloudObject>>::from(value.as_ref())
    }
}

/// Extension trait for CloudObjectMetadata with methods that require AppContext.
pub trait CloudObjectMetadataExt {
    /// Returns a semantic summary of the last edit to the object. For example, "Alice edited 4 weeks ago".
    /// Returns None if the revision and last_editor are None.
    fn semantic_editing_history(&self, app: &AppContext) -> Option<String>;
}

impl CloudObjectMetadataExt for CloudObjectMetadata {
    fn semantic_editing_history(&self, _app: &AppContext) -> Option<String> {
        self.revision
            .clone()
            .map(|r| format!("Edited {}", format_approx_duration_from_now_utc(r.utc())))
    }
}

#[derive(Default, Clone, Copy, Debug, Eq, Derivative)]
#[derivative(PartialEq, Hash)]
pub enum Space {
    #[default]
    Personal,
}

impl Space {
    pub fn name(&self, _app: &AppContext) -> String {
        match self {
            Space::Personal => "Personal".to_string(),
        }
    }
}

/// Enum for specifying the location of a local object.
/// Objects can live in top level spaces, or a specific folder.
#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub enum CloudObjectLocation {
    Space(Space),
    Folder(SyncId),
    Trash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedModel(String);

impl SerializedModel {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn model_as_str(&self) -> &str {
        &self.0
    }

    pub fn take(self) -> String {
        self.0
    }
}

impl From<String> for SerializedModel {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<Space> for WorkflowSource {
    fn from(space: Space) -> Self {
        match space {
            Space::Personal => WorkflowSource::Saved,
        }
    }
}

impl From<Owner> for WorkflowSource {
    fn from(owner: Owner) -> WorkflowSource {
        match owner {
            Owner::User { .. } => Self::Saved,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RevisionAndLastEditor {
    pub revision: Revision,
    pub last_editor_uid: Option<String>,
}
