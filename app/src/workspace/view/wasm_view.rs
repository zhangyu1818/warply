//! WASM-only view functions for the Workspace.

use warpui::elements::{ChildView, Element};
use warpui::{AppContext, SingletonEntity, ViewContext, ViewHandle};

use super::PanelPosition;

use crate::ai::conversation_details_panel::{
    ConversationDetailsData, ConversationDetailsPanel, ConversationDetailsPanelEvent,
};
use crate::terminal::TerminalView;
use crate::ui_components::icons;
use crate::view_components::action_button::{ActionButton, ButtonSize, NakedTheme};
use crate::workspace::action::WorkspaceAction;
use crate::workspace::view::Workspace;
use crate::BlocklistAIHistoryModel;

const TRANSCRIPT_PANEL_WIDTH: f32 = 280.0;

impl Workspace {
    pub(super) fn build_transcript_info_button(
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<ActionButton> {
        ctx.add_typed_action_view(|_ctx| {
            ActionButton::new("", NakedTheme)
                .with_icon(icons::Icon::Info)
                .with_size(ButtonSize::Small)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(
                        WorkspaceAction::ToggleConversationTranscriptDetailsPanel,
                    );
                })
        })
    }

    pub(super) fn build_transcript_details_panel(
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<ConversationDetailsPanel> {
        let panel = ctx.add_typed_action_view(|ctx| {
            ConversationDetailsPanel::new(false, TRANSCRIPT_PANEL_WIDTH, ctx)
        });

        ctx.subscribe_to_view(&panel, |me, _, event, ctx| match event {
            ConversationDetailsPanelEvent::Close => {
                me.current_workspace_state.is_transcript_details_panel_open = false;
                me.transcript_info_button.update(ctx, |button, ctx| {
                    button.set_active(false, ctx);
                });
                ctx.notify();
            }
        });

        panel
    }

    /// Check if we should show the conversation details panel, given the focused terminal view.
    /// Returns true for conversation transcript viewers or active conversations.
    pub(super) fn should_show_conversation_details_panel(
        focused_terminal_view: &ViewHandle<TerminalView>,
        ctx: &AppContext,
    ) -> bool {
        let terminal_view_ref = focused_terminal_view.as_ref(ctx);
        let model = terminal_view_ref.model.lock();

        // Always show for conversation transcript viewers
        if model.is_conversation_transcript_viewer() {
            return true;
        }

        false
    }

    /// Renders the transcript details panel for WASM conversation transcript and shared session viewing.
    pub(super) fn render_transcript_details_panel(
        &self,
        app: &AppContext,
    ) -> Option<Box<dyn Element>> {
        let terminal_view = self
            .active_tab_pane_group()
            .as_ref(app)
            .focused_session_view(app)?;

        if !Self::should_show_conversation_details_panel(&terminal_view, app) {
            return None;
        }

        Some(self.render_panel(
            app,
            ChildView::new(&self.transcript_details_panel).finish(),
            &PanelPosition::Right,
        ))
    }

    pub(super) fn update_transcript_details_panel_data(&mut self, ctx: &mut ViewContext<Self>) {
        // Get the focused terminal view
        let Some(terminal_view) = self
            .active_tab_pane_group()
            .as_ref(ctx)
            .focused_session_view(ctx)
        else {
            return;
        };

        if !Self::should_show_conversation_details_panel(&terminal_view, ctx) {
            return;
        }

        let terminal_view_id = terminal_view.id();
        self.transcript_details_panel.update(ctx, |panel, ctx| {
            let history_model = BlocklistAIHistoryModel::handle(ctx).as_ref(ctx);
            if let Some(conversation) = history_model.active_conversation(terminal_view_id) {
                let details = ConversationDetailsData::from_conversation(conversation, ctx);
                panel.set_conversation_details(details, ctx);
            }
            ctx.notify();
        });
    }
}
