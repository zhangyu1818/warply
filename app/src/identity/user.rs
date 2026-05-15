use super::UserUid;

#[cfg(any(test, feature = "integration_tests"))]
pub use local_object_model::identity::TEST_USER_UID;

#[derive(Debug, Clone)]
pub struct User {
    pub local_id: UserUid,
    display_name: String,
}

impl User {
    pub fn username_for_display(&self) -> &str {
        &self.display_name
    }

    #[cfg(any(test, feature = "integration_tests"))]
    pub fn test() -> Self {
        Self {
            local_id: UserUid::new(TEST_USER_UID),
            display_name: "Test User".to_string(),
        }
    }

    pub fn local() -> Self {
        Self {
            local_id: UserUid::new("local-user"),
            display_name: "Local".to_string(),
        }
    }
}
