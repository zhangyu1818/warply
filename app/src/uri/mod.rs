mod docker;

use crate::launch_configs::launch_config::LaunchConfig;
use crate::linear::{LinearAction, LinearIssueWork};
use crate::root_view::{open_new_window_get_handles, OpenLaunchConfigArg};
use crate::tab_configs::TabConfig;
use crate::ui_events::LaunchConfigUiLocation;
use crate::util::openable_file_type::{
    is_file_openable_in_warp, is_runnable_shell_script, starts_with_shebang,
};
use crate::workspace::active_terminal_in_window;
use crate::workspace::util::PaneViewLocator;
use crate::workspace::{Workspace, WorkspaceAction, WorkspaceRegistry};
use crate::{view_components::DismissibleToast, workspace::ToastStack};

use crate::settings_view::{settings_widget_deeplink_target, SettingsSection};
use crate::user_config::{load_launch_configs, load_tab_configs, tab_configs_dir};
use crate::{quake_mode_window_id, quake_mode_window_is_open, ChannelState, OpenPath};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::Url;
use warp_core::{features::FeatureFlag, safe_info};
use warpui::{SingletonEntity as _, TypedActionView};

use warpui::{AppContext, WindowId};

use self::docker::open_docker_container;

/// Args for the `warp://settings` deeplink family, dispatched to the
/// `root_view:open_settings_in_{existing,new}_window` actions.
pub enum OpenSettingsArgs {
    /// `warp://settings` — open a settings tab on the default page.
    Default,
    /// `warp://settings?q=<query>` — open settings with the search bar pre-filled.
    Search { query: String },
    /// `warp://settings?widget=<widget_id>` — open settings scrolled to a widget.
    Widget {
        page: SettingsSection,
        widget_id: &'static str,
    },
}

/// Resolves a simple `warp://settings/<sub_page>` slug (with no query params)
/// to the settings section it should open. Sub-pages requiring extra params
/// (e.g. `teams`, `mcp`) are handled by their own branches in the
/// `UriHost::Settings` arm.
fn settings_section_for_simple_subpage(sub_page: &str) -> Option<SettingsSection> {
    let section = match sub_page {
        "appearance" => SettingsSection::Appearance,
        _ => return None,
    };
    Some(section)
}

#[derive(Debug, PartialEq, Eq)]
pub enum UriHost {
    /// A host prefix for all actions (e.g.: new tab, new window).
    Action,
    /// A host prefix for all actions that involve launch configurations
    Launch,
    /// Supports opening warp's settings panel via URI
    Settings,
    /// A host prefix for a general-purpose home/landing page. Unlike other intent URIs, the home
    /// page behavior may change over time and vary from platform to platform.
    Home,
    /// Actions triggered from Linear integrations (e.g. work on issue).
    Linear,
    TabConfig,
    /// Focuses a specific terminal pane by its persistent session UUID.
    Session,
}

impl FromStr for UriHost {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "action" => Ok(Self::Action),
            "launch" => Ok(Self::Launch),
            "settings" => Ok(Self::Settings),
            "home" => Ok(Self::Home),
            "linear" => Ok(Self::Linear),
            "tab_config" if FeatureFlag::TabConfigs.is_enabled() => Ok(Self::TabConfig),
            "session" => Ok(Self::Session),
            _ => Err(anyhow!("Received url with unexpected host: {}", s)),
        }
    }
}

