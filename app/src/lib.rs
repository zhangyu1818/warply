// Suppress warnings about rustdoc style.
#![allow(clippy::doc_lazy_continuation)]

mod ai;
mod alloc;
mod app_menus;
mod app_services;
mod app_state;
mod banner;
mod chip_configurator;
mod cloud_object;
mod code;
mod code_review;
mod coding_panel_enablement_state;
mod command_palette;
mod completer;
#[allow(dead_code)]
mod context_chips;
mod datetime_ext;
mod debounce;
mod default_terminal;
mod drive;
mod env_vars;
mod external_secrets;
mod global_resource_handles;
mod gpu_state;
mod http_api;
mod identity;
mod input_classifier;
mod interval_timer;
mod linear;
mod login_item;
mod menu;
mod modal;
mod notebooks;
mod notification;
mod object_ids;
mod palette;
mod persistence;
#[cfg(feature = "plugin_host")]
mod plugin;
mod prefix;
mod profiling;
mod projects;
mod prompt;
mod quit_warning;
#[allow(dead_code)]
mod remote_server;
mod resource_limits;
mod safe_triangle;
mod search_bar;
mod session_management;
mod shell_indicator;
mod suggestions;
mod system;
mod tab;
#[cfg(test)]
mod test_util;
mod throttle;
mod tips;
mod tracing;
mod ui_components;
mod ui_events;
mod undo_close;
mod updater;
mod uri;
mod user_config;
pub mod util;
mod view_components;
mod vim_registers;
mod voltron;
mod warp_managed_paths_watcher;
mod window_settings;

// PLEASE DO NOT ADD MORE PUBLIC MODULES!
//
// Any modules which we make public outside of the `warp` crate lose dead code
// checking support, as the compiler cannot make any assumptions about whether
// or not the function/type is used by another crate that pulls in this one as
// a dependency.
//
// If you feel the need to export a module so that a type or function within it
// can be used by an integration test, you should define a new assertion function
// in the warp::integration_testing::assertions module (or a sub-module).  These
// functions will allow us to keep types internal to this crate and expose a
// simpler API for integration tests to consume.
pub mod appearance;
pub mod channel;
pub mod editor;
pub mod features;
pub mod input_suggestions;
#[cfg(feature = "integration_tests")]
pub mod integration_testing;
pub mod keyboard;
pub mod launch_configs;
pub mod pane_group;
pub mod resource_center;
pub mod root_view;
pub mod search;
pub mod settings;
pub mod settings_view;
pub mod tab_configs;
pub mod terminal;
pub mod themes;
use crate::ai::active_agent_views_model::ActiveAgentViewsModel;
use ::ai::project_context::model::ProjectContextModel;
pub use ai::agent::{todos::AIAgentTodoList, AIAgentActionResultType, FileEdit, TodoOperation};
use ai::agent_conversations_model::AgentConversationsModel;
use ai::blocklist::{BlocklistAIHistoryModel, BlocklistAIPermissions};
use ai::execution_profiles::editor::ExecutionProfileEditorManager;
use ai::execution_profiles::profiles::AIExecutionProfilesModel;
use ai::persisted_workspace::PersistedWorkspace;
use code::editor_management::CodeManager;
use code::opened_files::OpenedFilesModel;
use code_review::GlobalCodeReviewModel;
use identity::local_identity::LocalIdentity;
use identity::local_identity::LocalIdentityProvider;
use quit_warning::UnsavedStateSummary;
#[cfg(feature = "local_fs")]
use settings::import::model::ImportedConfigModel;

#[cfg(feature = "local_fs")]
use repo_metadata::{
    repositories::DetectedRepositories, watcher::DirectoryWatcher, RepoMetadataModel,
};
#[cfg(feature = "local_fs")]
use watcher::HomeDirectoryWatcher;

use settings_view::pane_manager::SettingsPaneManager;
use terminal::general_settings::GeneralSettings;
use terminal::keys_settings::KeysSettings;
#[cfg(feature = "local_tty")]
use terminal::local_shell::LocalShellState;
pub use util::bindings::cmd_or_ctrl_shift;
pub mod workflows;
pub mod workspace;

#[cfg(feature = "integration_tests")]
pub use persistence::testing as sqlite_testing;

use ::settings::{Setting, ToggleableSetting};

#[cfg(feature = "plugin_host")]
pub use plugin::{run_plugin_host, PLUGIN_HOST_FLAG};
use warpui::platform::app::ApproveTerminateResult;
use window_settings::WindowSettings;
use workflows::manager::WorkflowManager;

use crate::ai::document::ai_document_model::AIDocumentModel;
use crate::ai::facts::manager::AIFactManager;
use crate::ai::outline::RepoOutlines;
use crate::ai::restored_conversations::RestoredAgentConversations;
use crate::cloud_object::model::actions::ObjectActions;
use crate::cloud_object::update_manager::UpdateManager;
use crate::code::global_buffer_model::GlobalBufferModel;
#[cfg(feature = "local_fs")]
use crate::code::language_server_shutdown_manager::LanguageServerShutdownManager;
use crate::context_chips::prompt::Prompt;
use crate::default_terminal::DefaultTerminal;
use crate::env_vars::manager::EnvVarCollectionManager;
use crate::gpu_state::GPUState;
use crate::notebooks::editor::keys::NotebookKeybindings;
use crate::palette::PaletteMode;
use crate::persistence::PersistenceWriter;
use crate::projects::ProjectManagementModel;
use crate::session_management::{RunningSessionSummary, SessionNavigationData};
use crate::settings::manager::SettingsManager;
use crate::settings::{
    log_setting_result, AccessibilitySettings, ScrollSettings, SelectionSettings,
};
use crate::settings_view::keybindings::KeybindingChangedNotifier;
use crate::settings_view::DisplayCount;
use crate::suggestions::ignored_suggestions_model::IgnoredSuggestionsModel;
use crate::system::SystemStats;
use crate::terminal::cli_agent_sessions::CLIAgentSessionsModel;
use crate::terminal::keys::TerminalKeybindings;
use crate::terminal::resizable_data::ResizableData;
use crate::terminal::{AudibleBell, History};
use crate::undo_close::UndoCloseStack;
use crate::updater::WarplyUpdater;
use crate::user_config::WarpConfig;
use crate::vim_registers::VimRegisters;
use crate::warp_managed_paths_watcher::{ensure_warp_watch_roots_exist, WarpManagedPathsWatcher};
use crate::workflows::aliases::WorkflowAliases;
use crate::workflows::local_workflows::LocalWorkflows;
use crate::workspace::{ActiveSession, ToastStack};
#[cfg(feature = "local_tty")]
use anyhow::Context;
use anyhow::{anyhow, Result};
use appearance::{Appearance, AppearanceManager};
use channel::ChannelState;
use http_api::HttpApiProvider;
use interval_timer::IntervalTimer;
use itertools::Itertools;
use rust_embed::RustEmbed;
use settings::ExtraMetaKeys;
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::Arc;
use terminal::input;
use terminal::session_settings::SessionSettings;
use url::Url;
use warp_core::execution_mode::{AppExecutionMode, ExecutionMode};
use workspace::sync_inputs::SyncedInputState;

