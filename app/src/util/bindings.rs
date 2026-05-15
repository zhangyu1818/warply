use crate::keyboard::{remove_custom_keybinding, write_custom_keybinding, UserDefinedKeybinding};
use crate::settings_view::keybindings::{KeybindingChangedEvent, KeybindingChangedNotifier};
use enum_iterator::{all, Sequence};
use fuzzy_match::match_indices_case_insensitive;
use itertools::Itertools;
use lazy_static::lazy_static;

use regex::Regex;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    sync::Arc,
};
use warpui::keymap::{BindingId, IsBindingValid};
use warpui::{
    actions::StandardAction,
    keymap::{
        BindingDescription, BindingLens, CustomTag, DescriptionContext, EditableBindingLens,
        Keystroke, Trigger,
    },
    Action,
};
use warpui::{AppContext, SingletonEntity};

pub const MAC_MENUS_CONTEXT: DescriptionContext = DescriptionContext::Custom("mac_menus");

// CustomActions are attached to menu items, and may be attached to Bindings.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Sequence)]
#[repr(isize)]
pub enum CustomAction {
    NewTab,
    NewFile,
    ShowAboutWarp,
    ShowSettings,
    ConfigureKeybindings,
    ShowAppearance,
    FocusInput,
    ClearBlocks,
    AddNextOccurrence,
    AddCursorAbove,
    AddCursorBelow,
    CycleNextSession,
    CyclePrevSession,
    Cut,
    Copy,
    Paste,
    Undo,
    Redo,
    CommandPalette,
    AISearch,
    ClearEditor,
    Find,
    SelectAll,
    Workflows,
    HistorySearch,
    SaveCurrentConfig,
    History,
    IncreaseFontSize,
    DecreaseFontSize,
    ResetFontSize,
    IncreaseZoom,
    DecreaseZoom,
    ResetZoom,
    RenameTab,
    SplitPaneRight,
    SplitPaneLeft,
    SplitPaneUp,
    SplitPaneDown,
    MoveTabLeft,
    MoveTabRight,
    ActivateNextTab,
    ActivatePreviousTab,
    ActivateNextPane,
    ActivatePreviousPane,
    NavigationPalette,
    SelectBlockAbove,
    SelectBlockBelow,
    SelectAllBlocks,
    ToggleBookmarkBlock,
    FindWithinBlock,
    CopyBlock,
    CopyBlockCommand,
    CopyBlockOutput,
    CloseTab,
    CloseOtherTabs,
    CloseTabsRight,
    ToggleMaximizePane,
    LaunchConfigPalette,
    FilesPalette,
    TriggerWelcomeBlock,
    CommandSearch,
    ToggleResourceCenter,
    ToggleKeybindingsPage,
    ScrollToTopOfSelectedBlocks,
    ScrollToBottomOfSelectedBlocks,
    ToggleSyncAllTerminalInputsInAllTabs,
    ToggleSyncTerminalInputsInCurrentTab,
    DisableSyncTerminalInputs,
    ReopenClosedSession,
    AddWindow,
    CloseCurrentSession,
    CloseWindow,
    NewAgentModePane,
    AttachSelectionAsAgentModeContext,
    OpenAIFactCollection,
    ToggleProjectExplorer,
    OpenRepository,
    NewTerminalTab,
    NewAgentTab,
    GoToLine,
    ToggleGlobalSearch,
    ToggleConversationListView,
}

