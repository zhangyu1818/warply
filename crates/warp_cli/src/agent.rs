use std::fmt;

use clap::ValueEnum;

/// Output format for agent results.
#[derive(Debug, Copy, Clone, ValueEnum, Eq, PartialEq, Default)]
pub enum OutputFormat {
    /// Output as JSON.
    #[value(name = "json")]
    Json,
    /// Output as newline-delimited JSON.
    #[value(name = "ndjson")]
    Ndjson,
    /// Output as human-readable text.
    #[default]
    #[value(name = "pretty")]
    Pretty,
    /// Output as plain text.
    #[value(name = "text")]
    Text,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.to_possible_value().expect("no values are skipped");
        f.write_str(value.get_name())
    }
}

/// The execution harness for an agent run.
#[derive(
    Debug, Copy, Clone, ValueEnum, Eq, PartialEq, Default, serde::Serialize, serde::Deserialize,
)]
pub enum Harness {
    /// Delegate to the `claude` CLI.
    #[value(name = "claude", alias = "claude-code")]
    Claude,
    /// Delegate to the `opencode` CLI.
    #[value(name = "opencode", alias = "open-code")]
    OpenCode,
    /// Delegate to the `gemini` CLI.
    #[value(name = "gemini")]
    Gemini,
    /// Delegate to the `codex` CLI.
    #[default]
    #[value(name = "codex")]
    Codex,
}

impl Harness {
    pub fn parse_harness(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
        <Self as ValueEnum>::from_str(&normalized, true).ok()
    }

    pub fn parse_local_child_harness(value: &str) -> Option<Self> {
        match Self::parse_harness(value) {
            Some(harness @ (Self::Claude | Self::OpenCode | Self::Codex)) => Some(harness),
            Some(Self::Gemini) | None => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Claude => "Claude Code",
            Self::OpenCode => "OpenCode",
            Self::Gemini => "Gemini CLI",
            Self::Codex => "Codex",
        }
    }

    pub fn from_config_name(name: &str) -> Option<Self> {
        match name {
            "claude" => Some(Harness::Claude),
            "opencode" => Some(Harness::OpenCode),
            "gemini" => Some(Harness::Gemini),
            "codex" => Some(Harness::Codex),
            _ => None,
        }
    }

    pub fn config_name(self) -> &'static str {
        match self {
            Harness::Claude => "claude",
            Harness::OpenCode => "opencode",
            Harness::Gemini => "gemini",
            Harness::Codex => "codex",
        }
    }
}

impl fmt::Display for Harness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.config_name())
    }
}

#[cfg(test)]
#[path = "agent_tests.rs"]
mod tests;
