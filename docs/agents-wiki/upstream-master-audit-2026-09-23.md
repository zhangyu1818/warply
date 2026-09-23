# Upstream Master Audit 2026-09-23

Range under review: `f4f9b8838..upstream/master` (12 commits)

Previous audited upstream tip: `f4f9b8838 REMOTE-3075: Test signal origin enrichment (#16081)`

Current upstream tip: `71088ba18 Report when a cloud agent run has computer use enabled but unavailable (#16114)`

Total upstream commits in this incremental range: 12

Status: triage complete. Two ports (one adapted tooling port, one adapted feature port); ten commits rejected, not applicable, or empty.

## Per-Commit Triage

### `ae06c14f6` — Fix scope-specific spend limit alerts (#16073)

Decision: **reject (no anchors)**. Spend-limit alert fixes for AI credits: scope-specific alert banner logic in `app/src/ai/blocklist/prompt/prompt_alert.rs`, `credit_availability.rs`, the `inline_banner/prompt_suggestions.rs` banner, plus GraphQL schema and `skills-lock.json`. The fork has none of these files (billing/credits surfaces removed at fork creation); verified by path inspection. Nothing separable.

### `93b4f91e9` — Remove stable orchestration feature flags (#15887)

Decision: **N/A**. Upstream deleted the `WaitForEventsParentRegistration` and `OrchestrationUnifiedStack` cargo features and `FeatureFlag`s, making orchestration unconditional, and removed the `EnsureSharedSessionViewerChildPane` terminal event variant plus legacy drains/hydration in `orchestration_event_streamer`, `pane_group/child_agent/*`, and `shared_session/viewer/*`. The fork deleted orchestration wholesale at creation; searches show no `WaitForEventsParentRegistration`/`OrchestrationUnifiedStack`/`MultiLevelOrchestration` flags, no orchestration files, and no `Ensure*ViewerChildPane` events. Every hunk targets fork-absent code.

### `64f652a8f` — Add conservative ShellCheck gate for Bash bootstrap (#16104)

Decision: **accept (adapted)**. Retained bootstrap/dev-tooling hardening: a blocking warning-level ShellCheck gate for `app/assets/bundled/bootstrap/bash_body.sh` via a new `script/lint_shellcheck`, wired into `script/presubmit` and `script/macos/bootstrap`, a `.shellcheckrc` baseline (SC2213/SC2155 disabled), and behavior-preserving quoting fixes in `bash_body.sh` (quoted array expansions, `kill "${pids[@]}"`, array emptiness via `${#pids[@]}`, quoted `$(...)` in echo/if, `declare -F` for function probes, quoted `HISTFILESIZE`/`HISTSIZE` comparisons, and line-level SC2164/SC2124/SC2034/SC2154 suppressions where semantics would be risky).

Port: exact three-way patch of the retained paths. Recorded adaptations and omissions:

- The ctrl-r detection context in `bash_body.sh` keeps the fork's ungated structure (fork dropped `WARP_IN_MSYS2`); upstream's SC2154 suppression lands before the atuin indirect-dispatcher check at the fork's indent level.
- The new gate flags `RESET_GRID_OSC` (SC2034) because the fork removed both consumers (the `warp_maybe_send_reset_grid_osc` helper and the `warp_update_prompt_vars` arm were Windows ConPTY branches). The dead assignment is dropped instead of suppressed, consistent with the fork's prior ConPTY removals.
- `.github/workflows/ci.yml` omitted: the fork-owned macOS-only CI has no Linux lint job for the apt-based install step.
- `script/linux/install_test_deps` omitted: fork-absent path.

Verified: `bash -n` passes; `./script/lint_shellcheck` passes locally (ShellCheck 0.9.x via Homebrew) against the ported file.

### `796fa478f` — Fix master build: remove dangling pending_remote_child_hydrations use (#16116)