lazy_static! {
    /// Maps for converting from custom tags back to the action enum
    /// This layer of indirection is necessary because the UI framework can't
    /// know about particular Warp specific actions, so it deals with all actions
    /// as plain isizes.  Within Warp though we want to deal with them as the enum type.
    pub static ref CUSTOM_TAG_TO_ACTION: HashMap<isize, CustomAction> = HashMap::from_iter(all::<CustomAction>().map(|action| {
        (action as isize, action)
    }));

    /// Regex that matches whether the the normalized form of a [`Keystroke`] matches a control
    /// character. ASCII control characters constitute the first 31 values of ASCII characters.
    /// Though they have their own ASCII codepoints, they are typed into the keyboard using
    /// `ctrl-XX`, see <https://en.wikipedia.org/wiki/Caret_notation>.
    ///
    /// As an example, the ETX character (represented as `^C` in caret notation) is sent to
    /// the PTY when the user presses `ctrl-c`.
    ///
    /// ## Control Characters List
    /// The full list of these control characters (and their corresponding name) are documented
    /// below:
    /// * `^@`: Null
    /// * `^A`: Start of Header
    /// * `^B`: Start of Text
    /// * `^C`: End of Text
    /// * `^D`: End of Transmission
    /// * `^E`: Enquiry
    /// * `^F`: Acknowledge
    /// * `^G`: Bell
    /// * `^H`: BackSpace
    /// * `^I`: Horizontal Tabulation
    /// * `^J`: Line Feed
    /// * `^K`: Vertical Tabulation
    /// * `^L`: Form Feed
    /// * `^M`: Carriage Return
    /// * `^N`: Shift Out
    /// * `^O`: Shift In
    /// * `^P`: Data Link Escape
    /// * `^Q`: Device Control 1 (XON)
    /// * `^R`: Device Control 2
    /// * `^S`: Device Control 3 (XOFF)
    /// * `^T`: Device Control 4
    /// * `^U`: Negative acknowledge
    /// * `^V`: Synchronous Idle
    /// * `^W`: End of Transmission Block
    /// * `^X`: Cancel
    /// * `^Y`: End of Medium
    /// * `^Z`: Substitute
    /// * `^[`: Escape
    /// * `^\`: File Separator
    /// * `^]`: Group Separator
    /// * `^^`: Record Separator
    /// * `^_`: Unit Separator
    /// * `^?`: Delete
    ///
    /// ## Note
    /// Though caret notation uses uppercase letters (`^C` instead of `^c`), we validate using
    /// _lowercase_ characters because it is impossible to create a [`Keystroke`] of the form
    /// `ctrl-[A-Z]`. See [`Keystroke::parse`].
    pub static ref CONTROL_CHARACTER_KEY_REGEX: Regex = Regex::new(r"^ctrl-[a-z@\[\\\]^_?]$").expect("should be able to construct regex");

    /// Set of actions on Mac that should be considered valid bindings even though they aren't PTY
    /// compliant. We weren't always diligent about avoiding bindings that could conflict with
    /// character codes, unfortunately some bindings on Mac currently conflict with the PTY. We have
    /// this allowlist to special case these legacy actions for the purposes of binding validation.
    pub static ref MAC_PTY_NON_COMPLIANT_ACTIONS: HashSet<&'static str> = HashSet::from_iter(["terminal:warpify_subshell", "terminal:open_block_list_context_menu_via_keybinding"]);

    /// Set of actions on Windows that should be considered valid bindings even though they aren't
    /// PTY compliant. Windows users expect pasting to work using both `ctrl-v` and `ctrl-shift-v`,
    /// so we allowlist the terminal paste action for the purposes of binding validation.
    /// Set of keystrokes that should be considered valid bindings on all platforms even though
    /// they aren't PTY compliant.
    pub static ref PTY_NON_COMPLIANT_KEYSTROKES: HashSet<Keystroke> = HashSet::from_iter([
        // Windows users expect ctrl-c to copy any selected text to the clipboard. To avoid
        // introducing multiple codepaths for handling ctrl-c, we register ctrl-c as a binding
        // on TerminalView on all platforms.
        Keystroke::parse("ctrl-c").expect("should be able to construct ctrl-c keystroke"),
        // The resume conversation binding uses cmd-shift-R on Mac and should be allowed
        Keystroke::parse("cmd-shift-R").expect("should be able to construct cmd-shift-R keystroke")
    ]);
}

