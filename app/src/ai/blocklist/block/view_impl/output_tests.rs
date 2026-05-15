use agent_client_protocol::schema::{
    ContentBlock, Diff, ImageContent, ResourceLink, Terminal, TextContent, ToolCall,
    ToolCallContent, ToolCallLocation, ToolKind,
};

use crate::ai::acp::{acp_raw_images, AcpTerminalTrace, AcpToolCall, ACP_RAW_IMAGE_SOURCE_PREFIX};
use crate::ai::agent::AIAgentTextSection;

use super::{
    acp_tool_call_content_sections, acp_tool_call_location_display_strings,
    acp_tool_call_render_kind, format_acp_terminal_trace, format_acp_tool_call_content,
    AcpToolCallRenderKind, AcpToolCallSurfaceKind,
};

#[test]
fn format_acp_tool_call_content_includes_text_content() {
    let call = AcpToolCall::from_acp(
        ToolCall::new("tool-1", "Inspect file")
            .kind(ToolKind::Read)
            .content(vec![ToolCallContent::from(ContentBlock::Text(
                TextContent::new("read src/main.rs"),
            ))]),
    );

    assert_eq!(
        format_acp_tool_call_content(&call).as_deref(),
        Some("read src/main.rs")
    );
}

#[test]
fn format_acp_tool_call_content_does_not_render_non_text_content_as_plain_text() {
    let call = AcpToolCall::from_acp(
        ToolCall::new("tool-1", "Inspect file")
            .kind(ToolKind::Read)
            .content(vec![
                ToolCallContent::from(ContentBlock::Image(ImageContent::new(
                    "base64",
                    "image/png",
                ))),
                ToolCallContent::from(Diff::new("src/main.rs", "fn main() {}\n")),
                ToolCallContent::Terminal(Terminal::new("term-1")),
            ]),
    );

    assert_eq!(format_acp_tool_call_content(&call), None);
}

#[test]
fn acp_tool_call_content_sections_include_resource_links_and_images() {
    let call = AcpToolCall::from_acp(
        ToolCall::new("tool-1", "Inspect file")
            .kind(ToolKind::Read)
            .content(vec![
                ToolCallContent::from(ContentBlock::ResourceLink(
                    ResourceLink::new("spec", "https://example.com/spec").title("Spec"),
                )),
                ToolCallContent::from(ContentBlock::Image(ImageContent::new(
                    "aW1hZ2U=",
                    "image/png",
                ))),
            ]),
    );

    let sections = acp_tool_call_content_sections(&call);

    assert!(matches!(
        &sections[0],
        AIAgentTextSection::PlainText { text }
            if text.text().contains("[Spec](https://example.com/spec)")
    ));
    assert!(matches!(
        &sections[1],
        AIAgentTextSection::Image { image }
            if image.source.starts_with(ACP_RAW_IMAGE_SOURCE_PREFIX)
                && image.markdown_source.contains(&image.source)
    ));
}

#[test]
fn acp_raw_images_use_protocol_image_blocks() {
    let call = AcpToolCall::from_acp(
        ToolCall::new("tool-1", "Inspect image")
            .kind(ToolKind::Read)
            .content(vec![ToolCallContent::from(ContentBlock::Image(
                ImageContent::new("aW1hZ2U=", "image/png"),
            ))]),
    );

    let images = acp_raw_images(&call);

    assert_eq!(images.len(), 1);
    assert_eq!(images[0].asset_id, "acp-tool-1-0.png");
    assert!(images[0].source.starts_with(ACP_RAW_IMAGE_SOURCE_PREFIX));
    assert_eq!(images[0].data, "aW1hZ2U=");
    assert_eq!(images[0].mime_type, "image/png");
}

#[test]
fn acp_terminal_trace_sections_use_matching_terminal_reference() {
    let mut call = AcpToolCall::from_acp(
        ToolCall::new("tool-1", "Run tests")
            .kind(ToolKind::Execute)
            .content(vec![ToolCallContent::Terminal(Terminal::new("term-1"))]),
    );

    call.set_terminal_trace(
        "term-1".to_string(),
        AcpTerminalTrace {
            command: Some("cargo test".to_string()),
            cwd: Some("/repo".to_string()),
            output: "running 1 test".to_string(),
            exit_code: Some(0),
        },
    );

    let sections = acp_tool_call_content_sections(&call);

    assert!(sections.iter().any(|section| matches!(
        section,
        AIAgentTextSection::PlainText { text }
            if text.text().contains("$ cargo test")
                && text.text().contains("cwd: /repo")
                && text.text().contains("running 1 test")
                && text.text().contains("exit: 0")
    )));
}