use warpui::{integration::TestDriver, App, AssetProvider, Event};

use self::features::FeatureFlag;
use crate::app_state::AppState;
use crate::cloud_object::model::persistence::CloudModel;
use crate::drive::CloudObjectTypeAndId;
pub use crate::global_resource_handles::{GlobalResourceHandles, GlobalResourceHandlesProvider};
use crate::notification::NotificationContext;
use crate::root_view::{
    quake_mode_window_id, quake_mode_window_is_open, OpenFromRestoredArg, OpenPath,
};
use crate::ui_events::PaletteSource;
use crate::util::bindings::is_binding_supported_on_mac;
use crate::workspace::{PaneViewLocator, Workspace, WorkspaceAction};
use warp_logging::LogDestination;

#[cfg(feature = "local_fs")]
use warp_files::FileModel;
use warpui::platform::TerminationMode;
use warpui::windowing::state::ApplicationStage;
use warpui::{AppContext, SingletonEntity, WindowId};

#[derive(Clone, Copy, RustEmbed)]
#[folder = "assets"]
#[include = "bundled/**"] // Should be kept in sync with BUNDLED_ASSETS_DIR.
#[include = "async/**"]
// Should be kept in sync with ASYNC_ASSETS_DIR.
// Excludes take precedence.
// Standalone CLI builds are headless and never render the async image set, so
// we exclude those bytes to keep the CLI binary small.
#[cfg_attr(feature = "standalone", exclude = "async/**")]
pub struct Assets;

pub static ASSETS: Assets = Assets;

/// Launch mode for how to start up Warp.
#[allow(clippy::large_enum_variant)]
pub enum LaunchMode {
    /// Run the regular GUI application.
    App { args: warp_cli::AppArgs },

    /// Run a test - this may be an integration test or an eval.
    Test {
        driver: Box<Option<TestDriver>>,
        is_integration_test: bool,
    },

    /// Remote server proxy — bridges SSH stdio to the daemon's Unix socket.
    /// This is a short-lived process that runs for the lifetime of an SSH session.
    RemoteServerProxy,

    /// Remote server daemon — long-lived headless process serving remote
    /// connections via a Unix domain socket.
    RemoteServerDaemon {
        /// Stable identity key used to partition the daemon's socket/PID
        /// directory on the remote host.
        identity_key: String,
    },
}

impl LaunchMode {
    fn args(&self) -> Cow<'_, warp_cli::AppArgs> {
        match self {
            LaunchMode::App { args, .. } => Cow::Borrowed(args),
            LaunchMode::Test { .. }
            | LaunchMode::RemoteServerProxy
            | LaunchMode::RemoteServerDaemon { .. } => Cow::Owned(warp_cli::AppArgs::default()),
        }
    }

    /// Returns `true` if this process is running an integration test.
    fn is_integration_test(&self) -> bool {
        match self {
            LaunchMode::Test {
                is_integration_test,
                ..
            } => *is_integration_test,
            LaunchMode::App { .. }
            | LaunchMode::RemoteServerProxy
            | LaunchMode::RemoteServerDaemon { .. } => false,
        }
    }

    fn take_test_driver(&mut self) -> Option<TestDriver> {
        match self {
            LaunchMode::Test { driver, .. } => driver.take(),
            LaunchMode::App { .. }
            | LaunchMode::RemoteServerProxy
            | LaunchMode::RemoteServerDaemon { .. } => None,
        }
    }

    /// Add an URL to open. Only supported for [`LaunchMode::App`]
    #[allow(dead_code)]
    fn add_url(&mut self, url: Url) {
        if let LaunchMode::App { ref mut args, .. } = self {
            args.urls.push(url);
        }
    }

    fn execution_mode(&self) -> ExecutionMode {
        match self {
            LaunchMode::App { .. } => ExecutionMode::App,
            LaunchMode::Test { .. } => ExecutionMode::App,
            LaunchMode::RemoteServerProxy | LaunchMode::RemoteServerDaemon { .. } => {
                ExecutionMode::Headless
            }
        }
    }

    /// Returns `true` if Warp should run headlessly, without a visible UI.
    fn is_headless(&self) -> bool {
        match self {
            LaunchMode::RemoteServerProxy | LaunchMode::RemoteServerDaemon { .. } => true,
            LaunchMode::App { .. } | LaunchMode::Test { .. } => false,
        }
    }

    /// Whether profiling and tracing should be initialized.
    pub(crate) fn needs_profiling(&self) -> bool {
        match self {
            LaunchMode::App { .. }
            | LaunchMode::Test { .. }
            | LaunchMode::RemoteServerDaemon { .. }
            | LaunchMode::RemoteServerProxy => true,
        }
    }

    /// Log destination for this mode.
    fn log_destination(&self) -> Option<LogDestination> {
        match self {
            // Proxy must log to stderr because stdout is the protocol channel.
            LaunchMode::RemoteServerProxy => Some(LogDestination::Stderr),
            LaunchMode::RemoteServerDaemon { .. } => Some(LogDestination::File),
            LaunchMode::App { .. } | LaunchMode::Test { .. } => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_unit_test() -> Self {
        LaunchMode::Test {
            driver: Box::new(None),
            is_integration_test: false,
        }
    }
}

impl AssetProvider for Assets {
    fn get(&self, path: &str) -> Result<Cow<'_, [u8]>> {
        <Assets as RustEmbed>::get(path)
            .map(|f| f.data)
            .ok_or_else(|| anyhow!("no asset exists at path {}", path))
    }
}

/// If the given event is a key down event containing alt modifiers, and those
/// alt modifiers should be treated as meta keys, then remove the alts and
/// prefix the keys with an escape. See WAR-472.
fn apply_extra_meta_keys(event: &mut Event, extra_metas: ExtraMetaKeys) {
    if let Event::KeyDown {
        keystroke, details, ..
    } = event
    {
        let left_as_meta = extra_metas.left_alt && details.left_alt;
        let right_as_meta = extra_metas.right_alt && details.right_alt;
        if left_as_meta || right_as_meta {
            let side = match (left_as_meta, right_as_meta) {
                (true, true) => "left+right alt",
                (true, false) => "left alt",
                (false, true) => "right alt",
                (false, false) => unreachable!(),
            };
            log::info!("Treating {side} as meta");
            keystroke.alt = false;
            keystroke.meta = true;
        }
    }
}

fn apply_scroll_multiplier(event: &mut Event, app: &AppContext) {
    if let Event::ScrollWheel { delta, precise, .. } = event {
        if !*precise {
            let scroll_multiplier = *ScrollSettings::as_ref(app).mouse_scroll_multiplier.value();
            *delta *= scroll_multiplier;
        }
    }
}

