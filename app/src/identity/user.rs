use serde::{Deserialize, Serialize};

use super::UserUid;

#[cfg(any(test, feature = "integration_tests"))]
pub use warp_server_client::identity::{TEST_USER_EMAIL, TEST_USER_UID};

#[derive(Debug, Clone)]
pub struct User {
    pub local_id: UserUid,
    pub metadata: UserMetadata,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserMetadata {
    pub email: String,
    pub display_name: Option<String>,
}

impl User {
    pub fn username_for_display(&self) -> &str {
        let user_metadata = &self.metadata;
        user_metadata
            .display_name
            .as_deref()
            .unwrap_or(user_metadata.email.as_str())
    }

    #[cfg(any(test, feature = "integration_tests"))]
    pub fn test() -> Self {
        Self {
            local_id: UserUid::new(TEST_USER_UID),
            metadata: UserMetadata {
                email: TEST_USER_EMAIL.to_string(),
                display_name: None,
            },
        }
    }

    pub fn local() -> Self {
        Self {
            local_id: UserUid::new("local-user"),
            metadata: UserMetadata {
                email: "local@warp.local".to_string(),
                display_name: Some("Local".to_string()),
            },
        }
    }
}
