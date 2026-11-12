# Upstream Master Audit — 2026-11-12

Incremental audit for `619c82aa7..upstream/master` (`0515627a4`), 10 commits. The previous audit is `upstream-master-audit-2026-11-11.md`.

## Summary

Zero-port audit. Every commit in the range lives entirely on fork-absent surfaces — the removed `agent_sdk` harness/prewarm/git-credential stack, the removed `server_api`/billing/usage/Teams/GraphQL clients, the absent `warp_tui` crate, upstream-internal CI workflows, and the removed app-managed skills flag plumbing. Each fork-present shared-file hunk (`app/src/lib.rs`, `app/src/lib_tests.rs`, `app/src/tracing.rs`) was inspected individually and is anchored on symbols the fork does not carry (`TuiEntryPoint`, `StartupUserAuthentication`, `crate::ai::agent_sdk`, `crate::auth::credentials`, `tracing::native`, `AppManagedSecretsClient` — all verified absent by search). No removed product surface was reintroduced.

## Commit-by-Commit

### `d7fc3edd4` Add ACP harness stage, turn, steer, and cancel telemetry (#16776) — rejected

Adds stage/turn/steer/cancel telemetry to the upstream `agent_sdk` ACP harness (the `app/src/ai/agent_sdk/driver/harness/acp/**` stack rejected in the 2026-10-10 and 2026-11-11 audits — a different codebase from this fork's `app/src/ai/acp/` client), plus `setup_observability` expansion and `app/src/server/server_api/ai.rs` request plumbing. All 16 of 18 files sit under fork-absent `app/src/ai/agent_sdk/` and `app/src/server/`; the remaining two are `app/src/pane_group/pane/local_harness_launch.rs` (fork-absent — the fork's `app/src/pane_group/pane/` has no harness launch pane) and a one-line `harness::telemetry` re-export. The telemetry runtime owner is the removed cloud orchestration/observability path; the fork's ACP client emits no harness telemetry.

### `2b2ed236f` Support Scale shared monthly usage in Warp (#16758) — rejected

Scale-plan shared monthly usage: new `included_usage_pool` billing model under `app/src/settings_view/billing_and_usage/`, `app/src/ai/request_usage_model*` usage accounting, `app/src/workspaces/**` team scoping, `teams_page`/`cloud_agent_capacity_modal` UI, and `crates/graphql`/`crates/warp_graphql_schema` schema/query additions. Every path is fork-absent (verified by path listing: no `request_usage_model.rs`, no `billing_and_usage/`, no `workspaces/`, no `cloud_agent_capacity_modal/`, no `crates/graphql`, no `crates/warp_graphql_schema`). Billing/usage/Teams is a removed product area; nothing is separable into local persistence.

### `4fe3af1be` Change the name of the secret used for repo sync's Warp API key. (#16797) — not applicable

Two-line secret rename in `.github/workflows/repo-sync.yml`, upstream's internal repo-sync workflow. The file is fork-absent; this fork's CI is fork-owned and has no repo-sync job.

### `993226c2e` Disable nested sandboxing for Codex ACP cloud runs (#16799) — rejected

Seven-line guard in `app/src/ai/agent_sdk/driver/harness/mod.rs` disabling nested sandboxing for Codex ACP *cloud* runs. The file is fork-absent and the behavior targets cloud-run execution, which this fork does not own.

### `20ec69711` refresh github credentials only right before it will expire (#16803) — rejected

Expiry-driven refresh for restricted GitHub bot credentials in `app/src/ai/agent_sdk/driver/git_credentials{,/restricted.rs,/restricted_tests.rs}`. The entire `git_credentials` stack was rejected with `651109041`/`619c82aa7` in the 2026-11-11 audit; all three files are fork-absent (verified).

### `bca1d4416` feat(agent): persist setup variables through .warp.vars (#16749) — rejected

