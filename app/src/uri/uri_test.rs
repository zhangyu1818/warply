use super::*;
use crate::launch_configs::launch_config::make_mock_single_window_launch_config;
use crate::linear::{LinearAction, LinearIssueWork};
use crate::tab_configs::TabConfig;
use crate::ChannelState;
use std::{collections::HashMap, path::PathBuf};

#[test]
fn test_find_matching_config() {
    let mut configs: Vec<LaunchConfig> = vec![];
    for i in 0..5 {
        add_mock_config_with_name(
            (String::from("config") + i.to_string().as_str()).as_str(),
            &mut configs,
        );
    }

    let with_extension = "config1.yaml";
    assert_eq!(
        find_matching_config(with_extension, &configs),
        Some(&configs[1])
    );

    let no_extension = "config4";
    assert_eq!(
        find_matching_config(no_extension, &configs),
        Some(&configs[4])
    );

    let caps_insensitive = "ConFig3";
    assert_eq!(
        find_matching_config(caps_insensitive, &configs),
        Some(&configs[3])
    );

    let missing_config = "missing";
    assert_eq!(find_matching_config(missing_config, &configs), None);
}

#[test]
fn test_find_matching_config_with_spaces() {
    let mut configs: Vec<LaunchConfig> = vec![];
    for i in 0..3 {
        add_mock_config_with_name(
            (String::from("config") + i.to_string().as_str()).as_str(),
            &mut configs,
        );
    }

    let with_space = "config 3.yaml";
    add_mock_config_with_name(with_space, &mut configs);
    assert_eq!(
        find_matching_config(with_space, &configs),
        Some(&configs[3])
    );

    let more_space = " a ";
    add_mock_config_with_name(more_space, &mut configs);
    assert_eq!(
        find_matching_config(more_space, &configs),
        Some(&configs[4])
    );
}

#[test]
fn test_find_matching_configs_special_chars() {
    let mut configs: Vec<LaunchConfig> = vec![];
    for i in 0..3 {
        add_mock_config_with_name(
            (String::from("config") + i.to_string().as_str()).as_str(),
            &mut configs,
        );
    }

    // test special characters
    let special_ascii = "yes! this_works,too-even[braces}and(parens'.";
    add_mock_config_with_name(special_ascii, &mut configs);
    assert_eq!(
        find_matching_config(special_ascii, &configs),
        Some(&configs[3])
    );

    // test emojis
    let bread = "🍞";
    add_mock_config_with_name(bread, &mut configs);
    assert_eq!(find_matching_config(bread, &configs), Some(&configs[4]));
}

fn add_mock_config_with_name(name: &str, configs: &mut Vec<LaunchConfig>) {
    let mut new_config = make_mock_single_window_launch_config();
    new_config.name = name.to_string();
    new_config.windows[0].tabs[0].title = Some(String::from("First tab from config ") + name);
    configs.push(new_config);
}

#[test]
fn test_find_matching_tab_config() {
    let configs = vec![
        make_mock_tab_config("my tab", Some("/tab_configs/my_tab.toml")),
        make_mock_tab_config("Deploy", Some("/tab_configs/Deploy.yaml")),
        make_mock_tab_config("dotted", Some("/tab_configs/foo.bar.toml")),
        make_mock_tab_config("orphan", None),
    ];

    assert_eq!(
        find_matching_tab_config("my_tab", configs.clone()).map(|config| config.name),
        Some(String::from("my tab")),
    );
    assert_eq!(
        find_matching_tab_config("my_tab.toml", configs.clone()).map(|config| config.name),
        Some(String::from("my tab")),
    );
    assert_eq!(
        find_matching_tab_config("deploy", configs.clone()).map(|config| config.name),
        Some(String::from("Deploy")),
    );
    assert_eq!(
        find_matching_tab_config("foo.bar", configs.clone()).map(|config| config.name),
        Some(String::from("dotted")),
    );
    assert_eq!(
        find_matching_tab_config("foo.bar.toml", configs.clone()).map(|config| config.name),
        Some(String::from("dotted")),
    );
    assert!(find_matching_tab_config("unknown", configs.clone()).is_none());
    assert!(find_matching_tab_config("orphan", configs).is_none());
}

fn make_mock_tab_config(name: &str, source_path: Option<&str>) -> TabConfig {
    TabConfig {
        name: name.to_string(),
        title: None,
        color: None,
        panes: vec![],
        params: HashMap::new(),
        source_path: source_path.map(PathBuf::from),
    }
}

