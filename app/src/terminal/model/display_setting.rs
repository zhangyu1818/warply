use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DisplaySetting {
    Command,
    Output,
    CommandAndOutput,
    Other(String),
}
