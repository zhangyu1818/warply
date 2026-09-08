# Upstream Master Audit 2026-09-08

Range under review: `4b7798fca..upstream/master` (5 commits)

Previous audited upstream tip: `4b7798fca feat(agent-sdk): authenticate Azure CLI with Entra token (#15840)`

Current upstream tip detected: `3cb7b96ca [REV-2383] Add team selection to API key commands (#15806)`

Total upstream commits in this incremental range: 5

Status: triage complete. No commits ported. All five commits are verticals of the REV-2383 "Oz CLI team selection" multi-team migration: they scope Oz CLI agent runs, model listing, environment selection, agent catalogs, and API-key commands to the selected team, resolving a `TeamScope`/`RequestTeamScope` from `UserWorkspaces` and attaching `X-Warp-Team-Uid` at the Warp server transport boundary. Every commit's core requires the removed Teams/workspace-discovery surface and the removed Warp server API client, and every shared-file hunk was verified to have no anchor symbol in the fork.

## Per-Commit Triage

### `37bea3895` — Scope local & cloud agent runs to the selected team (#15808)

Decision: **reject** (multi-team/Teams + Warp server transport + removed agent_sdk/ambient/orchestration stack).

Adds `--team` to `oz agent run`, requires a resolved request scope for local task creation, attaches the team header only at transport, carries initiating scope through headless/GUI/TUI launch paths, and validates explicit models against the selected team's catalog. Of 42 touched paths, the bulk lives in removed surfaces: `app/src/ai/agent_sdk/**`, `app/src/ai/ambient_agents/**`, `app/src/ai/orchestration/**`, `app/src/server/server_api*` (new `RequestTeamScope` transport plumbing), `app/src/terminal/view/ambient_agent/**`, `app/src/workspaces/user_workspaces/**`, `crates/warp_cli/src/agent.rs`, `crates/warp_server_client/**`, and `crates/warp_tui/**` (crate absent).

Shared-file hunks are pure team-scope plumbing with no fork anchors:

- `app/src/ai/blocklist/child_agent_launch.rs` and `handoff/pipeline.rs` — files absent; they thread `RequestTeamScope` into `prepare_local_oz_child_launch`/`SpawnAgentRequest.team`/cloud-handoff spawn calls that do not exist here (`rg "RequestTeamScope|prepare_local_oz_child_launch|begin_local_to_cloud_handoff"` returns 0 hits).
- `app/src/pane_group/pane/local_harness_launch.rs` — absent (local harness launch stack removed).
- `app/src/pane_group/pane/terminal_pane.rs` — hunks add `team_context_for_operation`/`RequestTeamScope::from_scope` parameters to `launch_local_no_harness_child`/`launch_local_harness_child`/`launch_remote_child`, all part of the removed agent-run launch paths; `team_context_for_operation` has 0 hits in the fork.
- `app/src/root_view.rs`, `app/src/uri/mod.rs`, `app/src/workspace/view.rs` — add `initial_team_uid` to `NewWorkspaceSource::Session` and a `team_uid()` early-return; the fork's `NewWorkspaceSource::Session` has only `options`, `root_view.rs` has no `team_uid`, and `initial_team_uid` has 0 hits.
- `app/src/terminal/view_tests.rs`, `queued_prompts_tests.rs`, `shared_session/view_impl_tests.rs` — test adaptations calling `request_team_scope()`/`TeamlessScopeForTest`/`spawn_agent_with_request(.., team_scope, ..)` on the ambient-agent view model; `app/src/terminal/view/ambient_agent/` is absent and `ambient_agent_view_model` has 0 hits.
- `app/src/tui_export.rs` — file absent (no `warp_tui`).

### `51242b5f0` — [REV-2383] Scope model listing to selected team (#15804)

Decision: **reject** (Teams-scoped server workspace-metadata model catalog).

