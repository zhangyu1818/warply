# Upstream Master Audit — 2026-11-22

Incremental audit for `280d5eaed..upstream/master` (`f8223cef2`, 3 commits). The previous audit is `upstream-master-audit-2026-11-21.md`.

## Summary

Two-commit security/toolchain port. `f2ca97abb` bumps the direct zip dependency to 9 (CVE-2026-115905/06/09 on the locked zip 2.4.2) and `f8223cef2` moves the workspace to Rust 1.94.0 and Candle 0.11.1, which unifies the lockfile on zip 9.8.1 and deletes the zip 7.2.0 copy pulled in by candle-core 0.9.2 (CVE-2026-115905/06/07/08/09). Both are dependency-security and workspace-migration scope per the merge guide, applied source-first. `773752750` (GitHub MCP integration identity enum for the Warp server protocol) is rejected wholesale: every touched path is fork-absent and its runtime owner is the removed managed-MCP/server-API surface.

Final lock state: single `zip 9.8.1`, `candle-{core,nn,onnx} 0.11.1`, `zip 7.2.0` gone.

## Commit-by-Commit

### `f2ca97abb` Bump zip to 9 (GHSA-m665-v7pm-7grj, GHSA-q7v9-x93c-xqv3, GHSA-8hxf-qr8p-62x4) (#16928) — adapted

Dependency security update (accept class). Ported:

- `crates/node_runtime/Cargo.toml` — exact upstream hunk: `zip = "2.1"` → `zip = { version = "9.0.5", default-features = false, features = [...] }` without bzip2. Conflict resolved keeping the fork-only `warp_core` dependency line; the upstream hunk's trailing `[build-dependencies]` context lines are not fork sections and were not added.
- `crates/node_runtime/src/lib.rs` — exact upstream hunk: `ZipFile::name()` now returns `Result`, wrapped with `.context("Failed to read zip entry name")?`.
- `Cargo.lock` — regenerated with `cargo update -p zip:2.4.2 --precise 9.1.2` to match the upstream lock at this commit.

Omitted fork-absent paths: `app/src/ai/agent_sdk/driver/mcp_startup_tests.rs` (agent_sdk removed), `crates/mcp/src/runtime_tests.rs` (crate removed), `crates/warp_logging/Cargo.toml` + `crates/warp_logging/src/native_tests.rs` (the fork's warp_logging is trimmed and has no zip dependency, no log-panics, no native tests). Verified `node_runtime/src/lib.rs` is the fork's only zip API call site.

### `773752750` Send the GitHub MCP integration identity enum (#16846) — rejected

Sends `MCPIntegration::Github` for the reserved GitHub `warp_id` through the `warp_multi_agent_api` protocol when converting MCP context servers for the Warp server, plus a protocol rev pin (`Cargo.toml`) and `app/src/ai/agent/api/convert_to.rs` + `convert_to_tests.rs`. All paths are fork-absent (verified): the fork's workspace `Cargo.toml` has no `warp_multi_agent_api` dependency, and `app/src/ai/agent/api/convert_to*.rs` does not exist. Runtime ownership is the removed managed-MCP / Warp server API surface (the fork has no `crates/mcp` and no `warp_server_client`), so nothing was ported.

### `f8223cef2` Bump Rust 1.94 and Candle 0.11.1 to drop zip 7.2.0 (#16946) — adapted (workspace migration)

Toolchain/dependency migration, reviewed independently of product triage per the workspace-migration rule. Ported:

- `rust-toolchain.toml` — exact upstream hunk: `1.92.0` → `1.94.0`.
- `crates/input_classifier/Cargo.toml` — exact upstream hunk: `candle-{core,nn,onnx}` `0.9.2` → `0.11.1`. No source changes were needed upstream and none here: `cargo check -p input_classifier --features nld_classifier_v3 --all-targets` passes on the new candle.
- `app/src/ai/predict/next_command_model.rs`, `app/src/terminal/block_list_element.rs` — exact upstream clippy 1.94 `if let` hunks (replacing `is_some()` + `unwrap`/`expect`).
- `crates/warpui_core/src/elements/hoverable.rs` — exact upstream hunk from `elements/gui/hoverable.rs`, path-adapted to the fork's pre-gui-split layout (verified identical preimage code).
- `app/src/terminal/model/selection.rs` — exact upstream hunk from `crates/warp_terminal/src/model/selection.rs`, path-adapted (the fork's `warp_terminal` crate does not contain Selection; the model lives in the app path with identical preimage code).
- `Cargo.lock` — regenerated with `cargo update -p zip:9.1.2 --precise 9.8.1`, which also resolved candle to 0.11.1 and removed zip 7.2.0 (the candle 0.9.2 path) and a duplicate typed-path 0.12.2 entry.

Omitted fork-absent paths: `app/src/ai/blocklist/usage/conversation_usage_view.rs` (usage tracking removed), `app/src/terminal/shared_session/presence_manager.rs` (shared-session removed), `crates/warp_server_client/src/base_client.rs` (`#[allow(clippy::result_large_err)]` on the IAP eventsource wrapper; crate removed).

## Restored-Surface Scans

Scan 1: 234 hits, scan 2: 7 hits, scan 3: 16 hits — identical to the 2026-11-20/21 documented baselines; no new hits. The full `main..merge/upstream-2026-11-22` diff was additionally grepped for sentry/telemetry/graphql/mcp/auth/billing keywords with zero matches.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`, on the newly installed 1.94.0 toolchain:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p input_classifier --features nld_classifier_v3 --all-targets --message-format short` — passed.
- `cargo check -p warp --all-targets --message-format short` — passed (pre-existing unused match-binding warnings in `app/src/terminal/input/slash_commands/mod.rs`, untouched by this merge, no errors).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed, 3075 skipped.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo build --workspace --all-targets --message-format short` (workspace-migration gate) — passed in 1m19s, followed immediately by `cargo clean` (23.2 GiB removed).

## Tag

`v2026.11.22` on `main` (two port commits on `merge/upstream-2026-11-22`, merged with `--no-ff`).