impl From<CustomAction> for CustomTag {
    fn from(action: CustomAction) -> Self {
        action as CustomTag
    }
}

impl From<CustomTag> for CustomAction {
    fn from(tag: CustomTag) -> Self {
        *CUSTOM_TAG_TO_ACTION
            .get(&tag)
            .expect("All custom actions are handled.")
    }
}

pub fn trigger_to_keystroke(trigger: &Trigger) -> Option<Keystroke> {
    match trigger {
        Trigger::Keystrokes(keys) => keys.first().cloned(),
        // Custom actions don't have keyboard shortcuts associated with the actions themselves,
        // they are set separately in app/src/lib.rs as part of creating the Menu. As a result,
        // we need to map those to the appropriate keyboard shortcut.
        Trigger::Custom(custom) => custom_tag_to_keystroke(*custom),
        // Similarly, Standard Actions have their keyboard shortcuts set when creating the menu
        Trigger::Standard(standard) => match standard {
            StandardAction::Close => parse_keystroke("cmd-shift-W"),
            StandardAction::Quit => parse_keystroke("cmd-q"),
            StandardAction::Hide => parse_keystroke("cmd-h"),
            StandardAction::HideOtherApps => Keystroke::parse("cmdorctrl-alt-h").ok(),
            StandardAction::ToggleFullScreen => parse_keystroke("cmd-ctrl-f"),
            StandardAction::Paste => Keystroke::parse(cmd_or_ctrl_shift("v")).ok(),
            StandardAction::ShowAllApps
            | StandardAction::BringAllToFront
            | StandardAction::Minimize
            | StandardAction::Zoom => None,
        },
        Trigger::Empty => None,
    }
}