impl UriHost {
    fn handle(&self, primary_window_id: Option<WindowId>, url: &Url, ctx: &mut AppContext) {
        // Handle host
        match self {
            UriHost::Action => {
                match Action::parse(url) {
                    Ok(action) => action.handle(primary_window_id, url, ctx),
                    Err(err) => {
                        log::warn!("{err}");
                    }
                };
            }
            UriHost::Launch => {
                if let Some(desired_config_path) = get_launch_config_path(url.path()) {
                    let configs = load_launch_configs(&crate::user_config::launch_configs_dir());
                    if let Some(config) =
                        find_matching_config(desired_config_path.as_str(), &configs)
                    {
                        ctx.dispatch_global_action(
                            "root_view:open_launch_config",
                            &OpenLaunchConfigArg {
                                launch_config: config.clone(),
                                ui_location: LaunchConfigUiLocation::Uri,
                                open_in_active_window: false,
                            },
                        )
                    } else {
                        log::warn!(
                            "couldn't find a matching file path for '{}'",
                            desired_config_path.as_str()
                        );
                    }
                } else {
                    log::warn!("couldn't turn launch link '{}' into path", url.path());
                }
            }
            UriHost::Settings => {
                // Supported warp://settings deep links:
                // - warp://settings - opens a settings tab on the default page
                // - warp://settings?q={query} - opens settings with the search bar pre-filled
                // - warp://settings?widget={widget_id} - opens settings scrolled to a widget
                // - warp://settings/appearance - opens the appearance settings page
                let query_string: HashMap<_, _> = url.query_pairs().collect();
                // A bare `warp://settings` (or a trailing slash) yields an empty path
                // segment; treat that as "no sub-page" so the query-param routing below
                // handles it.
                let settings_sub_page: Option<String> = url
                    .path_segments()
                    .into_iter()
                    .flatten()
                    .last()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                match settings_sub_page.as_deref() {
                    // No special sub-page: route the bare host, the `q` (search) and
                    // `widget` (scroll-to) query params, and the simple section
                    // sub-pages (e.g. appearance) resolved via
                    // `settings_section_for_simple_subpage`.
                    maybe_simple_subpage => {
                        let simple_section =
                            maybe_simple_subpage.and_then(settings_section_for_simple_subpage);
                        // Pull the non-empty `q` search query out of the already
                        // parsed pairs to pre-fill the settings search bar.
                        let search_query = query_string
                            .get("q")
                            .map(|query| query.to_string())
                            .filter(|query| !query.is_empty());
                        let widget_target = query_string
                            .get("widget")
                            .and_then(|slug| settings_widget_deeplink_target(slug));

                        if let Some((page, widget_id)) = widget_target {
                            // `?widget=` scrolls to a specific widget; it takes
                            // precedence over `?q=` since searching would filter the
                            // target widget out of view.
                            let args = OpenSettingsArgs::Widget { page, widget_id };
                            dispatch_action_in_new_or_existing_window(
                                primary_window_id,
                                "root_view:open_settings_in_existing_window",
                                "root_view:open_settings_in_new_window",
                                &args,
                                ctx,
                            );
                        } else if let Some(query) = search_query {
                            let args = OpenSettingsArgs::Search { query };
                            dispatch_action_in_new_or_existing_window(
                                primary_window_id,
                                "root_view:open_settings_in_existing_window",
                                "root_view:open_settings_in_new_window",
                                &args,
                                ctx,
                            );
                        } else if let Some(section) = simple_section {
                            dispatch_action_in_new_or_existing_window(
                                primary_window_id,
                                "root_view:open_settings_page_in_existing_window",
                                "root_view:open_settings_page_in_new_window",
                                &section,
                                ctx,
                            );
                        } else if maybe_simple_subpage.is_none() {
                            // Bare `warp://settings` opens the default settings page.
                            let args = OpenSettingsArgs::Default;
                            dispatch_action_in_new_or_existing_window(
                                primary_window_id,
                                "root_view:open_settings_in_existing_window",
                                "root_view:open_settings_in_new_window",
                                &args,
                                ctx,
                            );
                        } else {
                            log::warn!("Failed to open settings pane: unrecognized sub-page");
                        }
                    }
                }
            }
            UriHost::Home => {
                ctx.dispatch_global_action("root_view::open_new", &());
            }
            UriHost::Linear => match LinearAction::parse(url) {
                Ok(LinearAction::WorkOnIssue) => {
                    let args = LinearIssueWork::from_url(url);
                    dispatch_action_in_new_or_existing_window(
                        primary_window_id,
                        "root_view:open_linear_issue_work_in_existing_window",
                        "root_view:open_linear_issue_work_in_new_window",
                        &args,
                        ctx,
                    );
                }
                Err(err) => {
                    log::warn!("{err}");
                }
            },
            UriHost::TabConfig => {
                handle_tab_config_uri(primary_window_id, url, ctx);
            }
            UriHost::Session => {
                let uuid_hex = url
                    .path_segments()
                    .into_iter()
                    .flatten()
                    .last()
                    .unwrap_or("");

                let Some(uuid_bytes) = decode_uuid_hex(uuid_hex) else {
                    log::warn!(
                        "session deep link received invalid UUID hex (safe: len={})",
                        uuid_hex.len()
                    );
                    return;
                };

                let result = WorkspaceRegistry::as_ref(ctx)
                    .all_workspaces(ctx)
                    .iter()
                    .find_map(|(win_id, workspace)| {
                        workspace.as_ref(ctx).tab_views().find_map(|pane_group| {
                            let pane_id = pane_group
                                .as_ref(ctx)
                                .find_terminal_pane_by_session_uuid(&uuid_bytes)?;
                            Some((
                                *win_id,
                                PaneViewLocator {
                                    pane_group_id: pane_group.id(),
                                    pane_id,
                                },
                            ))
                        })
                    });

                if let Some((window_id, locator)) = result {
                    ctx.windows().show_window_and_focus_app(window_id);
                    if let Some(root_view_id) = ctx.root_view_id(window_id) {
                        ctx.dispatch_action_for_view(
                            window_id,
                            root_view_id,
                            "root_view:handle_pane_navigation_event",
                            &locator,
                        );
                    }
                } else {
                    log::warn!("session deep link could not find pane with given UUID");
                }
            }
        }
    }
}

