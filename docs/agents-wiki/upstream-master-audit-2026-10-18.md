# Upstream Master Audit — 2026-10-18

Incremental audit for `b80b4e3e0..upstream/master` (`3f02b7028`), 2 commits. The previous audit is `upstream-master-audit-2026-10-17.md`.

## Summary

One adapted port: the bash `$BASH` pinning fix (`063a7643a`) for the retained local PTY shell-spawn path. One not-applicable commit: the agent-sdk pre-warmed standby sandbox stack (`3f02b7028`), which lives entirely in fork-absent surfaces with no shared-file anchors.

## Commit-by-Commit

### `063a7643a` Pin $BASH to the executed bash binary when bootstrapping bash sessions (#16520) — adapted, ported

Retained local terminal/shell fix. When bash is the session shell and Warp was launched from the Dock/Finder, `$BASH` resolved through launchd's PATH to `/bin/bash` instead of the executed binary (e.g. Homebrew's bash), corrupting the bootstrap's `shell_path` report and its `"$BASH" --norc -i` PS1 expansion. The fix pins the rcfile's first statement to `BASH='<escaped resolved path>'` and single-quote-escapes the exec path itself.

Applied via the exact upstream patch (`git diff 063a7643a^ 063a7643a` remapped from `crates/warp_terminal/src/local_tty/` to the fork's pre-split `app/src/terminal/local_tty/`, then `git apply --3way`). Fork adaptations, recorded against the upstream patch:

- `shell.rs` import hunk: upstream adds `shell_escape_single_quotes` to `use crate::shell::{...}` (warp_terminal crate layout). The fork keeps its grouped `crate::terminal::{...}` import and adds the separate `use crate::terminal::model::session::command_executor::shell_escape_single_quotes;` line, matching the existing idiom in `app/src/terminal/local_tty/unix.rs`. The function-body hunk (comment item 7, `escaped_shell_path`/`set_bash_var` locals, `exec -a bash '{}' --rcfile <(echo '{};{}')` format) is verbatim upstream.
- `shell_tests.rs`: both upstream tests ported verbatim (`bash_spawn_command_pins_bash_variable_to_resolved_path`, `spawned_bash_reports_the_executed_binary_as_bash_variable` with the PTY-running `bash_variable_for_spawned_shell` helper), plus the upstream `use tempfile::tempdir;` import. Omitted: `test_trim_wsl_err_from_output` and its preceding context — WSL output trimming is a removed surface and `take_until_utf16_crlf` has no fork anchor. Upstream's `direct_replacement_refreshes_embedded_bootstrap_session` test was already in the upstream file before this commit and is out of this commit's scope.
- Upstream's WSL-side effect (`wsl_arguments_for_session_spawning_command` reuse) is fork-absent by design; the shared-function fix covers the fork's macOS path.

### `3f02b7028` feat(agent-sdk): prepare pre-warmed standby sandboxes (#16449) — not applicable

Part 5 of the pre-warmed sandbox stack (REMOTE-3488): `agent run-cloud-standby --standby-id` prepares a Namespace sandbox before a task exists, fetches config via the `prewarmedSandboxConfig` GraphQL mutation, clones repositories, starts configured MCP servers, and reports readiness/heartbeats over HTTP. Every mechanism path is fork-absent: `app/src/ai/agent_sdk/**` (removed Agent SDK; parts 1–4 of this stack were already rejected as N.A. in the 2026-10-16/17 audits), `app/src/server/server_api/**` (no `app/src/server`), `crates/warp_cli/src/agent.rs` (fork `warp_cli` has no agent surface).

Shared-file hunks verified anchorless: `app/src/lib.rs` adds `CliCommand::Agent(AgentCommand::RunCloudStandby(_))` arms to `LaunchMode::command_line`/`supports_indexing`, and `app/src/lib_tests.rs` adds a `run_cloud_standby_is_sandboxed_and_leaves_computer_use_to_defaults` test using the `command_line_launch_mode` helper — the fork removed `LaunchMode::CommandLine`, `AgentCommand`, and the CLI agent-command parser at baseline (`rg "AgentCommand|LaunchMode::CommandLine|command_line_launch_mode"` over both fork files returns zero matches). Nothing separable.

## Verification

- `cargo fmt -- --check` — passed.
- `cargo check -p warp --all-targets --message-format short` — passed (SDKROOT pointed at the Xcode 26.5 SDK per the CLT 27 tapi prerequisite).
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — passed (156 run, 156 passed, 2972 skipped).
- Focused new tests: `bash_spawn_command_pins_bash_variable_to_resolved_path` and `spawned_bash_reports_the_executed_binary_as_bash_variable` both pass.
- `cargo build -p warp --all-targets --message-format short` — passed; `cargo clean` run after the push.
- Restored-surface scans — zero delta against `main` for all three scan patterns (pref-stripped `git grep` comparison over `app crates script Cargo.toml`).

## Tag

`v2026.10.18` on `main` (merge commit of `merge/upstream-2026-10-18`).
