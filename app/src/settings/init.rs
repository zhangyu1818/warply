use settings::Setting as _;
use warpui::{rendering::GPUPowerPreference, AppContext, SingletonEntity};
use warpui_extras::user_preferences;

use crate::{
    appearance,
    banner::BannerState,
    resource_center::TipsCompleted,
    search::command_search::settings::CommandSearchSettings,
    terminal::{
        alt_screen_reporting::AltScreenReporting,
        general_settings::GeneralSettings,
        keys_settings::KeysSettings,
        ligature_settings::LigatureSettings,
        safe_mode_settings::SafeModeSettings,
        session_settings::{SessionSettings, SessionSettingsChangedEvent},
        settings::TerminalSettings,
        warpify::settings::WarpifySettings,
        BlockListSettings,
    },
    undo_close::UndoCloseSettings,
    window_settings::WindowSettings,
    workflows::aliases::WorkflowAliases,
    workspace::tab_settings::TabSettings,
};

use warp_core::semantic_selection::SemanticSelection;

use super::{
    app_icon::AppIconSettings, AISettings, AccessibilitySettings, AliasExpansionSettings,
    AppEditorSettings, BlockVisibilitySettings, CodeSettings, DebugSettings, FontSettings,
    FontSettingsChangedEvent, GPUSettings, InputBoxType, InputModeSettings, InputSettings,
    PaneSettings, ScrollSettings, SelectionSettings, ThemeSettings, VimBannerSettings,
};

pub struct UserDefaultsOnStartup {
    pub should_restore_session: bool,
    pub tips_data: TipsCompleted,
    pub user_default_shell_unsupported_banner_state: BannerState,
    pub settings_file_error: Option<super::SettingsFileError>,
}

/// Registers all settings groups with the application context.
///
/// This populates the `SettingsManager` with storage keys, default values,
/// and hierarchy info for every setting. It does not set up appearance,
/// rendering config, or event subscriptions.
pub fn register_all_settings(ctx: &mut AppContext) {
    BlockListSettings::register(ctx);
    BlockVisibilitySettings::register(ctx);
    DebugSettings::register(ctx);
    SessionSettings::register(ctx);
    KeysSettings::register(ctx);
    FontSettings::register(ctx);
    TabSettings::register(ctx);
    WindowSettings::register(ctx);
    SafeModeSettings::register(ctx);
    TerminalSettings::register(ctx);
    PaneSettings::register(ctx);
    CommandSearchSettings::register(ctx);
    AliasExpansionSettings::register(ctx);
    CodeSettings::register(ctx);
    LigatureSettings::register(ctx);
    GPUSettings::register(ctx);
    GeneralSettings::register(ctx);
    AISettings::register_and_subscribe_to_events(ctx);
    ScrollSettings::register(ctx);
    SelectionSettings::register(ctx);
    InputModeSettings::register(ctx);
    ThemeSettings::register(ctx);
    AccessibilitySettings::register(ctx);
    AppIconSettings::register(ctx);
    AppEditorSettings::register(ctx);
    InputSettings::register(ctx);
    WarpifySettings::register(ctx);
    AltScreenReporting::register(ctx);
    UndoCloseSettings::register(ctx);
    VimBannerSettings::register(ctx);
    WorkflowAliases::register(ctx);
    SemanticSelection::register(ctx);

    #[cfg(feature = "local_fs")]
    crate::util::file::external_editor::EditorSettings::register(ctx);
}

pub fn init(
    startup_toml_parse_error: Option<user_preferences::Error>,
    ctx: &mut AppContext,
) -> UserDefaultsOnStartup {
    register_all_settings(ctx);

    let use_thin_strokes = *FontSettings::as_ref(ctx).use_thin_strokes;

    let general_settings = GeneralSettings::as_ref(ctx);
    let tips_features_used = general_settings.welcome_tips_features_used.clone();
    let tips_skipped_or_completed = *general_settings.welcome_tips_skipped_or_completed;
    let user_default_shell_unsupported_banner_state =
        *general_settings.user_default_shell_unsupported_banner_state;
    let should_restore_session = *general_settings.restore_session;

    // Validate all public settings to detect values that parsed as TOML
    // but cannot be deserialized into the expected Rust types.
    let invalid_setting_keys =
        settings::SettingsManager::as_ref(ctx).validate_all_public_settings(ctx);
    let settings_file_error = if let Some(err) = startup_toml_parse_error {
        Some(super::SettingsFileError::FileParseFailed(err.to_string()))
    } else if !invalid_setting_keys.is_empty() {
        Some(super::SettingsFileError::InvalidSettings(
            invalid_setting_keys,
        ))
    } else {
        None
    };

    let user_defaults_on_startup = UserDefaultsOnStartup {
        should_restore_session,
        tips_data: TipsCompleted::new(tips_features_used, tips_skipped_or_completed),
        user_default_shell_unsupported_banner_state,
        settings_file_error,
    };

    let gpu_settings = GPUSettings::as_ref(ctx);
    let prefer_low_power_gpu = *gpu_settings.prefer_low_power_gpu.value();

    // Update the rendering config.
    ctx.update_rendering_config(|config| {
        config.glyphs.use_thin_strokes = use_thin_strokes;
        config.gpu_power_preference = if prefer_low_power_gpu {
            GPUPowerPreference::LowPower
        } else {
            GPUPowerPreference::default()
        };
    });

    ctx.subscribe_to_model(&FontSettings::handle(ctx), |font_settings, event, ctx| {
        if matches!(event, FontSettingsChangedEvent::UseThinStrokes { .. }) {
            let use_thin_strokes = *font_settings.as_ref(ctx).use_thin_strokes;
            ctx.update_rendering_config(|config| {
                config.glyphs.use_thin_strokes = use_thin_strokes;
            });
        }
    });

    // Keep input_box_type in sync whenever honor_ps1 changes —
    // Classic when PS1 is honored, Universal otherwise.
    ctx.subscribe_to_model(
        &SessionSettings::handle(ctx),
        |session_settings, event, ctx| {
            if let SessionSettingsChangedEvent::HonorPS1 { .. } = event {
                let new_honor_ps1 = *session_settings.as_ref(ctx).honor_ps1;
                let new_type = if new_honor_ps1 {
                    InputBoxType::Classic
                } else {
                    InputBoxType::Universal
                };
                InputSettings::handle(ctx).update(ctx, |input_settings, ctx| {
                    let _ = input_settings.input_box_type.set_value(new_type, ctx);
                });
            }
        },
    );

    appearance::register(ctx);

    // Set up hot-reload for the settings file. When the WarpConfig watcher
    // detects a change to settings.toml, reload preferences from disk and
    // push changed values into setting models.
    #[cfg(feature = "local_fs")]
    {
        let prefs = <settings::PublicPreferences as warpui::SingletonEntity>::as_ref(ctx);
        if prefs.is_settings_file() {
            ctx.subscribe_to_model(
                &crate::user_config::WarpConfig::handle(ctx),
                handle_warp_config_change,
            );
        }
    }

    user_defaults_on_startup
}