/// Runs the app. If a subcommand was requested, it'll be run instead of the main application.
pub fn run() -> Result<()> {
    // Ensure feature flags are initialized before parsing command-line arguments.
    init_feature_flags();

    // Parse command-line arguments.
    let args = warp_cli::Args::from_env();

    if let Some(command) = args.command() {
        match command {
            warp_cli::Command::Worker(warp_cli::WorkerCommand::TerminalServer(args)) => {
                // If we were asked to run as a terminal server (as opposed to the main
                // GUI application), do so immediately.  Ideally, the terminal server would
                // be a separate binary, but it's much easier to distribute a single binary,
                // so starting the terminal server event loop immediately is the closest
                // approximation we can get to running a separate binary.
                crate::terminal::local_tty::server::run_terminal_server(args);
                return Ok(());
            }
            #[cfg(feature = "plugin_host")]
            warp_cli::Command::Worker(warp_cli::WorkerCommand::PluginHost { .. }) => {
                return crate::run_plugin_host();
            }
            warp_cli::Command::Worker(warp_cli::WorkerCommand::RemoteServerProxy(args)) => {
                // Proxy is a thin byte bridge (stdin/stdout ↔ Unix socket).
                // It only needs logging to stderr since stdout is the protocol
                // channel. No initialize_app.
                let launch_mode = LaunchMode::RemoteServerProxy;
                warp_logging::init(warp_logging::LogConfig {
                    is_cli: true,
                    log_destination: launch_mode.log_destination(),
                })?;
                return crate::remote_server::run_proxy(args.identity_key.clone());
            }
            warp_cli::Command::Worker(warp_cli::WorkerCommand::RemoteServerDaemon(args)) => {
                // Daemon handles its own full initialization inside run_daemon_app.
                return crate::remote_server::run_daemon(args.identity_key.clone());
            }
            warp_cli::Command::Worker(warp_cli::WorkerCommand::RipgrepSearch {
                parent,
                ignore_case,
                multiline,
                pattern,
                paths,
            }) => {
                warp_ripgrep::search::run_search_subprocess(
                    std::slice::from_ref(pattern),
                    paths.clone(),
                    *ignore_case,
                    *multiline,
                    parent.pid,
                )
                .map_err(|err| anyhow!(err.to_string()))?;
                return Ok(());
            }
            #[cfg(not(any(feature = "local_tty", feature = "plugin_host")))]
            warp_cli::Command::Worker(worker) => {
                // Need this case to handle platforms where there are no enum variants in
                // warp_cli::WorkerCommand, as we still need to check Command::Worker.
                panic!("Worker process not supported: {worker:?}")
            }
            warp_cli::Command::Completions { shell } => {
                return warp_cli::completions::generate_to_stdout(*shell);
            }
        }
    }

    if should_print_cli_help_without_command(std::env::var_os("WARPLY_CLI_MODE").is_some()) {
        warp_cli::Args::clap_command().print_help()?;
        return Ok(());
    }

    run_internal(LaunchMode::App {
        args: args.into_app_args(),
    })
}

fn should_print_cli_help_without_command(cli_mode_env_set: bool) -> bool {
    cfg!(feature = "standalone") || cli_mode_env_set
}

#[cfg(test)]
mod launch_mode_tests {
    use super::should_print_cli_help_without_command;

    #[cfg(not(feature = "standalone"))]
    #[test]
    fn gui_build_without_cli_mode_does_not_print_cli_help() {
        assert!(!should_print_cli_help_without_command(false));
    }

    #[test]
    fn explicit_cli_mode_prints_cli_help() {
        assert!(should_print_cli_help_without_command(true));
    }
}

/// Runs an integration test using the provided test driver.
pub fn run_integration_test(driver: TestDriver) -> Result<()> {
    let is_integration_test = std::env::var("WARP_INTEGRATION").is_ok();
    let launch = LaunchMode::Test {
        driver: Box::new(Some(driver)),
        is_integration_test,
    };
    run_internal(launch)
}

/// Runs the app (or CLI / daemon).
fn run_internal(mut launch_mode: LaunchMode) -> Result<()> {
    let mut timer = IntervalTimer::new();

    // ── Early initialization (pre-AppBuilder) ──────────────────────
    // These steps run before the platform event loop is started.
    // They must not depend on AppContext.

    if launch_mode.needs_profiling() {
        profiling::init();
    }

    // The `run` function already initializes feature flags, but ensure they're initialized here
    // for other entrypoints.
    init_feature_flags();

    if launch_mode.needs_profiling() {
        tracing::init()?;
    }

    let log_destination = launch_mode.log_destination();
    let is_cli = log_destination.is_some();

    warp_logging::init(warp_logging::LogConfig {
        is_cli,
        log_destination,
    })?;

    timer.mark_interval_end("LOG_FILE_SETUP_COMPLETE");

    // Adjust resource limits early, before doing other work, to ensure that
    // any children we spawn (like the terminal server) inherit our adjusted
    // rlimits.
    resource_limits::adjust_resource_limits();

    // Configure rustls to use its default crypto provider.  This MUST be called
    // before making any network requests that use TLS, otherwise rustls will
    // panic.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("must be able to initialize crypto provider for TLS support");

    // Collect errors that occur before the app is initialized.
    let pre_app_errors: Vec<anyhow::Error> = Vec::new();

    let private_preferences = settings::init_private_user_preferences();
    let (public_preferences, startup_toml_parse_error) = settings::init_public_user_preferences();

    // Set up the pty spawner before doing any meaningful work. We want to
    // ensure that the process is in the cleanest possible state (minimal opened
    // files, modified signal handlers, etc.) to avoid unexpected effects on
    // spawned ptys.
    #[cfg(feature = "local_tty")]
    let pty_spawner =
        terminal::local_tty::spawner::PtySpawner::new().context("Failed to create pty spawner")?;

    let mut app_builder = if launch_mode.is_headless() {
        warpui::platform::AppBuilder::new_headless(
            app_callbacks(launch_mode.is_integration_test()),
            Box::new(ASSETS),
            launch_mode.take_test_driver(),
        )
    } else {
        warpui::platform::AppBuilder::new(
            app_callbacks(launch_mode.is_integration_test()),
            Box::new(ASSETS),
            launch_mode.take_test_driver(),
        )
    };

    {
        use warpui::platform::mac::AppExt;

        let activate_on_launch = !launch_mode.is_integration_test()
            || std::env::var("WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS").is_ok();
        app_builder.set_activate_on_launch(activate_on_launch);

        let dev_icon = ASSETS.get("bundled/png/local.png")?;
        app_builder.set_dev_icon(dev_icon);

        app_builder.set_menu_bar_builder(app_menus::menu_bar);
        app_builder.set_dock_menu_builder(|_| app_menus::dock_menu());
    }

    app_builder.register_default_keystroke_triggers_for_custom_actions(
        crate::util::bindings::custom_tag_to_keystroke,
    );

    app_builder.run(move |ctx| {
        // Rotate the log files in the background.
        ctx.background_executor()
            .spawn(warp_logging::rotate_log_files())
            .detach();

        ctx.add_singleton_model(|ctx| AppExecutionMode::new(launch_mode.execution_mode(), ctx));
        // Add the terminal server singleton to the application.
        #[cfg(feature = "local_tty")]
        ctx.add_singleton_model(move |_ctx| pty_spawner);

        // Register user preferences before initializing feature flags.
        ctx.add_singleton_model(move |_ctx| ::settings::PublicPreferences::new(public_preferences));
        ctx.add_singleton_model(move |_ctx| private_preferences);
        let startup_toml_parse_error = startup_toml_parse_error;

        #[cfg(feature = "plugin_host")]
        ctx.add_singleton_model(move |ctx| {
            plugin::PluginHost::new(ctx).expect("Could not instantiate PluginHost")
        });
        let app_state = initialize_app(
            &launch_mode,
            timer,
            startup_toml_parse_error,
            ctx,
            pre_app_errors,
        );

        FeatureFlag::UseTantivySearch.set_enabled(true);

        launch(ctx, app_state, launch_mode);
    })
}

