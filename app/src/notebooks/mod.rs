pub mod editor;
pub mod events;
pub mod file;
pub mod link;
mod styles;

use serde::{Deserialize, Serialize};
use warpui::AppContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum NotebookLocation {
    LocalFile,
    RemoteFile,
}

/// Initialize notebooks-related keybindings.
pub fn init(app: &mut AppContext) {
    self::editor::view::init(app);
    self::file::init(app);
}
