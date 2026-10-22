# Upstream Master Audit — 2026-10-22

Incremental audit for `9b499e029..upstream/master` (`f8a9c719f`), 8 commits. The previous audit is `upstream-master-audit-2026-10-21.md`.

## Summary

One adapted port: the V4A blank-line preservation and independent hunk-outcome fix (`6ce705ac5`) in the retained `crates/ai/src/diff_validation` library and the restored-preview path of `code_diff_view.rs`. Seven commits are not applicable or rejected: the agent-sdk ACP-harness follow-up batching (`f3bb1306b`), its release-build flag promotion (`349cdd6f5`), and the five pre-warmed sandbox commits (`96d816ad2`, `4ff5753e2`, `c6c62e2fe`, `0616e9d38`, `f8a9c719f`) all live on fork-absent `agent_sdk`/`server_api`/`graphql`/`managed_secrets`/`conversation_restoration` paths with zero anchors in the fork's shared files.

## Commit-by-Commit

### `f3bb1306b` ACP: batch follow-ups and preserve delivery after cancel timeouts (#16571) — not applicable

All five files live in `app/src/ai/agent_sdk/driver/harness/acp/` (connection, turn_driver, and their tests), the ACP harness transport stack rejected/N.A. on 2026-10-09/10-10/10-20. The fix batches unsent follow-ups in the harness turn driver, splits prompt submission from response waiting, and makes `session/cancel` best-effort with a 15-second grace period. The fork's ACP client (`app/src/ai/acp/`) is an independent implementation: its `model.rs` owns cancellation through `pending_session_cancels` cancel channels with no turn driver, follow-up batching queue, or cancel-timeout ladder, so the fixed code structure has no anchor here. Porting the behavior would mean rewriting the fork's client from the upstream harness source shape it never shared.

### `96d816ad2` feat(prewarmed-sandboxes): prepare cloud provider credentials (#16576) — rejected (removed surfaces)

Extends the pre-warmed standby sandbox stack with cloud provider (AWS/GCP) credential preparation across `app/src/ai/agent_sdk/driver/cloud_provider*`, `app/src/server/server_api/prewarmed_sandbox*`, `crates/graphql`, `crates/warp_graphql_schema`, `crates/managed_secrets`, and `crates/warp_cli` federate commands. Every path is fork-absent (no `agent_sdk`, no `app/src/server`, no graphql/managed_secrets crates, no `warp_cli/src/federate.rs`); `rg` over the fork returns zero anchors for `prewarmed_sandbox`, `cloud_provider`, or the federate surface.

### `6ce705ac5` Preserve V4A blank lines and report independent hunk outcomes (#16306) — adapted, ported

Retained local library fix for the V4A (`V4AEdit`) diff format used by restored AgentView file-edit cards. Upstream adds `V4ALineData` (explicit per-hunk line slices, serde-defaulted) alongside the legacy strings, decodes hunks through `DecodedV4AHunk` so trailing blank lines survive matching, compares no-op outcomes as line lists instead of raw strings, keeps CR-bearing legacy replacements byte-exact, matches blank-only candidates safely across the exact/indentation-agnostic/fuzzy tiers via a `V4AScorer`, and exposes `fuzzy_match_v4a_preview` so restored previews match synthetic lines directly. In the fork this is live runtime behavior: `code_diff_view.rs` deserializes persisted V4A history and rebuilds lossy restored previews through `fuzzy_match_v4a_diffs`, so the blank-line bug (trailing-blank edits treated as no-ops, wrong-occurrence matches) is reproducible here.

Applied via `git diff 6ce705ac5^ 6ce705ac5 -- <paths> | git apply --3way` for the retained paths, with these recorded adaptations:

- `crates/ai/src/diff_validation/mod.rs`: all six hunks applied cleanly; the result is byte-identical to upstream except the fork's pre-existing import grouping, telemetry-wording comment removals, and `#[path = "mod_test.rs"]` test module name.
- `crates/ai/src/diff_validation/mod_test.rs` (upstream `mod_tests.rs`): the test hunk applied cleanly after path remap to the fork's singular test filename; all eight new tests and the `line_data: None` field additions are verbatim upstream. The two pre-existing test divergences (fork-only `test_multiline_prefix_tail_rescue_preserves_suffix_after_deleted_line`, absent `test_partial_last_line_in_multiline_replacement_preserves_suffix`) predate this port and are untouched.
- `crates/ai/src/diff_validation/testdata/blank_lines.json`: copied verbatim.
- `app/src/ai/blocklist/inline_action/code_diff_view.rs`: hunks applied with two conflict resolutions. The import block keeps the fork's grouped std import and only swaps `fuzzy_match_v4a_diffs` → `fuzzy_match_v4a_preview` in the `ai::diff_validation` import; the file-tail hunk keeps the fork's tail (the fork has no `changed_lines_for_result`/range-conversion helpers and no `code_diff_persistence_tests` module — both removed at baseline) and adds only the new `#[cfg(test)] #[path = "code_diff_view_tests.rs"] mod tests;` declaration. Because upstream's `FileDiff` (moved to the fork-absent `app/src/ai/blocklist/diff_types.rs`) carries a `line_stats()` method the new test asserts on, the exact upstream `line_stats` + `line_count` implementations were copied onto the fork's inline `FileDiff` impl.
- `app/src/ai/blocklist/inline_action/code_diff_view_tests.rs`: both new tests (`v4a_string_restore_previews`, `v4a_restore_preserves_cr_ending_line`) verbatim.
- `app/src/code/editor/model_tests.rs`: `test_apply_v4a_final_bytes` adapted — upstream constructs the hunk through the `warp_multi_agent_api` proto `ApplyFileDiffs` and the fork-absent `convert.rs` `From` impl; the fork deserializes the fixture's `dual` V4AHunk directly and passes it to `fuzzy_match_v4a_diffs` (identical observable, since the omitted proto conversion maps fields verbatim). The editor application and final-bytes assertions are verbatim.