/// Turn the launch config URL into a filename.
/// "/hello%20world" --> "hello world"
fn get_launch_config_path(path: &str) -> Option<String> {
    // Remove the leading slash before the filename.
    let (_, config_path) = path.split_once('/')?;

    // URL-decode the filename to recover spaces and
    // other non-URL-friendly characters
    let decoded = serde_urlencoded::from_str::<Vec<(String, String)>>(config_path).ok()?;

    // serde_urlencoded::from_str tries to find a vector key-value pairs,
    // so we'll take the first tuple in the vector...
    let decoded_config_name = decoded.first()?;

    // ... and read the first member of the tuple.
    let validated_path = validate_launch_config_path(decoded_config_name.0.as_str())?;

    // Finally, return the validated path.
    Some(validated_path.to_string())
}

/// Remove file extension, which consists of the last '.' in the filename
/// and whatever characters follow it.
fn remove_extension(full_path: &str) -> Option<&str> {
    let (no_extension, _) = full_path.rsplit_once('.')?;
    Some(no_extension)
}

/// Ensure that a path is relative and doesn't contain '/../',
/// to prevent launch config links from escaping the launch config directory.
fn validate_launch_config_path(path: &str) -> Option<&str> {
    if path.starts_with('/')
        || path.starts_with("../")
        || path.contains("/../")
        || path.ends_with("/..")
    {
        None
    } else {
        Some(path)
    }
}

/// Given a config path, find a matching launch config file
fn find_matching_config<'a>(
    target_path: &str,
    configs: &'a [LaunchConfig],
) -> Option<&'a LaunchConfig> {
    // first, try to match the exact filename.
    if let Some(matched_config) = find_matching_config_name(target_path, configs) {
        return Some(matched_config);
    }

    // next, try to match the filename without the extension
    let no_extension = remove_extension(target_path)?;
    find_matching_config_name(no_extension, configs)
}

/// Case-insensitive matching on the config's name
/// (field in the YAML file).
fn find_matching_config_name<'a>(
    target_name: &str,
    configs: &'a [LaunchConfig],
) -> Option<&'a LaunchConfig> {
    let target_name_lower = target_name.to_lowercase();
    configs
        .iter()
        .find(|&config| config.name.to_lowercase() == target_name_lower)
}

fn handle_tab_config_uri(primary_window_id: Option<WindowId>, url: &Url, ctx: &mut AppContext) {
    let Some(desired) = get_launch_config_path(url.path()) else {
        log::warn!("couldn't turn tab config link '{}' into name", url.path());
        return;
    };

    let (configs, _errors) = load_tab_configs(&tab_configs_dir());
    let Some(config) = find_matching_tab_config(desired.as_str(), configs) else {
        log::warn!("couldn't find a tab config matching '{}'", desired);
        return;
    };

    let force_new_window = url
        .query_pairs()
        .any(|(k, v)| k == "new_window" && matches!(v.as_ref(), "1" | "true"));
    let target_window_id = if force_new_window {
        None
    } else {
        primary_window_id.filter(|id| WorkspaceRegistry::as_ref(ctx).get(*id, ctx).is_some())
    };

    let workspace = match target_window_id {
        Some(window_id) => WorkspaceRegistry::as_ref(ctx).get(window_id, ctx),
        None => {
            let new_window_id = open_new_window_get_handles(None, ctx).0;
            WorkspaceRegistry::as_ref(ctx).get(new_window_id, ctx)
        }
    };

    let Some(workspace) = workspace else {
        log::warn!(
            "no workspace available to open tab config '{}'",
            config.name
        );
        return;
    };

    workspace.update(ctx, |workspace, ctx| {
        workspace.open_tab_config(config, ctx);
    });
}

