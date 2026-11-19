# Upstream Master Audit — 2026-11-19

Incremental audit for `1f357e8e2..upstream/master` (`a1ade81fc`, 20 commits), five of them the agent-hooks Phase 2 gap-closure stack, ported as five adapted port commits on `merge/upstream-2026-11-19`. The previous audit is `upstream-master-audit-2026-11-18.md`.

## Summary

Five commits ported (Cursor `tool_input` field names, per-file tool events with per-run hook timing, tool run duration on post-tool events, prompt attachments plus the per-event model, and the subagentStart library half with the shared `report_denial` factoring); fifteen rejected or not applicable — every one confined to fork-absent `agent_sdk`/server-api/GraphQL/prewarmed-sandbox/warp_tui/winit-windows/warp_graphql_schema surfaces or fork-absent anchors, each verified against the fork before rejection. No removed product surface was restored.

Port commits: `c03cc8320` (`02d525eeb`), `aaf7b9337` (`d89fbe47c`), `c2ae33ca1` (`076d2fa27`), `ffb758e49` (`69f2052a5`), `92d76c914` (`22b2a572b`).

## Commit-by-Commit

### `02d525eeb` hooks: Cursor field names in the Cursor adapter's tool_input (#16892) — adapted port

File edits name their file as `file_path` (was `file`) and a string replacement as `old_string`/`new_string` (was `search`/`replace`); a single-file edit batch gets a top-level `file_path` via `single_file`; the three generic tool events share the factored `insert_tool_fields`.

Fork adaptations: the ReadSkill `Read`/`ReadSkill` mapping, the MCP arms (`CallMCPTool` input as the whole arguments plus `mcp_server_id` beside it, `ReadMCPResource`), and the `mcp_server_id` helper are omitted — the fork's action enum has no skills or MCP variants, so `insert_tool_fields` carries Cursor's `tool_name`/`tool_input` only. The fork's `FileEdit::Create` keeps its no-`allow_overwrite` shape. Omitted: `specs/agent-hooks/**`.

### `d89fbe47c` hooks: one event per file for batched reads and edits; time each hook from when it starts (#16894) — adapted port

The adapter contract becomes `encode_inputs -> Vec<HookInput>` (payload + optional label). `parts.rs` splits a read per file and an edit batch per file it touches; the matcher tests each part (`^Delete$` selects the delete in a mixed batch); each part's `tool_output` is its slice of the result. `HookDispatcher::run_binding` runs the binding once per input under the runner's slots, prefixes denial reasons with the part's file (`src/secret.env: not allowed`), and merges a binding's runs with the cross-binding reduce rules (all-defer stays a deferral). The runner starts an invocation's clock when it holds one of the four slots; `HookProcessOutcome` reports `duration` (the run) and `queued` (the wait) separately, and the invocation log gains `part=`/`queued_ms=`.

Fork adaptations: `tool_parts`' catch-all lists the fork's variant set; `part_result_view`'s read arm has no `failed_files` (the fork's `ReadFilesResult::Success` carries only `files`); test literals drop `allow_overwrite`; `insert_tool_fields` keeps no MCP server naming. Omitted: `specs/agent-hooks/**`.

### `076d2fa27` hooks: tool run duration on postToolUse and postToolUseFailure (#16895) — adapted port

`AsyncExecutingAction` records `started` after every check and confirmation; `observe_terminal_result` carries `Option<Duration>` into both envelope builders (async completion and cancellation report the real elapsed run, a sync result `Duration::ZERO`, a never-started pending action `None`); `envelope_for_tool_result_with_duration` folds into `envelope_for_tool_result`. The fork's unknown-conversation negative test passes `None`. Omitted: `specs/agent-hooks/**`.

### `69f2052a5` hooks: prompt attachments on beforeSubmitPrompt, and the request's model on every event (#16896) — adapted port

`HookPayload::PromptSubmit` carries typed `PromptAttachment`s (File/Rule) instead of path strings; `attachments_in` reads them off the input (person-attached `FilePathReference`s in path order, then `ProjectRules` active rules); the Cursor adapter encodes `type`/`file_path` pairs. Pre-request boundaries name a model: `EnvelopeSite::before_request` stores a site model that `build` prefers over the latest exchange's, wired through `prepare_turn`, `prompt_dispatch`, and the session barrier.

Fork adaptations: the model is `Option<LLMId>` end to end and the controller passes `None` — ACP adapter configuration owns model choice, so a boundary that fires before the request exists knows no model and the envelope falls back to the conversation's last exchange (the fork's `RequestInput` keeps `LLMId::from("auto")`; there is no `LLMPreferences`/`TeamScope`, so upstream's `active_base_model_id` helper and call sites are omitted). `attachments_in` drops the fork-absent `DriveObject`/`Repository`/`PullRequest`/`Skills` arms. The controller-driven attachments e2e test is omitted with the absent MAA harness; the ported tests are the adapter-level attachment test and the pure `attachments_in` test (its `AgentWake` assert becomes `ResumeConversation`), and the session-start model assertion drives the fork's library-level `prepare_turn` path. Omitted: `specs/agent-hooks/**`.

