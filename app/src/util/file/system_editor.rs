use std::path::PathBuf;

use serde::Deserialize;
use warpui::AppContext;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EditorApp {
    pub id: String,
    pub bundle_identifier: String,
    pub display_name: String,
    pub bundle_url: PathBuf,
    pub icon_path: Option<PathBuf>,
}

#[derive(Deserialize)]
struct RawEditorApp {
    bundle_identifier: String,
    display_name: String,
    bundle_url: PathBuf,
    icon_path: Option<PathBuf>,
}

impl From<RawEditorApp> for EditorApp {
    fn from(raw: RawEditorApp) -> Self {
        Self {
            id: raw.bundle_identifier.clone(),
            bundle_identifier: raw.bundle_identifier,
            display_name: raw.display_name,
            bundle_url: raw.bundle_url,
            icon_path: raw.icon_path,
        }
    }
}

pub fn scan() -> Vec<EditorApp> {
    platform::scan()
}

pub fn open_path(editor: &EditorApp, path: PathBuf, ctx: &mut AppContext) {
    platform::open_path_with_bundle_identifier(&editor.bundle_identifier, path, ctx);
}

#[cfg(all(target_os = "macos", not(test)))]
mod platform {
    #![allow(deprecated)]

    use super::{EditorApp, RawEditorApp};
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSString};
    use command::r#async::Command;
    use std::path::PathBuf;
    use std::slice;
    use warp_core::paths;
    use warpui::{platform::mac::make_nsstring, AppContext};

    unsafe extern "C" {
        fn scan_editor_apps_json(icon_cache_directory: id) -> id;
    }

    pub fn scan() -> Vec<EditorApp> {
        let icon_cache_dir = paths::cache_dir().join("editor-icons");
        let Some(icon_cache_dir) = icon_cache_dir.to_str() else {
            return Vec::new();
        };

        let json = unsafe {
            let pool = NSAutoreleasePool::new(nil);
            let json = scan_editor_apps_json(make_nsstring(icon_cache_dir));
            let result = nsstring_to_string(json);
            pool.drain();
            result
        };

        let mut editors: Vec<EditorApp> = json
            .and_then(|json| serde_json::from_str::<Vec<RawEditorApp>>(&json).ok())
            .unwrap_or_default()
            .into_iter()
            .map(EditorApp::from)
            .collect();
        editors.sort_by(|left, right| {
            left.display_name
                .to_lowercase()
                .cmp(&right.display_name.to_lowercase())
        });
        editors
    }

    pub fn open_path_with_bundle_identifier(
        bundle_identifier: &str,
        path: PathBuf,
        ctx: &mut AppContext,
    ) {
        let mut command = Command::new("/usr/bin/open");
        command.arg("-b").arg(bundle_identifier).arg(&path);
        match command.spawn() {
            Ok(mut child) => {
                let bundle_identifier = bundle_identifier.to_string();
                ctx.background_executor()
                    .spawn(async move {
                        match child.status().await {
                            Ok(status) if status.success() => {
                                log::info!(
                                    "Opened path in editor bundle {bundle_identifier}: {}",
                                    path.display()
                                );
                            }
                            Ok(status) => {
                                log::error!(
                                    "Opening path in editor bundle {bundle_identifier} exited with status {status}: {}",
                                    path.display()
                                );
                            }
                            Err(err) => {
                                log::error!(
                                    "Unable to await editor bundle {bundle_identifier}: {err:?}"
                                );
                            }
                        }
                    })
                    .detach();
            }
            Err(err) => {
                log::error!("Unable to open path in editor bundle {bundle_identifier}: {err:?}");
                ctx.open_file_path(&path);
            }
        }
    }

    unsafe fn nsstring_to_string(nsstring: id) -> Option<String> {
        unsafe {
            if nsstring == nil {
                return None;
            }
            let cstr = nsstring.UTF8String() as *const u8;
            std::str::from_utf8(slice::from_raw_parts(cstr, nsstring.len()))
                .ok()
                .map(ToOwned::to_owned)
        }
    }
}

#[cfg(any(not(target_os = "macos"), test))]
mod platform {
    use super::EditorApp;
    use std::path::PathBuf;
    use warpui::AppContext;

    pub fn scan() -> Vec<EditorApp> {
        Vec::new()
    }

    pub fn open_path_with_bundle_identifier(
        _bundle_identifier: &str,
        path: PathBuf,
        ctx: &mut AppContext,
    ) {
        ctx.open_file_path(&path);
    }
}