Persists Oz agent setup variables through a `.warp.vars` file: new `app/src/ai/agent_sdk/driver/setup_vars.rs` (+592-line test file) with edits to `agent_sdk/driver.rs`, `driver/agent_setup.rs`, `driver/prewarmed.rs`, and `driver/terminal.rs`. All six files are under the removed `app/src/ai/agent_sdk/` surface (verified absent). The runtime owner is the cloud agent setup/prewarm machinery; the fork's ACP client has no `agent_setup`/`prewarmed` drivers (the only `agent_setup*` hits in the fork are the unrelated local `agent_setup_speedbump_banner` AgentView banner state in `app/src/terminal/view.rs`).

### `11dd3893d` Give ServerApi explicit Send/Sync impls to cut repeated auto-trait checks (#16787) — not applicable

Compile-time optimization adding `#[non_structural_derive(Send, Sync)]` impls to `ServerApi` in `app/src/server/server_api.rs`, plus the `non_structural_derive` 0.1.2 dependency. `ServerApi` and `app/src/server/` are fork-absent (verified by search), and the new dependency would have no consumer in the fork manifest, so neither the impl nor the dependency applies.

### `b826d1b4d` Added a tui command to contact the sever to mint a jwt token (#16726) — rejected

TUI `get-git-access-jwt` command that mints a short-lived server JWT for warp-hosted factory access, cached in secure storage. Core paths are fork-absent: `crates/warp_tui/**` (crate removed), `app/src/tui_export.rs`, `app/src/server/server_api/factory.rs`, `app/src/ai/agent_sdk/mod.rs`. The shared-file hunks were inspected individually: `app/src/lib.rs` restructures `startup_authentication` dispatch around `LaunchMode::Tui { entrypoint: TuiEntryPoint::CliCommand { .. } }` (both symbols fork-absent; the fork's `LaunchMode` has no TUI or CommandLine variants), and `app/src/lib_tests.rs` imports `crate::ai::agent_sdk::authenticate_then_execute` and `crate::auth::credentials::Credentials` (both fork-absent). Server JWT minting for warp-hosted factory auth is exactly the removed server/auth dependency chain.

### `de01cc6c5` feat(prewarm): export standby traces and switch to the run token at assignment (#16795) — rejected

Sandbox-scoped OTLP tracing for prewarmed cloud agents: standby span export, run-token swap at assignment, refresh generations. The `app/src/ai/agent_sdk/driver/prewarmed*` files are fork-absent, as are `app/src/tracing/cloud_agent_auth*` and `app/src/tracing/native.rs` (the fork's `app/src/tracing.rs` is a 13-line `NoSubscriber` stub with no `native` module and no `start_auth_refresh`). The `tracing.rs` hunk adds `assign_run_credentials` anchored on `crate::server::server_api::managed_secrets::AppManagedSecretsClient` — fork-absent. External telemetry and managed secrets are removed systems.

### `0515627a4` Enable Factory skill references for all release channels (#16805) — not applicable

Moves the `FactorySkillReferences` flag from Dogfood into the shared `RELEASE_FLAGS` list. The flag does not exist in the fork's `crates/warp_features/src/lib.rs` (verified by search); the underlying Factory cloud-run skill-reference feature was rejected on 2026-11-05, and app-managed/bundled skills remain a removed surface.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`. No source files changed in this audit (documentation only), so the checks re-confirm the existing main tree:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean`.
- Restored-surface scans re-run: the MCP/skills pattern scan returns zero hits, the platform scan returns the same six known retained occurrences (SSH `ForwardX11=no` ×2, retained ConPTY comments in `zsh_body.sh` ×2, `#[cfg(windows)]`-gated `warp_util` tests ×2), and the cloud/auth/billing/telemetry scan returns only the pre-existing allowed hits — unchanged from the 2026-11-11 baseline.

## Tag

`v2026.11.12` on `main` (zero-port audit; docs commit per the 2026-11-07 precedent, no merge branch).
