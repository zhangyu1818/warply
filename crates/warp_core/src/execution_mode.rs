use warpui::{Entity, ModelContext, SingletonEntity};

/// Execution mode that Warp is running under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Warp is running as a normal desktop app.
    App,
    /// Warp is running without the desktop app UI.
    Headless,
}

/// Model tracking the mode that Warp is running in.
///
/// This gates functionality that's disabled when Warp is running without the full desktop GUI.
#[derive(Clone, Debug)]
pub struct AppExecutionMode {
    mode: ExecutionMode,
}

impl AppExecutionMode {
    /// Create an `AppExecutionMode` model with the execution mode set.
    pub fn new(mode: ExecutionMode, _ctx: &mut ModelContext<Self>) -> Self {
        Self { mode }
    }

    /// True if running as the full desktop app.
    fn is_app(&self) -> bool {
        matches!(self.mode, ExecutionMode::App)
    }

    /// Whether Active AI features are allowed in this execution mode.
    ///
    /// Active AI should only run in the desktop app, where there's a user
    /// to engage with it.
    pub fn allows_active_ai(&self) -> bool {
        self.is_app()
    }

    /// Whether the app can save and restore sessions.
    pub fn can_save_session(&self) -> bool {
        self.is_app()
    }
}

impl Entity for AppExecutionMode {
    type Event = ();
}

impl SingletonEntity for AppExecutionMode {}
