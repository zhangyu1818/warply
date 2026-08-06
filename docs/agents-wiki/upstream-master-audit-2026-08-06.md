# Upstream Master Audit 2026-08-06

Range under review: `c8a166b6c..upstream/master` (22 commits)

Previous audited upstream tip: `c8a166b6c Only accent an onboarding credit pack when the credit option is chosen (#14712)`

Current upstream tip detected: `5f8734329 add /manage-billing and /upgrade slash commands (#14744)`

Total upstream commits in this incremental range: 22

Status: triage complete. A focused set of retained terminal/completion/markdown fixes were ported. The bulk of the range is TUI-only (`crates/warp_tui/`, absent in this fork), cloud/orchestration/agent-SDK/billing/credits/onboarding, MCP app-config/skills, cloud shared-session copy-link (removed surface), wasm/web, and Docker CI infra.

## Ported Or Adapted

- `1b65a8b9a` Ported the Tab path completion symlink fix for remote/SSH sessions. `ls_script_for_dir` in `app/src/completer/mod.rs` now uses `find -L` (follow symlinks) on both the directory and file invocations, so a symlink to a directory is classified as a directory and offered as a `cd` completion (trailing separator), matching standard terminal behavior. Also folded in the upstream cleanup of the trailing-whitespace on the `cd {escaped_dir} &&` line. Added regression test `test_session_context_follows_symlinked_directories_remotely` (`#[cfg(unix)]`) covering directory/file symlinks via the real remote `find` script.
- `e7c5febe1` + `ccc6bb185` Bumped `warp-command-signatures` from `ec1ae8e8` → `4990fa1d` (the fork's previous pin was an ancestor of upstream's `29cd61c3` base for `e7c5febe1`). Picks up the `journalctl` completion spec (86 options + 6 generators) and the `pkill`/`killall` process-name generator. The bump also pulled `serde_with` 3.19 → 3.21 (security fix GHSA-7gcf-g7xr-8hxj from command-signatures `7068528`) and added `bs58 v0.5.1` as a new transitive dep; both satisfy the existing `^3.18.0` requirement from `agent-client-protocol-schema`. `Cargo.lock` updated via `cargo update -p serde_with@3.19.0 --precise 3.21.0`.
- `5e49861d7` Ported the numeric CSS font-weight preservation fix for pasted rich text. `Styling` in `html_parser.rs` now tracks `weight: Option<CustomWeight>` instead of a `bold: bool`. Added `CustomWeight::from_css_numeric(value: i32)` in `weight.rs` that clamps to 1..=1000, rounds to the nearest hundred, and maps to the named step (400 → `None`). `bold`/`bolder` keywords and `<b>`/`<strong>` still map to `Bold`; `normal`/`lighter` clear the weight. Adapted: fork uses the singular test-file convention (`weight_test.rs`, `html_parser_test.rs`), so the new test module path is `#[path = "weight_test.rs"]`. Updated two existing `font-weight:600` test expectations from `bold` → `weighted(.., Some(CustomWeight::Semibold))`, and added `test_parse_numeric_font_weights_preserve_custom_weight` + `test_parse_keyword_font_weights`.

## Deferred

- `935f86b94` Deferred the TUI shell-completion alignment refactor. It extracts the GUI Tab-completion decision logic (`single_prefix_suggestion` / common-prefix insertion / `NoAction`) into shared `warp_completer` types (`PreparedSuggestion`, `ExplicitTabCompletion`, `prepare_for_query`, `explicit_tab_completion`) so the TUI can reuse them. The fork has no TUI, and the GUI behavior before and after the refactor is semantically identical — this is a pure enable-TUI-reuse refactor (~250 lines across `input.rs`, `input_suggestions.rs`, `warp_completer`) with no user-visible GUI bug fix. Revisit if a future retained-completion fix lands on top of these types.

## Rejected Or Not Applicable

The 22-commit range is dominated by removed-surface, TUI-only, or absent-in-fork work. Key decisions:

