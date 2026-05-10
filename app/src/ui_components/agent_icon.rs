use warp_cli::agent::Harness;
use warpui::AppContext;
use warpui::SingletonEntity;

use crate::ai::agent::conversation::ConversationStatus;
use crate::ai::agent_conversations_model::AgentConversationEntry;
use crate::terminal::cli_agent_sessions::listener::agent_supports_rich_status;
use crate::terminal::cli_agent_sessions::CLIAgentSessionsModel;
use crate::terminal::view::TerminalView;
use crate::terminal::CLIAgent;
use crate::ui_components::icon_with_status::IconWithStatusVariant;

/// Returns the agent-icon variant for a live [`TerminalView`], or `None` when the terminal is
/// not an agent surface (plain terminal / shell / empty conversation).
///
/// Resolution order:
/// 1. A [`CLIAgentSessionsModel`] session with a known agent wins. Plugin-backed sessions
///    surface rich status; command-detected sessions don't.
/// 2. A selected local conversation falls through to the
///    no-task waterfall.
/// 3. Everything else returns `None` so the caller renders a plain-terminal indicator.
pub(crate) fn terminal_view_agent_icon_variant(
    terminal_view: &TerminalView,
    app: &AppContext,
) -> Option<IconWithStatusVariant> {
    let cli_agent_session = CLIAgentSessionsModel::as_ref(app).session(terminal_view.id());

    let inputs = TerminalIconInputs {
        cli_session: cli_agent_session.map(|session| CLISessionInputs {
            agent: session.agent,
            has_listener: session.listener.is_some(),
            status: session.status.to_conversation_status(),
            supports_rich_status: agent_supports_rich_status(&session.agent),
        }),
        selected_conversation_status: terminal_view.selected_conversation_status_for_display(app),
        has_selected_conversation: terminal_view
            .selected_conversation_display_title(app)
            .is_some(),
    };
    agent_icon_variant_from_terminal_inputs(&inputs)
}

pub(crate) fn agent_conversation_entry_icon_variant(
    entry: &AgentConversationEntry,
) -> Option<IconWithStatusVariant> {
    let status = entry.display.status.to_conversation_status();
    agent_icon_variant_for_run(entry.display.harness?, status)
}

struct TerminalIconInputs {
    cli_session: Option<CLISessionInputs>,
    selected_conversation_status: Option<ConversationStatus>,
    has_selected_conversation: bool,
}

struct CLISessionInputs {
    agent: CLIAgent,
    has_listener: bool,
    status: ConversationStatus,
    supports_rich_status: bool,
}

fn agent_icon_variant_from_terminal_inputs(
    inputs: &TerminalIconInputs,
) -> Option<IconWithStatusVariant> {
    if let Some(session) = inputs
        .cli_session
        .as_ref()
        .filter(|s| !matches!(s.agent, CLIAgent::Unknown))
    {
        let status =
            (session.has_listener && session.supports_rich_status).then(|| session.status.clone());
        return Some(IconWithStatusVariant::CLIAgent {
            agent: session.agent,
            status,
        });
    }

    if inputs.has_selected_conversation {
        return Some(IconWithStatusVariant::Agent {
            status: inputs.selected_conversation_status.clone(),
        });
    }

    None
}

fn agent_icon_variant_for_run(
    harness: Harness,
    status: ConversationStatus,
) -> Option<IconWithStatusVariant> {
    CLIAgent::from_harness(harness).map(|agent| IconWithStatusVariant::CLIAgent {
        agent,
        status: Some(status),
    })
}

#[cfg(test)]
#[path = "agent_icon_tests.rs"]
mod tests;
