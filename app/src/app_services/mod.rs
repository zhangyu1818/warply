//! Functionality relating to services that the application provides
//! to the host system.
//!
//! For example, on macOS, this module sets up integrations with
//! Finder such that the user can open a new Warp tab or window
//! in a given directory.

#[cfg(target_os = "macos")]
mod mac;

pub fn init(_ctx: &mut warpui::AppContext) {
    log::info!("Initializing app services");

    mac::init();
}

pub fn teardown(_ctx: &mut warpui::AppContext) {
    log::info!("Tearing down app services...");
}
