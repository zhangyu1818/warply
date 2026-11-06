# Upstream Master Audit — 2026-11-06

Incremental audit for `3fe2ac676..upstream/master` (`f7fc2f2a0`), 7 commits. The previous audit is `upstream-master-audit-2026-11-05.md`.

## Summary

One adapted port: the Codex/Claude auth-command exclusion from CLI session detection (`03ceb39cf`), including the prerequisite `command_prefixes()`/`matches_command()` detection structure it generalizes. Six commits rejected or not applicable over fork-absent surfaces (`agent_sdk` cloud-run/prewarm stack, bundled harness skills, plugin manager, Linux Debian packaging, and the blocklist startup-queue machinery whose anchors are all absent).

## Commit-by-Commit

### `03ceb39cf` Exclude Codex and Claude auth commands from CLI session detection (#16723) — adapted, ported

Upstream fix: `CLIAgent::matches_command` generalizes the WarpTui `acp-bridge` exclusion into a `non_session_subcommands` list — Claude `auth`, Codex `login`/`logout` — so authentication commands like `codex login status` or `claude auth status --json` no longer register a live CLI agent session. The same false positive exists on the fork's local path: `TerminalView::detect_cli_agent_from_model` calls `CLIAgent::detect` on the active block's command, so running an auth command would raise the CLI-agent toolbar and route follow-up input as if an interactive agent session were live.

The upstream hunk modifies `matches_command`, which the fork never carried (it arrived with the Warp TUI parity stack `43ee27303` and `6aeacb6ad`, both rejected as TUI-only). Ported the fix together with its prerequisite structure from the exact upstream source:

- `command_prefixes() -> &'static [&'static str]` copied from upstream `43ee27303`/`03ceb39cf` state, adapted to the fork's variant set: Vibe `["vibe", "vibe-acp"]`, Antigravity `["agy", "antigravity"]`, Kiro `["kiro-cli", "kiro"]`, Grok/Muse single entries; Hermes/OhMyPi/WarpTui arms omitted (variants absent). `command_prefix()` now delegates to `command_prefixes().first()`, preserving every current return value.
- `matches_command()` copied verbatim from `03ceb39cf` (basename extraction over the resolved command, shell-aware subcommand check via `all_parsed_commands`), with only the `CLIAgent::WarpTui => &["acp-bridge"]` arm and its comment omitted (no WarpTui variant in the fork).
- `detect()` now delegates to `matches_command(&resolved_command, escape_char)` exactly as upstream, replacing the fork's first-word equality plus the Vibe/Kiro/Antigravity second-prefix special cases (subsumed by the prefix lists). The upstream `is_aifx_agent_run_claude` clause is omitted (Uber-team check over the removed `UserWorkspaces`; fork keeps `_ctx` unused).
- The four regression tests (`test_detect_does_not_match_auth_commands`, `..._with_shell_syntax`, `..._through_aliases`, `test_detect_matches_auth_words_as_prompt_arguments`) applied verbatim via `git apply --3way`.

Behavior note: detection now also matches path-form executables (`/usr/local/bin/claude` → basename `claude`), which is upstream's matching semantics; the fork's pre-existing detect tests (`claude_wrapper`, `vibe-other`, `npx muse` negatives) remain valid and pass.

### `765ebc8a9` Align cloud completion skills and platform plugin minimums (#16732) — rejected / not applicable

Bundled cloud-harness skill copy updates under `resources/bundled/harness_skills/` (removed skills surface, directory absent) plus minimum-version bumps for the Claude/Codex plugin installers under `app/src/terminal/cli_agent_sessions/plugin_manager/` — the fork is pre-plugin_manager lineage (`cli_agent_sessions/` contains only `event/`, `listener/`, `mod.rs`, `mod_tests.rs`). No fork file to apply.

### `9dbbea43e` [ACP] Fall back to the PTY harness without an adapter, report the transport, and attribute the opening prompt (#16742) — rejected

Despite the `[ACP]` tag this is the `agent_sdk` ACP harness for **cloud runs**, not the fork's `app/src/ai/acp/` client: PTY fallback in `agent_sdk/driver/harness/*` and `prewarmed.rs`, the new `transport_report.rs`, the `reportHarnessTransport` GraphQL mutation (`crates/graphql`, `crates/warp_graphql_schema`, `crates/warp_server_client`), `app/src/server/server_api/harness_support*`, and `AmbientAgents` telemetry. Every one of those paths is removed in this fork. The lone `app/src/ai/agent/base_user_query.rs` hunk (cfg-gate relaxation + error-copy rewording for the warp-server base64 `user_query_b64` decode) has no fork anchor either — the file is absent; the fork's ACP client never receives server-encoded user queries.

### `2699ab77b` fix(prewarm): remove the client harness restriction for pre-warmed sandboxes (#16748) — not applicable

Touches only `app/src/ai/agent_sdk/driver/prewarmed.rs`, `prewarmed_tests.rs`, and `execution_config.rs` — removed `agent_sdk` surface, absent in the fork.

### `a6b42da27` Update the pubkey installed by Debian Warp packages to match the new key (#16750) — rejected

Single file `resources/linux/debian/common/postinst.repo.template` — Linux packaging is a removed surface; the path does not exist here.

### `73b7d6f3c` Fix queued query submission at startup (#16755) — not applicable

The `agent_sdk/driver.rs` hunk removes an eager `dispatch_queued_warp_agent_prompt` call (removed surface). The blocklist hunks all target the native prompt-queue/startup-queue machinery the fork never ported: `app/src/ai/blocklist/controller/startup_queue.rs` and its tests are absent, and repo-wide search finds no `ready_query`, `dispatch_queued_warp_agent_prompt`, `can_dispatch_queued_warp_agent_prompt`, `finish_native_setup`, `native_prompt_conversation_id`, or `QueuedQueryEvent::PromptReady` — the fork's `queued_query.rs` is the older queue-next-prompt lineage, so every hunk is anchorless rather than a cherry-pick candidate.

### `f7fc2f2a0` Update check inbox wording in skill (#16752) — not applicable

Single file `resources/bundled/harness_skills/oz-child-agent-orchestration/SKILL.md` — bundled skills are a removed surface; the directory does not exist here.

## Verification

- `cargo nextest run -p warp --lib -E 'test(terminal::cli_agent)'` — 83 passed (includes the 4 ported regression tests).
- `cargo check -p warp --all-targets --message-format short` — passed (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean` (removed 16.9 GiB).
- Restored-surface scans re-run: cloud/auth/billing/telemetry scan returns only the pre-existing allowed hits (weak-handle `upgrade` wording, tests, doc comments); MCP/skills scan returns zero hits; platform scan returns exactly the six known retained occurrences (`ForwardX11=no` SSH config ×2, retained ConPTY explanatory comments in `zsh_body.sh`, `#[cfg(windows)]`-gated tests in `warp_util`) — unchanged from the 2026-11-05 baseline.

## Tag

`v2026.11.06` on `main` (merge branch `merge/upstream-2026-11-06`, one port commit merged with `--no-ff`).
