use warp_cli::agent::Harness;

use super::{
    agent_icon_variant_for_run, agent_icon_variant_from_terminal_inputs, CLISessionInputs,
    TerminalIconInputs,
};
use crate::ai::agent::conversation::ConversationStatus;
use crate::terminal::CLIAgent;
use crate::ui_components::icon_with_status::IconWithStatusVariant;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentIconFields {
    is_cli: bool,
    cli_agent: Option<CLIAgent>,
    status: Option<ConversationStatus>,
}

impl AgentIconFields {
    fn from_variant(variant: &IconWithStatusVariant) -> Option<Self> {
        match variant {
            IconWithStatusVariant::Agent { status } => Some(Self {
                is_cli: false,
                cli_agent: None,
                status: status.clone(),
            }),
            IconWithStatusVariant::CLIAgent { agent, status } => Some(Self {
                is_cli: true,
                cli_agent: Some(*agent),
                status: status.clone(),
            }),
            IconWithStatusVariant::Neutral { .. }
            | IconWithStatusVariant::NeutralElement { .. } => None,
        }
    }
}

#[test]
fn plain_terminal_has_no_agent_icon() {
    let inputs = TerminalIconInputs {
        cli_session: None,
        selected_conversation_status: None,
        has_selected_conversation: false,
    };

    assert!(agent_icon_variant_from_terminal_inputs(&inputs).is_none());
}

#[test]
fn selected_conversation_renders_agent_status() {
    let inputs = TerminalIconInputs {
        cli_session: None,
        selected_conversation_status: Some(ConversationStatus::InProgress),
        has_selected_conversation: true,
    };

    let variant = agent_icon_variant_from_terminal_inputs(&inputs).unwrap();
    assert_eq!(
        AgentIconFields::from_variant(&variant).unwrap(),
        AgentIconFields {
            is_cli: false,
            cli_agent: None,
            status: Some(ConversationStatus::InProgress),
        }
    );
}

#[test]
fn plugin_cli_session_renders_cli_status() {
    let inputs = TerminalIconInputs {
        cli_session: Some(CLISessionInputs {
            agent: CLIAgent::Claude,
            has_listener: true,
            status: ConversationStatus::Blocked {
                blocked_action: String::new(),
            },
            supports_rich_status: true,
        }),
        selected_conversation_status: None,
        has_selected_conversation: false,
    };

    let variant = agent_icon_variant_from_terminal_inputs(&inputs).unwrap();
    assert_eq!(
        AgentIconFields::from_variant(&variant).unwrap(),
        AgentIconFields {
            is_cli: true,
            cli_agent: Some(CLIAgent::Claude),
            status: Some(ConversationStatus::Blocked {
                blocked_action: String::new()
            }),
        }
    );
}

#[test]
fn command_detected_cli_session_has_no_status() {
    let inputs = TerminalIconInputs {
        cli_session: Some(CLISessionInputs {
            agent: CLIAgent::Claude,
            has_listener: false,
            status: ConversationStatus::InProgress,
            supports_rich_status: false,
        }),
        selected_conversation_status: None,
        has_selected_conversation: false,
    };

    let variant = agent_icon_variant_from_terminal_inputs(&inputs).unwrap();
    assert_eq!(
        AgentIconFields::from_variant(&variant).unwrap(),
        AgentIconFields {
            is_cli: true,
            cli_agent: Some(CLIAgent::Claude),
            status: None,
        }
    );
}

#[test]
fn run_harness_mapping_is_exact() {
    let codex = agent_icon_variant_for_run(Harness::Codex, ConversationStatus::Success).unwrap();
    assert_eq!(
        AgentIconFields::from_variant(&codex).unwrap(),
        AgentIconFields {
            is_cli: true,
            cli_agent: Some(CLIAgent::Codex),
            status: Some(ConversationStatus::Success),
        }
    );
}
