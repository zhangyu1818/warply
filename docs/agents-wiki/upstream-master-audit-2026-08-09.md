# Upstream Master Audit 2026-08-09

Range under review: `d78ced530..upstream/master` (6 commits)

Previous audited upstream tip: `d78ced530 Avoid Git probes for filesystem-only directory watchers (#14830)`

Current upstream tip detected: `7d93fa468 [QUALITY-1333] Prevent background TUI agents from stealing focus (#14829)`

Total upstream commits in this incremental range: 6

Status: triage complete. No commits ported. All six touch removed surfaces only (TUI, Warp Drive, Billing/cloud environment, cloud agent/ambient agent/agent-SDK, multi-level orchestration). Every code-bearing change was verified against the fork: the shared files that upstream edits (`pane_group/mod.rs`, `agent_conversations_model.rs`, `entry.rs`, `workspace/view.rs`, `terminal/view.rs`, `history_model.rs`, `warp_features/src/lib.rs`) do not contain the symbols the upstream hunks rely on, so the hunks have no anchor in this fork and are not applicable rather than cherry-pick candidates.

## Per-Commit Triage

### `7d93fa468` — Prevent background TUI agents from stealing focus (#14829)

Decision: **not applicable** (`crates/warp_tui/` is absent).

100% of the diff lives in `crates/warp_tui/` (`agent_block.rs`, `agent_block_tests.rs`, `handoff/block.rs`, `handoff/session.rs`, `terminal_session_view_tests.rs`, `transcript_view.rs`, `tui_ask_question_view.rs`, `tui_ask_question_view_tests.rs`, `tui_file_edits_view_tests.rs`, `tui_permission_prompt.rs`, `tui_permission_prompt_tests.rs`). The TUI app target was removed when the fork was created; `crates/warp_tui/` does not exist here, so there is nothing to port.

### `1f19fcb8c` — Scope each window's Warp Drive search corpus to that window (#14842)

Decision: **reject** (removed Warp Drive / Teams surfaces).

Touches the removed Warp Drive search corpus (`app/src/search/command_palette/warp_drive/data_source.rs`, `data_sources.rs`, `view.rs`), the removed Teams page (`app/src/settings_view/teams_page.rs`), the removed `user_workspaces` model (`app/src/workspaces/user_workspaces.rs`), and the prompt `data_source.rs`/`view.rs` whose edits in this commit are wired to the Warp Drive corpus scoping. Verified: `app/src/workspaces/user_workspaces.rs`, `app/src/settings_view/teams_page.rs`, and the `search/command_palette/warp_drive/` tree are absent in the fork; `rg "Warp Drive|warp_drive"` under `app/src/search` returns no hits. Restoring this would re-introduce the cloud Warp Drive search corpus and Teams membership model that the fork contract removes.

### `e8cb7b4e7` — Scope Billing & Usage to the selected team (#14831)

Decision: **reject** (removed Billing / GraphQL surfaces).

Touches `app/src/billing_and_usage/*` (`billing_cycle_usage_common*`, `billing_cycle_usage_rows*`, `billing_cycle_usage_section`, team-totals tests), `app/src/workspaces/{gql_convert.rs,workspace.rs}`, `crates/graphql/src/api/billing.rs`, `crates/graphql/src/api/queries/get_workspaces_metadata_for_user.rs`, and `crates/warp_graphql_schema/api/schema.graphql`. Verified: `app/src/billing_and_usage/` does not exist in the fork; the GraphQL client schema and billing API module are removed. The change is scoped entirely to team-billing accounting.

### `720871fe5` — Display the correct text in the environment selector when no environment is specified (#14837)

Decision: **not applicable** (removed cloud environment selector surface).

The single touched file is `app/src/ai/blocklist/agent_view/agent_input_footer/environment_selector.rs`, which does not exist in the fork (only `chips.rs`, `editor.rs`, `mod.rs`, `toolbar_item.rs` remain under `agent_input_footer/`). The change depends on `CloudEnvironmentCatalog`, `EnvironmentSelector`, `EnvironmentSelectorTarget::{CloudPane, Handoff}`, and `is_configuring_ambient_agent()` — verified all absent (`rg "CloudEnvironmentCatalog|EnvironmentSelector|cloud_environment"` returns nothing under `app crates`). The commit's self-described trigger is "Warp on Web" footer behavior for cloud agent runs; both the web target and the cloud environment catalog are removed.

### `5de276578` — Keep a failed cloud agent's session open for debugging and make it attachable (#14561)

Decision: **not applicable** (removed cloud agent / ambient agent / agent-SDK surfaces).

The commit keeps a failed cloud agent's sandboxed shared-session alive for a debug window (`--idle-on-fail` / `OZ_IDLE_ON_FAIL`) and makes a retained session writable again. The vast majority of the diff lives in removed modules: `app/src/ai/agent_sdk/` (driver, harness, terminal, tests), `app/src/ai/ambient_agents/` (task, tests), `app/src/ai/blocklist/local_agent_task_sync_model*`, `app/src/server/server_api/ai.rs`, `app/src/terminal/view/ambient_agent/mod.rs`, `app/src/terminal/view/shared_session/*`, `crates/graphql/src/api/mutations/update_agent_task.rs`, `crates/warp_cli/src/agent.rs` — all verified absent in the fork.