Decision: **N/A**. Two-line build fix for a symbol introduced by `93b4f91e9` in `app/src/pane_group/mod.rs`; the fork has no `pending_remote_child_hydrations` (orchestration hydration removed with the surface).

### `f2b401aed` — Support fzf Alt-C across shell integrations (#16094)

Decision: **accept (adapted)**. Retained shell-integration feature building on the fork's ctrl-r/ctrl-t handoff (ported 2026-09-03 from `bf2364bc9`/`7c360f772`): plugin detection now derives fzf/atuin identity from the effective ctrl-r binding and reports `fzf`/`atuin` `shell_plugins` tags (replacing `external_ctrl_r_history`/`external_ctrl_t_file`), ctrl-t handoff keys off the `fzf` tag alone, and a new editable context-gated `alt-c` binding (`ExternalAltCDirectorySearch`) hands off to fzf's directory widget with an ESC-c PTY fallback. Also adds macOS Bash 3 Readline macro binding detection, Fish 4 flagged binding output parsing, the `with_shell_plugins` session test helper, the `Input::is_voltron_open` accessor feeding `VoltronActive` on `TerminalView`'s keymap context, and `ctrl_t_apply_mode()` extraction.

Port: exact three-way patch with test-file paths remapped to the fork's singular naming. Recorded adaptations and omissions:

