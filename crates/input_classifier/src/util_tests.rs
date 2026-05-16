use warp_completer::ParsedTokensSnapshot;
use warp_completer::util::parse_current_commands_and_tokens;

use crate::test_utils::CompletionContext;

use super::*;

async fn mock_parsed_input_token(buffer_text: &str) -> ParsedTokensSnapshot {
    let completion_context = CompletionContext::new();
    parse_current_commands_and_tokens(buffer_text.to_string(), &completion_context).await
}

fn clear_all_token_descriptions(snapshot: &mut ParsedTokensSnapshot) {
    for token in snapshot.parsed_tokens.iter_mut() {
        token.token_description = None;
    }
}

#[test]
fn test_is_likely_shell_command_one_off_keyword_short_circuits() {
    futures::executor::block_on(async move {
        let mut token = mock_parsed_input_token("sudo apt update").await;
        let word_tokens_count = token.parsed_tokens.len();
        clear_all_token_descriptions(&mut token);

        assert!(is_likely_shell_command(&token, word_tokens_count).await);
    });
}

#[test]
fn test_is_likely_shell_command_requires_all_described_tokens() {
    futures::executor::block_on(async move {
        let mut token = mock_parsed_input_token("cargo build --release --workspace").await;
        let word_tokens_count = token.parsed_tokens.len();
        let description = token
            .parsed_tokens
            .iter()
            .find_map(|token| token.token_description.clone())
            .expect("test input should include a described token");

        for token in token.parsed_tokens.iter_mut() {
            token.token_description = Some(description.clone());
        }
        token
            .parsed_tokens
            .last_mut()
            .expect("test input should include tokens")
            .token_description = None;

        assert!(!is_likely_shell_command(&token, word_tokens_count).await);
    });
}

#[test]
fn test_is_likely_shell_command_ignores_shell_syntax_votes() {
    futures::executor::block_on(async move {
        let mut token = mock_parsed_input_token("git --foo=bar /tmp/file --baz").await;
        let word_tokens_count = token.parsed_tokens.len();

        for (idx, token) in token.parsed_tokens.iter_mut().enumerate() {
            if idx != 0 {
                token.token_description = None;
            }
        }

        assert!(word_tokens_count >= 3);
        assert!(!is_likely_shell_command(&token, word_tokens_count).await);
    });
}

#[test]
fn test_is_likely_shell_command_all_described_tokens_is_shell() {
    futures::executor::block_on(async move {
        let mut token = mock_parsed_input_token("cargo build --release --workspace").await;
        let word_tokens_count = token.parsed_tokens.len();
        let description = token
            .parsed_tokens
            .iter()
            .find_map(|token| token.token_description.clone())
            .expect("test input should include a described token");

        for token in token.parsed_tokens.iter_mut() {
            token.token_description = Some(description.clone());
        }

        assert!(is_likely_shell_command(&token, word_tokens_count).await);
    });
}
