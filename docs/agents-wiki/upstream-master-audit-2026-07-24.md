# Upstream Master Audit 2026-07-24

Range under review: `f1547fefc..upstream/master` (166 commits)

Previous audited upstream tip: `f1547fefc Add bottom padding below TUI footer/prompt (CODE-1878) (#13850)`

Current upstream tip detected: `940c50594 Build authenticated recording viewer URL in client (#14210)`

Total upstream commits in this incremental range: 166

Status: triage complete. A focused set of retained terminal/shell-integration/vim/editor/macOS/dependency fixes were ported or adapted. The bulk of the range is TUI-only (`crates/warp_tui/`, absent in this fork), computer-use recording work (`crates/computer_use`, removed surface), cloud/orchestration/agent-SDK (`run_agents`, `orchestration`, `agent_sdk`, GraphQL, managed secrets), onboarding, feature-flag promote/telemetry commits for flags the fork does not carry, or Linux/Windows/WASM platform code. Many `report_error!` → `log::error!`/`log::warn!` Sentry-noise commits are N/A because the fork removed the entire `report_error!`/Sentry infrastructure.

## Ported Or Adapted

- `0b07f7c2e` Adapted the dynamic zsh glitch-width prompt stripping fix. zsh's explicit-width constructs (`%n{...%}`, `%G`) emitted by dynamic prompt elements (e.g. `$(git_prompt_info)` under `PROMPT_SUBST`) caused command-grid column drift. The fix backs up `$PROMPT` into `$_WARP_RAW_PROMPT` and installs a live `_warp_stripped_prompt` function that strips glitch constructs on every render when `PROMPT_SUBST` is on, or strips statically at precmd when off. Replaces the fork's earlier static `WARP_STRIPPED_ORIGINAL_PROMPT` pre-strip. The `header_grid.rs` preexec command-grid reconciliation machinery (`reconcile_command_grid_with_preexec_command`, `has_leading_prefix_redraw_artifact`, etc.) was removed along with the upstream commit, since the dynamic strip eliminates the root cause. Adapted: resolved two `zsh_body.sh` conflict regions (the fork's `WARP_STRIPPED_ORIGINAL_PROMPT` block vs the upstream `_WARP_RAW_PROMPT` block); the upstream `_warp_stripped_prompt` function auto-merged cleanly. Also removed the obsolete `test_multiline_preexec_reconciles_command_grid_redraw_prefix` test from the fork's `block_test.rs` (upstream removed it from `block_tests.rs`; the fork's singular test filename meant auto-merge did not catch it).
- `a9cb364a5` Adapted the `WARP_DATA_PROFILE` macOS config-dir scoping fix. `macos_config_dir_name()` was not profile-scoped on macOS, so every development profile of a channel shared one `settings.toml`. Extracted a pure `macos_config_dir_name_for(channel, data_profile)` helper that appends the `-{profile}` suffix, mirroring the existing `warp_home_config_dir_name()` pattern. Adapted: the fork uses `WARPLY_CONFIG_DIR` (not `WARP_CONFIG_DIR`) and groups `Channel::Stable | Channel::Oss` together; the upstream's reintroduced `warp_home_skills_dir`/`warp_home_mcp_config_file_path` helpers were dropped (skills/MCP removed in this fork). The unit test was adjusted to assert `.warply` base names instead of `.warp`.
- `47234cc6d` Ported the vim `d%`/`c%`/`y%` code-editor fix. The operator-pending motions for matching brackets were dropped by a catch-all `_ =>` no-op arm in `vim_handler.rs`. Added explicit match arms for `JumpToMatchingBracket`/`JumpToUnmatchedBracket` and a `vim_select_for_matching_bracket` helper that extends the selection end by one to include the matching bracket char. The three upstream regression tests were manually appended (the auto-merge pulled in unrelated `set_viewport_lines`-dependent tests from the upstream base that do not exist in the fork).
- `730a4acc0` Ported the vim `gg` count fix. `Ngg` now jumps to line N like `NG` instead of always jumping to line 1. The `gg` mapping in `crates/vim/src/vim.rs` checks for a pending count and returns `JumpToLine(n)`. The new `crates/vim/src/vim_tests.rs` (333 lines) was added cleanly.
- `08fdf52fb` Ported the resizable-bounds small-window fix. Clamped the code-review comment-list and suggestions-mode-menu resize bounds to a `.max(100.0)` minimum so tiny windows don't produce zero/negative resize targets.
- `c86f67db3` Ported the stale classic completions fix. Classic Completions now close the stale suggestion menu when the user edits the trigger text, while still allowing Tab cycling (which rewrites the buffer) to keep its menu open. The fork's `input_test.rs` (singular) received the regression test.
- `b6d242037` + `06eedd6fc` Regenerated the diesel `2.3.9` → `2.3.10` and h2 `0.4.12` → `0.4.15` security/bug-fix bumps via `cargo update --precise` (the upstream commits only touch `Cargo.lock`).
- `7867010a0` Ported the render-test logging idempotency fix. `warp_editor`'s render-test `init_logging` helper now uses `try_init()` instead of `init()`, ignoring `SetLoggerError` when another test already installed the global logger.

## Deferred (Retained But Requires Dedicated Port)

- `50853a9b9` Cancel repo metadata tree walks on teardown is a retained repo-metadata lifecycle fix (1473 lines, 12 files) adding async cancellation, per-build generation guards, and duplicate-load coalescing. Deferred because of the breadth and test-file divergence (`.agents/skills` path references, `remote_codebase_indexing` feature gates). Port in a focused follow-up.

## Rejected Or Not Applicable

The 166-commit range is dominated by removed-surface or TUI-only work. Key decisions:

### TUI-only (N/A — `crates/warp_tui/` absent)

`fdebae8dc`, `49230240d`, `79a0e04c8`, `3fa45a48f`, `917ba672d`, `9edb612da`, `d7a44d435`, `0f9b32152`, `c7c66040a`, `49e9cba37`, `693f59feb`, `b1dd46f6c`, `7da3cb636`, `983a97cbe`, `0167b43a8`, `4207f5667`, `446c2f331`, `7d2304d17`, `6ab529be0`, `8d2759cff`, `a41e5846b`, `983a97cbe`, `a6ebba40e`, `074e59533`, `3993cf544`, `693f59feb`, `0fb6c02dc`, `e2c823bb9`, `132800ecb`, `4b77c4de2`, `b9226ffb0`, `1dcbfdf46`, `8d9f2e08a`, `ce5a0c5ef`, `615e47606`, `da50fd586`, `8b5291608`, `f7e9d2830`, `cc5199743`, `0d89f6c72`, `d31f7a636` (TUI binary rename detection — `command_is_warp_tui` absent), `8e61f9ced`, `7e4da0ff8`, `f74bfd97b`, `05e63c60d`, `1d9be246c`, `f48094345`, `7edf88f82`, `3dd6ea882`, `76176405d`, `b4199bb35`, `b4d011ccf`, `1d70a32b8`, `d02e14736`, `959e78b2d`, `a0d589460`, `a0d0dc83c`, `d21702711`, `c4cc7be94`, `7491c1156`, `031f39614`, `43a41099d` (cmov bump for TUI), `0b422baa7`.

### Computer-use recording (Reject — removed `crates/computer_use` + agent SDK)

`f99a900fe`, `59bb73fa3`, `fdbb7b1c6`, `b4d011ccf`, `ba38f12a3`, `88fe7be3b`, `d67694cfe`, `cd8951123`, `cd6347727`, `8c55ad2b8`, `402bb953a`, `940c50594`, `4d2de5ea4`.

### Cloud / orchestration / agent-SDK / managed secrets (Reject — removed)

`e24f75b21`, `524839aa1`, `0e3f9fb98`, `11e217e9d`, `3de1d78fe`, `af891a4ef`, `eeecb6744`, `6f24ea230`, `b4191bb35`, `05026636f`, `1388531b1`, `1dcbfdf46`, `10141bf51`, `8346c134d` (orchestration runners capability — `agent/api/impl.rs` absent), `8d7881159`, `a6b677247` (shared object limit banner — cloud sharing surface absent), `41b922e01` (staging IAP cache — cloud auth absent).

### Auth / onboarding / billing (Reject — removed)

`2e225aa98`, `8b4b5a0d7`, `5b8a3758e`, `1df06fe5a` (account-first onboarding), `78364bffa` (Warp Agent CLI device auth URLs), `10f5ab483` (Warp Agent CLI download endpoints).

### MCP / skills (Reject — removed)

`d34aaf06e` (`/fast-forward` GUI+TUI — MCP/skills), `995ef1393` (common skills lock), `b89c08b37` (TUI setup migration skill).

### Telemetry / Sentry (N/A — `report_error!`/Sentry removed)

`e6aaaf9b8`, `2eb079f69`, `3ff9e3ccc`, `ad45c3089`, `a1abe3270`, `97f441bc4`, `e3d4cd663` (flex — already `log::error!` in fork), `12dde64ee`, `faba5dee1` (LLM cache — `MODELS_BY_FEATURE_CACHE_KEY` absent), `cb085fb89` (SQLite Sentry dedup — Sentry removed), `eca8c1a60`, `5a25993a7` (AgentTipShown staging analytics — `server_api` absent), `8a30a37ff` (oh-my-pi telemetry).

### Voice input (Reject — removed)

`67960ea54` (`voice_input`/`voice_transcriber` absent).

### Gemini Enterprise / cloud credentials (Reject — removed)

`7bebcd7c5` (GEAP credential failures — `geap_credentials.rs` absent).

### Linux / Windows / WASM platform (Reject — macOS-only)

`ebe3d363e` (Wayland IME — winit windowing absent), `f7ecf6657` (OSC 7 Windows drive-path — OSC 7 cwd parsing absent), `b231c358f` (Rust 2024 extern block — `crash_reporting/` absent), `60ec6c7f6`, `49e9cba37`, `c4cc7be94`, `7491c1156` (Linux TUI bundling), `abea51cd1` (Rust 2024 edition migration — 1893-file mechanical change, deferred), `0017f3059` (wasm bundle — `script/wasm/` absent).

### Feature flags the fork does not carry (N/A)

`054a5e4b3` (OscHyperlinks Preview — deferred flag), `88fe7be3b`/`d67694cfe` (computer-use flags), `2e225aa98` (account-first onboarding flag), `917ba672d`/`8e61f9ced`/`7e4da0ff8` (TUI NLD flags).

### Specs / CI (Reject)

`f005fba2c`, `5e86de433`, `917ba672d`, `b4199bb35`, `d21702711`, `c3ba49419`, `4b77c4de2`, `e7c409bad`, `a7a8f1ec7` (serde_with 3.x — blocked by `computer_use` crate's serde_with 2.x requirement), `60ec6c7f6`, `8cef29407`, `f7586a618`, `187916fe8`, `d6207523f` (`.github/` CI), `1713b8155` (`build_cache` crate absent).

### Other N/A (code paths absent in fork)

`2356ddab2` (settings subpage search — fork's `SettingsSection` has no AI/Code subpages), `f0e3db9cd` (conversation stuck loading — cloud conversation load-state infrastructure removed), `c7c66040a` (TUI conversation filtering — ambient/cloud-agent provenance fields absent), `dbaf6d50d` (PR url in InputContext — `AIAgentContext::Repository`/`PullRequest` variants absent), `f7e9d2830` (`/version` command — TuiOnly surface), `8ee421691` (git branch chip — 6-region conflict in `display_chip.rs`, cosmetic UI alignment deferred), `c353a2a41` (wgpu 30.0.0 — fork uses Metal, not wgpu).

## Verification

Commands run after porting:

- `cargo check -p warp --all-targets --message-format short` — passed (no errors).
- `cargo check --workspace --all-targets --message-format short` — passed (no errors).
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed, 2439 skipped.
- `cargo nextest run -p warp -E 'test(vim) | test(completions) | test(classic)'` — 202 passed.
- `cargo nextest run -p vim` — 57 passed (covers the new `gg` count tests).
- `cargo nextest run -p warp_core paths` — 9 passed (covers the new `test_macos_config_dir_name_scopes_to_data_profile`).
- `cargo nextest run -p warp -E 'test(test_vim_d_percent) | test(test_vim_c_percent) | test(test_vim_y_percent)'` — 3 passed (new d%/c%/y% tests).
- `cargo nextest run -p warp --no-fail-fast` — 2591 run, 2582 passed, 9 failed, 3 skipped. The 9 failures are identical to the pre-merge `main` baseline (`test_plan_markdown_content_preserves_copyable_structure`, `test_focused_pane_is_synchronized_with_application_focus`, `test_tokenizer_warp_special_chars`, `test_smart_selection_override`, `test_smart_selection_in_multiple_blocks`, `test_smart_selection_in_single_block`, `test_find_url_omits_trailing_periods`, `test_secrets_serialization`, `inline_agent_view_persists_across_transfer_takeover_for_monitored_long_running_command`); confirmed pre-existing on `main` and unrelated to these ports.
- Deleted-surface scan for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — no restored surfaces (all hits are `Arc::upgrade()`, doc comments, or retained local naming).
- Deleted-surface scan for `mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill` — no restored surfaces.
- Deleted-surface scan for `target_os = "linux"|target_os = "windows"|cfg(windows)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb` — only retained SSH `ForwardX11=no` config strings.
- Dependency changes: diesel 2.3.9 → 2.3.10, h2 0.4.12 → 0.4.15 (security/bug-fix bumps only; no cloud/API/reporting crates reintroduced).
