use crate::terminal::block_list_viewport::InputMode;
use settings::{Setting, SupportedPlatforms, macros::define_settings_group};

define_settings_group!(InputModeSettings, settings: [
    input_mode: InputModeState {
        type: InputMode,
        default: InputMode::PinnedToBottom,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        storage_key: "InputMode",
        toml_path: "appearance.input.input_mode",
        description: "The position of the terminal input.",
    },
]);

impl InputModeSettings {
    pub fn is_pinned_to_top(&self) -> bool {
        *self.input_mode.value() == InputMode::PinnedToTop
    }
}
