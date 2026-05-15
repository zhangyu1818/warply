pub use local_object_model::ids::{
    parse_sqlite_id_to_uid, ClientId, HashableId, HashedSqliteId, ObjectUid, ServerId, SyncId,
    ToServerId,
};

/// server_id_traits generates implementations for legacy server-style local object ID newtypes.
/// It implements different To/From, Display, and HashableId traits.
/// Takes type and desired prefix for HashableId.
///
/// Note: This macro uses `$crate::object_ids::*` paths, so it only works within the warp crate.
/// For types defined in local_object_model, use `local_object_model::server_id_traits!` instead.
#[macro_export]
macro_rules! server_id_traits {
    ($t:ty, $prefix:literal) => {
        #[cfg(test)]
        impl From<i64> for $t {
            fn from(id: i64) -> Self {
                Self(id.into())
            }
        }

        impl From<String> for $t {
            fn from(id: String) -> Self {
                Self(
                    $crate::object_ids::ServerId::try_from(id)
                        .expect("server-style object id should be valid"),
                )
            }
        }

        impl From<$t> for String {
            fn from(id: $t) -> String {
                id.0.into()
            }
        }

        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "{}", self.0)
            }
        }

        impl From<$t> for $crate::object_ids::ServerId {
            fn from(id: $t) -> Self {
                id.0
            }
        }

        impl $crate::object_ids::HashableId for $t {
            fn to_hash(&self) -> String {
                format!("{}-{}", $prefix, self)
            }

            fn from_hash(hash: &str) -> Option<$t> {
                hash.strip_prefix(&format!("{}-", $prefix))
                    .map(|s| s.to_string().into())
            }
        }

        impl From<$crate::object_ids::ServerId> for $t {
            fn from(id: $crate::object_ids::ServerId) -> Self {
                Self(id)
            }
        }

        impl $crate::object_ids::ToServerId for $t {
            fn to_server_id(&self) -> $crate::object_ids::ServerId {
                self.0
            }
        }
    };
}

#[cfg(test)]
#[path = "object_ids_test.rs"]
mod tests;
