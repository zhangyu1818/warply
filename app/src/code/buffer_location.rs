use warp_util::content_version::ContentVersion;
pub use warp_util::local_or_remote_path::LocalOrRemotePath;

pub type BufferLocation = LocalOrRemotePath;
pub type FileLocation = LocalOrRemotePath;

/// Tracks sync state between client and server for a single remote buffer.
#[derive(Clone, Debug)]
pub struct SyncClock {
    pub server_version: ContentVersion,
    pub client_version: ContentVersion,
}

impl SyncClock {
    pub fn new() -> Self {
        Self {
            server_version: ContentVersion::from_raw(0),
            client_version: ContentVersion::from_raw(0),
        }
    }

    pub fn from_wire(server_version: u64, client_version: u64) -> Self {
        Self {
            server_version: ContentVersion::from_raw(server_version as usize),
            client_version: ContentVersion::from_raw(client_version as usize),
        }
    }

    pub fn bump_server(&mut self) -> ContentVersion {
        self.server_version = ContentVersion::new();
        self.server_version
    }

    pub fn server_push_matches(&self, expected_client_version: ContentVersion) -> bool {
        self.client_version == expected_client_version
    }

    pub fn client_edit_matches(&self, expected_server_version: ContentVersion) -> bool {
        self.server_version == expected_server_version
    }
}

#[cfg(test)]
#[path = "buffer_location_tests.rs"]
mod tests;
