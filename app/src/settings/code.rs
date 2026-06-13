use settings::{macros::define_settings_group, SupportedPlatforms};

define_settings_group!(CodeSettings, settings: [
    code_as_default_editor: CodeAsDefaultEditor {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "code.editor.use_warp_as_default_editor",
        description: "Whether Warp is used as the default code editor.",
    }
    // Controls whether the project explorer / file tree appears in the tools panel.
    show_project_explorer: ShowProjectExplorer {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "code.editor.show_project_explorer",
        description: "Whether the project explorer is shown in the tools panel.",
    },
    // Controls whether global file search appears in the tools panel.
    show_global_search: ShowGlobalSearch {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "code.editor.show_global_search",
        description: "Whether global file search is shown in the tools panel.",
    },
    format_on_save: FormatOnSave {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        private: false,
        toml_path: "code.editor.format_on_save",
        description: "Whether the language server automatically formats files on save.",
    },
]);
