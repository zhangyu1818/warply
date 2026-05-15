//! Module containing operating system information such as the name, category, and version.

use serde::Serialize;
use serde_with::SerializeDisplay;
use std::fmt::{Display, Formatter};
use std::sync::OnceLock;

static OS_INFO: OnceLock<Result<OperatingSystemInfo, OperatingSystemInfoError>> = OnceLock::new();

#[derive(Serialize)]
pub struct OperatingSystemInfo {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    category: OperatingSystemCategory,
}

impl OperatingSystemInfo {
    fn new() -> Result<Self, OperatingSystemInfoError> {
        let os_category = OperatingSystemCategory::Mac;

        Ok(Self {
            name: os_category.to_string(),
            version: sysinfo::System::os_version(),
            category: os_category,
        })
    }

    pub fn get() -> Result<&'static Self, OperatingSystemInfoError> {
        let inner = OS_INFO.get_or_init(Self::new);
        inner.as_ref().map_err(|error| *error)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn category(&self) -> &OperatingSystemCategory {
        &self.category
    }
}

#[derive(SerializeDisplay, PartialEq)]
pub enum OperatingSystemCategory {
    Mac,
}

impl Display for OperatingSystemCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            OperatingSystemCategory::Mac => "macOS",
        };
        write!(f, "{str}")
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum OperatingSystemInfoError {
    #[error("unable to compute the operating system information")]
    Unknown,
}
