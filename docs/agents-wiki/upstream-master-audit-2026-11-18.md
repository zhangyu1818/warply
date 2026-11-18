# Upstream Master Audit — 2026-11-18

Incremental audit for `9c9f0b579..upstream/master` (`1f357e8e2`, 16 commits), five of them the agent-hooks Phase 1 completion stack (PRs 6–10), ported as five adapted port commits on `merge/upstream-2026-11-18`. The previous audit is `upstream-master-audit-2026-11-17.md`.

## Summary

Five commits ported (hooks dispatcher/permission policy, prompt/turn/session/shell-completion boundaries, run-ending on an unanswerable ask, the library e2e suite, and the flag removal with login-shell PATH capture); eleven rejected or not applicable — all confined to fork-absent `agent_sdk`/prewarmed-sandbox/server-api/GraphQL/skills/orchestration/preview-proxy/managed-MCP/computer-use-recording surfaces with every shared-file hunk verified anchorless. No removed product surface was restored.

The `AgentHooks` feature flag is gone with upstream PR 10; hooks in the fork now load from `<cwd>/.cursor/hooks.json` at startup through new fork glue (see `70b4354b6` below), so the feature is live when a project configures it and inert otherwise.

Port commits: `33707b3f1` (PR 6, `ee38f86f4`), `ba0611230` (PR 7, `c98d0be32`), `cdceb7fbc` (PR 8, `d1b1daa0c`), `fcb747d3c` (PR 9, `288af8a37`), `d6963b2de` (PR 10, `70b4354b6`).

## Commit-by-Commit

### `ee38f86f4` hooks: dispatch bindings at once and apply the permission policy (#16590) — adapted port

Sixth PR of the stack. The dispatcher runs every matching binding under `join_all` and merges in snapshot order; the permission policy composes the merged verdict with Warp's own decision (hooks only tighten); the pre-tool verdict key covers argument and cwd changes; denied actions of any type finish `DeniedByHook`; strict/on-ask come from each hook source; `AppExecutionMode` gains `is_headless`/`new_with_headlessness`.

Fork adaptations: `is_headless` derives from the fork's `App|Headless` variant set (`new_with_headlessness` drops `is_sandboxed`; the lib.rs production caller keeps `new`); the envelope surface maps headless to `CliRun` with no Oz/Tui branches and `user_email` stays `None`; the floors drop the team scope, the MCP floor family, and fork-absent action variants (`Inconclusive`/`ProtectedPath` reasons do not exist here); the run-agents denial telemetry is omitted; `ReadSkillResult` `is_failed` is not applicable (no `ReadSkill` action). Omitted with absent anchors: the child-conversation envelope test, the protected-path floors test module (`.mcp.json` protection left with MCP), and `add_child_conversation`.

### `c98d0be32` hooks: prompt, turn, session, and shell completion boundaries (#16593) — adapted port

Seventh PR. `prompt.submit` becomes a waitable step in `send_query`; `turn.stop` holds a completed turn's success and can queue a follow-up with provenance and loop limits; `session.start` fires once per conversation before the first request, which waits for it and carries its env; `afterShellExecution` fires on a long-running command's real exit.

Fork adaptations: `send_query` splits into prepare/conclude/send with `PreparedQuery` minus the orchestration piggyback and team scope, and the in-controller slash-command re-read is omitted (the fork routes slash commands before the controller); the turn lifecycle runs from a new controller subscription to `AcpEvent::Completed` (which now carries `conversation_id`) instead of upstream's MAA response-stream path — `has_active_stream_for_conversation` reads the latest exchange's `Streaming` status, `settle_turn_after_stop` dispatches a queued follow-up through `send_queued_user_query_in_conversation`, `cancel_conversation_progress` drops the pending prompt and settles a dropped held turn, and `mark_request_completed` returns `turn_ended` with the `Success` write moved into the controller's `finalize_turn`; `tool_result` drops the result timestamps the fork's action results never carried (envelope duration stays empty), the `ShellRecovered` arms, and fork-absent action families; the history-event subscription maps to the fork's event set (`ClearedConversationsInTerminalView`, `ConversationOwnershipTransferred`). `ai_document_model`'s orchestration-config tweak and `queued_prompts_panel`'s suffix-list edit are covered by the fork's existing structure. Omitted with the absent MAA harness: `controller_tests.rs` (never in the fork), the ten controller/mock-stream-driven turn tests, the controller-driven session/prompt/stream tests — the retained boundary tests drive `prepare_turn`/`finish_turn_preparation`, `hold_turn`/`start_turn_stop`/`resolve_turn_stop`, and the session barrier directly; a `new_conversation` helper replaces `send_prompt`'s per-call conversation.

### `d1b1daa0c` hooks: deliver hook sources and settings to Oz and CLI runs (#16602) — adapted port

Eighth PR, fork-relevant subset: `CancellationReason::HookEndedRun` with `FinalizedExternally` outcome and `hook_ended_run` envelope name; `fail_conversation_due_to_hook` plus the shared `fail_conversation` core; the `has_failed` guard so an action finishing after its conversation was failed cannot move it out of `Error`; `RunSurface`/`RunIdentity` run tracking; `NotebookKeybindings` registration moved into the shared test app init.

