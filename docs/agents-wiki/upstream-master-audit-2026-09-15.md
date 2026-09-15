# Upstream Master Audit 2026-09-15

Range under review: `3959ea721..upstream/master` (9 commits)

Previous audited upstream tip: `3959ea721 Route AI history updates through TerminalView (#15943)`

Current upstream tip detected: `e3464f102 Attach the ambient workload token to the built-in Factory MCP server (#15992)`

Total upstream commits in this incremental range: 9

Status: triage complete. One commit ported to `merge/upstream-2026-09-15`:

- `c72f36d6a` — adapted port of tab-group persistence in launch configurations: `WindowTemplate` gains `tab_groups: Vec<TabGroupTemplate>` and `TabTemplate` gains a `group: Option<usize>` index. Save (`From<WindowSnapshot>`) carries membership on each tab's own `group_id`, drops memberless groups, and remaps surviving indices around unsaveable tabs. Restore (`open_launch_config_window`) resolves contiguous membership via `resolve_group_memberships`, mints fresh `TabGroupId`s, reads back real insertion indices (`NewTabPlacement` aware), re-anchors the restored block past the host group (`move_restored_block_past_group`), moves pinned groups to the pinned boundary, and resolves the active tab by pane-group id.

## Per-Commit Triage

### `c72f36d6a` — Save and restore tab groups in launch configurations (#14624)

Decision: **adapt** (purely local retained feature: tab grouping was ported 2026-08-12; launch configs are retained local persistence). Applied via `git cherry-pick --no-commit` with conflict resolution; all ported regions follow the upstream post-image.

Fork adaptations:

- `open_launch_config_window`: the fork has no `FeatureFlag::GroupedTabs`/`FeatureFlag::PinnedTabs` (removed with the rollout flags when grouping was ported), so `group_count` is unconditionally `window.tab_groups.len()` and `pinned` honors `group_template.pinned` directly, matching the fork's session-restore precedent (`configure_new_workspace` sets `pinned: group_snapshot.pinned` without a gate).
- Integration tests: the four `FeatureFlag::GroupedTabs.set_enabled(true)` / `FeatureFlag::PinnedTabs.set_enabled(true)` lines dropped for the same reason. The four tests (`test_launch_config_restores_tab_groups`, `test_launch_config_restores_tab_groups_into_active_window`, `test_launch_config_restores_pinned_tab_group_into_pinned_prefix`, `test_launch_config_restore_keeps_existing_group_contiguous`) are ported verbatim otherwise and registered in both `crates/integration/src/bin/integration.rs` and `tests/integration/ui_tests.rs`. Compile-verified only (integration tests are headless-unrunnable on this machine).
- `launch_config_tests.rs`: `terminal_tab` fixture drops `llm_model_override: None` (field removed at fork baseline); `unsaveable_tab` uses `LeafContents::Welcome { startup_directory: None }` instead of `LeafContents::Notebook(NotebookPaneSnapshot::CloudNotebook { .. })` (no Notebook pane variant in this fork — `Welcome` is the existing unsaveable-pane precedent in this test file). Imports keep the fork's grouped `use crate::{...}` form without `NotebookPaneSnapshot`/`crate::drive::OpenWarpDriveObjectSettings`.
- `assertions.rs`: import conflict resolved to the fork's single-line `use warpui::{async_assert, async_assert_eq, integration::AssertionCallback};` (fork already imported `async_assert`). `assert_tab_groups` ported verbatim.
- Everything else (schema fields with `skip_serializing_if`, `TabGroupTemplate`, `From<&TabGroupSnapshot>`, `resolve_group_memberships` with its contiguity/out-of-range normalization, `move_restored_block_past_group`, mock-config field updates, `Duration`/`HashSet` usage) matches the upstream post-image; edition-2024 let-chains compile as-is.

No upstream paths omitted: every file in the commit exists in the fork.

### Rejected / not applicable (8 commits)

- `3d7895792` — **N/A**. `RunnerOS` `WINDOWS` variant: `crates/graphql`, `crates/warp_graphql_schema`, `app/src/ai/agent_sdk/runner.rs`, `app/src/ai/runner_display.rs`, `crates/warp_cli/src/runner.rs` all fork-absent.
- `82b52132c` — **N/A**. `oz secret docker-registry` help copy: `crates/managed_secrets*`, `crates/warp_cli/src/secret.rs` all fork-absent.
- `8aef29240` — **N/A**. `PlatformError` fidelity: agent_sdk driver/error-classification, `app/src/ai/agent_events/`, `app/src/server/server_api*`, `crates/graphql`, `crates/warp_server_client` all fork-absent; the `app/src/pane_group/mod_tests.rs` hunk targets `finish_seed_child_conversations_from_task`/`new_ambient_agent_task_id`/`HttpStatusError` — no anchors in the fork.
- `016de4052` — **N/A**. Sentry `warp.task_id` tag: `app/src/crash_reporting/`, `app/src/ai/agent_sdk/` fork-absent; the `app/src/lib.rs` hunk needs `LaunchMode::CommandLine`, `ambient_agent_task_id`, `server_api.set_ambient_agent_task_id` — all fork-absent.
- `af0fcc542` — **N/A**. Team-metadata refresh failure: `app/src/ai/agent_sdk/*`, `app/src/workspaces/update_manager.rs` fork-absent.
- `39a936e06` — **N/A**. MAA stream-failure classification: classification core (`agent_sdk/driver/error_classification*`, `crates/graphql`, `server_api/ai`) fork-absent; shared-file hunks lack anchors — the fork's blocklist has no `StreamFinished`/`warp_multi_agent_api`/`RequestTeamScope` server-stream arms, no `local_agent_task_sync_model.rs`, no `agent_message_bar.rs`, and no `AgentStreamFailure`/`AgentExitedShell` variants in `RenderableAIError`. Adding the variant alone would be dead code with no producer.
- `385cc203c` — **N/A**. Benchmark repository overrides: `agent_sdk/driver/environment*`, `crates/warp_cli/src/agent.rs`, `crates/warp_cli/src/lib_tests.rs` (deleted in the 2026-09-13 port) all fork-absent.
- `e3464f102a` — **N/A**. Factory MCP ambient workload token: `agent_sdk/driver/mcp_startup*`, `app/src/ai/mcp/`, `crates/warp_server_client` fork-absent; the `app/src/pane_group/mod_tests.rs` hunk needs `ServerApiProvider`/`set_ambient_workload_token_for_test` — no anchors in the fork.

## Verification

Run on `merge/upstream-2026-09-15` after the port (with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo nextest run -p warp launch_config` — pass (18/18, including the three new snapshot tests and four `resolve_group_memberships` tests).
- `cargo nextest run -p warp -E 'test(tab_group) | test(workspace::view)'` — pass (126/126, 1 leaky).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass (156/156).
- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass (compiles the touched `crates/integration` targets; integration tests are compile-verified only on this headless machine).
- `cargo fmt -- --check` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass; `target/debug/warply` Mach-O arm64 binary produced fresh this session.
- Deleted-surface scan over the port's added lines: zero hits for cloud/auth/billing/telemetry, MCP/skills, or native Linux/Windows host patterns.

`cargo clean` run after merge/tag/push.

## Notes

- The port adds new serialized launch-config fields (`tab_groups`, `group`) with `skip_serializing_if` + `default`, so existing local launch configs keep round-tripping byte-for-byte; `test_config_from_snapshot_omits_tab_groups_when_there_are_none` guards this.