The commit also edits five files that **do** exist in the fork; each edit was checked and is inert here because the symbol it targets is absent:

- `app/src/ai/agent_conversations_model/entry.rs` — upstream swaps `active_execution_session_id()` for `active_live_session_state()` / `AmbientAgentLiveSessionState::Attachable`. Fork's `entry.rs` has neither symbol; it only retains local `AgentRunDisplayStatus`/`from_conversation_status` logic. No anchor.
- `app/src/pane_group/mod.rs` — upstream edits `attach_execution_session_to_ambient_pane`, adds a `is_conversation_transcript_viewer()` refusal, and calls `prepare_for_live_session_reattach`. Fork's `pane_group/mod.rs` (5202 lines vs upstream's ~7000+) has no `attach_execution_session*` method, no `ambient_agent_view_model` field access, and `prepare_for_live_session_reattach` does not exist anywhere. No anchor.
- `app/src/terminal/view.rs` — upstream adds `SharedSessionViewerInput`/`prepare_for_live_session_reattach`/`supports_live_session` handling. Verified all three absent in fork's `view.rs`. No anchor.
- `app/src/workspace/view.rs` — upstream's ambient/live-session fallback path. Verified `rg "ambient|live_session|attach_execution"` returns no hits in fork's `workspace/view.rs`. No anchor.
- `app/src/pane_group/mod_tests.rs` — upstream tests for the above; the production code under test does not exist here.

Restoring any of this would re-introduce the cloud agent shared-session / ambient-agent attach machinery that the fork contract removes.

### `306257bfe` — Multi-level orchestration: depth-capable client core and drill-down UI (#14762)

Decision: **not applicable** (removed cloud orchestration surfaces).

The commit adds client-side multi-level (configurable-depth) agent orchestration — child conversations auto-executing their own `run_agents`, a drill-down orchestration pill bar, breadcrumbs, and a recursive topology walker. Most of the diff lives in removed modules: `orchestration_topology*`, `orchestration_event_streamer*`, `agent_view/orchestration_pill_bar*`, `agent_view/orchestration_conversation_links.rs`, `action_model/execute/run_agents*`, `inline_action/run_agents_card_view.rs`, `telemetry.rs`, plus `crates/warp_tui/src/terminal_session_view.rs` (TUI absent).

The commit edits five files that **do** exist in the fork; each edit is inert here:

- `app/src/ai/agent_conversations_model.rs` — upstream imports `orchestration_topology::orchestration_aware_conversation_status` and rolls an orchestration subtree into the root card status via `AmbientAgentTaskState`. Fork has neither `orchestration_topology` nor `AmbientAgentTaskState`/`from_task_state`. No anchor.
- `app/src/ai/agent_conversations_model/entry.rs` — upstream adds the same `orchestration_aware_conversation_status` import/call into `conversation_display_status`. Fork's `entry.rs` has no such symbol and no `AmbientAgentTask` import. No anchor.
- `app/src/ai/blocklist/history_model.rs` — upstream only adds a doc comment to `resolved_parent_conversation_id_from_refs`, which does not exist in the fork (`rg resolved_parent_conversation_id_from_refs|conversation_id_for_agent_id` → no hits). No anchor.
- `app/src/terminal/view.rs` — upstream adds orchestration drill-down navigation/anchor handling depending on the removed `orchestration_*` modules. No anchor.
- `crates/warp_features/src/lib.rs` — upstream adds the `MultiLevelOrchestration` flag and registers it in `DOGFOOD_FLAGS`. Verified fork's `lib.rs` has no `Orchestration`/`WaitForEvents`/`MultiLevel` flags at all (the orchestration feature-flag surface was cleaned out); the surrounding `WaitForEventsParentRegistration` anchor that upstream inserts next to is also absent. No anchor.

## Verification

No code was ported in this cycle, so the fork tree is unchanged and the build state from the 2026-08-08 merge carries over. Deleted-surface scans were re-run to confirm no drift:

- `rg "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment"` — only allowed hits (`Arc::upgrade()`/`WeakViewHandle::upgrade()` weak-handle calls, `toolchain upgrades` comment).
- `rg "mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill"` — no hits.
- `rg "target_os = \"linux\"|target_os = \"windows\"|cfg\(windows\)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb"` — only the retained `ConPTY` explanatory comment in `zsh_body.sh` and retained SSH `ForwardX11=no` config strings (all allowed).

## Notes

- The upstream `agent_conversations_model.rs` / `entry.rs` shared-file edits in both `5de276578` and `306257bfe` are the kind of "shared file, removed symbol" pattern called out in `fork-contract.md` under Ambiguous Names. They were not rejected on the filename alone; each was checked for a viable anchor symbol before classifying as not applicable.
- No deferred ports this cycle. The `da4da09f8` Agent Mode Cmd-Up/Cmd-Down prompt navigation deferred from the 2026-08-08 audit remains deferred pending either a follow-up retained change on those navigation-cursor types or resolution of the keymap-context `Terminal` gate that the upstream PR reports as inert on its test platform.
