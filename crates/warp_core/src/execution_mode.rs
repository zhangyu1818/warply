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
    is_headless: bool,
}

impl AppExecutionMode {
    /// Create an `AppExecutionMode` model with the execution mode set.
    pub fn new(mode: ExecutionMode, _ctx: &mut ModelContext<Self>) -> Self {
        let is_headless = matches!(mode, ExecutionMode::Headless);
        Self { mode, is_headless }
    }

    /// Create an `AppExecutionMode` with the launch's interactive capability set explicitly.
    pub fn new_with_headlessness(
        mode: ExecutionMode,
        is_headless: bool,
        _ctx: &mut ModelContext<Self>,
    ) -> Self {
        Self { mode, is_headless }
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

    /// Whether this launch has no interactive surface where a user can respond.
    pub fn is_headless(&self) -> bool {
        self.is_headless
    }
}

impl Entity for AppExecutionMode {
    type Event = ();
}

impl SingletonEntity for AppExecutionMode {}