pub struct UpdateQuakeModeEventArg {
    active_window_id: Option<WindowId>,
}

pub(crate) fn initialize_app(
    launch_mode: &LaunchMode,
    mut timer: IntervalTimer,
    startup_toml_parse_error: Option<warpui_extras::user_preferences::Error>,
    ctx: &mut warpui::AppContext,
    _pre_app_errors: impl IntoIterator<Item = anyhow::Error>,
) -> Option<AppState> {
    let data_domain = ChannelState::data_domain();

    // Register an implementation of the secure storage service.
    cfg_if::cfg_if! {
        if #[cfg(feature = "integration_tests")] {
            warpui_extras::secure_storage::register_noop(&data_domain, ctx);
        } else {
            warpui_extras::secure_storage::register(&data_domain, ctx);
        }
    }

    ensure_warp_watch_roots_exist();
    ctx.add_singleton_model(WarpManagedPathsWatcher::new);

    ctx.add_singleton_model(WarpConfig::new);
    ctx.add_singleton_model(|_ctx| SettingsManager::default());

    let user_defaults_on_startup = settings::init(startup_toml_parse_error, ctx);
    timer.mark_interval_end("READ_USER_DEFAULTS_AND_INITIALIZE_SETTINGS");

    if FeatureFlag::UIZoom.is_enabled() {
        ctx.set_zoom_factor(WindowSettings::as_ref(ctx).zoom_level.as_zoom_factor());
    }

    let local_identity = Arc::new(LocalIdentity::initialize(ctx));
    timer.mark_interval_end("LOCAL_IDENTITY_INITIALIZED");

    ctx.add_singleton_model(HttpApiProvider::new);

    ctx.add_singleton_model(|_ctx| LocalIdentityProvider::new(local_identity.clone()));

    ctx.add_singleton_model(|_ctx| GPUState::new());

    // If any part of sqlite initialization fails, we just don't do session restoration (i.e.
    // feature degradation).
    let (sqlite_data, writer_handles) = persistence::initialize(ctx);
    timer.mark_interval_end("SQLITE_INITIALIZED");

    let persistence_writer = PersistenceWriter::new(writer_handles);

    let model_event_sender = persistence_writer.sender();

    let tips_handle = ctx.add_model(|_| user_defaults_on_startup.tips_data);
    let user_default_shell_unsupported_banner_model_handle =
        ctx.add_model(|_| user_defaults_on_startup.user_default_shell_unsupported_banner_state);
    let settings_file_error = user_defaults_on_startup.settings_file_error;
    ctx.add_singleton_model(move |_ctx| {
        GlobalResourceHandlesProvider::new(GlobalResourceHandles {
            model_event_sender,
            tips_completed: tips_handle,
            user_default_shell_unsupported_banner_model_handle,
            settings_file_error,
        })
    });

    let (
        cloud_objects,
        app_state,
        command_history,
        object_actions,
        ai_queries,
        persisted_workspaces,
        workspace_language_servers,
        agent_conversations,
        persisted_projects,
        persisted_project_rules,
        persisted_ignored_suggestions,
    ) = sqlite_data
        .map(|sqlite_data| {
            (
                sqlite_data.cloud_objects,
                Some(sqlite_data.app_state),
                sqlite_data.command_history,
                sqlite_data.object_actions,
                sqlite_data.ai_queries,
                sqlite_data.code_workspaces,
                sqlite_data.workspace_language_servers,
                sqlite_data.agent_conversations,
                sqlite_data.projects,
                sqlite_data.project_rules,
                sqlite_data.ignored_suggestions,
            )
        })
        .unwrap_or_else(|| {
            (
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            )
        });

    ctx.set_fallback_font_source_provider(|url| ::asset_cache::url_source(url));

    ctx.set_default_binding_validator(is_binding_supported_on_mac);

    // Initialize timestamp for session id and last active event
    App::record_last_active_timestamp();

    ctx.add_singleton_model(|_| SettingsPaneManager::new());
    ctx.add_singleton_model(WarplyUpdater::new);
    ctx.add_singleton_model(|_| AIFactManager::new());
    ctx.add_singleton_model(|_| ExecutionProfileEditorManager::default());
    if !launch_mode.is_headless() {
        AppearanceManager::as_ref(ctx).set_app_icon(ctx);
    }

    #[cfg(feature = "local_tty")]
    terminal::available_shells::register(ctx);

    // Add truly global actions that don't depend on the existence of any view here
    ctx.add_global_action("app:toggle_user_ps1", move |_args: &(), ctx| {
        SessionSettings::handle(ctx).update(ctx, |session_settings, ctx| {
            log_setting_result(
                session_settings.honor_ps1.toggle_and_save_value(ctx),
                "honor_ps1",
            );
        });
    });
    ctx.add_global_action("app:toggle_copy_on_select", move |_args: &(), ctx| {
        SelectionSettings::handle(ctx).update(ctx, |selection_settings, ctx| {
            log_setting_result(
                selection_settings.copy_on_select.toggle_and_save_value(ctx),
                "copy_on_select",
            );
        });
    });

    ctx.add_singleton_model(|_ctx| SyncedInputState::new());

    ctx.add_singleton_model(remote_server::manager::RemoteServerManager::new);
    log::info!(
        "Starting warp with channel state {} and version {:?}",
        ChannelState::debug_str(),
        ChannelState::app_version()
    );

    // Teach our app that sometimes option means meta.
    ctx.set_event_munger(move |event, ctx| {
        let extra_meta_keys = *KeysSettings::as_ref(ctx).extra_meta_keys;
        apply_extra_meta_keys(event, extra_meta_keys);
        apply_scroll_multiplier(event, ctx);
    });

    ctx.set_a11y_verbosity(*AccessibilitySettings::as_ref(ctx).a11y_verbosity);

    ctx.on_first_frame_drawn(move |ctx| {
        IntervalTimer::handle(ctx).update(ctx, |timer, _| {
            timer.mark_interval_end("FIRST_FRAME_DRAWN");
        });

        GPUState::handle(ctx).update(ctx, |gpu_state, ctx| {
            gpu_state.set_has_lower_power_gpu(warpui::rendering::is_low_power_gpu_available(), ctx);
        });
    });

    #[cfg(feature = "local_fs")]
    {
        ctx.add_singleton_model(DirectoryWatcher::new);
        ctx.add_singleton_model(|_| DetectedRepositories::default());
        if let Some(home_dir) = dirs::home_dir() {
            ctx.add_singleton_model(|ctx| HomeDirectoryWatcher::new(home_dir, ctx));
        } else {
            log::info!("Home directory not found; skipping HomeDirectoryWatcher registration");
        }
    }

    #[cfg(feature = "local_fs")]
    {
        let imported_config_model = ctx.add_singleton_model(ImportedConfigModel::new);

        if ChannelState::channel() != warp_core::channel::Channel::Integration {
            imported_config_model.update(ctx, |model, ctx| {
                model.search_for_settings_to_import(ctx);
            });
        }

        let emit_incremental_updates = matches!(launch_mode, LaunchMode::RemoteServerDaemon { .. });
        ctx.add_singleton_model(|ctx| {
            let model = if emit_incremental_updates {
                RepoMetadataModel::new_with_incremental_updates(ctx)
            } else {
                RepoMetadataModel::new(ctx)
            };

            // Subscribe to RemoteServerManager push events so that remote repo
            // metadata snapshots and incremental updates populate the remote
            // sub-model and trigger RepoMetadataEvent emissions.
            {
                use remote_server::manager::{RemoteServerManager, RemoteServerManagerEvent};
                let mgr = RemoteServerManager::handle(ctx);
                ctx.subscribe_to_model(&mgr, |me, event, ctx| match event {
                    RemoteServerManagerEvent::RepoMetadataSnapshot { host_id, update } => {
                        me.insert_remote_snapshot(host_id.clone(), update, ctx);
                    }
                    RemoteServerManagerEvent::RepoMetadataUpdated { host_id, update }
                    | RemoteServerManagerEvent::RepoMetadataDirectoryLoaded { host_id, update } => {
                        me.apply_remote_incremental_update(host_id, update, ctx);
                    }
                    RemoteServerManagerEvent::HostDisconnected { host_id } => {
                        me.remove_remote_repositories_for_host(host_id, ctx);
                    }
                    _ => {}
                });
            }

            model
        });
    }

    {
        use code_review::git_status_update::GitStatusUpdateModel;
        ctx.add_singleton_model(|_| GitStatusUpdateModel::new());
    }

    ctx.add_singleton_model(|ctx| {
        ProjectManagementModel::new(persisted_projects, persistence_writer.sender(), ctx)
    });

    ctx.add_singleton_model(move |_| History::new(command_history));

    // Register initial keybindings prior to creating menus
    ai::init(ctx);
    app_services::init(ctx);
    code::editor::find::view::init(ctx);
    workspace::init(ctx);
    pane_group::init(ctx);
    terminal::init(ctx);
    input::init(ctx);
    editor::init(ctx);
    menu::init(ctx);
    tips::tip_view::init(ctx);
    launch_configs::init(ctx);
    workflows::init(ctx);
    themes::theme_chooser::init(ctx);
    themes::theme_creator_modal::init(ctx);
    themes::theme_deletion_modal::init(ctx);
    root_view::init(ctx);
    voltron::init(ctx);
    crate::view_components::find::init(ctx);
    prompt::editor_modal::init(ctx);
    ai::blocklist::agent_view::editor::init(ctx);
    undo_close::init(ctx);
    tab_configs::new_worktree_modal::init(ctx);
    tab_configs::params_modal::init(ctx);
    ai::blocklist::init(ctx);
    ai::blocklist::block::status_bar::init(ctx);
    env_vars::env_var_collection_block::init(ctx);
    terminal::ssh::install_tmux::init(ctx);
    terminal::ssh::warpify::init(ctx);
    terminal::ssh::error::init(ctx);
    context_chips::display_menu::init(ctx);
    context_chips::node_version_popup::init(ctx);
    env_vars::view::env_var_collection::init(ctx);
    ai::agent::todos::popup::init(ctx);
    code_review::init(ctx);

    let display_count = ctx.windows().display_count();
    ctx.add_singleton_model(|_| DisplayCount(display_count));

    ctx.add_singleton_model(|_| SystemStats::new());
    ctx.add_singleton_model(|_| KeybindingChangedNotifier::new());
    ctx.add_singleton_model(|_| search::command_palette::SelectedItems::new());
    ctx.add_singleton_model(search::files::model::FileSearchModel::new);
    ctx.add_singleton_model(|_| VimRegisters::new());
    ctx.add_singleton_model(UndoCloseStack::new);
    ctx.add_singleton_model(|_| ToastStack);
    ctx.add_singleton_model(|_| GlobalCodeReviewModel);
    #[cfg(feature = "local_fs")]
    ctx.add_singleton_model(FileModel::new);
    ctx.add_singleton_model(GlobalBufferModel::new);
    #[cfg(feature = "local_fs")]
    ctx.add_singleton_model(|_| LanguageServerShutdownManager::new());

    ctx.add_singleton_model(|_ctx| CloudModel::new(persistence_writer.sender(), cloud_objects));

    {
        let conversations = &agent_conversations;
        ctx.add_singleton_model(move |_| BlocklistAIHistoryModel::new(ai_queries, conversations));
    }
    ctx.add_singleton_model(move |_| RestoredAgentConversations::new(agent_conversations));
    ctx.add_singleton_model(ai::acp::registry::AcpRegistryModel::new);
    ctx.add_singleton_model(ai::acp::model::AcpAgentModel::new);
    ctx.add_singleton_model(|_| CLIAgentSessionsModel::new());
    // ActiveAgentViewsModel is used to track active agent conversations and notify listeners when they change.
    ctx.add_singleton_model(|_| ActiveAgentViewsModel::new());
    ctx.add_singleton_model(BlocklistAIPermissions::new);
    ctx.add_singleton_model(RepoOutlines::new);
    ctx.add_singleton_model(|_| ObjectActions::new(object_actions));

    ctx.add_singleton_model(|_| AudibleBell::new());

    ctx.add_singleton_model(|ctx| UpdateManager::new(persistence_writer.sender(), ctx));

    // LogManager must be registered before subsystems that create file-based loggers.
    ctx.add_singleton_model(|_| simple_logger::manager::LogManager::new());

    ctx.add_singleton_model(AIDocumentModel::new);

    // AgentConversationsModel subscribes to UpdateManager for RTC task updates.
    ctx.add_singleton_model(AgentConversationsModel::new);

    ctx.add_singleton_model(|_| CodeManager::default());
    ctx.add_singleton_model(|_| OpenedFilesModel::new());
    ctx.add_singleton_model(NotebookKeybindings::new);
    ctx.add_singleton_model(TerminalKeybindings::new);
    ctx.add_singleton_model(|_| ActiveSession::default());
    #[cfg(feature = "local_tty")]
    {
        ctx.add_singleton_model(LocalShellState::new);
        ctx.add_singleton_model(system::SystemInfo::new);
    }

    // Add a singleton model that holds the current prompt configuration.
    ctx.add_singleton_model(Prompt::new);

    // Add a singleton model for resizable modals whose size should be persisted through restarts.
    ctx.add_singleton_model(|_| ResizableData::default());

    ctx.add_singleton_model(EnvVarCollectionManager::new);
    ctx.add_singleton_model(WorkflowManager::new);

    ctx.add_singleton_model(LocalWorkflows::new);

    timer.mark_interval_end("SINGLETON_MODELS_REGISTERED");

    ctx.add_singleton_model(move |_| timer);

    ctx.add_singleton_model(|ctx| AIExecutionProfilesModel::new(launch_mode, ctx));

    ctx.add_singleton_model(DefaultTerminal::new);

    ctx.add_singleton_model(|ctx| {
        ProjectContextModel::new_from_persisted(persisted_project_rules, ctx)
    });
    ctx.add_singleton_model(|ctx| {
        PersistedWorkspace::new(
            persisted_workspaces,
            workspace_language_servers,
            persistence_writer.sender(),
            ctx,
        )
    });
    ctx.add_singleton_model(move |_| persistence_writer);

    ctx.add_singleton_model(input_classifier::InputClassifierModel::new);

    ctx.add_singleton_model(move |_| IgnoredSuggestionsModel::new(persisted_ignored_suggestions));

    // Subscribe WorkflowAliases to the UpdateManager so that it can be notified when objects are
    // trashed.
    WorkflowAliases::handle(ctx).update(ctx, |aliases, ctx| {
        aliases.connect(ctx);
    });

    ctx.add_singleton_model(move |ctx| {
        let routers = vec![profiling::make_router()];
        http_server::HttpServer::new(routers, ctx)
    });

    app_state
}

