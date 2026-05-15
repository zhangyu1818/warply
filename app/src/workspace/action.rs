use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use warp_util::path::LineAndColumnArg;

use crate::ai::agent::conversation::AIConversationId;
use crate::ai::agent::AIAgentExchangeId;
use crate::ai::document::ai_document_model::{AIDocumentId, AIDocumentVersion};
use crate::drive::CloudObjectTypeAndId;
use crate::object_ids::SyncId;
use crate::palette::PaletteMode;
use crate::prompt::editor_modal::OpenSource as PromptEditorOpenSource;
use crate::search;
use crate::settings_view::{SettingsAction as SettingsTabAction, SettingsSection};
use crate::tab::{NewSessionMenuItem, SelectedTabColor};
use crate::tab_configs::TabConfig;
use crate::terminal::available_shells::AvailableShell;
use crate::terminal::view::inline_banner::ZeroStatePromptSuggestionType;
use crate::themes::theme::AnsiColorIdentifier;
use crate::themes::theme_chooser::ThemeChooserMode;
use crate::ui_events::PaletteSource;
use crate::workflows::{WorkflowSelectionSource, WorkflowSource, WorkflowType};
use crate::workspace::PaneViewLocator;

use ui_components::lightbox;
use warpui::accessibility::AccessibilityVerbosity;
use warpui::geometry::rect::RectF;
use warpui::geometry::vector::Vector2F;
use warpui::platform::Cursor;
use warpui::{EntityId, WindowId};

use super::global_actions::{ForkFromExchange, ForkedConversationDestination};
use super::tab_settings::{
    VerticalTabsCompactSubtitle, VerticalTabsDisplayGranularity, VerticalTabsPrimaryInfo,
    VerticalTabsTabItemMode, VerticalTabsViewMode,
};
use super::view::WorkspaceBanner;

/// This enum determines how the search query is initialized when opening command search.
#[derive(Clone, Default, Debug)]
pub enum InitContent {
    /// Read the content of the active terminal input, and make that the initial search query.
    #[default]
    FromInputBuffer,
    /// Specify an exact string to initialize the query to.
    Custom(String),
}

/// To initialize command search, we may want to specify a search filter, or the content of the
/// query itself.
#[derive(Clone, Default, Debug)]
pub struct CommandSearchOptions {
    pub filter: Option<search::QueryFilter>,
    pub init_content: InitContent,
}

/// Specifies how to restore a conversation when it's not already open in a pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum RestoreConversationLayout {
    /// Restore the conversation into the currently active pane.
    ActivePane,
    /// Restore the conversation in a new split pane.
    SplitPane,
    /// Restore the conversation in a new tab.
    #[default]
    NewTab,
}

#[derive(Debug, Clone, Copy)]
pub enum TabContextMenuAnchor {
    Pointer(Vector2F),
    VerticalTabsKebab,
}

#[derive(Debug, Clone, Copy)]
pub enum VerticalTabsPaneContextMenuTarget {
    ClickedPane(PaneViewLocator),
    ActivePane(PaneViewLocator),
}