/// Handles a `WarpConfig` change event, reloading settings from disk when
/// the settings file is modified, created, or deleted.
#[cfg(feature = "local_fs")]
fn handle_warp_config_change(
    _: warpui::ModelHandle<crate::user_config::WarpConfig>,
    event: &crate::user_config::WarpConfigUpdateEvent,
    ctx: &mut AppContext,
) {
    use crate::user_config::{WarpConfig, WarpConfigUpdateEvent};

    if !matches!(event, WarpConfigUpdateEvent::Settings) {
        return;
    }
    let prefs = <settings::PublicPreferences as warpui::SingletonEntity>::as_ref(ctx);
    if let Err(err) = prefs.reload_from_disk() {
        log::warn!("Settings file reload failed: {err}");
        WarpConfig::handle(ctx).update(ctx, |_, ctx| {
            ctx.emit(WarpConfigUpdateEvent::SettingsErrors(
                super::SettingsFileError::FileParseFailed(err.to_string()),
            ));
        });
        return;
    }
    let failed_keys = settings::SettingsManager::handle(ctx)
        .update(ctx, |manager, ctx| manager.reload_all_public_settings(ctx));
    WarpConfig::handle(ctx).update(ctx, |_, ctx| {
        if failed_keys.is_empty() {
            ctx.emit(WarpConfigUpdateEvent::SettingsErrorsCleared);
        } else {
            ctx.emit(WarpConfigUpdateEvent::SettingsErrors(
                super::SettingsFileError::InvalidSettings(failed_keys),
            ));
        }
    });
}
/// Returns the platform-native preferences backend.
///
/// Used directly for private settings.
fn init_platform_native_preferences() -> user_preferences::Model {
    cfg_if::cfg_if! {
        if #[cfg(test)] {
            Box::<user_preferences::in_memory::InMemoryPreferences>::default()
        } else if #[cfg(feature = "integration_tests")] {
            match user_preferences::file_backed::FileBackedUserPreferences::new(super::user_preferences_file_path()) {
                Ok(prefs) => Box::new(prefs) as user_preferences::Model,
                Err(_) => {
                    Box::<user_preferences::in_memory::InMemoryPreferences>::default()
                }
            }
        } else {
            Box::new(user_preferences::user_defaults::UserDefaultsPreferencesStorage::new(
                warp_core::channel::ChannelState::data_domain_if_not_default()
            ))
        }
    }
}

/// Creates the platform-native preferences backend for private settings.
///
/// Private settings are always stored in the platform-native store (e.g.
/// UserDefaults on macOS) and never appear in the user-visible TOML file.
pub fn init_private_user_preferences() -> settings::PrivatePreferences {
    settings::PrivatePreferences::new(init_platform_native_preferences())
}

/// Initializes the public UserPreferences provider.
///
/// Public settings are stored in `settings.toml` so they are user-visible and
/// editable.
/// Returns `(preferences_backend, optional_parse_error)`. The parse error
/// is `Some` only when the TOML settings file existed but could not be
/// parsed; it should be propagated to the UI so the user sees a banner.
pub fn init_public_user_preferences() -> (user_preferences::Model, Option<user_preferences::Error>)
{
    cfg_if::cfg_if! {
        if #[cfg(test)] {
            (Box::<user_preferences::in_memory::InMemoryPreferences>::default(), None)
        } else {
            let (prefs, parse_error) =
                user_preferences::toml_backed::TomlBackedUserPreferences::new(
                    super::user_preferences_toml_file_path(),
                );
            if let Some(err) = &parse_error {
                log::warn!("Settings file has syntax errors and could not be parsed: {err}");
            }
            (Box::new(prefs) as user_preferences::Model, parse_error)
        }
    }
}

#[cfg(test)]
pub fn init_and_register_user_preferences(ctx: &mut AppContext) {
    let (public_prefs, _parse_error) = init_public_user_preferences();
    ctx.add_singleton_model(move |_| settings::PublicPreferences::new(public_prefs));
    ctx.add_singleton_model(move |_| init_private_user_preferences());
}
