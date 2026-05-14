//! Module containing helper functions for opening links within the terminal.

use warpui::event::ModifiersState;

/// Returns a string denoting the keybinding to directly open a link.
pub fn directly_open_link_keybinding_string() -> &'static str {
    "Cmd +"
}

/// Returns true if a link should directly be opened (instead of showing a tooltip) given the
/// current [`ModifiersState`].
///
pub fn should_directly_open_link(modifiers: &ModifiersState) -> bool {
    modifiers.cmd
}
