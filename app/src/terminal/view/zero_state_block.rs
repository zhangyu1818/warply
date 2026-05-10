use warp_core::ui::Icon;
use warpui::{
    elements::{
        ChildAnchor, Container, CrossAxisAlignment, Flex, MainAxisSize, OffsetPositioning,
        ParentAnchor, ParentElement, ParentOffsetBounds, Stack, Text,
    },
    fonts::{Properties, Weight},
    keymap::Keystroke,
    prelude::{vec2f, ConstrainedBox, Cursor, Empty, Hoverable, MouseStateHandle},
    scene::Border,
    AppContext, Element, Entity, ModelHandle, SingletonEntity, TypedActionView, View, ViewContext,
};

use crate::{
    ai::blocklist::agent_view::{AgentViewController, AgentViewControllerEvent},
    appearance::Appearance,
    settings::{AISettings, AISettingsChangedEvent, InputModeSettings},
    terminal::{
        self,
        event::BlockType,
        input::message_bar::{common::render_standard_message, Message, MessageItem},
        model_events::{ModelEvent, ModelEventDispatcher},
        settings::{TerminalSettings, TerminalSettingsChangedEvent},
        view::TerminalAction,
    },
    util::bindings::keybinding_name_to_keystroke,
    workspace::tab_settings::TabSettings,
    workspace::tab_settings::TabSettingsChangedEvent,
    workspace::view::TOGGLE_RIGHT_PANEL_BINDING_NAME,
    WorkspaceAction,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalViewZeroStateAction {
    Dismiss,
}

#[derive(Default)]
struct StateHandles {
    dismiss_button: MouseStateHandle,
    open_history_menu: MouseStateHandle,
    open_code_review: MouseStateHandle,
}

pub struct TerminalViewZeroStateBlock {
    state_handles: StateHandles,
    should_hide: bool,
}

impl TerminalViewZeroStateBlock {
    pub fn new(
        agent_view_controller: &ModelHandle<AgentViewController>,
        model_events_dispatcher: &ModelHandle<ModelEventDispatcher>,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        let controller_clone = agent_view_controller.clone();
        ctx.subscribe_to_model(
            model_events_dispatcher,
            move |me, model_events_dispatcher, event, ctx| {
                if let ModelEvent::BlockCompleted(block_completed) = event {
                    if matches!(block_completed.block_type, BlockType::User(..)) {
                        me.should_hide = true;
                        ctx.unsubscribe_to_model(&model_events_dispatcher);
                        ctx.unsubscribe_to_model(&controller_clone);
                        ctx.notify();
                    }
                }
            },
        );

        let model_events_clone = model_events_dispatcher.clone();
        ctx.subscribe_to_model(agent_view_controller, move |me, controller, event, ctx| {
            if let AgentViewControllerEvent::ExitedAgentView {
                original_exchange_count,
                final_exchange_count,
                ..
            } = event
            {
                if original_exchange_count != final_exchange_count {
                    me.should_hide = true;
                    ctx.unsubscribe_to_model(&model_events_clone);
                    ctx.unsubscribe_to_model(&controller);
                    ctx.notify()
                }
            }
        });

        ctx.subscribe_to_model(&AISettings::handle(ctx), |me, _, event, ctx| {
            if matches!(event, AISettingsChangedEvent::IsAnyAIEnabled { .. })
                && !TerminalSettings::as_ref(ctx).should_show_zero_state_block(ctx)
            {
                me.should_hide = true;
                ctx.notify();
            }
        });

        ctx.subscribe_to_model(&TerminalSettings::handle(ctx), |me, _, event, ctx| {
            if matches!(
                event,
                TerminalSettingsChangedEvent::ShowTerminalZeroStateBlock { .. }
            ) && !TerminalSettings::as_ref(ctx).should_show_zero_state_block(ctx)
            {
                me.should_hide = true;
                ctx.notify();
            }
        });

        ctx.subscribe_to_model(&TabSettings::handle(ctx), |_, _, event, ctx| {
            if matches!(event, TabSettingsChangedEvent::ShowCodeReviewButton { .. }) {
                ctx.notify();
            }
        });

        Self {
            should_hide: false,
            state_handles: Default::default(),
        }
    }
}

impl View for TerminalViewZeroStateBlock {
    fn ui_name() -> &'static str {
        "TerminalViewZeroStateBlock"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        if self.should_hide {
            return Empty::new().finish();
        }

        let appearance = Appearance::handle(app).as_ref(app);
        let theme = appearance.theme();

        let title_font_size = appearance.monospace_font_size() + 6.;
        let title = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                Container::new(
                    ConstrainedBox::new(
                        Icon::Warp
                            .to_warpui_icon(theme.main_text_color(theme.background()))
                            .finish(),
                    )
                    .with_height(title_font_size)
                    .with_width(title_font_size)
                    .finish(),
                )
                .with_margin_right(8.)
                .finish(),
            )
            .with_child(
                Text::new(
                    "New terminal session",
                    appearance.ui_font_family(),
                    title_font_size,
                )
                .with_color(theme.main_text_color(theme.background()).into_solid())
                .with_style(Properties::default().weight(Weight::Bold))
                .finish(),
            )
            .finish();

        let mut content = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_main_axis_size(MainAxisSize::Min)
            .with_child(
                Container::new(title)
                    .with_margin_bottom(styles::TITLE_MARGIN_BOTTOM)
                    .finish(),
            );

        let mut items = vec![render_standard_message(
            Message::new(vec![MessageItem::clickable(
                vec![
                    MessageItem::keystroke(Keystroke {
                        key: "up".to_owned(),
                        ..Default::default()
                    }),
                    MessageItem::text("cycle past commands and conversations"),
                ],
                |ctx| {
                    ctx.dispatch_typed_action(TerminalAction::OpenInlineHistoryMenu);
                },
                self.state_handles.open_history_menu.clone(),
            )]),
            app,
        )];

        if *TabSettings::as_ref(app).show_code_review_button {
            if let Some(keystroke) =
                keybinding_name_to_keystroke(TOGGLE_RIGHT_PANEL_BINDING_NAME, app)
            {
                items.push(render_standard_message(
                    Message::new(vec![MessageItem::clickable(
                        vec![
                            MessageItem::keystroke(keystroke),
                            MessageItem::text("open code review"),
                        ],
                        |ctx| {
                            ctx.dispatch_typed_action(WorkspaceAction::ToggleRightPanel);
                        },
                        self.state_handles.open_code_review.clone(),
                    )]),
                    app,
                ));
            }
        }

        if InputModeSettings::handle(app)
            .as_ref(app)
            .input_mode
            .is_pinned_to_top()
        {
            items.reverse();
        }

        let item_count = items.len();
        for (i, item) in items.into_iter().enumerate() {
            content.add_child(if i < item_count - 1 {
                Container::new(item).with_margin_bottom(8.).finish()
            } else {
                item
            });
        }

        let dismiss_button = Hoverable::new(self.state_handles.dismiss_button.clone(), |state| {
            let color = if state.is_hovered() {
                theme.sub_text_color(theme.background())
            } else {
                theme.disabled_text_color(theme.background())
            };
            Text::new(
                "Don't show again",
                appearance.ui_font_family(),
                appearance.monospace_font_size() - 4.,
            )
            .with_color(color.into_solid())
            .finish()
        })
        .on_click(|ctx, _, _| {
            ctx.dispatch_typed_action(TerminalViewZeroStateAction::Dismiss);
        })
        .with_cursor(Cursor::PointingHand)
        .finish();

        Stack::new()
            .with_child(
                Container::new(content.finish())
                    .with_horizontal_padding(*terminal::view::PADDING_LEFT)
                    .with_vertical_padding(styles::CONTAINER_VERTICAL_PADDING)
                    .with_border(
                        Border::new(1.)
                            .with_sides(true, false, true, false)
                            .with_border_fill(theme.outline()),
                    )
                    .finish(),
            )
            .with_positioned_child(
                dismiss_button,
                OffsetPositioning::offset_from_parent(
                    vec2f(-8., -8.),
                    ParentOffsetBounds::ParentBySize,
                    ParentAnchor::BottomRight,
                    ChildAnchor::BottomRight,
                ),
            )
            .finish()
    }
}

impl TypedActionView for TerminalViewZeroStateBlock {
    type Action = TerminalViewZeroStateAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            TerminalViewZeroStateAction::Dismiss => {
                self.should_hide = true;
                TerminalSettings::handle(ctx).update(ctx, |_settings, _ctx| {});
                ctx.notify();
            }
        }
    }
}

impl Entity for TerminalViewZeroStateBlock {
    type Event = ();
}

mod styles {
    pub const CONTAINER_VERTICAL_PADDING: f32 = 16.;

    pub const TITLE_MARGIN_BOTTOM: f32 = 8.;
}