#[test]
fn test_get_launch_config_path() {
    assert_eq!(
        get_launch_config_path("/path/to/a/config"),
        Some(String::from("path/to/a/config")),
    );
    assert_eq!(
        get_launch_config_path("/hello%20world.yaml"),
        Some(String::from("hello world.yaml")),
    );
    assert_eq!(
        get_launch_config_path("/%3Bhello%20%23world!"),
        Some(String::from(";hello #world!")),
    );
    assert_eq!(
        get_launch_config_path("/yes%21%20this_works%2Ctoo-even%5Bbraces%7Dand%28parens%27."),
        Some(String::from("yes! this_works,too-even[braces}and(parens'."))
    );
    assert_eq!(
        get_launch_config_path("/%F0%9F%8D%9E"),
        Some(String::from("🍞"))
    );
    assert_eq!(
        get_launch_config_path("/..filename_.with_dots.."),
        Some(String::from("..filename_.with_dots.."))
    );
}

#[test]
fn test_get_launch_config_path_invalid() {
    assert_eq!(get_launch_config_path(""), None);
    assert_eq!(get_launch_config_path("/"), None);
    assert_eq!(get_launch_config_path("%2F"), None);
    assert_eq!(get_launch_config_path("/../outside"), None);
    assert_eq!(get_launch_config_path("/..%2Foutside"), None);
    assert_eq!(get_launch_config_path("/A/.."), None);
    assert_eq!(get_launch_config_path("/A/../B"), None);
    assert_eq!(get_launch_config_path("//absolute"), None);
    assert_eq!(get_launch_config_path("/%2Fabsolute sneaky"), None);
    assert_eq!(get_launch_config_path("//../very_bad/.."), None);
}

#[test]
fn test_remove_extension() {
    assert_eq!(remove_extension(""), None);
    assert_eq!(remove_extension(".yaml"), Some(""));
    assert_eq!(remove_extension(" .yaml"), Some(" "));
    assert_eq!(remove_extension("config.yaml"), Some("config"));
    assert_eq!(remove_extension("..yaml"), Some("."));
    assert_eq!(remove_extension("config"), None);
    assert_eq!(remove_extension("🍞.yaml"), Some("🍞"));
}

#[test]
fn test_action_new_agent_conversation_parse() {
    let url = Url::parse(&format!(
        "{}://action/new_agent_conversation",
        ChannelState::url_scheme()
    ))
    .unwrap();

    let action = Action::parse(&url).unwrap();
    assert!(matches!(action, Action::NewAgentConversation));
}

