use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum TargetOS {
    #[serde(rename = "macos")]
    MacOS,
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "windows")]
    Windows,
    #[serde(rename = "web")]
    Web,

    #[serde(untagged)]
    Unknown(String),
}

impl TargetOS {
    pub fn current() -> Option<Self> {
        Some(TargetOS::MacOS)
    }

    /// Returns the name of the [`TargetOS`], or None if it is unknown.
    pub fn name(&self) -> Option<String> {
        let name = match self {
            TargetOS::MacOS => "MacOS".to_owned(),
            TargetOS::Linux => "Linux".to_owned(),
            TargetOS::Windows => "Windows".to_owned(),
            TargetOS::Web => "Web".to_owned(),
            _ => return None,
        };
        Some(name)
    }
}