pub(crate) fn app_callbacks(is_integration_test: bool) -> warpui::platform::AppCallbacks {
    warpui::platform::AppCallbacks {
        on_internet_reachability_changed: None,
        on_become_active: None,
        on_screen_changed: Some(Box::new(move |ctx| {
            ctx.dispatch_global_action(
                "root_view:move_quake_mode_window_from_screen_change",
                &KeysSettings::as_ref(ctx)
                    .quake_mode_settings
                    .value()
                    .clone(),
            );

            let new_display_count = ctx.windows().display_count();
            DisplayCount::handle(ctx).update(ctx, |display_count, ctx| {
                display_count.0 = new_display_count;
                ctx.notify();
            });
        })),
        on_cpu_awakened: Some(Box::new(move |ctx| {
            SystemStats::handle(ctx).update(ctx, move |system, ctx| {
                log::info!("System has returned from sleep");
                system.dispatch_cpu_was_awakened(ctx);
            });
        })),
        on_cpu_will_sleep: Some(Box::new(move |ctx| {
            SystemStats::handle(ctx).update(ctx, move |system, ctx| {
                log::info!("System is going to sleep...");
                system.dispatch_cpu_will_sleep(ctx);
            });
        })),
        on_resigned_active: Some(Box::new(move |ctx| {
            let active_window_id = ctx.windows().active_window();
            let update_quake_mode_arg = UpdateQuakeModeEventArg { active_window_id };

            ctx.dispatch_global_action("root_view:update_quake_mode_state", &update_quake_mode_arg);
        })),
        on_will_terminate: Some(Box::new(move |ctx| {
            PersistenceWriter::handle(ctx).update(ctx, |writer, _ctx| {
                writer.terminate();
            });

            // Shutdown all LSP servers gracefully before app termination
            lsp::LspManagerModel::handle(ctx).update(ctx, |manager, ctx| {
                manager.terminate(ctx);
            });

            #[cfg(feature = "local_tty")]
            terminal::local_tty::spawner::PtySpawner::handle(ctx).update(ctx, |pty_spawner, _| {
                pty_spawner.prepare_for_app_termination();
            });

            // Tear down app services before spawning the new process, to
            // ensure that the new process doesn't find the old process while
            // attempting to enforce our single-instance policy on Linux.
            app_services::teardown(ctx);

            // Tear down any application profilers that are running, writing
            // results to disk.
            profiling::teardown();
        })),
        on_should_close_window: Some(Box::new(move |window_id, ctx| {
            let general_settings = GeneralSettings::as_ref(ctx);
            let quit_on_last_window_closed = *general_settings.quit_on_last_window_closed;
            if ctx.window_ids().count() == 1 && quit_on_last_window_closed {
                log::info!("No windows left, terminating app");
                ctx.terminate_app(TerminationMode::Cancellable, None);
                return ApproveTerminateResult::Cancel;
            }

            let summary = UnsavedStateSummary::for_window(window_id, ctx);

            // Don't show dialog on integration test. Machine can't press buttons.
            if !is_integration_test && summary.should_display_warning(ctx) {
                let shown = summary
                    .dialog()
                    .on_confirm(move |ctx| {
                        ctx.windows()
                            .close_window(window_id, TerminationMode::ForceTerminate);
                    })
                    .on_cancel(move |ctx| {
                        on_close_window_cancelled(window_id, false, ctx);
                    })
                    .on_show_processes(move |ctx| {
                        on_close_window_cancelled(window_id, true, ctx);
                    })
                    .show(ctx);
                if shown {
                    ApproveTerminateResult::Cancel
                } else {
                    ApproveTerminateResult::Terminate
                }
            } else {
                ApproveTerminateResult::Terminate
            }
        })),
        on_should_terminate_app: Some(Box::new(move |ctx| {
            let summary = UnsavedStateSummary::for_app(ctx);
            // Don't show dialog on integration test. Machine can't press buttons.
            if !is_integration_test && summary.should_display_warning(ctx) {
                let shown = summary
                    .dialog()
                    .on_confirm(|ctx| ctx.terminate_app(TerminationMode::ForceTerminate, None))
                    .on_show_processes(|ctx| on_close_app_cancelled(true, ctx))
                    .on_cancel(|ctx| on_close_app_cancelled(false, ctx))
                    .show(ctx);
                if shown {
                    return ApproveTerminateResult::Cancel;
                }
            }

            ApproveTerminateResult::Terminate
        })),
        on_disable_warning_modal: Some(Box::new(move |ctx| {
            GeneralSettings::handle(ctx).update(ctx, |general_settings, ctx| {
                log_setting_result(
                    general_settings
                        .show_warning_before_quitting
                        .set_value(false, ctx),
                    "show_warning_before_quitting",
                );
            });
        })),
        on_notification_clicked: Some(Box::new(move |notification_response, ctx| {
            if let Some(notification_data) = notification_response.data() {
                let context: serde_json::Result<NotificationContext> =
                    serde_json::from_str(notification_data);
                if let Ok(NotificationContext::BlockOrigin {
                    window_id,
                    pane_group_id,
                    pane_id,
                }) = context
                {
                    // Ensure the window ID exists, if so dispatch an action to focus
                    // the correct pane.
                    if ctx.window_ids().contains(&window_id) {
                        if let Some(root_view_id) = ctx.root_view_id(window_id) {
                            ctx.dispatch_action(
                                window_id,
                                &[root_view_id],
                                "root_view:handle_notification_click",
                                &PaneViewLocator {
                                    pane_group_id,
                                    pane_id,
                                },
                                log::Level::Info,
                            );
                        }
                    }
                }
            }
        })),
        on_new_window_requested: Some(Box::new(move |ctx| {
            // This one is called when the app is requested to open a new window,
            // e.g. clicking on the Dock icon. It is NOT called from the New Window
            // menu item.
            App::record_last_active_timestamp();
            ctx.dispatch_global_action("root_view:open_new", &());
            ctx.dispatch_global_action("workspace:save_app", &());
        })),
        on_open_urls: Some(Box::new(move |urls, ctx| {
            for url in &urls {
                let parsed_url = Url::parse(url);
                match parsed_url {
                    Ok(url) => uri::handle_incoming_uri(&url, ctx),
                    Err(e) => log::warn!("Unable to parse received url: {e}"),
                }
            }
        })),
        on_os_appearance_changed: Some(Box::new(move |ctx| {
            AppearanceManager::handle(ctx).update(ctx, |appearance_manager, ctx| {
                appearance_manager.refresh_theme_state(ctx);
            });
        })),
        on_active_window_changed: Some(Box::new(move |ctx| {
            let windowing_model = ctx.windows();
            let active_window_id = windowing_model.active_window();
            let key_window_is_modal_panel = windowing_model.key_window_is_modal_panel();

            if !key_window_is_modal_panel {
                let update_quake_mode_arg = UpdateQuakeModeEventArg { active_window_id };
                ctx.dispatch_global_action(
                    "root_view:update_quake_mode_state",
                    &update_quake_mode_arg,
                );
            }

            ctx.dispatch_global_action("workspace:save_app", &());
        })),
        on_window_will_close: Some(Box::new(move |closed_window_data, ctx| {
            if ctx.windows().stage() == ApplicationStage::Terminating {
                return;
            }

            if let Some(window_data) = closed_window_data {
                UndoCloseStack::handle(ctx).update(ctx, |stack, ctx| {
                    stack.handle_window_closed(window_data, ctx);
                });
            }
            ctx.dispatch_global_action("workspace:save_app", &());
        })),
        on_window_moved: Some(Box::new(move |ctx| {
            ctx.dispatch_global_action("workspace:save_app", &());
        })),
        on_window_resized: Some(Box::new(move |ctx| {
            ctx.dispatch_global_action("workspace:save_app", &());
        })),
        ..Default::default()
    }
}

