# Upstream Master Audit — 2026-10-14

Incremental audit for `21f068328..upstream/master` (`d0d50955d`), 9 commits, completing the currently fetched upstream master tip. The previous audit is `upstream-master-audit-2026-10-13.md`.

## Summary

One accepted port and eight rejections/not-applicable commits. `c79019b55` bumps the rustls cargo group for the security fix set (rustls 0.23.39 → 0.23.45, rustls-webpki 0.103.13 → 0.103.15, aws-lc-rs 1.16.2 → 1.18.1, aws-lc-sys 0.39.1 → 0.45.0 with its new `pkg-config` build dep); the fork's lockfile carried the identical pre-bump versions and was regenerated to upstream's exact resolution via targeted `cargo update --precise` pins (a three-way patch of the upstream lockfile conflicted in unrelated diverged regions, so the lockfile was regenerated per the merge guide's "regenerate dependency state from retained manifests" rule — resulting diff is content-identical to upstream: same four versions, checksums, and the `pkg-config` addition). The two ACP-harness commits (`37fcf683f` server-backed native conversation persistence, `cd328546a` CLI session-transcript upload) live entirely in the removed `app/src/ai/agent_sdk/` harness and server-API surfaces, and the Claude-transcript Windows fix (`626a4d4eb`) touches only `agent_sdk`'s fork-absent `claude_transcript.rs`. The cloud-agent startup-metadata retry (`8aa20c3ee`) is fork-absent in both its `agent_sdk` and `workspaces/update_manager` paths. Four dependency/CI bumps hit fork-absent files (private channel-config script pin, feature-flag-cleanup workflow, `warp_graphql_schema` npm lock, trunk analytics-uploader steps in the fork-owned `ci.yml`).

## Commit-by-Commit

### `c79019b55` fix: bump rustls from 0.23.39 to 0.23.45 in the cargo group across 1 directory (#16322) — accepted

Dependabot lockfile-only security bump of the rustls group. The fork's `Cargo.lock` had the exact pre-bump state (`rustls` 0.23.39, `rustls-webpki` 0.103.13, `aws-lc-rs` 1.16.2, `aws-lc-sys` 0.39.1) and retains rustls consumers (`app` and `crates/ui_components` via the workspace `rustls = "0.23.39"` caret requirement, satisfied by 0.23.45). Application notes:

- `git apply --3way` of the upstream lockfile patch conflicted in unrelated diverged lockfile regions (`walkdir` context), so the bump was regenerated with cargo's own resolution, then pinned back to upstream's exact versions (`cargo update -p rustls@0.23.39 --precise 0.23.45`, `-p aws-lc-rs@1.18.2 --precise 1.18.1`, which also settled `aws-lc-sys` at 0.45.0; the intermediate 1.18.2/0.46.0 resolution was newer than upstream and was deliberately re-pinned).
- The resulting `Cargo.lock` diff is content-identical to the upstream commit: the same four version+checksum pairs and the same `"pkg-config",` addition to `aws-lc-sys`' build dependencies. No manifest changes (upstream made none).

### `37fcf683f` ACP: persist runs as server-backed native conversations (#16393) — rejected (server-backed persistence, zero anchors)

Persists third-party-harness runs to the Warp server as native conversations and materializes them back on load. Every anchor is fork-absent: `app/src/ai/agent_sdk/**` (driver, ACP harness mapping, save/checkpoint coordinators), `app/src/server/server_api/harness_support.rs`, and `app/src/ai/blocklist/controller/external_harness_tests.rs` are removed surfaces. The two fork-present shared files have no anchors for the commit's hunks: `app/src/ai/agent/conversation.rs` gains `to_conversation_data()` building `api::ConversationData` (the server-API type; the fork has no `api::ConversationData` — only the unrelated local-persistence `AgentConversationData` — and no `Task::source_for_persistence`), and `app/src/ai/blocklist/history_model/conversation_loader.rs` edits `load_conversation_from_server`/`server_api.get_block_snapshot`, which do not exist in the fork (the only `get_block_snapshot` is an unrelated local bookmarks helper).

### `626a4d4eb` Fix Claude transcript project-directory encoding on Windows (#16428) — not applicable (fork-absent file)

Windows path-encoding fix inside `app/src/ai/agent_sdk/driver/harness/claude_transcript.rs` and its tests. No `claude_transcript*` file exists anywhere in the fork (`agent_sdk` removed at baseline); the fork's ACP client owns no Claude-transcript reader.

### `4f3360deb` Fix dev workload audience for local server overrides (#16441) — not applicable (fork-absent script)

One-line revision pin bump in `script/install_channel_config`, which cargo-installs a binary from the private `warpdotdev/warp-channel-config` repository. The script does not exist in the fork and the pinned repo is Warp-internal dev infrastructure.

### `d23c22a4c` fix: bump namespacelabs/nscloud-cache-action from 1.6.1 to 1.7.0 (#16310) — not applicable (fork-absent workflow)

Dependabot bump against `.github/workflows/feature_flag_cleanup.yml`, which does not exist in the fork (the fork has no feature-flag cleanup workflow and no `features.rs` bridge).

### `714c51f1e` fix: update busboy to resolve multipart vulnerabilities (#16421) — not applicable (fork-absent crate)

npm `busboy` override in `crates/warp_graphql_schema/package.json` + `yarn.lock`. `crates/warp_graphql_schema/` is a removed surface (cloud GraphQL schema).

### `cd328546a` ACP: upload the CLI's own session transcript (#16394) — rejected (removed harness surface, zero anchors)

Adds `agent_sdk/driver/harness/acp/cli_transcript.rs` (new) and threads CLI-owned transcript upload through the ACP harness, harness persistence, and the Claude/Codex/Gemini harnesses. All seven touched files live under the removed `app/src/ai/agent_sdk/`; no fork file references `cli_transcript`/`CliTranscript`. The upload consumer is the removed server-API persistence path from `37fcf683f`.

### `8aa20c3ee` [REMOTE-3479] Retry cloud-agent startup metadata for 30 seconds (#16438) — rejected (removed surfaces, zero anchors)

Adds a retry window for cloud-agent startup metadata in `app/src/ai/agent_sdk/mod.rs` and `app/src/workspaces/update_manager.rs` with tests in both plus a `tokio` `test-util` dev-feature addition to `app/Cargo.toml`. Both source surfaces are fork-absent (`app/src/workspaces/` does not exist; the fork's `crate::cloud_object::update_manager::UpdateManager` is the retained local-object persistence manager, an unrelated legacy-named surface), and the tokio feature has no consumer without `update_manager_tests.rs`.

### `d0d50955d` fix: bump trunk-io/analytics-uploader from 2.1.2 to 2.1.5 (#16397) — not applicable (fork-owned CI)

Repin of `trunk-io/analytics-uploader` SHAs across upstream's `ci.yml` test-result upload steps. The fork's `.github/workflows/ci.yml` is fork-owned and contains no `trunk-io` steps (verified: zero hits under `.github/`).

## Verification

- `cargo check -p warp --all-targets --message-format short`: clean (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo fmt -- --check`: clean.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156/156 passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` follows after push per the release workflow.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): 232/0/6 hits — identical to the previously documented allowances; zero hits in `Cargo.lock`, the only changed file.