### TUI-only (N/A — `crates/warp_tui/` absent)

`935f86b94` (deferred — see above; the `crates/warp_tui/` portions are N/A), `9c0cb4566`, `9b6572261`, `05f986c81`, `5f8734329` (the `crates/warp_tui/` + `tui_export.rs` portions).

### Cloud / orchestration / agent-SDK / billing / credits / onboarding (Reject — removed)

`5f8734329` (`/manage-billing` and `/upgrade` slash commands + `user_workspaces` billing helpers — billing/upgrade surface, removed), `fe6b6755c` (`run_agents` repo-qualified child skill specs in `ai/orchestration/remote_child.rs` — orchestration + skills, removed), `668f4f371` (Warp Agent CLI launch modal + onboarding banner PNG — cloud-agent/onboarding, removed), `06c67ba6f` (Team.settings GQL + `gql_convert`/`workspace.rs`/`teams_page.rs` — GraphQL/Teams, removed), `d2cf270b8` (post-checkout onboarding advance — onboarding/billing, removed), `fae567e91` (plan + credit packs onboarding card — onboarding/billing, removed), `2ac05867f` (deferred startup API key validation in `ai/agent_sdk/mod.rs` + `warp_server_auth/auth_state.rs` — agent-SDK/auth, removed), `632af8a44` (workspace admin panel from Billing & Usage CTA — billing/Teams, removed).

### MCP / skills (Reject — removed)

None in this range directly. (`fe6b6755c` touches skill-spec resolution but is classified under orchestration above.)

### Cloud shared-session copy-link (N/A — removed surface)

`82fbde288` (copy-link menu item during cloud agent session setup — touches `terminal/shared_session/manager.rs`, `view/shared_session/view_impl.rs`, `CopySharedSessionLinkFromTab`, all absent in the fork; cloud shared-session sharing is a removed surface).

### WASM / web (Reject — macOS-only)

`2e6d78d7e` (`fix(wasm)` conversation-details pane-header reopen button on web — wasm target, removed).

### macOS Dock bouncing during bundled-CLI runs (Reject — depends on removed bundled-CLI surface)

`6eff52f9d` (stop Dock icon bounce during bundled-CLI `oz`/`oz-dev` runs). Adds `platform/mac.rs::mark_process_as_background_only` and gates the macOS Dock/menu-bar/autoupdate setup behind `!launch_mode.is_headless()` + a new `AppExecutionMode::can_autoupdate()` guard. The fork's `platform/mod.rs` is empty (wasm/windows branches already removed) and the fork has no bundled `oz`/`oz-dev` CLI surface; the `can_autoupdate` method does not exist. The headless-launch path this protects is part of the removed cloud-agent CLI surface, so the whole change is rejected rather than partially ported.

### CI / Docker infra (Reject — absent in fork)

`437721e5b`, `f81da1ea9`, `e09973d71` (`.github/workflows/publish-agent-dev-image.yml` — Docker Hub OIDC + agent-dev image publish workflow; file absent in fork).

## Verification

Commands run after porting:

- `cargo check -p markdown_parser --all-targets` — passed.
- `cargo check -p warp -p warp_completer --all-targets` — passed.
- `cargo check --workspace --all-targets` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(completer)' -p warp_completer` — 126 passed (including the new `test_session_context_follows_symlinked_directories_remotely`).
- `cargo nextest run -p markdown_parser` — 154 passed (including 4 new `weight::tests::*` and 2 new `html_parser` font-weight tests, plus 2 updated `font-weight:600` expectations).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- Deleted-surface scan for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — no restored surfaces (all hits are `Arc::upgrade()`/`WeakViewHandle::upgrade()` calls and tokenizer JSON vocabulary).
- Deleted-surface scan for `mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill` — no hits.
- Deleted-surface scan for `target_os = "linux"|target_os = "windows"|cfg(windows)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb` — only retained SSH `ForwardX11=no` config strings and a `ConPTY` explanatory comment in `zsh_body.sh`.
