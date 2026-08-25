use std::collections::HashMap;

use chrono::{Local, TimeZone, Utc};
use warp_editor::render::model::LineCount;

use super::{
    AIAgentAttachment, AIAgentContext, AnyFileContent, CurrentHead, DiffBase, DiffSetHunk,
    DocumentContentAttachmentSource, FileContext, ImageContext,
};
use crate::ai::block_context::BlockContext;
use crate::ai::execution_context::{AiExecutionContext, AiOsContext};

fn sample_block_context() -> BlockContext {
    BlockContext {
        id: "block-1".to_string().into(),
        index: 0.into(),
        command: "ls".to_string(),
        output: "file.txt".to_string(),
        exit_code: 0.into(),
        is_auto_attached: false,
        started_ts: None,
        finished_ts: None,
        pwd: Some("/tmp".to_string()),
        shell: None,
        username: None,
        hostname: None,
        git_branch: None,
        os: None,
        session_id: None,
    }
}

#[test]
fn ai_agent_context_round_trips_tagged_variants() {
    let contexts = vec![
        AIAgentContext::Directory {
            pwd: Some("/tmp/project".to_string()),
            home_dir: Some("/Users/me".to_string()),
            are_file_symbols_indexed: true,
        },
        AIAgentContext::SelectedText("selected text".to_string()),
        AIAgentContext::ExecutionEnvironment(AiExecutionContext {
            os: AiOsContext {
                category: Some("MacOS".to_string()),
                distribution: None,
            },
            shell_name: "zsh".to_string(),
            shell_version: Some("5.9".to_string()),
        }),
        AIAgentContext::CurrentTime {
            current_time: Utc
                .with_ymd_and_hms(2024, 1, 15, 10, 30, 0)
                .unwrap()
                .with_timezone(&Local),
        },
        AIAgentContext::Image(ImageContext {
            data: "aGVsbG8=".to_string(),
            mime_type: "image/png".to_string(),
            file_name: "shot.png".to_string(),
            is_figma: false,
        }),
        AIAgentContext::Codebase {
            path: "/tmp/project".to_string(),
            name: "project".to_string(),
        },
        AIAgentContext::ProjectRules {
            root_path: "/tmp/project".to_string(),
            active_rules: vec![FileContext::new(
                "WARP.md".to_string(),
                AnyFileContent::StringContent("Be nice.".to_string()),
                None,
                None,
            )],
            additional_rule_paths: vec!["sub/WARP.md".to_string()],
        },
        AIAgentContext::File(FileContext::new(
            "a.txt".to_string(),
            AnyFileContent::StringContent("hey\nyou".to_string()),
            None,
            None,
        )),
        AIAgentContext::Git {
            head: "abc1234".to_string(),
            branch: Some("main".to_string()),
        },
    ];
    for context in contexts {
        let json = serde_json::to_value(&context).unwrap();
        let deserialized: AIAgentContext = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, context);
    }
}

#[test]
fn ai_agent_context_round_trips_untagged_block_variant() {
    let context = AIAgentContext::Block(Box::new(sample_block_context()));
    let json = serde_json::to_value(&context).unwrap();
    // The Block variant must serialize untagged, as a bare object.
    assert!(
        json.get("block_id").is_some(),
        "expected untagged block object, got {json}"
    );
    let deserialized: AIAgentContext = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, context);
}

#[test]
fn ai_agent_context_rejects_unknown_variants() {
    let result = serde_json::from_str::<AIAgentContext>(r#"{"NotARealVariant":{}}"#);
    assert!(result.is_err());
}

#[test]
fn ai_agent_attachment_round_trips_tagged_variants() {
    let attachments = vec![
        AIAgentAttachment::PlainText("hello".to_string()),
        AIAgentAttachment::DocumentContent {
            document_id: "doc-1".to_string(),
            content: "# Plan".to_string(),
            source: DocumentContentAttachmentSource::UserAttached,
            line_range: Some(LineCount::range(1..5)),
        },
        AIAgentAttachment::DiffHunk {
            file_path: "src/main.rs".to_string(),
            line_range: LineCount::range(1..3),
            diff_content: "+fn main() {}".to_string(),
            lines_added: 1,
            lines_removed: 0,
            current: Some(CurrentHead::BranchName("feature".to_string())),
            base: DiffBase::BranchName("main".to_string()),
        },
        AIAgentAttachment::DiffSet {
            file_diffs: HashMap::from([(
                "src/main.rs".to_string(),
                vec![DiffSetHunk {
                    line_range: LineCount::range(1..3),
                    diff_content: "+use std::fmt;".to_string(),
                    lines_added: 1,
                    lines_removed: 0,
                }],
            )]),
            current: None,
            base: DiffBase::UncommittedChanges,
        },
        AIAgentAttachment::FilePathReference {
            file_id: "file-1".to_string(),
            file_name: "report.txt".to_string(),
            file_path: "/tmp/report.txt".to_string(),
        },
    ];
    for attachment in attachments {
        let json = serde_json::to_value(&attachment).unwrap();
        let deserialized: AIAgentAttachment = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, attachment);
    }
}

#[test]
fn ai_agent_attachment_round_trips_untagged_block_variant() {
    let attachment = AIAgentAttachment::Block(sample_block_context());
    let json = serde_json::to_value(&attachment).unwrap();
    // The Block variant must serialize untagged, as a bare object.
    assert!(
        json.get("block_id").is_some(),
        "expected untagged block object, got {json}"
    );
    let deserialized: AIAgentAttachment = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, attachment);
}
