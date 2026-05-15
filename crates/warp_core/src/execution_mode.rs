use warpui::{Entity, ModelContext, SingletonEntity};

/// Execution mode that Warp is running under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Warp is running as a normal desktop app.
    App,
    /// Warp is running as a CLI.
    CommandLine,
}

/// Model tracking the mode that Warp is running in.
///
/// This gates functionality that's disabled when Warp is running without the full desktop GUI.
#[derive(Clone, Debug)]
pub struct AppExecutionMode {
    mode: ExecutionMode,
    is_sandboxed: bool,
}

impl AppExecutionMode {
    /// Create an `AppExecutionMode` model with the execution mode set.
    pub fn new(mode: ExecutionMode, is_sandboxed: bool, _ctx: &mut ModelContext<Self>) -> Self {
        Self { mode, is_sandboxed }
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

    /// If true, the app is running autonomously, without a user present.
    pub fn is_autonomous(&self) -> bool {
        matches!(self.mode, ExecutionMode::CommandLine)
    }

    /// If true, Warp is running in a sandbox like a Docker container or VM, rather than directly
    /// on a user machine.
    pub fn is_sandboxed(&self) -> bool {
        self.is_sandboxed
    }
}

impl Entity for AppExecutionMode {
    type Event = ();
}

impl SingletonEntity for AppExecutionMode {}