- `bash_body.sh` detection keeps the fork's ungated structure; upstream's MSYS2-gated body is applied verbatim dedented one level. The ctrl-t detection block is removed as upstream did (fzf tag now covers it).
- `zsh_body.sh` and `fish.sh` hunks applied content-identical to upstream (verified by content diff).
- `workspace/view.rs`: `trigger_external_alt_c_directory_search` follows the fork's ctrl-t shape and omits upstream's `is_readonly_shared_session_active` early-return (shared-session surface removed in this fork). The `EXTERNAL_ALT_C_BINDING_CONTEXT` insertion in `View for Workspace` and the TypedActionView arm match upstream's hunks on the fork's anchors.
- `workspace/action.rs`: the new variant and match arms are added around the fork's local `CreateEnvVarCollection` variant instead of upstream's cloud Drive/Teams variant block.
- `input.rs`: the `is_voltron_open` accessor is ported alone; upstream's surrounding `handle_prompt_suggestions_event`/`is_input_at_top` context is the removed billing/signup surface and stays out.
- `input_test.rs`: the `ctrl_t_binding_is_ineligible_when_shell_widget_handoff_flag_is_disabled` test is deleted as upstream did (flag is default-on in this fork's runtime wiring).
- `view_test.rs`: upstream's HashSet import removal does not apply — the fork file still uses `HashSet` elsewhere; import kept.
- `EXTERNAL_CTRL_T_FILE_PLUGIN_TAG`/`EXTERNAL_CTRL_R_HISTORY_PLUGIN_TAG` constants and all fork references were swapped to the upstream `FZF_PLUGIN_TAG`/`ATUIN_PLUGIN_TAG` scheme by the applied hunks; no fork stragglers remain (searched).

### `c0380ce89` — [APP-5545] Publish metrics from native transcript captures (#15929)

Decision: **reject**. Agent-SDK harness usage reporting: new `usage_reporting.rs`/`harness_persistence.rs`/`transcript_persistence.rs` under `app/src/ai/agent_sdk/driver/harness/`, new `warp_harness_usage` crate, `server_api/harness_usage` publication stack, and telemetry metrics. All fork-absent surfaces (no `agent_sdk`, no `server_api`, no telemetry); the fork's ACP client owns agent execution with no usage reporting.

### `114bacea6` — fix: update rmcp to 2.2.0 to resolve CVE-2026-64684 (#16092)

Decision: **N/A (transitive-only exposure)**. The version pin lives in `crates/mcp/Cargo.toml` (fork-absent crate) with code changes in agent-sdk/MCP transport files that are also fork-absent. The fork's `Cargo.lock` carries `rmcp` 1.6.0 only as a transitive dependency of the crates.io `agent-client-protocol` 0.11.1 crate (used for its MCP-bridging schema types by the ACP client); the fork has no direct rmcp dependency to bump and does not run MCP servers through rmcp. Revisit only if `agent-client-protocol` publishes a semver-compatible fix or the fork ever takes a direct rmcp dependency.

### `a9520c97d` — fix: update js-yaml to 4.3.2 to resolve GHSA-2883-xcg3-v3hh (#15917)

Decision: **N/A**. `crates/warp_graphql_schema/package.json` + `yarn.lock`; the fork removed the GraphQL schema crate.

### `5b9771acf` — Avoid panic when /model targets a closed window (#16102)

Decision: **N/A**. The fix converts an `inline_model_selector_view.update` to `try_update` with `ViewUpdateError::WindowClosed` handling at the `/model` keybinding call site. The fork removed the inline model selector surface (no `inline_model_selector_view`/`InlineModelSelector` anchors in `input.rs`). The `ViewHandle::try_update` framework API itself is already ported (2026-09-02, `97037ec83`).

### `200153270` — Fix vulnerability: CVE-2026-84375 (#16119)

Decision: **N/A (empty commit)**. The commit object contains no file changes (`git diff-tree` empty); the substantive fix is not in the public tree.

### `c4351aee3` — [Chat][MCP] Send the MCP server's Warp-side identity in MCPContext (#15942)

Decision: **reject**. App-managed MCP identity plumbing: `TemplatableMCPServerInstallation.warp_id`, `MCPServerIdentity`/`MCPIntegration` in the agent API `convert_to` path, `crates/mcp` runtime plumbing, and managed-client config parsing. All fork-absent (no `app/src/ai/mcp`, no `crates/mcp`, no `agent/api/` convert modules); MCP configuration belongs to the ACP agent process in this fork. The `crates/warpui/build.rs` hunk is part of the same stacked protocol-pin series and carries no retained behavior.

### `71088ba18` — Report when a cloud agent run has computer use enabled but unavailable (#16114)

Decision: **reject**. Cloud-agent computer-use reporting through `app/src/ai/agent/api.rs` request params and `app/src/server/telemetry/events.rs`; both surfaces fork-absent.

## Verification

Run on `merge/upstream-2026-09-23` (with `SDKROOT` pinned to the Xcode 26.5 SDK per the standing CLT 27 toolchain prereq, `CARGO_PROFILE_DEV_DEBUG=0`):

- `bash -n app/assets/bundled/bootstrap/bash_body.sh` — pass.
- `./script/lint_shellcheck` — pass (0 warning/error diagnostics).
- `cargo nextest run -p warp -E 'test(ctrl_t) | test(external) | test(shell_widget)'` — 46 passed (includes the retained ctrl-t handoff suite).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo check -p warp --all-targets --message-format short` — pass (8 pre-existing lib warnings, all queued-prompts/dead-code categories unrelated to this range).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo fmt -- --check` — pass.
- `cargo build -p warp --all-targets --message-format short` — pass.

Deleted-surface scans (`access token|AuthState|billing|...`, `mcp.*capab|ReadSkill|...`, `target_os = "linux"|...`) show no new hits: remaining matches are the standing allowed categories (code-review/project-context wording, `Weak::upgrade` call sites, retained SSH `ForwardX11=no` config and remote-terminal platform parsing, pre-existing bootstrap ConPTY comments). The `zsh_body.sh` ConPTY comment hits predate this merge (verified against `HEAD~2`).

`cargo clean` run after merge/tag/push.

## Notes

- `200153270` is the first empty upstream commit observed in an audit range; recorded so future range accounting does not miscount it as a missed port.
- The rmcp CVE note above is the standing answer for future rmcp/security-bump commits: the fork's exposure is transitive via `agent-client-protocol`, not a direct dependency.