fn find_matching_tab_config(target: &str, configs: Vec<TabConfig>) -> Option<TabConfig> {
    let raw = target.to_lowercase();
    let stripped = remove_extension(target).map(str::to_lowercase);
    configs.into_iter().find(|config| {
        config
            .source_path
            .as_ref()
            .and_then(|path| path.file_stem())
            .and_then(|stem| stem.to_str())
            .map(|stem| {
                let stem = stem.to_lowercase();
                stem == raw || Some(stem.as_str()) == stripped.as_deref()
            })
            .unwrap_or(false)
    })
}

/// Extract the `path` query parameter, expanding a leading `~` to the
/// user's home directory.
fn parse_tab_path(url: &Url) -> Option<PathBuf> {
    let raw = url.query_pairs().find(|(k, _)| k == "path")?.1;
    Some(PathBuf::from(shellexpand::tilde(&raw).into_owned()))
}

#[derive(Debug)]
enum Action {
    NewTab,
    NewWindow,
    Docker,
    OpenRepo,
    NewAgentConversation,
}

impl Action {
    fn parse(url: &Url) -> Result<Self> {
        match url.path() {
            "/new_tab" => Ok(Self::NewTab),
            "/new_window" => Ok(Self::NewWindow),
            "/docker/open_subshell" => Ok(Self::Docker),
            "/open-repo" => Ok(Self::OpenRepo),
            "/new_agent_conversation" => Ok(Self::NewAgentConversation),
            _ => Err(anyhow!(
                "Received \"action\" intent with unexpected action: {}",
                url.path()
            )),
        }
    }

    fn handle(&self, primary_window_id: Option<WindowId>, url: &Url, ctx: &mut AppContext) {
        match self {
            Self::NewTab | Self::NewWindow => {
                let window_id = if let Self::NewTab = self {
                    primary_window_id
                } else {
                    None
                };
                let Some(path) = parse_tab_path(url) else {
                    log::warn!("Could not parse path to open a new tab/window");
                    return;
                };
                open_file(window_id, path, ctx);
            }
            Action::Docker => {
                if let Err(err) = open_docker_container(url, ctx) {
                    if let Some(window_id) = primary_window_id {
                        ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                            let toast =
                                DismissibleToast::error("Custom URI is invalid.".to_owned());
                            toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                        });
                    }

                    log::warn!("error opening docker container: {err}");
                }
            }
            Action::OpenRepo => {
                let window_id =
                    primary_window_id.or_else(|| Some(open_new_window_get_handles(None, ctx).0));

                let Some(window_id) = window_id else {
                    log::warn!("unable to determine window for open repo action");
                    return;
                };

                let Some(mut workspaces) = ctx.views_of_type::<Workspace>(window_id) else {
                    log::warn!("no workspace found in window {window_id} for open repo action");
                    return;
                };

                if let Some(workspace) = workspaces.pop() {
                    workspace.update(ctx, |workspace, ctx| {
                        workspace
                            .handle_action(&WorkspaceAction::OpenRepository { path: None }, ctx);
                    });
                } else {
                    log::warn!("no workspace views in window {window_id} for open repo action");
                }
            }
            Action::NewAgentConversation => {
                let window_id =
                    primary_window_id.or_else(|| Some(open_new_window_get_handles(None, ctx).0));

                let Some(window_id) = window_id else {
                    log::warn!("unable to determine window for new agent conversation action");
                    return;
                };

                let Some(workspace) = WorkspaceRegistry::as_ref(ctx).get(window_id, ctx) else {
                    log::warn!(
                        "no workspace found in window {window_id} for new agent conversation action"
                    );
                    return;
                };

                workspace.update(ctx, |workspace, ctx| {
                    workspace.handle_action(&WorkspaceAction::AddAgentTab, ctx);
                });
            }
        }
    }
}

/// Handles all incoming urls.
pub fn handle_incoming_uri(url: &Url, ctx: &mut AppContext) {
    safe_info!(
        safe: ("received url {}", safe_url_log_fields(url)),
        full: ("received url {:?}", &url)
    );

    // Pick the window that should be handling the URI.  This has some
    // additional logic to handle the hotkey window and there being no
    // currently-active window.
    let primary_window_id = get_primary_window(ctx.windows().frontmost_window_id(), ctx);

    // If we're running on a platform where we can spawn local TTYs,
    // check if this is a file:// URL and if so, spawn a new session
    // with an initial working directory based on the provided path.
    #[cfg(feature = "local_tty")]
    if url.scheme() == "file" {
        if let Ok(path) = url.to_file_path() {
            open_file(primary_window_id, path, ctx);
        }
        return;
    }

    match validate_custom_uri(url) {
        Ok(host) => {
            host.handle(primary_window_id, url, ctx);
        }
        Err(e) => {
            if let Some(window_id) = primary_window_id {
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    let toast = DismissibleToast::error(format!("Custom URI is invalid: {e:?}"));
                    toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                });
            }
            log::warn!("Custom URI is invalid: {e:?}");
        }
    }
}