### `22b2a572b` hooks: subagentStart for each child of a run_agents call (#16897) — adapted port (library half)

Library side: `HookPayload::SubagentStart` with `SubagentLaunch`/`SubagentExecutionMode`, the boundary moved into Phase 1, the Cursor adapter's `encode_subagent_start` with the `subagentStart` fixture set, the `SubagentType` matcher value (`generalPurpose`), and the shared `report_denial` factoring in `hooks/mod.rs` that `pre_tool::decide` now calls.

Fork omissions: the app-side producer stays out with the removed run_agents orchestration — `execute/run_agents.rs`, `run_agents_tests.rs`, `hooks/subagent.rs`, and `envelope_for_subagent_start` are not ported, so the boundary parses, validates, matches, and encodes but nothing fires it in this fork. `mcp_tool_call` keeps its inert stub (the fork has no `CallMCPTool` action). Omitted: `specs/agent-hooks/**`.

### Rejected or not applicable (15 commits)

- `384f9f674` (skip sandbox deadline FAILED report while idle): `agent_sdk/driver*` plus `local_agent_task_sync_model.rs` — both fork-absent.
- `76e4ab7d1` (inherited Git with managed GitHub MCP for local Oz runs): `agent_sdk` driver + `server_api` + GraphQL schema.
- `a4ad25c41` (Factory experiment values through the execution bootstrap): `agent_sdk`/`server_api`/GraphQL prewarmed stack.
- `01ff35f8a` (Claude launch commands in PowerShell): `agent_sdk/driver/harness/claude_code*` — the removed Oz harness, not the fork's ACP client.
- `7599c8433` (Claude ACP sessions Manual permission mode): same harness path (`driver/harness/acp`), zero fork anchors; the fork's ACP client owns permission modes through AI settings.
- `11a809346` (aws-smithy-json CVE bump): `aws-smithy-json` is not in the fork's `Cargo.lock`.
- `51031f387` (Windows clipboard sequential test): `warpui/src/windowing/winit/windows/` absent.
- `ecd06934d` (Preview URLs MCP in Stable): the `preview_proxy` stack was rejected 2026-11-14; the flag does not exist here.
- `11cfab87f` (task in progress on acp steered follow-ups): `agent_sdk` harness turn driver + `local_agent_task_sync_model.rs` — fork-absent steering/task-sync surfaces.
- `cfea9b732` (disable native Codex orchestration in Oz harness setup): `agent_sdk/driver/harness/codex*`.
- `cad3f6071` (bot-only gh login for managed sandbox credentials): `agent_sdk/driver/git_credentials/restricted*`.
- `8514d38bc` (browser login requirement for Factory Git access): `warp_tui` — absent.
- `76a9f5139` (shell-quote CVE bump): `warp_graphql_schema/package.json` — absent.
- `3e08d54c8` (restricted credentials for code.storage Factory definitions): `agent_sdk` git credentials.
- `a1ade81fc` (beforeSubmitPrompt for the first prompt of an Oz run): every retained-file hunk serves the ambient-run first prompt — `AIAgentInput::StartFromAmbientRunPrompt` (with `ResolvedRunPrompt`), the `RunPromptHalted` controller event, and the `PromptDecision::Halted { reason }` plumbing for it all live on the fork-absent ambient-agent/`agent_sdk` driver lineage; the fork's `send_query` already routes the first typed prompt through `prompt.submit` via `prepare_turn`.

## Restored-Surface Scans

All three skill scans re-run. Scan 1 returns the same 207 non-doc hits as `main` (no new hits; the hooks-runner doc comments about cloud launch environments are the documented copied-library set). Scan 2 hits remain the documented inert set (`mcp_server_denylisted` permission floor, the never-fired MCP event fixtures and the inert `insert_mcp_call_fields` stub). Scan 3 hits remain SSH `ForwardX11`, bootstrap ConPTY comments, and `#[cfg(windows)]` arms in copied library/test code that compile out on macOS.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p ai --all-targets` / `cargo check -p warp --all-targets` — passed after every port.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p ai` — 313 passed (including the new parts/subagentStart fixture tests; 312 after port 2, +1 with subagentStart).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed.
- `cargo nextest run -p warp -E 'test(hooks) | test(prompt) | test(session) | test(turn) | test(envelope) | test(pre_tool) | test(action_model) | test(real_process) | test(queued_query) | test(stream) | test(menu) | test(parts) | test(dispatch) | test(command) | test(subagent)'` — 887 passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed by `cargo clean`.

Known gaps, recorded honestly: the subagentStart boundary has library-level fixture coverage but no fork producer (run_agents stays removed), and the controller-driven prompt-attachment e2e test is omitted with the absent MAA harness — the attachment path is verified at the adapter and `attachments_in` level.

## Tag

`v2026.11.19` on `main` (five adapted port commits plus this audit, merged with `--no-ff`).
