# Upstream Master Audit — 2026-10-11

Incremental zero-port audit for `721449fde..upstream/master` (`c1ec372cb`), 2 commits, completing the currently fetched upstream master tip. The previous audit is `upstream-master-audit-2026-10-10.md`.

## Summary

Both commits extend removed cloud workstreams. `2055f0f48` continues the Factory cloud-execution stack (deferred repository checkouts via `environment-checkout`, Factory-scoped git credentials, a new `FactoryDeferredRepositories` dogfood flag, and a bundled `factory-deferred-repositories` skill); `c1ec372cb` relaxes the protected-path MCP-config write guard for sandboxed cloud agents. Every primary surface is fork-absent (`app/src/ai/agent_sdk/`, `app/src/server/server_api/`, `app/src/ai/skills/`, `crates/graphql`, `crates/warp_graphql_schema`, `crates/warp_cli/src/environment_checkout.rs`, `resources/bundled/skills/`) or anchors on symbols this fork never received (`ProtectedPath`, `check_protected_write_paths`, `mcp_provider_from_file_path`, `AppExecutionMode::is_sandboxed`, `CloudAgentExecutionConfig`/`NamedAgents`/`GitCredentialRefresh` flag lineages, the `enabled_features()` body in `app/src/features.rs`). No removed product surface was restored and no code was ported.

## Commit-by-Commit

### `2055f0f48` factory: use environment-checkout for live deferred repositories (REMOTE-2960) (#15386) — rejected (removed surface)

Paired task/execution-ID cloud launches request live Factory eager/deferred repositories over GraphQL (`supportsDeferredRepositories`/`useFactoryRepositories`), bind typed `environment-checkout` request identities to a workspace root and the running executable, publish the canonical bundled skill to Oz/Claude Code/Codex/Gemini over PTY and ACP, and add an opt-in `--fail-if-target-exists` strict-target checkout mode plus a `FactoryDeferredRepositories` dogfood-only capability flag. All implementation paths are fork-absent: the `agent_sdk` driver/execution-config/harness/skill-publish modules, `server_api/ai.rs`, `crates/graphql` execution-config and task-git-credentials queries, `crates/warp_graphql_schema`, `crates/warp_cli/src/environment_checkout.rs`, `app/src/ai/skills/**`, `app/src/pane_group/pane/local_harness_launch.rs`, and `resources/bundled/skills/factory-deferred-repositories/SKILL.md` (bundled-skills directories must remain absent). The fork-present hunks are anchorless or would land dead:

- `app/Cargo.toml` adds a `factory_deferred_repositories = []` cargo feature whose only consumers (the `FactoryDeferredRepositories` enum variant and `enabled_features()` cfg arm) do not exist here — a permanently dead feature.
- `app/src/features.rs` in the fork is the one-line `pub use warp_core::features::*;` re-export; the upstream hunk edits the `enabled_features()` body with cfg-gated flags, which this fork never had (flags wire via `app/src/lib.rs` + Cargo defaults).
- `crates/warp_features/src/lib.rs` exists here but predates the upstream flag lineage — `CloudAgentExecutionConfig`, `GitMirrorCache`, `GitCredentialRefresh`, and `NamedAgents` anchors are all absent, and `crates/warp_features/src/features_tests.rs` is fork-absent.

Same classification as the `b61e9ca31` (execution-config query) and `b113a6c95` (git-mirror checkout cache) commits rejected on 2026-10-10: the runtime owner end-to-end is the removed cloud Factory/agent-sdk driver.

### `c1ec372cb` [REMOTE-3445] Allow protected-path file writes in cloud agents (#16402) — rejected (removed surface, anchor absent)

Gates the protected-path file-write denial on `!AppExecutionMode::as_ref(ctx).is_sandboxed()`, so cloud sandboxed agents may write MCP config files under profile permissions while local sessions still require explicit approval; rewords the `ProtectedPath` doc comments and adds four tests (`cloud_mcp_config_writes_follow_profile_permissions`, `cloud_auto_approve_allows_mcp_config_writes`, `local_app_auto_approve_requires_mcp_config_approval`, `local_cli_auto_approve_requires_mcp_config_approval`) built on `initialize_permissions_test_sandboxed`. The fork has none of the stack this delta modifies: `app/src/ai/blocklist/permissions.rs` here has no `ProtectedPath` or `Inconclusive` variant in `FileWritePermissionDeniedReason`, no `check_protected_write_paths`, no `mcp_provider_from_file_path` (the guard protects Warp's app-managed MCP config files, a removed area), no `permissions_tests.rs`, and no sandboxed execution mode (`is_sandboxed` absent; this fork's `ExecutionMode` covers local launch modes only). For an all-local fork the gated predicate would be constant-true, so even a hypothetical port of the call-site hunk would be a no-op against an absent guard.

## Verification

Zero-port audit: no source changes, so verification re-confirms the unchanged tree.

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: clean.
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` follows after push per the release workflow.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): no new hits beyond the previously documented allowances (232/0/6 hits, all previously recorded — the six platform hits remain the retained SSH `ForwardX11` options, zsh bootstrap ConPTY comments, and `local_or_remote_path_tests.rs` `cfg(windows)` remote-path tests).