#[test]
fn test_validate_custom_uri_linear() {
    let url = Url::parse(&format!(
        "{}://linear/work?prompt=hello",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let host = validate_custom_uri(&url).unwrap();
    assert!(matches!(host, UriHost::Linear));
}

#[test]
fn test_linear_action_parse_work() {
    let url = Url::parse(&format!(
        "{}://linear/work?prompt=hello",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let action = LinearAction::parse(&url).unwrap();
    assert_eq!(action, LinearAction::WorkOnIssue);
}

#[test]
fn test_linear_action_parse_unknown_path() {
    let url = Url::parse(&format!("{}://linear/unknown", ChannelState::url_scheme())).unwrap();
    assert!(LinearAction::parse(&url).is_err());
}

#[test]
fn test_linear_issue_work_with_prompt() {
    let url = Url::parse(&format!(
        "{}://linear/work?prompt=fix+the+bug",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let args = LinearIssueWork::from_url(&url);
    assert_eq!(args.prompt.as_deref(), Some("fix the bug"));
}

#[test]
fn test_linear_issue_work_without_prompt() {
    let url = Url::parse(&format!("{}://linear/work", ChannelState::url_scheme())).unwrap();
    let args = LinearIssueWork::from_url(&url);
    assert!(args.prompt.is_none());
}

#[test]
fn test_linear_issue_work_empty_prompt() {
    let url = Url::parse(&format!(
        "{}://linear/work?prompt=",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let args = LinearIssueWork::from_url(&url);
    assert!(args.prompt.is_none());
}

// -- handle_incoming_uri redaction -------------------------------------------
//
// These tests validate URI log redaction and the error messages produced by
// `validate_custom_uri` so invalid URIs never embed query strings in logs.

/// The redacted log representation must contain scheme/host/path for triage
/// but must never contain the query string or any token material.
#[test]
fn safe_url_log_fields_redacts_sensitive_query_params() {
    let url = Url::parse(&format!(
        "{}://linear/work?prompt=SECRET_PROMPT&secret=SECRET_VALUE&token=RAW",
        ChannelState::url_scheme()
    ))
    .unwrap();

    let logged = safe_url_log_fields(&url);

    for forbidden in [
        "prompt=",
        "SECRET_PROMPT",
        "secret",
        "SECRET_VALUE",
        "token=RAW",
    ] {
        assert!(
            !logged.contains(forbidden),
            "redacted log must not contain {forbidden:?}: {logged}"
        );
    }
    assert!(logged.contains("host=linear"), "expected host: {logged}");
    assert!(logged.contains("path=/work"), "expected path: {logged}");
}

/// URL fragments are not currently used as secret carriers by Warp today, but
/// the entry log's contract is "scheme + host + path only", so fragments must
/// be dropped as well.
#[test]
fn safe_url_log_fields_drops_fragment() {
    let url = Url::parse(&format!(
        "{}://linear/work#sensitive_fragment",
        ChannelState::url_scheme()
    ))
    .unwrap();

    let logged = safe_url_log_fields(&url);

    assert!(
        !logged.contains("sensitive_fragment"),
        "redacted log must not contain url fragment: {logged}"
    );
    assert!(
        !logged.contains('#'),
        "redacted log must not contain any fragment separator: {logged}"
    );
}

/// `file://` URLs route through the same entry log. `file://` URLs on macOS
/// have no host; the helper must not panic and must report `host=-` so the
/// format string stays well-formed.
#[test]
fn safe_url_log_fields_handles_file_urls_without_host() {
    let url = Url::parse("file:///tmp/foo.md").unwrap();

    let logged = safe_url_log_fields(&url);

    assert!(logged.contains("scheme=file"), "expected scheme: {logged}");
    assert!(
        logged.contains("host=-"),
        "expected host placeholder: {logged}"
    );
    assert!(
        logged.contains("path=/tmp/foo.md"),
        "expected path: {logged}"
    );
}

/// `validate_custom_uri` returns `anyhow::Error`s whose messages are logged for
/// invalid URIs. Those messages must never embed query strings or fragments.
#[test]
fn validate_custom_uri_errors_do_not_leak_query_string() {
    // Unexpected scheme.
    let url = Url::parse("https://linear/work?token=LEAKED").unwrap();
    let err = validate_custom_uri(&url).unwrap_err();
    let msg = format!("{err:?}");
    assert!(!msg.contains("token="), "{msg}");
    assert!(!msg.contains("LEAKED"), "{msg}");

    // Unexpected host.
    let url = Url::parse(&format!(
        "{}://unknown_host/start?token=LEAKED",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let err = validate_custom_uri(&url).unwrap_err();
    let msg = format!("{err:?}");
    assert!(!msg.contains("token="), "{msg}");
    assert!(!msg.contains("LEAKED"), "{msg}");
}

#[test]
fn test_parse_tab_path_expands_tilde() {
    let url = Url::parse("warp://action/new_tab?path=~/Projects").unwrap();
    let home = dirs::home_dir().expect("HOME must be set for this test");
    assert_eq!(parse_tab_path(&url), Some(home.join("Projects")));
}

#[test]
fn test_parse_tab_path_expands_url_encoded_tilde() {
    // `%7E` and `%2F` are URL-encoded `~` and `/`.
    let url = Url::parse("warp://action/new_tab?path=%7E%2FProjects").unwrap();
    let home = dirs::home_dir().expect("HOME must be set for this test");
    assert_eq!(parse_tab_path(&url), Some(home.join("Projects")));
}

#[test]
fn test_parse_tab_path_absolute_path_unchanged() {
    let url = Url::parse("warp://action/new_tab?path=/tmp/foo").unwrap();
    assert_eq!(parse_tab_path(&url), Some(PathBuf::from("/tmp/foo")));
}

#[test]
fn test_parse_tab_path_relative_path_unchanged() {
    let url = Url::parse("warp://action/new_tab?path=relative/dir").unwrap();
    assert_eq!(parse_tab_path(&url), Some(PathBuf::from("relative/dir")));
}

#[test]
fn test_parse_tab_path_missing_returns_none() {
    let url = Url::parse("warp://action/new_tab").unwrap();
    assert_eq!(parse_tab_path(&url), None);
}

#[test]
fn test_parse_tab_path_bare_tilde() {
    let url = Url::parse("warp://action/new_tab?path=~").unwrap();
    let home = dirs::home_dir().expect("HOME must be set for this test");
    assert_eq!(parse_tab_path(&url), Some(home));
}

// Regression coverage for issue #9005: shell scripts opened via `file://` should run,
// not open in the editor. Exercised through the pure routing helper to avoid standing
// up a full `AppContext`.

#[test]
fn test_open_file_executable_sh_routes_to_execute() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("run.sh");
    std::fs::write(&p, b"#!/bin/sh\n:\n").unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        classify_open_file_action(&p),
        OpenFileAction::ExecuteInSession
    );
}

#[test]
fn test_open_file_non_executable_sh_routes_to_editor() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("view.sh");
    std::fs::write(&p, b"#!/bin/sh\n:\n").unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(classify_open_file_action(&p), OpenFileAction::Editor);
}

#[test]
fn test_open_file_executable_bash_zsh_fish_route_to_execute() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    for name in ["run.bash", "run.zsh", "run.fish"] {
        let p = dir.path().join(name);
        std::fs::write(&p, b"#!/bin/sh\n:\n").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            classify_open_file_action(&p),
            OpenFileAction::ExecuteInSession,
            "{name} should route to ExecuteInSession",
        );
    }
}

#[test]
fn test_open_file_markdown_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("README.md");
    std::fs::write(&p, b"# hi\n").unwrap();
    assert_eq!(classify_open_file_action(&p), OpenFileAction::Editor);
}

#[test]
#[cfg(feature = "local_fs")]
fn test_open_file_rust_source_still_opens_in_editor() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("main.rs");
    std::fs::write(&p, b"fn main() {}\n").unwrap();
    assert_eq!(classify_open_file_action(&p), OpenFileAction::Editor);
}

#[test]
fn test_open_file_directory_routes_to_session() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        classify_open_file_action(dir.path()),
        OpenFileAction::ExecuteInSession
    );
}

#[test]
fn test_open_file_non_runnable_shebang_routes_to_editor() {
    // Extensionless `#!/bin/sh` file without the user-execute bit. Without the
    // shebang fall-through this would hit `ExecuteInSession` and the shell would
    // refuse to run it; the editor is the right place to view it.
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("noext");
    std::fs::write(&p, b"#!/bin/sh\necho hi\n").unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(classify_open_file_action(&p), OpenFileAction::Editor);
}

#[test]
fn test_session_uri_host_parsing() {
    let result = UriHost::from_str("session");
    assert!(matches!(result, Ok(UriHost::Session)));
}

#[test]
fn test_session_uri_validation() {
    let url = Url::parse(&format!(
        "{}://session/A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let host = validate_custom_uri(&url).unwrap();
    assert!(matches!(host, UriHost::Session));
}

#[test]
fn test_session_uri_empty_path_does_not_panic() {
    let url = Url::parse(&format!("{}://session/", ChannelState::url_scheme())).unwrap();
    let host = validate_custom_uri(&url).unwrap();
    assert!(matches!(host, UriHost::Session));
}

#[test]
fn test_session_uri_invalid_hex_does_not_panic() {
    let url = Url::parse(&format!(
        "{}://session/ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
        ChannelState::url_scheme()
    ))
    .unwrap();
    let host = validate_custom_uri(&url).unwrap();
    assert!(matches!(host, UriHost::Session));
}

#[test]
fn test_session_uri_case_insensitive_hex() {
    let upper = "A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4";
    let lower = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4";
    let upper_bytes = super::decode_uuid_hex(upper).expect("upper hex should decode");
    let lower_bytes = super::decode_uuid_hex(lower).expect("lower hex should decode");
    assert_eq!(upper_bytes, lower_bytes);
    assert_eq!(upper_bytes.len(), 16);
}

#[test]
fn test_decode_uuid_hex_rejects_wrong_length() {
    assert!(super::decode_uuid_hex("ABCD").is_none());
    assert!(super::decode_uuid_hex("").is_none());
    assert!(super::decode_uuid_hex("A1B2C3D4E5F6A1B2C3D4E5F6A1B2C3D4FF").is_none());
}

#[test]
fn test_decode_uuid_hex_rejects_invalid_chars() {
    assert!(super::decode_uuid_hex("ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ").is_none());
}
