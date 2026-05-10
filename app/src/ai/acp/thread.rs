use crate::ai::agent::util::parse_markdown_into_text_and_code_sections;
use crate::ai::agent::{AIAgentTextSection, AgentOutputImage, AgentOutputImageLayout};
use agent_client_protocol::schema::{
    AvailableCommand, ConfigOptionUpdate, ContentBlock, CurrentModeUpdate,
    EmbeddedResourceResource, Meta, Plan, SessionInfoUpdate, ToolCall, ToolCallContent,
    ToolCallLocation, ToolCallStatus, ToolCallUpdate, ToolKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const ACP_RAW_IMAGE_SOURCE_PREFIX: &str = "acp-raw-image:";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AcpToolCall {
    pub id: String,
    pub title: String,
    pub kind: ToolKind,
    pub status: ToolCallStatus,
    pub content: Vec<ToolCallContent>,
    pub locations: Vec<ToolCallLocation>,
    pub raw_input: Option<serde_json::Value>,
    pub raw_output: Option<serde_json::Value>,
    pub meta: Option<Meta>,
    pub terminal_traces: HashMap<String, AcpTerminalTrace>,
}

impl AcpToolCall {
    pub fn from_acp(call: ToolCall) -> Self {
        Self {
            id: call.tool_call_id.0.to_string(),
            title: call.title,
            kind: call.kind,
            status: call.status,
            content: call.content,
            locations: call.locations,
            raw_input: call.raw_input,
            raw_output: call.raw_output,
            meta: call.meta,
            terminal_traces: HashMap::new(),
        }
    }

    pub fn apply_update(&mut self, update: ToolCallUpdate) {
        if let Some(title) = update.fields.title {
            self.title = title;
        }
        if let Some(kind) = update.fields.kind {
            self.kind = kind;
        }
        if let Some(status) = update.fields.status {
            self.status = status;
        }
        if let Some(content) = update.fields.content {
            self.content = content;
        }
        if let Some(locations) = update.fields.locations {
            self.locations = locations;
        }
        if let Some(raw_input) = update.fields.raw_input {
            self.raw_input = Some(raw_input);
        }
        if let Some(raw_output) = update.fields.raw_output {
            self.raw_output = Some(raw_output);
        }
        if let Some(meta) = update.meta {
            self.meta = Some(meta);
        }
    }

    pub fn references_terminal(&self, terminal_id: &str) -> bool {
        self.content.iter().any(|content| {
            matches!(
                content,
                ToolCallContent::Terminal(terminal)
                    if terminal.terminal_id.0.as_ref() == terminal_id
            )
        })
    }

    pub fn set_terminal_trace(&mut self, terminal_id: String, trace: AcpTerminalTrace) {
        if self.references_terminal(&terminal_id) {
            self.terminal_traces.insert(terminal_id, trace);
        }
    }
}

impl Eq for AcpToolCall {}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcpTerminalTrace {
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub output: String,
    pub exit_code: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcpRawImage {
    pub asset_id: String,
    pub source: String,
    pub data: String,
    pub mime_type: String,
}

pub fn acp_raw_images(tool_call: &AcpToolCall) -> Vec<AcpRawImage> {
    tool_call
        .content
        .iter()
        .enumerate()
        .filter_map(|(index, content)| match content {
            ToolCallContent::Content(content) => match &content.content {
                agent_client_protocol::schema::ContentBlock::Image(image) => {
                    let asset_id = acp_raw_image_asset_id(&tool_call.id, index, &image.mime_type);
                    Some(AcpRawImage {
                        source: acp_raw_image_source(&asset_id),
                        asset_id,
                        data: image.data.clone(),
                        mime_type: image.mime_type.clone(),
                    })
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

pub fn acp_raw_image_source(asset_id: &str) -> String {
    format!("{ACP_RAW_IMAGE_SOURCE_PREFIX}{asset_id}")
}

pub fn acp_raw_image_id_from_source(source: &str) -> Option<&str> {
    source.strip_prefix(ACP_RAW_IMAGE_SOURCE_PREFIX)
}

pub fn acp_tool_call_content_sections(tool_call: &AcpToolCall) -> Vec<AIAgentTextSection> {
    let mut sections = Vec::new();

    for (index, content) in tool_call.content.iter().enumerate() {
        match content {
            ToolCallContent::Content(content) => {
                sections.extend(acp_content_block_sections(
                    tool_call,
                    index,
                    &content.content,
                ));
            }
            ToolCallContent::Terminal(terminal) => {
                if let Some(trace) = tool_call
                    .terminal_traces
                    .get(terminal.terminal_id.0.as_ref())
                {
                    sections.extend(parse_markdown_into_text_and_code_sections(
                        &format_acp_terminal_trace(trace),
                    ));
                }
            }
            ToolCallContent::Diff(_) => {}
            _ => unreachable!("unhandled ACP ToolCallContent"),
        }
    }

    sections
}

pub fn format_acp_terminal_trace(trace: &AcpTerminalTrace) -> String {
    let mut parts = Vec::new();
    if let Some(command) = trace.command.as_ref().filter(|command| !command.is_empty()) {
        parts.push(format!("$ {command}"));
    }
    if let Some(cwd) = trace.cwd.as_ref().filter(|cwd| !cwd.is_empty()) {
        parts.push(format!("cwd: {cwd}"));
    }
    if !trace.output.is_empty() {
        parts.push(trace.output.clone());
    }
    if let Some(exit_code) = trace.exit_code {
        parts.push(format!("exit: {exit_code}"));
    }

    parts.join("\n")
}

fn acp_raw_image_asset_id(tool_call_id: &str, index: usize, mime_type: &str) -> String {
    let extension = match mime_type {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "image/bmp" => "bmp",
        "image/tiff" => "tiff",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        _ => "img",
    };
    format!("acp-{tool_call_id}-{index}.{extension}")
}

fn acp_content_block_sections(
    tool_call: &AcpToolCall,
    index: usize,
    content: &ContentBlock,
) -> Vec<AIAgentTextSection> {
    match content {
        ContentBlock::Text(text) => parse_markdown_into_text_and_code_sections(&text.text),
        ContentBlock::Image(image) => {
            let asset_id = acp_raw_image_asset_id(&tool_call.id, index, &image.mime_type);
            let source = acp_raw_image_source(&asset_id);
            let alt_text = image.uri.clone().unwrap_or_else(|| asset_id.clone());
            let markdown_source =
                warp_editor::content::text::format_image_markdown(&alt_text, &source, None);
            vec![AIAgentTextSection::Image {
                image: AgentOutputImage {
                    alt_text,
                    source,
                    title: None,
                    markdown_source,
                    layout: AgentOutputImageLayout::Block,
                },
            }]
        }
        ContentBlock::ResourceLink(resource) => {
            let label = resource
                .title
                .as_deref()
                .filter(|title| !title.is_empty())
                .unwrap_or(&resource.name);
            let mut text = format!("[{}]({})", escape_markdown_link_label(label), resource.uri);
            let mut details = Vec::new();
            if let Some(description) = resource
                .description
                .as_deref()
                .filter(|detail| !detail.is_empty())
            {
                details.push(description.to_string());
            }
            if let Some(mime_type) = resource
                .mime_type
                .as_deref()
                .filter(|detail| !detail.is_empty())
            {
                details.push(mime_type.to_string());
            }
            if let Some(size) = resource.size {
                details.push(format!("{size} bytes"));
            }
            let details = details.join(" · ");
            if !details.is_empty() {
                text.push('\n');
                text.push_str(&details);
            }
            parse_markdown_into_text_and_code_sections(&text)
        }
        ContentBlock::Resource(resource) => match &resource.resource {
            EmbeddedResourceResource::TextResourceContents(resource) => {
                let mut text = format!("[{}]({})", resource.uri, resource.uri);
                if let Some(mime_type) = resource.mime_type.as_deref() {
                    text.push('\n');
                    text.push_str(mime_type);
                }
                if !resource.text.is_empty() {
                    text.push_str("\n\n");
                    text.push_str(&resource.text);
                }
                parse_markdown_into_text_and_code_sections(&text)
            }
            EmbeddedResourceResource::BlobResourceContents(resource) => {
                let mut text = format!("[{}]({})", resource.uri, resource.uri);
                if let Some(mime_type) = resource.mime_type.as_deref() {
                    text.push('\n');
                    text.push_str(mime_type);
                }
                text.push_str(&format!("\n{} bytes", resource.blob.len()));
                parse_markdown_into_text_and_code_sections(&text)
            }
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

fn escape_markdown_link_label(label: &str) -> String {
    label.replace('[', "\\[").replace(']', "\\]")
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub struct AcpPlan {
    pub plan: Plan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[derive(Default)]
pub struct AcpCommands {
    pub commands: Vec<AvailableCommand>,
}

impl AcpCommands {
    pub fn new(commands: Vec<AvailableCommand>) -> Self {
        Self { commands }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[derive(Default)]
pub struct AcpSessionConfig {
    pub current_mode: Option<CurrentModeUpdate>,
    pub config: Option<ConfigOptionUpdate>,
}

impl AcpSessionConfig {
    pub fn with_current_mode(mut self, current_mode: CurrentModeUpdate) -> Self {
        self.current_mode = Some(current_mode);
        self
    }

    pub fn with_config(mut self, config: ConfigOptionUpdate) -> Self {
        self.config = Some(config);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct AcpSessionInfo {
    pub info: SessionInfoUpdate,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AcpSessionState {
    pub commands: AcpCommands,
    pub config: AcpSessionConfig,
    pub info: Option<AcpSessionInfo>,
    pub terminal_traces: HashMap<String, AcpTerminalTrace>,
}
