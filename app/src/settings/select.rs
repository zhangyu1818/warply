use std::ops::Not;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use warpui::{AppContext, clipboard::ClipboardContent};

use settings::{Setting, SupportedPlatforms, macros::define_settings_group};

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Default,
    JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "What a bare right-click does in the terminal.",
    rename_all = "snake_case"
)]
pub enum RightClickBehavior {
    #[default]
    /// Right-click opens the context menu.
    ContextMenu,
    /// Right-click pastes from the clipboard. Shift+right-click opens the context menu instead.
    Paste,
}

impl RightClickBehavior {
    pub fn as_dropdown_label(&self) -> &str {
        match self {
            Self::ContextMenu => "Open the context menu",
            Self::Paste => "Paste from the clipboard",
        }
    }
}

define_settings_group!(SelectionSettings, settings: [
    copy_on_select: CopyOnSelect {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "terminal.copy_on_select",
        description: "Whether text is automatically copied to the clipboard when selected.",
    },
    middle_click_paste_enabled: MiddleClickPasteEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::MAC,
        private: false,
        toml_path: "terminal.input.middle_click_paste_enabled",
        description: "Whether middle-click pastes from the clipboard.",
    },
    right_click_behavior: RightClickBehaviorSetting {
        type: RightClickBehavior,
        default: RightClickBehavior::ContextMenu,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "terminal.input.right_click_behavior",
        description: "What a bare right-click does in the terminal.",
    }
]);

impl SelectionSettings {
    pub fn copy_on_select_enabled(&self) -> bool {
        *self.copy_on_select.value()
    }

    pub fn right_click_pastes(&self) -> bool {
        *self.right_click_behavior.value() == RightClickBehavior::Paste
    }

    /// Writes the selection content to the user's clipboard if `copy_on_select` is enabled.
    pub fn maybe_copy_on_select(&self, clipboard_content: ClipboardContent, ctx: &mut AppContext) {
        if self.copy_on_select_enabled() && !clipboard_content.plain_text.is_empty() {
            ctx.clipboard().write(clipboard_content);
        }
    }

    /// Implements the correct middle-click paste behavior for the current platform.
    pub fn read_for_middle_click_paste(&self, ctx: &mut AppContext) -> Option<ClipboardContent> {
        (self
            .middle_click_paste_enabled
            .is_supported_on_current_platform()
            && *self.middle_click_paste_enabled.value())
        .then(|| ctx.clipboard().read())
        .filter(|content| content.is_empty().not())
    }
}
