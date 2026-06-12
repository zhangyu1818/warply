use crate::terminal::shell::ShellType;

use super::shell_escape_single_quotes;

#[test]
fn no_quotes_returns_input_unchanged() {
    for shell_type in [
        ShellType::Bash,
        ShellType::Zsh,
        ShellType::Fish,
        ShellType::PowerShell,
    ] {
        assert_eq!(
            shell_escape_single_quotes("/home/user/histfile", shell_type),
            "/home/user/histfile"
        );
    }
}

#[test]
fn bash_escapes_single_quote_with_concatenation() {
    assert_eq!(
        shell_escape_single_quotes("it's a test", ShellType::Bash),
        r#"it'"'"'s a test"#
    );
}

#[test]
fn zsh_escapes_single_quote_with_concatenation() {
    assert_eq!(
        shell_escape_single_quotes("it's a test", ShellType::Zsh),
        r#"it'"'"'s a test"#
    );
}

#[test]
fn fish_escapes_single_quote_with_backslash() {
    assert_eq!(
        shell_escape_single_quotes("it's a test", ShellType::Fish),
        r"it\'s a test"
    );
}

#[test]
fn powershell_escapes_single_quote_by_doubling() {
    assert_eq!(
        shell_escape_single_quotes("it's a test", ShellType::PowerShell),
        "it''s a test"
    );
}