/// Returns the corresponding [`Keystroke`], if any, of a [`CustomTag`].
pub fn custom_tag_to_keystroke(custom: CustomTag) -> Option<Keystroke> {
    match custom.into() {
        CustomAction::FocusInput => Keystroke::parse(cmd_or_ctrl_shift("l")).ok(),
        CustomAction::NewTab => Keystroke::parse(cmd_or_ctrl_shift("t")).ok(),
        CustomAction::Cut => Keystroke::parse("cmdorctrl-x").ok(),
        CustomAction::Copy => Keystroke::parse(cmd_or_ctrl_shift("c")).ok(),
        CustomAction::Paste => Keystroke::parse(cmd_or_ctrl_shift("v")).ok(),
        CustomAction::Undo => Keystroke::parse("cmdorctrl-z").ok(),
        CustomAction::Redo => Keystroke::parse("cmdorctrl-shift-Z").ok(),
        CustomAction::ClearEditor => Keystroke::parse("ctrl-c").ok(),
        CustomAction::CycleNextSession => Keystroke::parse("ctrl-tab").ok(),
        CustomAction::CyclePrevSession => Keystroke::parse("ctrl-shift-tab").ok(),
        CustomAction::ShowSettings => Keystroke::parse("cmdorctrl-,").ok(),
        CustomAction::AddNextOccurrence => Keystroke::parse("ctrl-g").ok(),
        CustomAction::AddCursorAbove => Keystroke::parse("ctrl-shift-up").ok(),
        CustomAction::AddCursorBelow => Keystroke::parse("ctrl-shift-down").ok(),
        CustomAction::CommandPalette => Keystroke::parse(cmd_or_ctrl_shift("p")).ok(),
        CustomAction::AISearch => Keystroke::parse("ctrl-`").ok(),
        CustomAction::Find => Keystroke::parse(cmd_or_ctrl_shift("f")).ok(),
        CustomAction::SelectAll => Keystroke::parse("cmdorctrl-a").ok(),
        CustomAction::CommandSearch => Keystroke::parse("ctrl-r").ok(),
        CustomAction::Workflows => Keystroke::parse("ctrl-shift-R").ok(),
        CustomAction::History => Keystroke::parse("up").ok(),
        CustomAction::IncreaseFontSize => Keystroke::parse("shift-cmdorctrl-+").ok(),
        CustomAction::DecreaseFontSize => Keystroke::parse("shift-cmdorctrl-_").ok(),
        CustomAction::ResetFontSize => Keystroke::parse("cmdorctrl-0").ok(),
        CustomAction::IncreaseZoom => Keystroke::parse("cmdorctrl-=").ok(),
        CustomAction::DecreaseZoom => Keystroke::parse("cmdorctrl--").ok(),
        CustomAction::ResetZoom => Keystroke::parse("cmdorctrl-0").ok(),
        CustomAction::SplitPaneRight => Keystroke::parse(cmd_or_ctrl_shift("d")).ok(),
        CustomAction::SplitPaneDown => Keystroke::parse("cmd-shift-D").ok(),
        CustomAction::MoveTabLeft => Keystroke::parse("shift-ctrl-left").ok(),
        CustomAction::MoveTabRight => Keystroke::parse("shift-ctrl-right").ok(),
        CustomAction::ActivateNextTab => Keystroke::parse("shift-cmd-}").ok(),
        CustomAction::ActivatePreviousTab => Keystroke::parse("shift-cmd-{").ok(),
        CustomAction::ActivateNextPane => Keystroke::parse("cmd-]").ok(),
        CustomAction::ActivatePreviousPane => Keystroke::parse("cmd-[").ok(),
        CustomAction::NavigationPalette => parse_keystroke("cmd-shift-P"),
        CustomAction::LaunchConfigPalette => parse_keystroke("ctrl-cmd-l"),
        CustomAction::FilesPalette => Keystroke::parse(cmd_or_ctrl_shift("o")).ok(),
        CustomAction::ClearBlocks => Keystroke::parse(cmd_or_ctrl_shift("k")).ok(),
        CustomAction::SelectBlockAbove => Keystroke::parse("cmdorctrl-up").ok(),
        CustomAction::SelectBlockBelow => Keystroke::parse("cmdorctrl-down").ok(),
        CustomAction::ToggleBookmarkBlock => Keystroke::parse(cmd_or_ctrl_shift("b")).ok(),
        CustomAction::CopyBlockOutput => Keystroke::parse("cmdorctrl-alt-shift-C").ok(),
        CustomAction::CopyBlockCommand => parse_keystroke("cmd-shift-C"),
        CustomAction::ToggleMaximizePane => parse_keystroke("cmd-shift-enter"),
        // Note: The base character '/' is used instead of '?' as mac registers keybindings
        // differently compared to the app which saves the resulting character used with shift
        // TODO: resolve these keybinding differences
        CustomAction::ToggleResourceCenter => Keystroke::parse("ctrl-shift-/").ok(),
        CustomAction::ToggleKeybindingsPage => Keystroke::parse("cmdorctrl-/").ok(),
        CustomAction::ScrollToTopOfSelectedBlocks => Keystroke::parse("cmdorctrl-shift-up").ok(),
        CustomAction::ScrollToBottomOfSelectedBlocks => {
            Keystroke::parse("cmdorctrl-shift-down").ok()
        }
        CustomAction::CopyBlock => Keystroke::parse(cmd_or_ctrl_shift("c")).ok(),
        CustomAction::FindWithinBlock => Keystroke::parse(cmd_or_ctrl_shift("f")).ok(),
        CustomAction::ToggleSyncTerminalInputsInCurrentTab => {
            Keystroke::parse("alt-cmdorctrl-i").ok()
        }
        CustomAction::ReopenClosedSession => Keystroke::parse("cmd-shift-T").ok(),

        // This is one of the app's hardcoded keybindings.
        CustomAction::AddWindow => Keystroke::parse(cmd_or_ctrl_shift("n")).ok(),
        CustomAction::CloseWindow => parse_keystroke("cmd-shift-W"),
        CustomAction::CloseCurrentSession => Keystroke::parse(cmd_or_ctrl_shift("w")).ok(),
        CustomAction::NewAgentModePane => Keystroke::parse("ctrl-space").ok(),
        CustomAction::AttachSelectionAsAgentModeContext => {
            Keystroke::parse("ctrl-shift-space").ok()
        }
        CustomAction::ToggleProjectExplorer => Keystroke::parse("ctrl-2").ok(),
        CustomAction::OpenRepository => Keystroke::parse("cmd-shift-O").ok(),
        CustomAction::GoToLine => Keystroke::parse("ctrl-g").ok(),
        CustomAction::ToggleGlobalSearch => Keystroke::parse("ctrl-3").ok(),
        CustomAction::ToggleConversationListView => Keystroke::parse("ctrl-1").ok(),
        CustomAction::NewTerminalTab
        | CustomAction::NewFile
        | CustomAction::ShowAboutWarp
        | CustomAction::SplitPaneLeft
        | CustomAction::SelectAllBlocks
        | CustomAction::SplitPaneUp
        | CustomAction::ConfigureKeybindings
        | CustomAction::RenameTab
        | CustomAction::CloseTab
        | CustomAction::CloseOtherTabs
        | CustomAction::CloseTabsRight
        | CustomAction::ShowAppearance
        | CustomAction::SaveCurrentConfig
        | CustomAction::TriggerWelcomeBlock
        | CustomAction::HistorySearch
        | CustomAction::DisableSyncTerminalInputs
        | CustomAction::ToggleSyncAllTerminalInputsInAllTabs
        | CustomAction::OpenAIFactCollection
        | CustomAction::NewAgentTab => None,
    }
}