/// Gets the primary window ID, and returns `None` if it does not exist.
/// A primary window is the foregrounded window, or one of the inactive non-quake windows.
/// A closed quake window is not counted.
fn get_primary_window(
    active_window_id: Option<WindowId>,
    ctx: &mut AppContext,
) -> Option<WindowId> {
    // Return quake mode window if it's open
    if let Some(window_id) = quake_mode_window_id()
        .filter(|window_id| quake_mode_window_is_open() && ctx.is_window_open(*window_id))
    {
        return Some(window_id);
    }

    // Otherwise, return active window
    if let Some(window_id) = active_window_id {
        return Some(window_id);
    }

    let mut non_quake_mode_windows = ctx
        .window_ids()
        .filter(|window_id| Some(*window_id) != quake_mode_window_id());

    // There's no active window, return first non-quake mode window or None if none exist.
    non_quake_mode_windows.next()
}

/// What `open_file` should do with an incoming `file://` URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenFileAction {
    /// Open in Warp's code/text editor pane.
    Editor,
    /// Open a session at the parent directory and queue the file as the pending command,
    /// or just open a session at the directory path if `path` is a directory.
    ExecuteInSession,
}

/// Pure routing decision for `open_file`. Extracted so it can be unit-tested without
/// standing up a full `AppContext`.
///
/// The Markdown Viewer preference is passed in because macOS can hand Markdown
/// file URLs to Warp via the file type registration in `Info.plist`. Since Warp
/// cannot easily update that registration when the user toggles the viewer
/// preference, the URI handler must check the preference before routing a
/// Markdown file to the in-Warp markdown viewer. The actual preference-based
/// target resolution happens downstream in `resolve_file_target_to_open_in_warp`,
/// but the flag is plumbed through here for routing decisions and future use.
fn classify_open_file_action(path: &Path, _prefer_markdown_viewer: bool) -> OpenFileAction {
    if path.is_file() {
        if is_runnable_shell_script(path) {
            return OpenFileAction::ExecuteInSession;
        }
        // Anything we can show in the editor opens there. The second branch catches
        // shebang scripts that `is_file_openable_in_warp` rejects on extension alone
        // (e.g. an extensionless `#!/bin/sh` file without the user-execute bit) so
        // they don't fall through to the executor and produce a `permission denied`.
        if is_file_openable_in_warp(path).is_some() || starts_with_shebang(path) {
            return OpenFileAction::Editor;
        }
    }
    OpenFileAction::ExecuteInSession
}

