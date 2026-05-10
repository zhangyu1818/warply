//! Shared styles for notebooks.

use warpui::{
    elements::{Hoverable, MouseStateHandle},
    platform::Cursor,
    ui_components::components::UiComponent as _,
    Element,
};

use crate::{
    appearance::Appearance,
    ui_components::{buttons::icon_button, icons::Icon},
};

/// Builds an action button for a block's footer (such as the button to run a command or embedded
/// workflow).
pub(super) fn block_footer_action_button(
    appearance: &Appearance,
    icon: Icon,
    mouse_state_handle: MouseStateHandle,
    tooltip: impl Into<String> + 'static,
    keybinding: Option<String>,
) -> Hoverable {
    let tooltip_builder = appearance.ui_builder().clone();
    icon_button(appearance, icon, false, mouse_state_handle)
        .with_tooltip(move || match keybinding {
            Some(keybinding) => tooltip_builder
                .tool_tip_with_sublabel(tooltip.into(), keybinding)
                .build()
                .finish(),
            None => tooltip_builder.tool_tip(tooltip.into()).build().finish(),
        })
        .build()
        // Revert to the default cursor instead of the editor I-beam
        .with_cursor(Cursor::Arrow)
}