/// Get the keystroke currently assigned to a binding. Returns `None` if the binding does not exist
/// or is unassigned.
pub fn keybinding_name_to_keystroke(binding_name: &str, ctx: &AppContext) -> Option<Keystroke> {
    ctx.get_binding_by_name(binding_name)
        .and_then(|binding| trigger_to_keystroke(binding.trigger))
}

/// Get keybinding display string from binding name. Unset keybindings will return None.
pub fn keybinding_name_to_display_string(binding_name: &str, ctx: &AppContext) -> Option<String> {
    keybinding_name_to_keystroke(binding_name, ctx).map(|keystroke| keystroke.displayed())
}

/// Get normalized keybinding string from binding name. Unset keybindings will return None.
pub fn keybinding_name_to_normalized_string(
    binding_name: &str,
    ctx: &AppContext,
) -> Option<String> {
    keybinding_name_to_keystroke(binding_name, ctx).map(|keystroke| keystroke.normalized())
}

/// Sets a custom keybinding for an editable binding using the given keystroke. Will
/// persist the keybinding to the user's config file and emit a KeybindingChangedEvent.
pub fn set_custom_keybinding(binding_name: &str, keystroke: &Keystroke, ctx: &mut AppContext) {
    ctx.set_custom_trigger(
        binding_name.into(),
        Trigger::Keystrokes(vec![keystroke.clone()]),
    );
    write_custom_keybinding(
        binding_name.into(),
        UserDefinedKeybinding::keystroke(keystroke.clone()),
    );
    KeybindingChangedNotifier::handle(ctx).update(ctx, |_, ctx| {
        ctx.emit(KeybindingChangedEvent::BindingChanged {
            binding_name: binding_name.into(),
            new_trigger: Some(keystroke.clone()),
        })
    });
}

/// Reset an editable binding back to its default trigger. Will persist this change to
/// the user's config file and emit a KeybindingChangedEvent with the default trigger.
/// Returns the default keystroke for the binding.
pub fn reset_keybinding_to_default(binding_name: &str, ctx: &mut AppContext) -> Option<Keystroke> {
    ctx.remove_custom_trigger(binding_name);
    remove_custom_keybinding(binding_name);

    let default_keystroke = ctx
        .editable_bindings()
        .find(|binding| binding.name == binding_name)
        .and_then(|binding| trigger_to_keystroke(binding.trigger));

    KeybindingChangedNotifier::handle(ctx).update(ctx, |_, ctx| {
        ctx.emit(KeybindingChangedEvent::BindingChanged {
            binding_name: binding_name.into(),
            new_trigger: default_keystroke.clone(),
        })
    });

    default_keystroke
}