fn on_close_app_cancelled(open_navigation_palette: bool, ctx: &mut AppContext) {
    let sessions = SessionNavigationData::all_sessions(ctx).collect_vec();
    let sessions_summary = RunningSessionSummary::new(&sessions);

    // If open_navigation_palette is false, return early. Otherwise, we honor the open_navigation_palette
    // param which is true if the user clicked the modal button for that. However, if the running
    // processes in this window have finished since the modal popped, there is nothing to do now and we
    // can return early
    if !open_navigation_palette || sessions_summary.long_running_cmds.is_empty() {
        return;
    }

    let windowing_model = ctx.windows();
    let active_window_id = windowing_model.active_window();
    // show the nav palette in the active window. if there is no active window,
    // arbitrarily pick one of the windows having a running process
    let window_id_to_focus = active_window_id.unwrap_or_else(|| {
        *sessions_summary
            .windows_running()
            .iter()
            .next()
            .expect("already checked len > 0")
    });

    windowing_model.show_window_and_focus_app(window_id_to_focus);

    // open the nav palette in the selected window
    if let Some(workspaces) = ctx.views_of_type::<Workspace>(window_id_to_focus) {
        if let Some(handle) = workspaces.first() {
            ctx.dispatch_typed_action_for_view(
                window_id_to_focus,
                handle.id(),
                &WorkspaceAction::OpenPalette {
                    mode: PaletteMode::Navigation,
                    source: PaletteSource::QuitModal,
                    query: Some("running".to_owned()),
                },
            );
        }
    }
}

