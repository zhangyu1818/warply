# Upstream Master Audit 2026-09-21

Range under review: `03832b3a8..upstream/master` (2 commits)

Previous audited upstream tip: `03832b3a8 [Chat][Attribution] Attribute local, shared-session, and restored UserQuery inputs (#15947)`

Current upstream tip detected: `37a7ebeff Include third-party harness output in failure details (#15991)`

Total upstream commits in this incremental range: 2

Status: triage complete. **Zero-port audit** — every commit is rejected or not applicable. No fork code changes.

## Per-Commit Triage

### `c1a27a6a2` — Fix native workspace discovery in the client (#16083)

Decision: **reject**. Native workspace discovery in the client's Teams page: fetching discoverable native workspaces alongside legacy teams via the Warp GraphQL server API, a `Continue` flow for workspaces with open teams, a workspace-scoped team list with Back navigation, and a new `join_workspace_from_discovery` mutation for workspace/team joins. Teams, workspace discovery, and the Warp server API are removed areas with no local or ACP data flow.

Every touched path is fork-absent (verified by listing):

- `app/src/server/server_api/team.rs`: no `app/src/server/` in the fork.
- `app/src/settings_view/teams_page.rs` / `teams_page_tests.rs`: no Teams settings page.
- `app/src/workspaces/gql_convert.rs`, `team.rs`, `user_workspaces/mod.rs`, `user_workspaces_tests.rs`: no `app/src/workspaces/` directory.
- `crates/graphql/src/api/mutations/join_workspace_from_discovery.rs` (new), `mutations/mod.rs`, `queries/get_discoverable_teams.rs`, `api/user.rs`, `crates/warp_graphql_schema/api/client-schema.ts`, `schema.graphql`: both GraphQL crates removed at baseline.
- `app/src/workspace/view_tests.rs` (16-line hunk): the fork's `app/src/workspace/` has `view_test.rs`, not upstream's `view_tests.rs`; the hunk only injects `MockTeamClient`/`MockWorkspaceClient` into a `UserWorkspaces` singleton and imports `crate::server::*` singletons (`ServerApiProvider`, `SyncQueue`, telemetry context provider) — no `UserWorkspaces`/`TeamClient`/`ServerApiProvider` anchors exist anywhere in the fork's `app/src/workspace/` (verified by search).

### `37a7ebeff` — Include third-party harness output in failure details (#15991)

Decision: **N/A (reject)**. Extends the old Warp Agent SDK driver so a failed third-party harness exit carries the block's terminal output in the structured `PlatformError` task-status detail sent to the Warp server: `AgentDriverError::HarnessCommandFailed` gains an `output` field, `fetch_harness_failure_output` pulls `block_output_plaintext`, and `harness::prepare_harness_failure_output` applies an unconditional `redact_secrets_in_string` pass (from `crate::server::telemetry::secret_redaction`) plus head/tail truncation within the server's 4 KiB limit.

All four files live under `app/src/ai/agent_sdk/` (`driver.rs`, `driver/error_classification.rs`, `driver/harness/mod.rs`, `driver/harness/mod_tests.rs`), removed at baseline with the old Warp Agent SDK; imports also require the fork-absent `crate::server::telemetry` and `crate::server::server_api::harness_support` modules. No `PlatformError`, harness driver, or failure-detail reporting anchor exists in the fork (verified by search); the fork's ACP client owns agent execution and has no server task-status reporting path. The redaction utility itself already exists locally in the fork's retained secret-redaction code, so nothing is separable.

## Verification

Zero ports; verification confirms the released tree is healthy. Run on `merge/upstream-2026-09-21` (= `main` + this audit doc, with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.

Environment note: a CommandLineTools 27.0 SDK installed on 2026-09-20 broke the default SDK resolution — `/usr/bin/xcrun --show-sdk-path` returns `MacOSX27.0.sdk` even with `DEVELOPER_DIR` pointing at Xcode 26.6, and its `libSystem.tbd` uses an `arm64e.x1-macos` architecture entry the Xcode 26.6 linker's tapi cannot parse (`aws-lc-sys`'s `memcmp_invalid_stripped_check` was the first C-linking casualty). All verification commands were run with `SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk` to keep clang, ld, and SDK on Xcode 26.6. This is a toolchain prereq, not a code regression.

Deleted-surface scan not applicable (no code changes); the standing scans from the 2026-09-19 audit remain valid for this tree.

`cargo clean` run after merge/tag/push.

## Notes

- Upstream's Teams/workspace-discovery surface continues to churn in `app/src/server/`, `app/src/workspaces/`, and `crates/graphql/` with no fork anchors; those paths remain blanket-absent and each new commit still gets the per-file existence check recorded above.
- The `PlatformError` structured task-status input stack (#15975 → #15991) is server-reporting for the removed agent-SDK harness path. If a future upstream change routes equivalent failure-detail/redaction behavior through a local or ACP-visible boundary, re-triage that stack as a unit.