/// Handle an incoming `file://` URL.
/// * For directories, open a new session at the directory path.
/// * For other files, open a new session at the parent directory path, then possibly execute the
///   file.
fn open_file(window_id: Option<WindowId>, path: PathBuf, ctx: &mut AppContext) {
    let primary_window_and_view = window_id.and_then(|window_id| {
        ctx.root_view_id(window_id)
            .map(|view_id| (window_id, view_id))
    });

    #[cfg(feature = "local_fs")]
    let prefer_markdown_viewer = {
        use crate::util::file::external_editor::EditorSettings;
        *EditorSettings::as_ref(ctx).prefer_markdown_viewer
    };
    #[cfg(not(feature = "local_fs"))]
    let prefer_markdown_viewer = true;

    let action = classify_open_file_action(&path, prefer_markdown_viewer);
    if action == OpenFileAction::Editor {
        #[cfg(feature = "local_fs")]
        {
            use crate::code::editor_management::CodeSource;
            use crate::root_view::{open_new_with_workspace_source, NewWorkspaceSource};
            use crate::util::{
                file::external_editor::EditorSettings,
                openable_file_type::resolve_file_target_to_open_in_warp,
            };

            // Open text/code files in Warp's code editor, respecting the user's layout preference.
            let editor_settings = EditorSettings::as_ref(ctx);
            let target = resolve_file_target_to_open_in_warp(&path, editor_settings, None);

            let window_id = if let Some((wid, _)) = primary_window_and_view {
                wid
            } else {
                open_new_with_workspace_source(
                    NewWorkspaceSource::Session {
                        options: Box::default(),
                    },
                    ctx,
                )
                .0
            };

            ctx.windows().show_window_and_focus_app(window_id);

            if let Some(workspaces) = ctx.views_of_type::<Workspace>(window_id) {
                if let Some(workspace) = workspaces.into_iter().next() {
                    workspace.update(ctx, |workspace, ctx| {
                        let source = CodeSource::Finder { path: path.clone() };
                        workspace.open_file_with_target(path, target, None, source, ctx);
                    });
                }
            }
        }
    } else {
        let directory_path = if path.is_file() {
            match path.parent() {
                Some(parent) => parent.to_path_buf(),
                None => PathBuf::new(),
            }
        } else {
            path.clone()
        };

        if let Some((primary_window_id, root_view_id)) = primary_window_and_view {
            ctx.dispatch_action(
                primary_window_id,
                &[root_view_id],
                "root_view:add_session_at_path",
                &directory_path,
                log::Level::Info,
            );

            // Run command after session has been added
            if path.is_file() {
                if let Some(path_str) = path.to_str() {
                    execute_file(primary_window_id, path_str, ctx);
                }
            }
        } else {
            let open_path = OpenPath {
                path: directory_path,
            };
            ctx.dispatch_global_action("root_view:open_new_from_path", &open_path);

            // Run command after window has been added
            if path.is_file() {
                let active_window_id = ctx.windows().active_window();
                if let Some(primary_window_id) = get_primary_window(active_window_id, ctx) {
                    if let Some(path_str) = path.to_str() {
                        execute_file(primary_window_id, path_str, ctx);
                    }
                }
            }
        }
    }
}

fn execute_file(window_id: WindowId, path_str: &str, ctx: &mut AppContext) {
    active_terminal_in_window(window_id, ctx, |term, t_ctx| {
        let path_str = term.shell_family(t_ctx).shell_escape(path_str);
        term.input().update(t_ctx, |input, i_ctx| {
            input.set_pending_command(&path_str, i_ctx);
        })
    });
}

/// Helper function to dispatch an action to an existing window
/// or create new window if none exist.
fn dispatch_action_in_new_or_existing_window<T: 'static>(
    primary_window_id: Option<WindowId>,
    existing_window_action: &str,
    new_window_action: &str,
    args: &T,
    ctx: &mut AppContext,
) {
    let primary_window_and_view = primary_window_id.and_then(|window_id| {
        ctx.root_view_id(window_id)
            .map(|view_id| (window_id, view_id))
    });

    if let Some((primary_window_id, root_view_id)) = primary_window_and_view {
        ctx.dispatch_action(
            primary_window_id,
            &[root_view_id],
            existing_window_action,
            args,
            log::Level::Info,
        );
    } else {
        ctx.dispatch_global_action(new_window_action, args);
    }
}

/// Validates an incoming custom URI for security and returns the host.
fn validate_custom_uri(url: &Url) -> Result<UriHost> {
    // For now the only scheme we support is `[scheme_name]://[host_str]/...
    // Ignore all other urls that don't match this scheme for security purposes.
    if url.scheme() != ChannelState::url_scheme() {
        return Err(anyhow!(
            "Received url with unexpected scheme: {} ",
            url.scheme()
        ));
    }

    let host_str = url
        .host_str()
        .ok_or_else(|| anyhow!("Received url with no host str"))?;

    let host = UriHost::from_str(host_str)?;

    Ok(host)
}

/// Formats the non-sensitive components of an incoming URL for logging on
/// release channels.
///
/// The returned string contains only the URL's scheme, host, and path — never
/// its query string, fragment, or userinfo component.
///
/// `url.host_str()` can return `None` for schemes that don't require a host
/// (e.g. some `file://` URLs on certain platforms); the literal `-` is used
/// as a placeholder in that case so the formatter never panics.
fn safe_url_log_fields(url: &Url) -> String {
    format!(
        "scheme={} host={} path={}",
        url.scheme(),
        url.host_str().unwrap_or("-"),
        url.path(),
    )
}

fn decode_uuid_hex(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.as_bytes();
    if hex.len() != 32 {
        return None;
    }

    hex.chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            Some(((high << 4) | low) as u8)
        })
        .collect()
}

#[cfg(test)]
#[path = "uri_test.rs"]
mod tests;