Omitted paths:

- `Cargo.toml`/`Cargo.lock` `warp_multi_agent_api` pin bump (`c5aa1833` → `8320a948`): the fork has no `warp_multi_agent_api` dependency; the pin only adds the proto `line_data` wire field whose only consumer is the omitted `convert.rs`.
- `crates/ai/src/agent/action/convert.rs`: proto → `V4AHunk.line_data` mapping; the fork's `agent/action` is a one-import `mod.rs` with no proto source.
- `app/src/ai/blocklist/action_model/execute/request_file_edits/{diff_application,diff_application_tests,telemetry.rs}`: the fork's `request_file_edits.rs` is an ACP-adapted executor delegating to `CodeDiffView` with no upstream diff-application machinery, and the `V4AMatchFinished` telemetry event is removed-surface telemetry.
- `crates/warp_tui/src/tui_diff_storage_tests.rs`: no `warp_tui` crate in the fork.

Pre-existing defect fixed alongside: the fork's `test_no_trailing_newline` in `app/src/code/editor/model_tests.rs` had lost its `#[test]` attribute (upstream still has it), so the test never ran; the attribute is restored.

### `4ff5753e2` fix(prewarmed): honor the standby control origin (#16592) — not applicable

Adds a `prewarmed_sandbox_control_url` override (env const, `WarpServerConfig` field, `ChannelState` override/getter, `initialize_app` wiring) so a standby client honors the control origin it was spawned with. The fork's `warp_core/channel` has no `WarpServerConfig`, `server_root_url`, `rtc_server_url`, `oz_config`, or `workload_audience_url` anchors; `warp_cli/src/lib.rs` has no `*_OVERRIDE_ENV` consts; the integration bin constructs no server config; and the `initialize_app` hunk extends the `set_prewarmed_sandbox_id` branch of the removed standby machinery. The pre-warmed standby control plane is a removed cloud surface (rejected 2026-10-16..21).

### `349cdd6f5` Enable the ACP harness transport in release builds (#16599) — not applicable

Promotes the `acp_harness` cargo feature to default and moves `FeatureFlag::AcpHarness` out of `DOGFOOD_FLAGS`. The fork has no `acp_harness` cargo feature, no `FeatureFlag::AcpHarness` (the harness transport stack was never ported), and `app/src/features.rs` is a one-line stub. The upstream description confirms the flag exists to let the Warp server choose ACP for cloud runs — a server-selected transport concept that has no owner in this fork's client-owned ACP path.

### `c6c62e2fe` perf(agent-sdk): resolve managed MCP client configs concurrently (#16597) — not applicable

Rewrites `app/src/ai/agent_sdk/driver/mcp_startup.rs` to resolve managed MCP client configs concurrently. Both the agent SDK and app-managed MCP are removed surfaces; the path is fork-absent.

### `0616e9d38` fix(prewarmed): replace standby git credentials and author identity (#16601) — not applicable

Adds standby git-credential/author replacement in `app/src/ai/agent_sdk/driver/git_credentials.rs` and `prewarmed.rs`. Fork-absent paths; the pre-warmed standby stack is a removed cloud surface.

### `f8a9c719f` feat(prewarmed): restore resumed runs in standby terminals (#16598) — not applicable

Restores resumed cloud runs into standby terminals. The behavioral change is confined to `app/src/terminal/conversation_restoration.rs` (fork-absent; it captures `first_restored_command = active_block_index()` and makes `command_block_indices_for_exchanges` skip pre-existing blocks — meaningful only when serialized blocklist items are inserted into a terminal that already has standby blocks). The fork keeps its own private `command_block_indices_for_exchanges` in `load_ai_conversation.rs`, has no `to_serialized_blocklist_items`/`prepare_conversation_block_restoration` machinery, and the one fork-present hunk (`load_ai_conversation.rs` passing `BlockIndex::from(0)`) is a signature adaptation whose behavior is identical to the fork's current call sites. All remaining hunks are `agent_sdk` driver/terminal paths.

## Verification

- `cargo fmt -- --check` — passed.
- `cargo check -p warp --all-targets --message-format short` — passed (SDKROOT pointed at the Xcode MacOSX SDK per the CLT tapi prerequisite); remaining warnings are pre-existing dead-code lints.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 tests passed, 0 failed.
- Focused new tests: `cargo nextest run -p ai -E 'test(diff_validation) or test(v4a)'` — 40 passed including the eight new V4A tests; `cargo nextest run -p warp -E 'test(code_diff_view) or test(apply_v4a) or test(no_trailing_newline)'` — 5 passed including both new restored-preview tests, the adapted final-bytes editor test, and the revived `test_no_trailing_newline`.
- `cargo build -p warp --all-targets --message-format short` — passed; `cargo clean` run after the push.
- Restored-surface scans over the diff against `main` — zero hits for all three scan patterns.

## Tag

`v2026.10.22` on `main` (merge commit of `merge/upstream-2026-10-22`).
