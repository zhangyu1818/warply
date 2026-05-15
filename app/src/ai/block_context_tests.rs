use super::*;
use serde_json::Value;

#[test]
fn block_context_serializes_id_as_block_id() {
    let ctx = BlockContext {
        id: "test-block-123".to_string().into(),
        index: 0.into(),
        command: "ls".to_string(),
        output: "file.txt".to_string(),
        exit_code: 0.into(),
        is_auto_attached: false,
        started_ts: None,
        finished_ts: None,
        pwd: None,
        shell: None,
        username: None,
        hostname: None,
        git_branch: None,
        os: None,
        session_id: None,
    };

    let json: Value = serde_json::to_value(&ctx).unwrap();
    // The `id` field should serialize as `block_id` on the wire.
    assert!(
        json.get("block_id").is_some(),
        "expected 'block_id' key in JSON"
    );
    assert!(json.get("id").is_none(), "should not have 'id' key in JSON");
    assert_eq!(json["block_id"], "test-block-123");
}

#[test]
fn block_context_serializes_none_environment_fields_as_null() {
    let ctx = BlockContext {
        id: "b1".to_string().into(),
        index: 0.into(),
        command: "echo hi".to_string(),
        output: "hi".to_string(),
        exit_code: 0.into(),
        is_auto_attached: false,
        started_ts: None,
        finished_ts: None,
        pwd: None,
        shell: None,
        username: None,
        hostname: None,
        git_branch: None,
        os: None,
        session_id: None,
    };

    let json_str = serde_json::to_string(&ctx).unwrap();
    // Environment fields with None are part of the current payload and serialize as null.
    let json: Value = serde_json::from_str(&json_str).unwrap();
    assert!(json.get("pwd").is_some());
    assert!(json["pwd"].is_null());
}

#[test]
fn block_context_requires_auto_attached_field() {
    let json = r#"{
        "block_id": "b1",
        "index": 0,
        "command": "ls",
        "output": "file.txt",
        "exit_code": 0
    }"#;

    let result = serde_json::from_str::<BlockContext>(json);
    assert!(result.is_err());
}
