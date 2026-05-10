mod model;

pub use model::*;
use pathfinder_color::ColorU;

use std::borrow::Cow;

use warp_core::{features::FeatureFlag, ui::appearance::Appearance};
use warpui::{
    elements::{Border, Container, CrossAxisAlignment, Expanded, Flex, ParentElement, Text},
    keymap::Keystroke,
    ui_components::components::{Coords, UiComponent, UiComponentStyles},
    AppContext, Element, SingletonEntity,
};

use crate::{
    ai::blocklist::agent_view::ENTER_AGENT_VIEW_NEW_CONVERSATION_KEYSTROKE,
    search::slash_command_menu::static_commands::commands,
    terminal,
    ui_components::blended_colors,
    util::bindings::keybinding_name_to_keystroke,
    workspace::view::{
        TOGGLE_CONVERSATION_LIST_VIEW_BINDING_NAME, TOGGLE_RIGHT_PANEL_BINDING_NAME,
    },
};

#[derive(Copy, Clone, Debug, Default)]
pub struct AgentShortcutsViewContext {
    pub has_submitted_first_prompt: bool,
}

#[derive(Default)]
pub struct ShortcutProps {
    pub keystroke: Keystroke,
    pub text: Cow<'static, str>,
    pub text_color: Option<ColorU>,
}

pub fn render_shortcut(props: ShortcutProps, app: &AppContext) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();
    let font_size = styles::font_size(appearance);
    let font_color = props.text_color.unwrap_or_else(|| {
        theme
            .sub_text_color(blended_colors::neutral_1(theme).into())
            .into()
    });
    Flex::row()
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(
            Container::new(render_keystroke(&props.keystroke, app))
                .with_margin_right(4.)
                .finish(),
        )
        .with_child(
            Expanded::new(
                1.,
                Text::new(props.text, appearance.ui_font_family(), font_size)
                    .with_color(font_color)
                    .finish(),
            )
            .finish(),
        )
        .finish()
}

pub fn render_keystroke(keystroke: &Keystroke, app: &AppContext) -> Box<dyn Element> {
    render_keystroke_with_color_overrides(keystroke, None, None, app)
}

pub fn render_keystroke_with_color_overrides(
    keystroke: &Keystroke,
    color: Option<ColorU>,
    background_color: Option<ColorU>,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let theme = appearance.theme();
    let font_size = styles::font_size(appearance);
    appearance
        .ui_builder()
        .keyboard_shortcut(keystroke)
        .lowercase_modifier()
        .with_space_between_keys(2.)
        .with_style(UiComponentStyles {
            margin: Some(Coords::default()),
            padding: Some(Coords::default()),
            border_width: Some(1.),
            background: Some(
                background_color
                    .unwrap_or_else(|| blended_colors::neutral_3(theme))
                    .into(),
            ),
            font_color: Some(color.unwrap_or_else(|| theme.foreground().into_solid())),
            font_family_id: Some(appearance.ui_font_family()),
            font_size: Some(font_size),
            width: Some(styles::keystroke_size(appearance)),
            height: Some(styles::keystroke_size(appearance)),
            ..Default::default()
        })
        .with_line_height_ratio(1.0)
        .build()
        .finish()
}

pub fn render_agent_shortcuts_view(
    context: AgentShortcutsViewContext,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let shortcuts = agent_shortcut_props(context, agent_shortcut_keybindings(app))
        .into_iter()
        .map(|props| render_shortcut(props, app))
        .collect::<Vec<_>>();

    Container::new(
        Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_spacing(8.)
            .with_children(shortcuts)
            .finish(),
    )
    .with_vertical_padding(16.)
    .with_padding_left(*terminal::view::PADDING_LEFT)
    .with_border(
        Border::new(1.)
            .with_sides(true, false, true, false)
            .with_border_color(blended_colors::neutral_2(appearance.theme())),
    )
    .finish()
}

#[derive(Default)]
struct AgentShortcutKeybindings {
    code_review: Option<Keystroke>,
    conversation_list: Option<Keystroke>,
    conversation_search: Option<Keystroke>,
}