fn on_close_window_cancelled(
    window_id: WindowId,
    open_navigation_palette: bool,
    ctx: &mut AppContext,
) {
    let sessions = SessionNavigationData::all_sessions(ctx).collect_vec();
    let sessions_summary = RunningSessionSummary::new(&sessions);
    let num_processes_in_window = sessions_summary.processes_in_window(&window_id).len();

    // If open_navigation_palette is false, return early. Otherwise, we honor the
    // open_navigation_palette param which is true if the user clicked the modal
    // button for that. However, if the running processes in this window have finished
    // since the modal popped, there is nothing to do now and we can return early
    if !open_navigation_palette || num_processes_in_window == 0 {
        return;
    }

    ctx.windows().show_window_and_focus_app(window_id);

    // if we haven't returned early, it means open_navigation_palette is true as the
    // user pressed the modal button for opening the navigation palette to show their
    // running processes
    if let Some(workspaces) = ctx.views_of_type::<Workspace>(window_id) {
        if let Some(handle) = workspaces.first() {
            ctx.dispatch_typed_action_for_view(
                window_id,
                handle.id(),
                &WorkspaceAction::OpenPalette {
                    mode: PaletteMode::Navigation,
                    source: PaletteSource::QuitModal,
                    query: Some("running".to_owned()),
                },
            );
        }
    }
}

fn launch(ctx: &mut warpui::AppContext, app_state: Option<AppState>, launch_mode: LaunchMode) {
    IntervalTimer::handle(ctx).update(ctx, |timer, _ctx| {
        timer.mark_interval_end("APP_LAUNCHED");
    });

    keyboard::load_custom_keybindings(ctx);

    IntervalTimer::handle(ctx).update(ctx, |timer, _ctx| {
        timer.mark_interval_end("KEYBINDINGS_LOADED");
    });

    match launch_mode {
        LaunchMode::App { .. } | LaunchMode::Test { .. } => {
            // Attempt to restore windows from the persisted application state.
            let arg = OpenFromRestoredArg { app_state };
            ctx.dispatch_global_action("root_view:open_from_restored", &arg);

            // Process any URLs that were provided on the command line (which may be
            // file:// URLs or ones using our custom URL scheme).
            for url in launch_mode.args().urls.iter() {
                uri::handle_incoming_uri(url, ctx);
            }

            // If, after session restoration and command-line argument handling, we
            // haven't opened any windows, open a new window.
            if ctx.window_ids().count() == 0 {
                ctx.dispatch_global_action("root_view:open_new", &());
            }

            IntervalTimer::handle(ctx).update(ctx, |timer, _| {
                timer.mark_interval_end("WINDOWS_CREATED");
            });

            {
                use crate::login_item::maybe_register_app_as_login_item;
                use crate::terminal::general_settings::GeneralSettingsChangedEvent;
                ctx.subscribe_to_model(&GeneralSettings::handle(ctx), |_, event, ctx| {
                    if matches!(event, GeneralSettingsChangedEvent::LoginItem { .. }) {
                        maybe_register_app_as_login_item(ctx);
                    }
                });
                maybe_register_app_as_login_item(ctx);
            }
        }
        // Proxy should never reach launch() — it's a thin byte bridge.
        LaunchMode::RemoteServerProxy => {
            log::error!("Proxy mode should not use the launch() path");
            std::process::exit(1);
        }
        LaunchMode::RemoteServerDaemon { identity_key } => {
            remote_server::unix::launch_daemon(&identity_key, ctx);
        }
    }
}

