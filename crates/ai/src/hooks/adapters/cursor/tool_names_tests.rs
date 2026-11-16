use warp_terminal::model::BlockId;

use super::*;
use crate::agent::action::{
    AIAgentPtyWriteMode, AskUserQuestionItem, AskUserQuestionType, ReadFilesRequest,
    SearchCodebaseRequest,
};
use crate::hooks::adapters::cursor::test_support::shell_action;

#[test]
fn tool_names_follow_cursors_vocabulary() {
    let cases: Vec<(AIAgentActionType, &str)> = vec![
        (shell_action("ls"), "Shell"),
        (
            AIAgentActionType::ReadFiles(ReadFilesRequest { locations: vec![] }),
            "Read",
        ),
        (
            AIAgentActionType::RequestFileEdits {
                file_edits: vec![FileEdit::Create {
                    file: Some("a.txt".to_owned()),
                    content: None,
                }],
                title: None,
            },
            "Write",
        ),
        (
            AIAgentActionType::RequestFileEdits {
                file_edits: vec![FileEdit::Delete {
                    file: Some("a.txt".to_owned()),
                }],
                title: None,
            },
            "Delete",
        ),
        (
            AIAgentActionType::RequestFileEdits {
                file_edits: vec![
                    FileEdit::Delete {
                        file: Some("a.txt".to_owned()),
                    },
                    FileEdit::Create {
                        file: Some("b.txt".to_owned()),
                        content: None,
                    },
                ],
                title: None,
            },
            "Write",
        ),
        (
            AIAgentActionType::RequestFileEdits {
                file_edits: vec![],
                title: None,
            },
            "Write",
        ),
        (
            AIAgentActionType::Grep {
                queries: vec![],
                path: ".".to_owned(),
            },
            "Grep",
        ),
        (
            AIAgentActionType::FileGlobV2 {
                patterns: vec![],
                search_dir: None,
            },
            "Glob",
        ),
        (
            AIAgentActionType::SearchCodebase(SearchCodebaseRequest {
                query: String::new(),
                partial_paths: None,
                codebase_path: None,
            }),
            "SearchCodebase",
        ),
        (
            AIAgentActionType::WriteToLongRunningShellCommand {
                block_id: BlockId::new(),
                input: bytes::Bytes::new(),
                mode: AIAgentPtyWriteMode::Raw,
            },
            "ShellWrite",
        ),
        (
            AIAgentActionType::ReadShellCommandOutput {
                block_id: BlockId::new(),
                delay: None,
            },
            "ShellRead",
        ),
        (
            AIAgentActionType::TransferShellCommandControlToUser {
                reason: String::new(),
            },
            "ShellHandoff",
        ),
        (AIAgentActionType::InitProject, "InitProject"),
        (AIAgentActionType::OpenCodeReview, "OpenCodeReview"),
        (
            AIAgentActionType::AskUserQuestion {
                questions: vec![AskUserQuestionItem {
                    question_id: String::new(),
                    question: String::new(),
                    question_type: AskUserQuestionType::MultipleChoice {
                        is_multiselect: false,
                        options: vec![],
                        supports_other: false,
                    },
                }],
            },
            "AskUserQuestion",
        ),
    ];
    for (action, expected) in cases {
        assert_eq!(tool_name(&action), expected, "{action}");
    }
}