Fork adaptations: the agent_sdk driver that delivers sources/settings to runs is absent, so `ApprovalUnavailable` is consumed by the controller (the fork's run owner), guarded on the runtime being registered; `fail_conversation` reads in-flight streams from the conversation's tracked response streams (`response_stream_ids_being_processed`, added to `AIConversation`) and sets the direct error through `update_conversation_status_with_error_message`; the shell-exit failure route and the `CLISubagentUserTakeover`/`AgentExitedShell` variants stay out with their absent CLI-subagent lineage; `RunSurface` drops the server-dispatched `Oz` variant. Omitted: driver/agent_setup/execution_config/standby paths, server_api fixtures, graphql schema files, and `code_diff_persistence_tests.rs` (deleted at baseline).

### `288af8a37` hooks: end-to-end tests and a sample hooks fixture (#16605) — adapted port

Ninth PR, library side: `e2e_slice_tests` is replaced by `e2e_tests` running the checked-in fixtures (sample project, malformed config, factory layout) through the real registry loaders, dispatcher, and process runner with an explicit `HOME`/`PATH` per runner. Omitted: the app-side controller-harness e2e suite (needs the absent MAA harness), the agent_sdk driver hook hunks, the bundled factory-files skill reference, and specs.

### `70b4354b6` hooks: remove the client feature flag; capture the login-shell PATH when hooks load (#16861) — adapted port

Tenth PR: `FeatureFlag::AgentHooks` is gone (variant, the fork's `agent_hooks` cargo feature and lib.rs bridge, every test override); `is_installed` is just the singleton existing; the login-shell PATH read moved out of startup into a per-process capture started only when hooks load (`read_interactive_path_env_var_now`, a fresh read, settled within ten seconds so every hook sees the same PATH, with a diagnostic when unreadable); the flag-off tests became registry-finalization tests (staged bindings are never consulted before `finalize`; a finalized empty registry behaves like one never finalized).

Fork adaptations: upstream loads a run's configured hook list in the agent_sdk driver (absent), so the fork gains startup glue — `HookRuntime::load_project_hook_sources` reads `<cwd>/.cursor/hooks.json` through `ExplicitHookSource::from_directory` (Worker directory, `FactoryWide` kind, default policy), finalizes the registry (empty when no file exists, so hooks stay inert), then starts and settles the PATH capture exactly as upstream's run setup does, wired from `run_internal` next to `install_command_runner`. The controller-driven flag-off tests were re-created at library level. Omitted: agent_sdk driver/agent_setup/prewarmed hunks.

### Rejected or not applicable (11 commits)

- `54d2a150d` (prewarmed deferred-repository advertisement), `9bb8d8083` (runner standby resources), `9a38f1c88` (code.storage Factory checkout), `0f81fad40` (prewarmed hook support advertisement): `agent_sdk`/`server_api`/GraphQL prewarmed-sandbox stack, zero fork anchors.
- `844591cc4` (cloud skill listings scoped to cwd): `agent_sdk` driver + app-managed `skill_manager` + `specs/QUALITY-2387`; skills belong to the ACP agent process.
- `4e518f161` (notify native agents another run waits): `orchestration_event_streamer` (removed) + `server_api`; no anchors.
- `429ade6e6` (normalize preview origins): `preview_proxy` crate rejected 2026-11-14, still absent.
- `d0bdf5beb` (agent-tip analytics dual-write removal): the fork's status bar never had the dual-write (telemetry removed at baseline); `server_api.rs` absent.
- `a45a95193` (targeted wait-for-events CLI): `agent_sdk` harness + `warp_cli/src/harness_support.rs`, both absent; the fork has no `WaitForEvents` action.
- `856c3249d` (ManagedMcpCredentialRefresh → Stable): promotes a managed-MCP flag the fork never had.
- `1f357e8e2` (3PCU computer use through a built-in MCP server): the harness/MCP-server paths are `agent_sdk`-confined; the lone fork-present file's hunk (`use_computer.rs` pointer-sink refactor) has no anchors — the fork's computer-use recording stack is trimmed away.

## Restored-Surface Scans

All three skill scans re-run; hits limited to this merge's touched files are the documented inert set: upstream doc comments in `crates/ai/src/hooks/runner/env.rs` (copied library code), and the `mcp_server_denylisted` permission-floor boolean plus its matrix tests in `crates/ai/src/hooks/permission.rs` (copied upstream policy, not MCP server management). `#[cfg(windows)]` arms in `crates/command` and the hooks runner compile out on macOS. No auth/billing/GraphQL/Sentry/telemetry/MCP-management/skills surface was reintroduced.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide after each port.
- `cargo check -p warp --all-targets --message-format short` — passed after every port.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p ai` — 300 passed (hooks library incl. the new e2e fixture suite).
- `cargo nextest run -p command` — 6 passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed.
- `cargo nextest run -p warp -E 'test(hooks) | test(prompt) | test(session) | test(turn) | test(envelope) | test(pre_tool) | test(action_model) | test(real_process) | test(queued_query) | test(stream) | test(menu)'` — 763 passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean` (21.2 GiB reclaimed).

Known gaps, recorded honestly: the ten controller/mock-stream-driven turn-lifecycle tests and the app-side e2e suite are omitted with the absent MAA harness, so the controller's turn-hold/settle/follow-up-dispatch wiring and the `AcpEvent::Completed` subscription are verified by compile and the library-level tests only; the fork's hook loading (`<cwd>/.cursor/hooks.json`) is startup glue with no dedicated test.

## Tag

`v2026.11.18` on `main` (five adapted port commits plus this audit, merged with `--no-ff`).