/// Initializes the logger before running tests.
///
/// The `ctor` attribute here means that this runs BEFORE main(), whenever the
/// binary is executed. For this reason, we need to ensure that this function
/// only exists within unit test code. Production bundles and integration tests
/// also initialize the logging system, and initializing it twice causes a panic.
///
/// Additionally, we must not write anything to stdout in this function, as it
/// can interfere with test harnesses collecting the set of tests to run. (This
/// is why we're not simply calling the init() function above.)
#[ctor::ctor]
#[cfg(test)]
fn init_logging_for_unit_tests_glue() {
    // Initialize terminal-friendly logging for tests from the shared logger crate.
    warp_logging::init_logging_for_unit_tests();
}

/// Mark all features which should be enabled on the current channel as enabled.
/// This sets global feature flag state and should never be called in a unit test.
pub fn init_feature_flags() {
    for flag in enabled_features() {
        flag.set_enabled(true);
    }
    features::mark_initialized();
}

/// Returns all feature flags which should be enabled in the current channel.
pub fn enabled_features() -> HashSet<FeatureFlag> {
    // Enable features overridden for the given channel.
    let mut flags = ChannelState::additional_features();

    // Enable flags for release builds, if appropriate.
    if ChannelState::is_release_bundle() {
        flags.extend(features::RELEASE_FLAGS);
    }

    flags.extend([
        #[cfg(feature = "runtime_feature_flags")]
        FeatureFlag::RuntimeFeatureFlags,
        #[cfg(feature = "sequential_storage")]
        FeatureFlag::SequentialStorage,
        #[cfg(feature = "in_band_generators_ssh")]
        FeatureFlag::InBandGeneratorsForSSH,
        #[cfg(feature = "ligatures")]
        FeatureFlag::Ligatures,
        #[cfg(feature = "selectable_prompt")]
        FeatureFlag::SelectablePrompt,
        #[cfg(feature = "resize_fix")]
        FeatureFlag::ResizeFix,
        #[cfg(feature = "richtext_multiselect")]
        FeatureFlag::RichTextMultiselect,
        #[cfg(feature = "rect_selection")]
        FeatureFlag::RectSelection,
        #[cfg(feature = "alacritty_settings_import")]
        FeatureFlag::AlacrittySettingsImport,
        #[cfg(feature = "dynamic_workflow_enums")]
        FeatureFlag::DynamicWorkflowEnums,
        #[cfg(feature = "shell_selector")]
        FeatureFlag::ShellSelector,
        #[cfg(feature = "full_screen_zen_mode")]
        FeatureFlag::FullScreenZenMode,
        #[cfg(feature = "minimalist_ui")]
        FeatureFlag::MinimalistUI,
        #[cfg(feature = "workflow_aliases")]
        FeatureFlag::WorkflowAliases,
        #[cfg(feature = "ime_marked_text")]
        FeatureFlag::ImeMarkedText,
        #[cfg(feature = "iterm_images")]
        FeatureFlag::ITermImages,
        #[cfg(feature = "kitty_images")]
        FeatureFlag::KittyImages,
        #[cfg(feature = "command_correction_key")]
        FeatureFlag::CommandCorrectionKey,
        #[cfg(feature = "use_tantivy_search")]
        FeatureFlag::UseTantivySearch,
        #[cfg(feature = "markdown_tables")]
        FeatureFlag::MarkdownTables,
        #[cfg(feature = "markdown_mermaid")]
        FeatureFlag::MarkdownMermaid,
        #[cfg(feature = "editable_markdown_mermaid")]
        FeatureFlag::EditableMarkdownMermaid,
        #[cfg(feature = "code_find_replace")]
        FeatureFlag::CodeFindReplace,
        #[cfg(feature = "command_palette_file_search")]
        FeatureFlag::CommandPaletteFileSearch,
        #[cfg(feature = "expand_edit_to_pane")]
        FeatureFlag::ExpandEditToPane,
        #[cfg(feature = "tab_close_button_on_left")]
        FeatureFlag::TabCloseButtonOnLeft,
        #[cfg(feature = "tabbed_editor_view")]
        FeatureFlag::TabbedEditorView,
        #[cfg(feature = "undo_closed_panes")]
        FeatureFlag::UndoClosedPanes,
        #[cfg(feature = "welcome_tab")]
        FeatureFlag::WelcomeTab,
        #[cfg(feature = "projects")]
        FeatureFlag::Projects,
        #[cfg(feature = "vim_code_editor")]
        FeatureFlag::VimCodeEditor,
        #[cfg(feature = "revert_diff_hunk")]
        FeatureFlag::RevertDiffHunk,
        #[cfg(feature = "ui_zoom")]
        FeatureFlag::UIZoom,
        #[cfg(feature = "global_search")]
        FeatureFlag::GlobalSearch,
        #[cfg(feature = "configurable_toolbar")]
        FeatureFlag::ConfigurableToolbar,
        #[cfg(feature = "classic_completions")]
        FeatureFlag::ClassicCompletions,
        #[cfg(feature = "force_classic_completions")]
        FeatureFlag::ForceClassicCompletions,
        #[cfg(feature = "inline_history_menu")]
        FeatureFlag::InlineHistoryMenu,
        #[cfg(feature = "pluggable_notifications")]
        FeatureFlag::PluggableNotifications,
        #[cfg(feature = "new_tab_styling")]
        FeatureFlag::NewTabStyling,
        #[cfg(feature = "incremental_auto_reload")]
        FeatureFlag::IncrementalAutoReload,
        #[cfg(feature = "kitty_keyboard_protocol")]
        FeatureFlag::KittyKeyboardProtocol,
        #[cfg(feature = "inline_menu_headers")]
        FeatureFlag::InlineMenuHeaders,
        #[cfg(feature = "directory_tab_colors")]
        FeatureFlag::DirectoryTabColors,
        #[cfg(feature = "vertical_tabs")]
        FeatureFlag::VerticalTabs,
        #[cfg(feature = "vertical_tabs_summary_mode")]
        FeatureFlag::VerticalTabsSummaryMode,
        #[cfg(feature = "tab_configs")]
        FeatureFlag::TabConfigs,
    ]);

    flags
}