#[derive(Clone, Debug)]
pub struct CommandBinding {
    pub name: String,
    pub description: BindingDescription,
    pub trigger: Option<Keystroke>,
    pub action: Option<Arc<dyn Action>>,
    pub group: Option<BindingGroup>,
    /// The ID of the binding.  If the [`CommandBinding`] was created from an
    /// [`EditableBindingLens`] or [`BindingLens`] this is the id of the lens. Otherwise a new ID
    /// is constructed.
    pub id: BindingId,
}

/// SearchScore is a helper struct for ranking the result of the search.
/// `keystroke_score` represents the proximity between search term keystroke with
/// the candidate keystroke. If the score is None, it means the candidate keystroke is not
/// a valid subset of the search term keystroke.
/// The fuzzy_search_score helps on the secondary ranking -- if two candidate keystrokes are
/// both not valid subset of the search term keystroke, then we rank these by the score
/// of the fuzzy_search.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SearchScore {
    keystroke_score: Option<usize>,
    fuzzy_search_score: i64,
}

impl Ord for SearchScore {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.keystroke_score.cmp(&other.keystroke_score) {
            Ordering::Equal => self.fuzzy_search_score.cmp(&other.fuzzy_search_score),
            ordering => ordering,
        }
    }
}

impl PartialOrd for SearchScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn convert_search_term_to_keystroke(search_term: &str) -> Option<Keystroke> {
    let mut search_keystroke: Keystroke = Default::default();
    let mut key_set = false;

    for element in search_term.split_whitespace() {
        match element.to_ascii_lowercase().as_str() {
            "command" | "cmd" => {
                if search_keystroke.cmd {
                    return None;
                }
                search_keystroke.cmd = true
            }
            "control" | "ctrl" => {
                if search_keystroke.ctrl {
                    return None;
                }
                search_keystroke.ctrl = true
            }
            "alt" | "option" => {
                if search_keystroke.alt {
                    return None;
                }
                search_keystroke.alt = true
            }
            "shift" => {
                if search_keystroke.shift {
                    return None;
                }
                search_keystroke.shift = true
            }
            "meta" => {
                if search_keystroke.meta {
                    return None;
                }
                search_keystroke.meta = true
            }
            key => {
                if key_set {
                    return None;
                }
                key_set = true;
                search_keystroke.key = key.to_string()
            }
        }
    }

    // Internally we uppercase the key when shift modifier is true.
    if search_keystroke.shift && search_keystroke.key.len() == 1 {
        search_keystroke.key = search_keystroke.key.to_ascii_uppercase();
    }

    Some(search_keystroke)
}

