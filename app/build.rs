#![allow(clippy::disallowed_types)]

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{env, fs, process::Command};
use walkdir::WalkDir;
use warp_util::path::app_target_dir;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    add_features();

    println!("cargo:rustc-link-lib=framework=MetalKit");
    println!("cargo:rustc-link-lib=framework=UserNotifications");
    println!("cargo:rerun-if-changed=src/platform/mac/objc/app_bundle.h");
    println!("cargo:rerun-if-changed=src/platform/mac/objc/app_bundle.m");
    println!("cargo:rerun-if-changed=src/platform/mac/objc/services.h");
    println!("cargo:rerun-if-changed=src/platform/mac/objc/services.m");
    println!("cargo:rerun-if-changed=src/platform/mac/objc/updater.h");
    println!("cargo:rerun-if-changed=src/platform/mac/objc/updater.m");

    cc::Build::new()
        .file("src/platform/mac/objc/app_bundle.m")
        .file("src/platform/mac/objc/services.m")
        .file("src/platform/mac/objc/updater.m")
        .compile("warp_objc");

    // Build the dock tile plugin
    println!("cargo:rerun-if-changed=DockTilePlugin/WarplyDockTilePlugin.m");
    println!("cargo:rerun-if-changed=DockTilePlugin/WarplyDockTilePlugin.h");
    println!("cargo:rerun-if-changed=DockTilePlugin/Info.plist");
    println!("cargo:rerun-if-changed=DockTilePlugin/Makefile");

    let min_macos_version = env::var("MACOSX_DEPLOYMENT_TARGET")
        .expect("MACOSX_DEPLOYMENT_TARGET must be set for macos builds");
    let status = Command::new("make")
        .current_dir("DockTilePlugin")
        .env("MACOSX_DEPLOYMENT_TARGET", min_macos_version)
        .status()
        .expect("Failed to build dock tile plugin");
    if !status.success() {
        panic!("Dock tile plugin build failed");
    }

    // Copy the dock tile plugin to the output directory
    let profile = get_build_profile_name();
    let target_dir = app_target_dir(&profile).expect("Failed to get app target directory");
    let plugin_src = Path::new("DockTilePlugin/WarplyDockTilePlugin.docktileplugin");
    let plugin_dst = target_dir.join("WarplyDockTilePlugin.docktileplugin");

    if plugin_src.exists() {
        fs::remove_dir_all(&plugin_dst).ok(); // Remove existing if any
        fs::create_dir_all(&plugin_dst).expect("Failed to create plugin directory");

        // Copy the plugin directory recursively
        for entry in WalkDir::new(plugin_src) {
            let entry = entry.expect("Failed to read plugin directory");
            let path = entry.path();
            let relative = path
                .strip_prefix(plugin_src)
                .expect("Failed to strip path prefix");
            let target = plugin_dst.join(relative);

            if path.is_dir() {
                fs::create_dir_all(target).expect("Failed to create plugin subdirectory");
            } else {
                fs::copy(path, target).expect("Failed to copy plugin file");
            }
        }

        // Clean up the source plugin directory after copying
        fs::remove_dir_all(plugin_src).expect("Failed to clean up plugin directory");
    }

    // In standalone mode, embed the Info.plist file. We don't use embed_plist! for this
    // because the plist file is dynamically generated.
    if env::var("CARGO_FEATURE_STANDALONE").is_ok() {
        // Don't fail if INFO_PLIST_PATH is unset, since CI runs clippy with --all-features.
        if let Ok(info_plist_path) = env::var("INFO_PLIST_PATH") {
            println!("cargo:rerun-if-env-changed=INFO_PLIST_PATH");
            println!("cargo:rerun-if-changed={info_plist_path}");
            println!("cargo:rustc-link-arg=-sectcreate");
            println!("cargo:rustc-link-arg=__TEXT");
            println!("cargo:rustc-link-arg=__info_plist");
            println!("cargo:rustc-link-arg={info_plist_path}");
        } else {
            eprintln!("Expected INFO_PLIST_PATH to be set")
        }
    }

    Ok(())
}

fn get_build_profile_name() -> String {
    // The profile name is always the 3rd last part of the path (with 1 based indexing).
    // e.g. /code/core/target/cli/build/my-build-info-9f91ba6f99d7a061/out
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set"));
    out_dir
        .ancestors()
        .nth(3)
        .and_then(Path::file_name)
        .expect("could not get profile name")
        .to_string_lossy()
        .into_owned()
}

fn add_features() {
    println!("cargo:rustc-cfg=feature=\"local_fs\"");
    println!("cargo:rustc-cfg=feature=\"local_tty\"");
    println!("cargo:rustc-cfg=feature=\"iterm_images\"");

    if env::var("PROFILE").ok().is_some_and(|val| val == "debug") {
        println!("cargo:rustc-cfg=feature=\"agent_mode_debug\"");
    }
}
