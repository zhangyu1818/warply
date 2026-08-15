# Upstream Master Audit 2026-08-15 — Incremental Fixes + File Explorer Chip

## Scope

- Current fork before this audit: `34e3a91e7` (`main`, `v2026.08.14`).
- Upstream source reviewed: `c9e562294..upstream/master` (15 commits, tip `d15645c77`).

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `59bda8db0` | Bump command-signatures to 15debaeb (kubectl) | **Accept (superseded)** | Intermediate bump; final state applied via `5bd9b8e15`. |
| `90fdba190` | fix(teams): route admin panel button for workspace admins | **Reject** | Teams/billing surfaces (`teams_page.rs`, `billing_and_usage_page_v2.rs`, workspace admin) absent in fork. |
| `8eb52216f` | QUALITY-928: Orchestration unified stack (M2) | **Reject** | Cloud-agent orchestration: `pane_group/child_agent/*`, `ambient_pane_restoration`, orchestration trackers, `is_remote_child`, ambient task cache, `/agent/runs` fetches — all removed/absent surfaces. Hunks in fork-existing files (`ai/agent/conversation.rs`, `agent_conversations_model.rs`, blocklist files) are orchestration-gated only. |
| `eb56df113` | Default sandbox git identity to Warp instead of Oz | **Reject** | Only touches `app/src/ai/agent_sdk/driver/git_credentials.rs` (removed) and a GitHub workflow. |
| `4cd1c77c4` | Add a File explorer chip to the native agent input toolbelt | **Accept (adapted)** | See port notes below. |
| `b277c0eb0` | Add postgres client/server to agent-dev image | **Reject** | `docker/` removed in macOS-only fork. |
| `3d4ee7236` | REMOTE-2111: enable PeriodicHandoffCheckpoints for dogfood | **Reject** | `PeriodicHandoffCheckpoints` flag absent in fork; cloud handoff checkpoints are a removed remote-control area. |
| `9d3f3e1ec` | Couple small changes to reduce cloning | **Accept (adapted)** | See port notes below. |
| `73bd01431` | Fix ancestor-list request loop in child agent restoration | **Reject** | Fixes orchestration child-agent restoration (`pane_group/child_agent/restoration.rs` absent; `pane_group/mod.rs` hunks add `PendingParentChildSeed` machinery for server `?ancestor_run_id=` listings). |
| `57c460874` | Let workspace admins manage team member roles | **Reject** | Teams settings page absent in fork. |
| `5bd9b8e15` | Bump command-signatures to d79e09c (paru v2.1.0) | **Accept** | Cargo.toml rev `32a7fd56`→`d79e09c`; Cargo.lock updated with exactly the two git source entries (upstream's lock diff). Upstream `winit`/`x11rb` Cargo.toml lines not added (Linux/X11 deps removed). A `cargo update -p` side effect that downgraded `windows-sys 0.59`→`0.52` in unrelated lock entries was reverted to keep the lock diff identical to upstream's. |
| `1d1abaedb` | Add copy button to Initial query section in conversation details panel | **Accept (adapted)** | Ported onto the fork's slim panel: `copy_initial_query` mouse state, `CopyButtonKind::InitialQuery`, `CopyInitialQuery` action, `trimmed_initial_query` helper, copyable `render_source_section` with wrap. Fork's panel only had Directory/ConversationId copy buttons; upstream's FetchError/Error/SetupCommands kinds belong to cloud-mode fields absent here. |
| `294033bb1` | zsh: rebind kill-buffer on all keymaps to fix bootstrap residue leak (#7099) | **Accept** | `zsh_body.sh` applied exactly (warp_kill_buffer_and_reset_insert_mode + rebind across main/emacs/viins/vicmd). Integration test registered after `test_zsh_bootstraps_with_nounset_option`; fork's tmux SSH test registrations retained; fork's import structure kept with `clear_blocklist_to_remove_bootstrapped_blocks` added. |
| `e1bcf5d07` | Split the AI settings page into Warp Agent, Agent Profiles, Knowledge, and Third-Party CLI Agents pages | **Not applicable (structure); widget-level review recorded** | See below. |
| `d15645c77` | Add ORCHESTRATION variant to client AgentSource | **Reject** | `ambient_agents/task.rs` absent in fork. |

## File explorer chip port (`4cd1c77c4`)

- `toolbar_item.rs`: `FileExplorer` moved to the "Both" availability group (enum regrouped exactly as upstream), `is_available()` added gating on `cfg!(feature = "local_fs") && show_project_explorer` (matches `Workspace::compute_left_panel_views` and the `SHOW_PROJECT_EXPLORER` keybinding predicate), `FileExplorer` added to `all_available()` as opt-in (absent from defaults), `toolbar_item_tests.rs` added with upstream's two regression tests, `is_local_agent_view_control` (fork-specific guard) extended.
- Footer `mod.rs`: `file_explorer_button` doc moved above the CLI-only buttons; `CodeSettings`/`ShowProjectExplorer` subscription repaints the footer; both render paths gate `FileExplorer` through `is_available`; `ToggleFileExplorer` now emits `Option<CLIAgent>` (works without a CLI session), matching upstream's event type change.
- `cli_agent_footer.rs`: both `AgentInputFooterEvent::ToggleFileExplorer` and `CLIAgentFooterEvent::ToggleFileExplorer` carry `Option<CLIAgent>`; handler still calls `toggle_file_tree`.
- AgentView path: upstream wires the event through `terminal/view/use_agent_footer/mod.rs`, which does not exist in this fork. Fork integration glue: `terminal/input.rs` handles `AgentInputFooterEvent::ToggleFileExplorer(_)` by emitting the new `Event::ToggleFileExplorer`, handled in `terminal/view.rs` `handle_input_event` via `self.toggle_file_tree(ctx)` (the same path the CLI footer uses).
- Omitted: `app/src/server/telemetry/events.rs` `FileTreeSource::AgentToolbelt` (telemetry removed), the `toggle_file_tree` `source` parameter and `use_agent_footer` telemetry plumbing (their only purpose is the telemetry payload).

## Clone-reduction port (`9d3f3e1ec`)

- Applied exactly (3-way clean): `ai/facts/view/mod.rs`, `ai/facts/view/rule_editor.rs`, `drive/workflows/modal.rs`, `env_vars/view/env_var_collection.rs`, `integration_testing/cloud_object/assertion.rs`, `workflows/workflow_view.rs`.
- Applied with minimal adaptation: `cloud_object/mod.rs` (`is_trashed_internal` reorder + clone removal; `semantic_editing_history` clone removal — fork keeps its no-account rendering and `None => true` arm, which equals upstream's `SharedWithMe`-disabled behavior), `drive/workflows/enum_creation_dialog.rs` and `workflow_arg_type_helpers.rs` (clone removals only; fork's `is_visible_to_other_workflows` field naming and local-object comments kept).
- Prerequisite fix: restored the upstream `Copy` derive on `local_object_model::cloud_object::Revision` (upstream `crates/cloud_objects` `Revision` is `Copy`; the fork's localization commit `3dfa8418fd` dropped it, and every upstream clone-removal in this commit assumes it). `ServerTimestamp` is already `Copy`.
- Omitted (no host in fork): `update_object_queue_item` hunks in `ai/execution_profiles/mod.rs`, `ai/facts/mod.rs`, `env_vars/mod.rs`, `notebooks/mod.rs`, `workflows/mod.rs`, `workflows/workflow_enum.rs` (sync queue removed); `conflicting_object_revision`/`check_and_maybe_clear_current_conflict` hunks in `cloud_object/mod.rs` + `model/persistence.rs` (server-conflict machinery removed); removed-area files (`agent_sdk`, `ambient_agents`, `cloud_agent_config`, `cloud_environments`, `mcp/*`, `server/cloud_objects/*`, `sync_queue*`, `cloud_preferences`, `environments_page`, `crates/cloud_objects`).

## AI settings page split (`e1bcf5d07`) — widget-level review

The page-split structure (Warp Agent / Agent Profiles / Knowledge / Third-Party CLI Agents pages, `slug()`/`from_slug()` pane persistence, `AgentMCPServers` rename) targets upstream's settings-page organization, which this fork does not share (fork: About/Appearance/Features/Keybindings/Warpify/AI with its own widget model) — same ruling as the Code-page split in the 2026-08-14 audit.

Widget-level classification of the new pages:

- Third-Party CLI Agents page widgets: already ported into the fork's AI page on 2026-08-13/14.
- Knowledge page `RulesWidget` (`memory_enabled`): the fork keeps `memory_enabled` and the local `ai/facts` RuleView, but nothing serializes facts/rules into ACP prompt context in this fork, so the toggle has no live ACP/local runtime data flow for agent behavior — not ported (legacy Warp-Agent-behavior setting, per the ACP-ownership rule). `SuggestedRulesWidget` and `WarpDriveContextWidget` are removed surfaces (Suggested Rules, cloud Drive context).
- Agent Profiles page: tied to upstream's profile model selectors (removed in fork); the fork exposes execution-profile editing through its own local editor panes (`ai/execution_profiles/editor/`), so no settings-page port is needed.
- `warp_agent_page.rs` content is the upstream Warp Agent cloud settings surface.

## Verification

- `cargo fmt -- --check`: clean (after applying fmt).
- `cargo check -p warp --all-targets --message-format short`: passes (pre-existing warnings only; the queued-prompts dead-code trio predates this merge, introduced by the earlier queued-prompts port `dd7e10e908` and only surfaced in the integration-crate profile).
- `cargo check -p integration --all-targets --message-format short`: passes.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions) | test(toolbar_item) | test(conversation_details_panel) | test(agent_input_footer)'`: 158 passed.
- `cargo nextest run -p warp -E 'test(toolbar_item)'`: 2 passed (new file-explorer chip tests).
- `cargo nextest run -p warp -E 'test(cloud_object) | test(workflow) | test(env_var) | test(notebook) | test(facts)'`: 148 passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded, followed immediately by `cargo clean`.
- Deletion-surface scans: all hits are pre-existing documentation/fork-note comments, tokenizer vocabulary, or retained SSH/remote-terminal behavior; the port diff introduces none.

## Omitted paths and concrete reasons

See per-commit notes above. Highlights: all orchestration/ambient/Teams/sandbox-identity commits (removed areas), telemetry plumbing around the file explorer chip, sync-queue and server-conflict hunks of the clone sweep, and the settings-page split structure.