fn agent_shortcut_keybindings(app: &AppContext) -> AgentShortcutKeybindings {
    AgentShortcutKeybindings {
        code_review: keybinding_name_to_keystroke(TOGGLE_RIGHT_PANEL_BINDING_NAME, app),
        conversation_list: if FeatureFlag::AgentViewConversationListView.is_enabled() {
            keybinding_name_to_keystroke(TOGGLE_CONVERSATION_LIST_VIEW_BINDING_NAME, app)
        } else {
            None
        },
        conversation_search: keybinding_name_to_keystroke(commands::CONVERSATIONS.name, app),
    }
}

fn agent_shortcut_props(
    context: AgentShortcutsViewContext,
    keybindings: AgentShortcutKeybindings,
) -> Vec<ShortcutProps> {
    let _has_submitted_first_prompt = context.has_submitted_first_prompt;
    let mut shortcuts = vec![
        ShortcutProps {
            keystroke: Keystroke {
                key: "!".to_owned(),
                ..Default::default()
            },
            text: "input shell command".into(),
            ..Default::default()
        },
        ShortcutProps {
            keystroke: Keystroke {
                key: "/".to_owned(),
                ..Default::default()
            },
            text: "for slash commands".into(),
            ..Default::default()
        },
        ShortcutProps {
            keystroke: Keystroke {
                key: "@".to_owned(),
                ..Default::default()
            },
            text: "for file paths and attaching other context".into(),
            ..Default::default()
        },
    ];

    if let Some(keystroke) = keybindings.code_review {
        shortcuts.push(ShortcutProps {
            keystroke,
            text: "open code review".into(),
            ..Default::default()
        });
    }

    if let Some(keystroke) = keybindings.conversation_list {
        shortcuts.push(ShortcutProps {
            keystroke,
            text: "toggle conversation list".into(),
            ..Default::default()
        });
    }

    if let Some(keystroke) = keybindings.conversation_search {
        shortcuts.push(ShortcutProps {
            keystroke,
            text: "search and continue conversations".into(),
            ..Default::default()
        });
    }

    shortcuts.extend([
        ShortcutProps {
            keystroke: ENTER_AGENT_VIEW_NEW_CONVERSATION_KEYSTROKE.clone(),
            text: "start a new conversation".into(),
            ..Default::default()
        },
        ShortcutProps {
            keystroke: Keystroke {
                key: "c".to_owned(),
                ctrl: true,
                ..Default::default()
            },
            text: "pause agent".into(),
            ..Default::default()
        },
        ShortcutProps {
            keystroke: Keystroke {
                key: "escape".to_owned(),
                ..Default::default()
            },
            text: "go back to terminal".into(),
            ..Default::default()
        },
    ]);

    shortcuts
}

pub mod styles {
    use warp_core::ui::appearance::Appearance;

    pub fn keystroke_size(appearance: &Appearance) -> f32 {
        font_size(appearance) + 2.
    }

    pub fn font_size(appearance: &Appearance) -> f32 {
        appearance.monospace_font_size() - 2.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_keystroke(key: &str) -> Keystroke {
        Keystroke {
            key: key.to_owned(),
            ..Default::default()
        }
    }

    #[test]
    fn acp_shortcuts_keep_generic_items_without_legacy_agent_items() {
        let shortcuts = agent_shortcut_props(
            AgentShortcutsViewContext {
                has_submitted_first_prompt: false,
            },
            AgentShortcutKeybindings {
                code_review: Some(test_keystroke("r")),
                conversation_list: Some(test_keystroke("l")),
                conversation_search: Some(test_keystroke("y")),
            },
        );
        let labels = shortcuts
            .iter()
            .map(|shortcut| shortcut.text.as_ref())
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec![
                "input shell command",
                "for slash commands",
                "for file paths and attaching other context",
                "open code review",
                "toggle conversation list",
                "search and continue conversations",
                "start a new conversation",
                "pause agent",
                "go back to terminal",
            ]
        );
        assert!(!labels.contains(&"toggle auto-accept"));
        assert_eq!(
            shortcuts
                .iter()
                .find(|shortcut| shortcut.text == "search and continue conversations")
                .map(|shortcut| shortcut.keystroke.key.as_str()),
            Some("y")
        );
    }
}