#[test]
fn acp_terminal_trace_sections_ignore_unreferenced_terminal_trace() {
    let mut call = AcpToolCall::from_acp(
        ToolCall::new("tool-1", "Run tests")
            .kind(ToolKind::Execute)
            .content(vec![ToolCallContent::Terminal(Terminal::new("term-1"))]),
    );

    call.set_terminal_trace(
        "term-2".to_string(),
        AcpTerminalTrace {
            command: Some("cargo test".to_string()),
            cwd: Some("/repo".to_string()),
            output: "running 1 test".to_string(),
            exit_code: Some(0),
        },
    );

    assert!(acp_tool_call_content_sections(&call).is_empty());
}

#[test]
fn acp_tool_call_render_kind_uses_protocol_kind_and_structured_fields() {
    let read_locations = AcpToolCall::from_acp(
        ToolCall::new("read", "Read file")
            .kind(ToolKind::Read)
            .locations(vec![ToolCallLocation::new("/repo/src/main.rs")]),
    );
    let edit_diff = AcpToolCall::from_acp(ToolCall::new("edit", "Edit file").kind(ToolKind::Edit));
    let search = AcpToolCall::from_acp(ToolCall::new("search", "Search").kind(ToolKind::Search));
    let execute =
        AcpToolCall::from_acp(ToolCall::new("execute", "Run tests").kind(ToolKind::Execute));
    let other = AcpToolCall::from_acp(ToolCall::new("other", "Custom").kind(ToolKind::Other));

    assert_eq!(
        acp_tool_call_render_kind(&read_locations, false),
        AcpToolCallRenderKind::ReadLocations
    );
    assert_eq!(
        acp_tool_call_render_kind(&edit_diff, true),
        AcpToolCallRenderKind::FileDiff
    );
    assert_eq!(
        acp_tool_call_render_kind(&search, false),
        AcpToolCallRenderKind::Search
    );
    assert_eq!(
        acp_tool_call_render_kind(&execute, false),
        AcpToolCallRenderKind::Execute
    );
    assert_eq!(
        acp_tool_call_render_kind(&other, false),
        AcpToolCallRenderKind::Other
    );
}

#[test]
fn acp_tool_call_location_display_strings_use_structured_locations() {
    let call = AcpToolCall::from_acp(
        ToolCall::new("read", "A title that should not be parsed")
            .kind(ToolKind::Read)
            .locations(vec![
                ToolCallLocation::new("/repo/src/main.rs").line(12),
                ToolCallLocation::new("/repo/src/lib.rs"),
            ]),
    );

    assert_eq!(
        acp_tool_call_location_display_strings(&call, None, None),
        vec![
            "/repo/src/main.rs (12-12)".to_string(),
            "/repo/src/lib.rs".to_string(),
        ]
    );
}

#[test]
fn acp_tool_call_surface_kind_keeps_execute_off_title_action_cards() {
    assert_eq!(
        AcpToolCallSurfaceKind::from(AcpToolCallRenderKind::Execute),
        AcpToolCallSurfaceKind::Header
    );
    assert_eq!(
        AcpToolCallSurfaceKind::from(AcpToolCallRenderKind::Search),
        AcpToolCallSurfaceKind::Header
    );
    assert_eq!(
        AcpToolCallSurfaceKind::from(AcpToolCallRenderKind::ReadLocations),
        AcpToolCallSurfaceKind::RenderableAction
    );
}

#[test]
fn format_acp_terminal_trace_uses_structured_output() {
    let trace = AcpTerminalTrace {
        command: Some("printf hi".to_string()),
        cwd: Some("/tmp".to_string()),
        output: "hi".to_string(),
        exit_code: Some(0),
    };

    let text = format_acp_terminal_trace(&trace);

    assert!(text.contains("$ printf hi"));
    assert!(text.contains("cwd: /tmp"));
    assert!(text.contains("hi"));
    assert!(text.contains("exit: 0"));
    assert!(!text.contains("terminal_id"));
}
