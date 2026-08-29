# Upstream Master Audit 2026-08-29

## Scope

- Current fork before this audit: `5b7a452cd` (`main`, `v2026.08.28`).
- Upstream source reviewed: `c2e4ee491..upstream/master` (8 commits, tip `066ec71b7`).
- Result: zero ports. Every commit touches only removed or fork-absent surfaces; each shared-file edit was verified to have no live anchor symbol in this fork.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `f8aa4b98e` | CLIAgentSessionStatus updates should update ConversationStatus (#15622) | **Not applicable** | Fixes `TerminalView::child_conversation_id_for_cli_status_updates`, a conversation-status bridge introduced upstream in `1148ae3e8` (#9399, remote Claude Code agent wakeup) that the fork creation commit `19659d12` removed with the ambient-agent/orchestration stack. Anchors absent: the function itself, `LocalAgentTaskSyncModel` (upstream `530ca5229e`, post-baseline merge of the ambient task-sync/link models), `AmbientAgentTaskId`, `AIConversation::task_id()`/`set_task_id`, `is_child_agent_conversation`, `start_new_child_conversation`, `all_live_conversations_for_terminal_surface`. The fork's `CLIAgentSessionStatus` path (`terminal/view.rs` rich-input auto-toggle + desktop notifications, and `agent_conversations_model`/`agent_icon`/`vertical_tabs`/`conversation_list` icon status via `to_conversation_status`) derives status from the live session model and has no stale `ConversationStatus` bridge to fix. Porting would require restoring removed ambient-agent machinery. |
| `061318ff7` | Fix local Oz run_agents child rendering as duplicate local + remote pills (#15583) | **Not applicable / reject** | Oz orchestration run-id indexing race: `child_agent_launch.rs`, `assign_run_id_for_conversation`, `discard_stale_placeholder_for_run_id`, `is_remote_child`, `ensure_remote_child_placeholder`, `launch_local_no_harness_child`, `orchestration_event_streamer_tests.rs`, `app/src/tui_export.rs`, and `crates/warp_tui/` are all absent (removed orchestration/TUI surfaces). The shared-file hunks only touch those anchors (`mod.rs` re-exports `finish_local_oz_child_conversation` from the absent module; `history_model.rs` rewrites the absent `assign_run_id_for_conversation`). |
| `9ee2fa1d4` | Fix built-in Factory MCP reconnect race (#15628) | **Not applicable** | `app/src/ai/mcp/` does not exist in the fork; app-managed MCP (templatable manager and its native reconnect loop) is a removed surface. |
| `8c2cc7325` | REMOTE-2467: add a Windows CLI release artifact (#15637) | **Reject** | Windows packaging for the `oz` CLI (sidecar DLLs, `script/windows/bundle.ps1`, signing). Windows packaging and the oz CLI distribution are removed surfaces; the fork's `create_release.yml` is fork-owned (Sparkle DMG appcast flow) and accepts no upstream release-job hunks. `script/windows/bundle.ps1` is absent. |
| `49db23158` | Add Cancel to the SuperGrok / X Premium connect flow (APP-5638) (#15576) | **Not applicable** | All destination surfaces absent: `crates/ai/src/grok_subscription/` (Grok OAuth rejected), `app/src/settings_view/warp_agent_page.rs` (old Warp Agent page removed), `app/src/server/telemetry/events.rs` (telemetry removed), `crates/warp_tui/` (absent). |
| `f9adce601` | Send initiating view share_with_team_uid in shared session InitPayload (#15631) | **Not applicable** | Cloud shared-session team ACL: `app/src/terminal/shared_session/` and `app/src/terminal/local_tty/terminal_view_adaptor.rs` are absent; the fork pins no `session-sharing-protocol` dependency. Consistent with standing shared-session/Teams rejections. |
| `44357a02f` | Support mixed GitHub and GitLab environments in the agent client (#15616) | **Not applicable** | Entirely over removed surfaces: `app/src/ai/agent_sdk/`, `app/src/server/server_api/`, `crates/cloud_object_models/`, `crates/graphql/`, `crates/warp_graphql_schema/` are all absent. The server-side mixed-forge credential protocol has no local counterpart. |
| `066ec71b7` | factory-files: teach the skill and validator about webhook sources (#15494) | **Not applicable** | `resources/bundled/skills/factory-files/` and `script/test_factory_files_skill.py` are absent; bundled skills remain a rejected surface (third bundled-skill attempt, consistent with the 2026-08-17/08-18/08-25 rejections). |

## Provenance

- No patches applied; `git show` inspection per commit plus anchor-symbol verification (`rg` for every function/type/module the hunks modify, and file-existence checks) against the fork tree.
- Upstream history consulted for `f8aa4b98e` (`git log -S` for `child_conversation_id_for_cli_status_updates`, `LocalAgentTaskSyncModel`) to confirm the fixed code path was removed at fork creation rather than never reviewed.

## Verification

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: pass.
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: pass.
- `cargo build -p warp --all-targets --message-format short`: pass; `cargo clean` run after the release push.
- Deletion-surface scans: no code changes this cycle, so no new hits possible; scans unchanged from the 2026-08-28 audit baseline.
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