`oz model list --team` resolves `TeamScopeForCli` and reads the selected team's model catalog from refreshed workspace metadata through the TeamScope-based `LLMPreferences` accessor. All five touched paths are absent: `app/src/ai/agent_sdk/mod.rs`/`model.rs` (removed SDK; `ModelCommand::List(_)` parser arms), `app/src/workspaces/user_workspaces/user_workspaces_tests.rs` (Teams surface removed), and `crates/warp_cli/src/model.rs`/`lib_tests.rs` (the fork's `warp_cli` keeps only completions/config/json-filter/lib, no Oz command tree). The fork also removed server-fetched model catalogs entirely; `LLMPreferences` here is local static metadata.

### `6c7975d61` — [REV-2383] Scope Oz environment selection to a team (#15805)

Decision: **reject** (cloud environments removed).

Adds `TeamSelection` to `oz environment list`, limiting results to personal plus team-owned cloud environments via `EnvironmentChoice::resolve_for_create`. Every touched path is absent: `app/src/ai/agent_sdk/{ambient,common,common_tests,environment,integration,mod,schedule}.rs` (cloud environment/Oz surface removed) and `crates/warp_cli/src/environment.rs` (no Oz CLI in fork).

### `f32b33be8` — [REV-2383] Scope agent catalogs to active team (#15807)

Decision: **reject** (agent catalogs + Warp server REST/GraphQL transport).

Scopes `oz agent list/create/skills` and `oz memory-store list` catalogs to the resolved team, sending `X-Warp-Team-Uid` at the public API transport. All paths absent: `app/src/ai/agent_sdk/{agent_config,agent_management,agent_management_tests,common,memory_store,mod}.rs`, `app/src/server/server_api*.rs`, `app/src/settings_view/platform/create_api_key_modal.rs` (API-key settings surface removed), `crates/warp_cli/src/{agent,memory_store}.rs`, `crates/warp_server_client/**`.

### `3cb7b96ca` — [REV-2383] Add team selection to API key commands (#15806)

Decision: **reject** (Warp account API keys + server companion dependency).

Adds `--team[=UID]` to `oz api-key list/create/expire`, lowers `RequestTeamScope` to `X-Warp-Team-Uid` in `AuthClient::list_api_keys`, and filters owner types in the settings platform page. All paths absent: `app/src/ai/agent_sdk/api_key*.rs`, `app/src/settings_view/platform_page.rs` (file removed with the API-key platform page; `rg "list_api_keys|create_api_key" app crates` returns 0 hits), `crates/warp_cli/src/api_key*.rs`, `crates/warp_server_client/**`. The commit description also declares a hard dependency on warp-server#16819, independently placing it in the rejected server-API category.

## Verification

No code was ported in this cycle, so the fork tree is unchanged and the build state from the 2026-09-06 merge (`cargo build -p warp --all-targets` clean, then `cargo clean`) carries over. `cargo fmt -- --check` passes. Deleted-surface scans were re-run with only allowed hits:

- Cloud/auth/billing/telemetry pattern — weak-handle `upgrade()` calls, doc-comment wording (settings macros, sync/anchor comments), and the pre-existing `Warp Drive` doc-comment in `crates/warp_util/src/sync.rs`.
- MCP/skills pattern — no hits.
- Platform pattern — retained `ForwardX11=no` SSH config strings, the retained ConPTY explanatory comment in `zsh_body.sh`, and `#[cfg(windows)]`-gated tests in `warp_util`.

## Notes

- This is the fourth zero-port cycle after 2026-08-09, 2026-08-29, and 2026-09-07. REV-2383 is a continuation of the multi-team stack already rejected in bulk on 2026-08-26/27; the recurring rule held again: the `UserWorkspaces`/`TeamScope` resolution machinery and the `X-Warp-Team-Uid` server transport do not exist in this fork, so every hunk — including shared-file hunks in retained trees like `root_view.rs`, `uri/mod.rs`, `workspace/view.rs`, and `terminal_pane.rs` — is team-scope plumbing without an anchor symbol.
- Deferred ports unchanged: `9921300b7` (Ctrl-C harness cancel, waiting on upstream local-keystroke wiring) and the mermaid toggle from `#10431` architecture work.