impl VerticalTabsPaneContextMenuTarget {
    pub fn locator(self) -> PaneViewLocator {
        match self {
            Self::ClickedPane(locator) | Self::ActivePane(locator) => locator,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorkspaceAction {
    ActivateTab(usize),
    ActivatePrevTab,
    ActivateNextTab,
    ActivateLastTab,
    CyclePrevSession,
    CycleNextSession,
    MoveActiveTabLeft,
    MoveActiveTabRight,
    MoveTabLeft(usize),
    MoveTabRight(usize),
    RenameTab(usize),
    ResetTabName(usize),
    RenamePane(PaneViewLocator),
    ResetPaneName(PaneViewLocator),
    RenameActiveTab,
    /// Renames the focused pane in the active tab. Mirrors `RenameActiveTab`
    /// so the action is reachable from the binding registry / Command Palette
    /// (see #9351). The context-menu path keeps using `RenamePane(locator)`.
    RenameActivePane,
    SetActiveTabName(String),
    /// Sets the manual color override for the active tab.
    ///
    /// - `Color(_)` — apply that color.
    /// - `Cleared` — explicitly clear (suppresses any directory default).
    /// - `Unset` — remove the manual override (lets the directory default apply, if any).
    SetActiveTabColor(SelectedTabColor),
    ToggleTabRightClickMenu {
        tab_index: usize,
        anchor: TabContextMenuAnchor,
    },
    ToggleVerticalTabsPaneContextMenu {
        tab_index: usize,
        target: VerticalTabsPaneContextMenuTarget,
        position: Vector2F,
    },
    TabHoverWidthStart {
        width: f32,
    },
    TabHoverWidthEnd,
    ToggleTabBarOverflowMenu,
    ToggleWelcomeTips,
    CloseTab(usize),
    CloseActiveTab,
    CloseOtherTabs(usize),
    CloseNonActiveTabs,
    CloseTabsRight(usize),
    CloseTabsRightActiveTab,
    AddDefaultTab,
    AddTerminalTab {
        hide_homepage: bool,
    },
    AddTabWithShell {
        shell: AvailableShell,
    },
    /// Add a new tab that immediately enters agent view with a new conversation.
    AddAgentTab,
    /// Add a new tab running a local Docker sandbox via `sbx`.
    AddDockerSandboxTab,
    OpenNewSessionMenu {
        position: Vector2F,
    },
    ToggleTabConfigsMenu,
    ToggleNewSessionMenu {
        position: Vector2F,
    },
    SelectNewSessionMenuItem(NewSessionMenuItem),
    CopyVersion(&'static str),
    ConfigureKeybindingSettings {
        keybinding_name: Option<String>,
    },
    ShowSettings,
    ShowSettingsPage(SettingsSection),
    ShowSettingsPageWithSearch {
        search_query: String,
        section: Option<SettingsSection>,
    },
    ShowThemeChooser(ThemeChooserMode),
    ShowThemeChooserForActiveTheme,
    IncreaseFontSize,
    DecreaseFontSize,
    ResetFontSize,
    IncreaseZoom,
    DecreaseZoom,
    ResetZoom,
    ActivateTabByNumber(usize),
    OpenPalette {
        mode: PaletteMode,
        source: PaletteSource,
        query: Option<String>,
    },
    TogglePalette {
        mode: PaletteMode,
        source: PaletteSource,
    },
    ChangeCursor(Cursor),
    ToggleBlockSnackbar,
    ToggleErrorUnderlining,
    ToggleSyntaxHighlighting,
    SetA11yVerbosityLevel(AccessibilityVerbosity),
    ToggleNotifications,
    ToggleTabColor {
        color: AnsiColorIdentifier,
        tab_index: usize,
    },
    OpenLaunchConfigSaveModal,
    SelectTabConfig(TabConfig),
    DispatchToSettingsTab(SettingsTabAction),
    ToggleResourceCenter,
    ToggleKeybindingsPage,
    ShowCommandSearch(CommandSearchOptions),
    CreateEnvVarCollection,
    ToggleMouseReporting,
    ToggleScrollReporting,
    ToggleFocusReporting,
    StartTabDrag,
    DragTab {
        tab_index: usize,
        tab_position: RectF,
    },
    DropTab,
    /// Toggles the left panel. This happens as explicit action from the user.
    ToggleLeftPanel,
    /// Toggles the right panel. This happens as an explicit action from the user.
    ToggleRightPanel,
    /// Opens the code review panel (right panel) without toggling. If already open,
    /// switches to the target pane's repo. Used by vertical tabs diff stats chip.
    OpenCodeReviewPanel(PaneViewLocator),
    /// Toggles the vertical tabs panel. This happens as an explicit action from the user.
    ToggleVerticalTabsPanel,
    ToggleVerticalTabsSettingsPopup,
    SetVerticalTabsDisplayGranularity(VerticalTabsDisplayGranularity),
    SetVerticalTabsTabItemMode(VerticalTabsTabItemMode),
    SetVerticalTabsViewMode(VerticalTabsViewMode),
    SetVerticalTabsPrimaryInfo(VerticalTabsPrimaryInfo),
    SetVerticalTabsCompactSubtitle(VerticalTabsCompactSubtitle),
    ToggleVerticalTabsShowPrLink,
    ToggleVerticalTabsShowDiffStats,
    ToggleVerticalTabsShowDetailsOnHover,
    /// Closes the focused panel. This happens as an explicit action from the user.
    ClosePanel,
    CopyTextToClipboard(String),
    DismissWorkspaceBanner(WorkspaceBanner),
    /// An action only registered in dev and local builds, which triggers a
    /// panic immediately when called.
    Panic,
    /// Stops the heap profiler (if one is running) and writes the profiling
    /// data to disk.
    DumpHeapProfile,
    /// An action to open a new window with a view hierarchy debugger.
    OpenViewTreeDebugWindow,
    /// An action to either mirror terminal input across all tabs or stop mirroring it.
    ToggleSyncAllTerminalInputsInAllTabs,
    /// An action to either mirror terminal input within one tab or cancel input mirroring.
    ToggleSyncTerminalInputsInTab,
    /// An action to force terminal input mirroring off
    DisableTerminalInputSync,
    HandleConflictingWorkflow(SyncId),
    HandleConflictingEnvVarCollection(SyncId),
    OpenPromptEditor {
        open_source: PromptEditorOpenSource,
    },
    OpenAgentToolbarEditor,
    OpenCLIAgentToolbarEditor,
    OpenHeaderToolbarEditor,
    ShowHeaderToolbarContextMenu {
        position: Vector2F,
    },
    OpenLink(String),
    ReopenClosedSession,
    AddWindow,
    AddWindowWithShell {
        shell: AvailableShell,
    },
    /// Moves focus to the panel on the left
    FocusLeftPanel,
    /// Moves focus to the panel on the right
    FocusRightPanel,
    ViewObject(CloudObjectTypeAndId),
    /// Open a local path in the file explorer.
    OpenInExplorer {
        path: PathBuf,
    },
    /// Open a local file with the system's default application.
    OpenFilePath {
        path: PathBuf,
    },
    TerminateApp,
    CloseWindow,
    /// Log review comment send eligibility for panes in the active tab.
    LogReviewCommentSendStatusForActiveTab,
    ToggleRecordingMode,
    ToggleInBandGenerators,
    ToggleShowMemoryStats,
    RunAISuggestedCommand(String),
    RunCommand(String),
    InsertInInput {
        content: String,
        replace_buffer: bool,
        /// Whether to ensure agent mode is enabled when inserting content
        ensure_agent_mode: bool,
    },
    /// Open a new tab with its input in AI mode.
    NewTabInAgentMode {
        /// The type of zero state prompt suggestion to start with (optional).
        zero_state_prompt_suggestion_type: Option<ZeroStatePromptSuggestionType>,
    },
    /// Open a new pane with its input in AI mode.
    NewPaneInAgentMode {
        /// The type of zero state prompt suggestion to start with (optional).
        zero_state_prompt_suggestion_type: Option<ZeroStatePromptSuggestionType>,
    },
    /// Open a new pane with its input in AI mode
    /// with query "Fix this" with error name and details from AI summary.
    FixInAgentMode {
        query: String,
    },
    OpenAIFactCollection,
    ToggleAIDocumentPane {
        document_id: AIDocumentId,
        document_version: AIDocumentVersion,
    },
    /// Closes all visible AI document panes in the active pane group.
    HideAIDocumentPanes,
    /// Closes any other ai document panes in the active pane group, and opens the specified document_id.
    OpenAIDocumentPane {
        document_id: AIDocumentId,
        document_version: AIDocumentVersion,
    },
    FocusTerminalViewInWorkspace {
        terminal_view_id: EntityId,
    },
    /// Focus a specific pane by its locator (pane_group_id and pane_id).
    FocusPane(PaneViewLocator),
    /// Start a new AI conversation in a terminal view. This sets the pending query state
    /// to default and focuses the terminal view.
    StartNewConversation {
        terminal_view_id: EntityId,
    },
    /// Open a file in a new tab with a code pane
    OpenFileInNewTab {
        full_path: PathBuf,
        line_and_column: Option<LineAndColumnArg>,
    },
    RunWorkflow {
        workflow: Arc<WorkflowType>,
        workflow_source: WorkflowSource,
        workflow_selection_source: WorkflowSelectionSource,
        argument_override: Option<HashMap<String, String>>,
    },
    ScrollToSettingsWidget {
        page: SettingsSection,
        widget_id: &'static str,
    },
    /// Navigate to an existing AI conversation, focusing on its terminal view.
    ///
    /// If the conversation is not in an open pane, restore it based on the layout setting or override.
    RestoreOrNavigateToConversation {
        pane_view_locator: Option<PaneViewLocator>,
        window_id: Option<WindowId>,
        conversation_id: AIConversationId,
        terminal_view_id: Option<EntityId>,
        /// If provided, use this layout to restore the conversation.
        /// Otherwise, fall back to the user's setting.
        restore_layout: Option<RestoreConversationLayout>,
    },
    /// Fork an existing AI conversation.
    ForkAIConversation {
        conversation_id: AIConversationId,
        fork_from_exchange: Option<ForkFromExchange>,
        /// Initial prompt to send in the forked conversation.
        initial_prompt: Option<String>,
        /// Where to open the forked conversation.
        destination: ForkedConversationDestination,
    },
    /// Fork an existing AI conversation into a new pane and prefill the input with a local
    /// continuation command (selecting all text).
    ContinueConversationLocally {
        conversation_id: AIConversationId,
    },
    /// Insert the /fork slash command into the active terminal's input.
    InsertForkSlashCommand,
    /// Queue a prompt to be sent after the current conversation finishes.
    QueuePromptForConversation {
        prompt: String,
    },
    UndoRevertInCodeReviewPane {
        window_id: WindowId,
        view_id: EntityId,
    },
    /// Handle a file being renamed in the file tree
    #[cfg(feature = "local_fs")]
    FileRenamed {
        old_path: PathBuf,
        new_path: PathBuf,
    },
    /// Handle a file being deleted in the file tree
    #[cfg(feature = "local_fs")]
    FileDeleted {
        path: PathBuf,
    },
    /// Open a repository directory via file picker. The `path` is an `Option` because some
    /// dispatchers don't know the path to open yet.
    OpenRepository {
        path: Option<String>,
    },
    /// Open the native folder picker for a repo param in the tab-config modal after the
    /// current interaction cycle finishes.
    OpenTabConfigRepoPicker {
        param_index: usize,
    },
    /// Open a new blank code file in the current tab
    NewCodeFile,
    NavigatePrevPaneOrPanel,
    NavigateNextPaneOrPanel,
    ToggleProjectExplorer,
    ToggleGlobalSearch,
    OpenGlobalSearch,
    ToggleConversationListView,
    /// Take a process sample of the app (equivalent to Activity Monitor > Sample Process).
    SampleProcess,
    /// Show the rewind confirmation dialog before rewinding an AI conversation
    ShowRewindConfirmationDialog {
        ai_block_view_id: EntityId,
        exchange_id: AIAgentExchangeId,
        conversation_id: AIConversationId,
    },
    /// Execute the actual rewind after confirmation
    ExecuteRewindAIConversation {
        ai_block_view_id: EntityId,
        exchange_id: AIAgentExchangeId,
        conversation_id: AIConversationId,
    },
    /// Execute the actual deletion of a conversation after confirmation
    ExecuteDeleteConversation {
        conversation_id: AIConversationId,
        terminal_view_id: Option<EntityId>,
    },
    /// Open a full-window lightbox displaying the given images.
    OpenLightbox {
        images: Vec<lightbox::LightboxImage>,
        /// The index of the image to display initially.
        initial_index: usize,
    },
    /// Update a single image in the currently open lightbox.
    UpdateLightboxImage {
        index: usize,
        image: lightbox::LightboxImage,
    },
    ShowSessionConfigModal,
    /// Open the "New worktree" modal for creating a reusable worktree tab config.
    OpenNewWorktreeModal,
    /// Open the native folder picker for the repo field in the new-worktree modal.
    OpenNewWorktreeRepoPicker,
    /// Create a new worktree in the given repo using the default worktree tab config.
    /// The branch name is auto-generated.
    OpenWorktreeInRepo {
        repo_path: String,
    },
    /// Open a folder picker to add a new repo to PersistedWorkspace (from the
    /// "New worktree config" submenu's "+ Add new repo..." item).
    OpenWorktreeAddRepoPicker,
    SaveCurrentTabAsNewConfig(usize),
    SyncTrafficLights,
    /// Opens a tab config file in the editor and dismisses the associated error toast.
    OpenTabConfigErrorFile {
        path: PathBuf,
        toast_object_id: String,
    },
    /// Sidecar action: set the hovered item as the Cmd+T default.
    TabConfigSidecarMakeDefault {
        mode: crate::settings::ai::DefaultSessionMode,
        tab_config_path: Option<PathBuf>,
        shell: Option<AvailableShell>,
    },
    /// Sidecar action: open the tab config TOML in the user's editor.
    TabConfigSidecarEditConfig {
        path: PathBuf,
    },
    /// Sidecar action: show the remove confirmation dialog for a tab config.
    TabConfigSidecarRemoveConfig {
        name: String,
        path: PathBuf,
    },
    /// Opens the settings.toml file in a code editor pane.
    OpenSettingsFile,
}

impl WorkspaceAction {
    /// Matches what actions require the app state to be saved, and which don't. We match all
    /// actions directly, rather than using _, so we're forced to make a conscious decision for each
    /// of them, rather than following some default.
    pub fn should_save_app_state_on_action(&self) -> bool {
        use WorkspaceAction::*;
        match self {
            ContinueConversationLocally { .. } => true,
            ActivateTab(_)
            | ActivateTabByNumber(_)
            | ActivatePrevTab
            | ActivateNextTab
            | ActivateLastTab
            | CyclePrevSession
            | CycleNextSession
            | MoveActiveTabLeft
            | MoveActiveTabRight
            | MoveTabLeft(_)
            | MoveTabRight(_)
            | DropTab
            | RenameTab(_)
            | ResetTabName(_)
            | RenamePane(_)
            | ResetPaneName(_)
            | RenameActiveTab
            | RenameActivePane
            | SetActiveTabName(_)
            | SetActiveTabColor(_)
            | CloseTab(_)
            | CloseActiveTab
            | CloseOtherTabs(_)
            | CloseNonActiveTabs
            | CloseTabsRight(_)
            | CloseTabsRightActiveTab
            | ToggleTabColor { .. }
            | AddDefaultTab
            | AddTerminalTab { .. }
            | AddTabWithShell { .. }
            | AddAgentTab
            | AddDockerSandboxTab
            | AddWindow
            | AddWindowWithShell { .. }
            | CloseWindow
            | ScrollToSettingsWidget { .. }
            | NewTabInAgentMode { .. }
            | NewPaneInAgentMode { .. }
            | FixInAgentMode { .. }
            | RunWorkflow { .. }
            | OpenFileInNewTab { .. }
            | RestoreOrNavigateToConversation { .. }
            | NewCodeFile
            | ForkAIConversation { .. }
            | OpenRepository { .. }
            | SelectTabConfig(_)
            | ToggleVerticalTabsPanel => true, // actions that actually change a state of the state of user's
            // workspace would most likely require a save, so that if the app gets
            // restarted, the user can continue working
            CopyVersion(_)
            | ConfigureKeybindingSettings { .. }
            | ShowSettings
            | ShowSettingsPage(_)
            | ShowSettingsPageWithSearch { .. }
            | ShowThemeChooser(_)
            | ShowThemeChooserForActiveTheme
            | IncreaseFontSize
            | DecreaseFontSize
            | ResetFontSize
            | IncreaseZoom
            | DecreaseZoom
            | ResetZoom
            | OpenPalette { .. }
            | TogglePalette { mode: _, source: _ }
            | ChangeCursor(_)
            | ToggleBlockSnackbar
            | ToggleErrorUnderlining
            | ToggleSyntaxHighlighting
            | OpenLaunchConfigSaveModal
            | ToggleTabRightClickMenu { .. }
            | ToggleVerticalTabsPaneContextMenu { .. }
            | OpenNewSessionMenu { .. }
            | ToggleTabConfigsMenu
            | ToggleNewSessionMenu { .. }
            | SelectNewSessionMenuItem(_)
            | ToggleTabBarOverflowMenu
            | SetA11yVerbosityLevel(_)
            | ToggleNotifications
            | DispatchToSettingsTab { .. }
            | ToggleResourceCenter
            | ToggleKeybindingsPage
            | ShowCommandSearch(_)
            | ToggleMouseReporting
            | ToggleScrollReporting
            | ToggleFocusReporting
            | CreateEnvVarCollection
            | OpenInExplorer { .. }
            | DragTab { .. }
            | StartTabDrag
            | ToggleLeftPanel
            | ClosePanel
            | ToggleRightPanel
            | OpenCodeReviewPanel(..)
            | ToggleVerticalTabsSettingsPopup
            | SetVerticalTabsDisplayGranularity(_)
            | SetVerticalTabsTabItemMode(_)
            | SetVerticalTabsViewMode(_)
            | SetVerticalTabsPrimaryInfo(_)
            | SetVerticalTabsCompactSubtitle(_)
            | ToggleVerticalTabsShowPrLink
            | ToggleVerticalTabsShowDiffStats
            | ToggleVerticalTabsShowDetailsOnHover
            | ToggleWelcomeTips
            | CopyTextToClipboard(_)
            | OpenTabConfigRepoPicker { .. }
            | OpenNewWorktreeModal
            | OpenNewWorktreeRepoPicker
            | OpenWorktreeInRepo { .. }
            | OpenWorktreeAddRepoPicker
            | Panic
            | DumpHeapProfile
            | OpenViewTreeDebugWindow
            | DismissWorkspaceBanner(..)
            | ToggleSyncAllTerminalInputsInAllTabs
            | ToggleSyncTerminalInputsInTab
            | DisableTerminalInputSync
            | HandleConflictingWorkflow(_)
            | HandleConflictingEnvVarCollection(_)
            | OpenPromptEditor { .. }
            | OpenAgentToolbarEditor
            | OpenCLIAgentToolbarEditor
            | OpenHeaderToolbarEditor
            | ShowHeaderToolbarContextMenu { .. }
            | OpenLink(_)
            | ReopenClosedSession
            | FocusLeftPanel
            | FocusRightPanel
            | LogReviewCommentSendStatusForActiveTab
            | ToggleRecordingMode
            | ToggleInBandGenerators
            | ToggleShowMemoryStats
            | RunAISuggestedCommand { .. }
            | RunCommand { .. }
            | InsertInInput { .. }
            | InsertForkSlashCommand
            | QueuePromptForConversation { .. }
            | OpenFilePath { .. }
            | ViewObject(_)
            | TerminateApp
            | TabHoverWidthStart { .. }
            | TabHoverWidthEnd
            | OpenAIFactCollection
            | FocusTerminalViewInWorkspace { .. }
            | FocusPane(..)
            | StartNewConversation { .. }
            | UndoRevertInCodeReviewPane { .. }
            | NavigatePrevPaneOrPanel
            | NavigateNextPaneOrPanel
            | ToggleProjectExplorer
            | ToggleGlobalSearch
            | OpenGlobalSearch
            | ToggleConversationListView
            | ToggleAIDocumentPane { .. }
            | HideAIDocumentPanes
            | OpenAIDocumentPane { .. }
            | ShowRewindConfirmationDialog { .. }
            | ExecuteRewindAIConversation { .. }
            | ExecuteDeleteConversation { .. }
            | OpenLightbox { .. }
            | UpdateLightboxImage { .. }
            | ShowSessionConfigModal
            | SaveCurrentTabAsNewConfig(_)
            | SyncTrafficLights
            | OpenTabConfigErrorFile { .. }
            | TabConfigSidecarMakeDefault { .. }
            | TabConfigSidecarEditConfig { .. }
            | TabConfigSidecarRemoveConfig { .. }
            | OpenSettingsFile => false,
            SampleProcess => false,
            #[cfg(feature = "local_fs")]
            FileRenamed { .. } => false, // File rename doesn't change workspace state
            #[cfg(feature = "local_fs")]
            FileDeleted { .. } => false, // File deletion doesn't change workspace state
                                         // actions that are related to updating user settings or
                                         // managing some ui elements (like closing/opening modals)
                                         // that don't reflect on actual workspace and don't need to
                                         // be preserved between restarts.
        }
    }
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod tests;