pub fn filter_bindings_including_keystroke<'a>(
    bindings_iter: impl Iterator<Item = &'a CommandBinding>,
    search_term: &str,
    description_for: DescriptionContext,
) -> impl Iterator<Item = (Option<Vec<usize>>, &'a CommandBinding)> {
    let search_keystroke = convert_search_term_to_keystroke(search_term);

    bindings_iter
        .filter_map(move |binding| {
            if search_term.is_empty() {
                Some((Default::default(), None, binding))
            } else {
                let keystroke_search_score = if let Some(search_keystroke) = &search_keystroke {
                    binding.trigger.as_ref().and_then(|candidate_keystroke| {
                        let score = keystroke_includes(search_keystroke, candidate_keystroke);
                        if score > 0 {
                            Some(score)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                let fuzzy_search_result = match_indices_case_insensitive(
                    binding.description.in_context(description_for),
                    search_term,
                );

                match (keystroke_search_score, fuzzy_search_result) {
                    // If keystroke matched, don't include fuzzy search highlights.
                    (Some(keystroke_score), Some(fuzzy_search_result)) => Some((
                        SearchScore {
                            keystroke_score: Some(keystroke_score),
                            fuzzy_search_score: fuzzy_search_result.score,
                        },
                        None,
                        binding,
                    )),
                    (None, Some(fuzzy_search_result)) => Some((
                        SearchScore {
                            fuzzy_search_score: fuzzy_search_result.score,
                            ..Default::default()
                        },
                        None,
                        binding,
                    )),
                    (Some(keystroke_score), None) => Some((
                        SearchScore {
                            keystroke_score: Some(keystroke_score),
                            ..Default::default()
                        },
                        None,
                        binding,
                    )),
                    _ => None,
                }
            }
        })
        .sorted_by(|(score1, _, _), (score2, _, _)| score2.cmp(score1))
        .map(|(_, indices, binding)| (indices, binding))
}

/// Check if the keystroke could be a possible candidate of the search keystroke and give a score
/// based on proximity.
/// The scores are generated as follows: for each field of the keystroke (alt, cmd, etc), the comparison
/// between the search and candidate keystroke could yield three possible results - a strict
/// match (candidate and search has the same value), a potential match (candidate has the value
/// set to true but search is missing the value), a mismatch (candidate has the value set to
/// false but search is set to true).
/// For a strict match, we increase the score by multiplying it by two. For a potential match,
/// we keep the original score by multiplying it by one. For a mismatch, we multiply by zero
/// to mark that the candidate keystroke is not a valid subset of the search keystroke.
fn keystroke_includes(search_keystroke: &Keystroke, candidate_keystroke: &Keystroke) -> usize {
    fn modifier_match(
        search_keystroke_condition: bool,
        candidate_keystroke_condition: bool,
    ) -> usize {
        match (search_keystroke_condition, candidate_keystroke_condition) {
            (false, false) | (true, true) => 2, // match gives a score of 2.
            (false, true) => 1, // keep the same score if the keystroke term is true but search_keystroke does not include the term.
            (true, false) => 0, // if the keystroke term is false but search_keystorke is true, return 0 score
        }
    }

    let key_match_score = if search_keystroke.key == candidate_keystroke.key {
        2
    } else {
        usize::from(search_keystroke.key.is_empty())
    };

    modifier_match(search_keystroke.alt, candidate_keystroke.alt)
        * modifier_match(search_keystroke.cmd, candidate_keystroke.cmd)
        * modifier_match(search_keystroke.ctrl, candidate_keystroke.ctrl)
        * modifier_match(search_keystroke.meta, candidate_keystroke.meta)
        * modifier_match(search_keystroke.shift, candidate_keystroke.shift)
        * key_match_score
}

impl CommandBinding {
    pub fn new(name: String, description: String, trigger: Option<Keystroke>) -> Self {
        CommandBinding {
            name,
            description: BindingDescription::new(description),
            trigger,
            action: None,
            group: None,
            id: BindingId::new(),
        }
    }

    /// Materializes a [`CommandBinding`] from a [`BindingLens`], resolving
    /// any dynamic description against `ctx` so downstream consumers that
    /// have no `&AppContext` (fuzzy/full-text search indices, accessibility
    /// labels, render paths that only see an `Appearance`) observe a plain
    /// string.
    ///
    /// This is intentionally the only way to build a `CommandBinding` from
    /// a lens; taking `&AppContext` by value here forces every cache-
    /// population site to thread context through, which in turn guarantees
    /// that a future dynamic description cannot silently go unresolved.
    ///
    /// Returns `None` when the source binding has no description.
    pub fn from_lens(lens: BindingLens<'_>, ctx: &AppContext) -> Option<Self> {
        lens.description.map(|desc| CommandBinding {
            description: materialize_description(desc, ctx),
            trigger: trigger_to_keystroke(lens.trigger),
            action: Some(lens.action.clone()),
            name: lens.name.to_string(),
            group: lens.group.and_then(BindingGroup::from_str),
            id: lens.id,
        })
    }

    /// Materializes a [`CommandBinding`] from an [`EditableBindingLens`].
    /// See [`Self::from_lens`] for why this takes `&AppContext`.
    pub fn from_editable_lens(lens: EditableBindingLens<'_>, ctx: &AppContext) -> Self {
        Self {
            description: materialize_description(lens.description, ctx),
            trigger: trigger_to_keystroke(lens.trigger),
            action: Some(lens.action.clone()),
            name: lens.name.into(),
            group: lens.group.and_then(BindingGroup::from_str),
            id: lens.id,
        }
    }

    pub fn placeholder(placeholder: String) -> Self {
        CommandBinding {
            name: Default::default(),
            description: placeholder.into(),
            trigger: None,
            action: None,
            group: None,
            id: BindingId::new(),
        }
    }
}

fn materialize_description(desc: &BindingDescription, ctx: &AppContext) -> BindingDescription {
    if desc.has_dynamic_override() {
        desc.materialized(ctx)
    } else {
        desc.clone()
    }
}

/// Possible groups a Binding can be part of. The string representation (produced in
/// [`BindingGroup::as_str`]) is used as the group identifier within
/// [`warpui::keymap::FixedBinding`] or [`EditableBinding`].
#[derive(Copy, Clone, Debug, Sequence)]
pub enum BindingGroup {
    Settings,
    Close,
    Navigation,
    Ai,
    Workflow,
    Folders,
    KeyboardShortcuts,
    Notifications,
    EnvVarCollection,
    Terminal,
}

impl BindingGroup {
    /// Returns a string representation of the [`BindingGroup`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Settings => "settings",
            Self::Ai => "ai",
            Self::Navigation => "navigation",
            Self::Workflow => "workflows",
            Self::Folders => "folders",
            Self::KeyboardShortcuts => "keyboard_shortcuts",
            Self::Close => "close",
            Self::Notifications => "notifications",
            Self::EnvVarCollection => "env_var_collections",
            Self::Terminal => "terminal",
        }
    }

    /// Creates a [`BindingGroup`] from a str. Returns `None` if there is no group that corresponds
    /// to the `str`.
    fn from_str(str: &'static str) -> Option<Self> {
        all::<Self>().find(|&item| item.as_str() == str)
    }
}

pub fn cmd_or_ctrl_shift(key: &str) -> String {
    format!("cmd-{key}")
}

/// Returns whether the given [`BindingLens`] is compliant with the PTY.
/// A binding is considered PTY compliant if it does not interfere with a control character that
/// needs to be sent to the PTY. A binding is considered to be a control character if the only
/// modifier set is `ctrl` and the key is one of `a-z@[\]^_?`.
pub fn is_binding_pty_compliant(binding: BindingLens) -> IsBindingValid {
    let trigger = binding.original_trigger.unwrap_or(binding.trigger);
    let Some(keystroke) = trigger_to_keystroke(trigger) else {
        return IsBindingValid::Yes;
    };

    let is_binding_in_allowlist = MAC_PTY_NON_COMPLIANT_ACTIONS.contains(binding.name)
        || PTY_NON_COMPLIANT_KEYSTROKES.contains(&keystroke);

    if CONTROL_CHARACTER_KEY_REGEX.is_match(keystroke.normalized().as_str())
        && !is_binding_in_allowlist
    {
        // The binding interferes with a control character so it is not valid.
        IsBindingValid::No
    } else {
        IsBindingValid::Yes
    }
}

pub fn is_binding_supported_on_mac(_binding: BindingLens) -> IsBindingValid {
    IsBindingValid::Yes
}

fn parse_keystroke(source: &str) -> Option<Keystroke> {
    Keystroke::parse(source).ok()
}

#[cfg(test)]
#[path = "bindings_tests.rs"]
mod tests;
