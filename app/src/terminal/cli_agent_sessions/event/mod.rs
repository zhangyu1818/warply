mod v1;

use serde::Deserialize;

use crate::terminal::CLIAgent;

#[cfg_attr(not(feature = "local_tty"), allow(dead_code))]
type EventParser = fn(&str) -> Option<CLIAgentEvent>;

/// Sentinel title that identifies structured CLI agent events sent via OSC 777.
/// The `"agent"` field in the JSON body distinguishes which agent sent it.
pub const CLI_AGENT_NOTIFICATION_SENTINEL: &str = "warp://cli-agent";

/// The event type encoded in the `"event"` field of the JSON body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CLIAgentEventType {
    SessionStart,
    PromptSubmit,
    ToolComplete,
    Stop,
    PermissionRequest,
    PermissionReplied,
    QuestionAsked,
    NeedsInput,
    IdlePrompt,
    Unknown(String),
}

/// Event-specific fields that vary by event type.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CLIAgentEventPayload {
    pub query: Option<String>,
    pub response: Option<String>,
    pub transcript_path: Option<String>,
    pub summary: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input_preview: Option<String>,
    /// Background tasks and scheduled wakeups still pending when a `Stop` fired. Claude Code
    /// wakes the agent again when any of them completes or fires, so a stop with pending work is
    /// a pause, not the end of the session. `None` when the plugin predates this field.
    pub pending_background_work_count: Option<u32>,
}

impl CLIAgentEventPayload {
    /// Whether the agent reported work that will wake it again without user input.
    pub fn has_pending_background_work(&self) -> bool {
        self.pending_background_work_count
            .is_some_and(|count| count > 0)
    }
}

/// A parsed event from a CLI agent plugin.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CLIAgentEvent {
    pub v: u32,
    pub agent: CLIAgent,
    pub event: CLIAgentEventType,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub project: Option<String>,
    pub payload: CLIAgentEventPayload,
}

#[cfg_attr(not(feature = "local_tty"), allow(dead_code))]
const VERSIONED_PARSERS: &[EventParser] = &[v1::parse];

/// Attempts to parse an OSC 777 `PluggableNotification` into a typed `CLIAgentEvent`.
/// Dispatches to the correct version-specific parser based on the `"v"` field. Returns `None`
/// if the title doesn't match the sentinel, the body isn't valid JSON, or the version is unsupported.
pub fn parse_event(title: Option<&str>, body: &str) -> Option<CLIAgentEvent> {
    if title? != CLI_AGENT_NOTIFICATION_SENTINEL {
        return None;
    }

    let version_probe: VersionProbe = serde_json::from_str(body).ok()?;
    let version = version_probe.v.unwrap_or(1);

    let index = (version as usize).checked_sub(1)?;
    match VERSIONED_PARSERS.get(index) {
        Some(parser) => parser(body),
        None => {
            log::error!(
                "Received CLI agent event with unsupported schema version \
                 {version}. The CLI agent plugin or Warp may need to be updated."
            );
            None
        }
    }
}

#[derive(Deserialize)]
struct VersionProbe {
    v: Option<u32>,
}
