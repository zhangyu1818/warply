use std::ops::Not;

use warpui::{AppContext, clipboard::ClipboardContent};

use settings::{Setting, SupportedPlatforms, macros::define_settings_group};

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
    }
]);

impl SelectionSettings {
    pub fn copy_on_select_enabled(&self) -> bool {
        *self.copy_on_select.value()
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
