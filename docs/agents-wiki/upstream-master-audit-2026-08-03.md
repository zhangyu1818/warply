# Upstream Master Audit 2026-08-03

Range under review: `9dcef6a88..upstream/master` (69 commits)

Previous audited upstream tip: `9dcef6a88 [REMOTE-2400] Hide API key values in CLI help (#14532)`

Current upstream tip detected: `956ae6be4 Keep TUI zero state stable during shell startup (#14632)`

Total upstream commits in this incremental range: 69

Status: triage complete. A focused set of retained warpui_core/repo_metadata/build/macOS-codesign fixes were ported or adapted. The bulk of the range is TUI-only (`crates/warp_tui/`, absent in this fork), computer-use recording (`crates/computer_use`, removed surface), cloud/orchestration/agent-SDK (`agent_sdk`, `ambient_agents`, GraphQL, managed secrets, Grok OAuth), auth/onboarding/billing/credits, MCP app-config/skills, telemetry (N/A — infrastructure removed), voice input, Linux/Windows/WASM platform, and `.github` CI.

## Ported Or Adapted

- `80a4e4654` Ported the cross-window terminal-lag fix in `warpui_core`. A terminal with an actively running process became laggy after dragging its tab into a different window because `TaskCallback::ViewFromStream` captured the original `window_id`, which became stale after a cross-window transfer. Removed the explicit `window_id` field from `ViewFromStream` and looked up the current window from `view_to_window` at item-delivery and completion time. Adapted: fork uses `as_any_mut()` (not upstream's `as_mut()`) and edition 2021, so the completion branch uses nested `if let` instead of upstream's `let`-chain (`if let .. && let ..`). Verified with 12 existing `transfer_view` tests.
- `6465abf58` Adapted the repository-watcher directory-symlink fix. Upstream added `should_watch_repo_directory` / `repo_watch_filter` with an `is_within_symlink` descend check, but the fork's `repo_metadata` watcher architecture differs: fork uses notify's single-argument `WatchFilter::with_filter` (a descend predicate) and has no `should_watch_repo_directory` wrapper. Added `is_within_symlink(path, repo_root)` to `entry.rs` and wired it into both watcher registration sites (`watcher.rs::start_watching_directory` and `local_model.rs::add_repository`), capturing the repo root in the filter closure so directory symlinks below the root (e.g. Nix `result -> /nix/store/...`) are pruned during recursive watch registration. Added 3 `#[cfg(unix)]` regression tests for symlink pruning, symlinked repo root, and symlinks above repo root.
- `baee9195f` Ported the `app/build.rs` path-API refactor. `get_build_profile_name()` now uses `PathBuf::from(env::var_os("OUT_DIR"))` with `ancestors().nth(3)` + `Path::file_name()` instead of string-splitting on `MAIN_SEPARATOR`, tolerating trailing separators. Added `PathBuf` to the existing `use std::path::Path` import.
- `179923ede` Ported the macOS codesign timestamp-retry fix into `script/macos/bundle`. Added `codesign_with_retry()` with bounded exponential backoff (default 4 attempts) and applied it to the timestamped app and DMG signing call sites so transient Apple timestamp-service failures no longer abort the release after compilation. Dev self-signing (ad-hoc, no timestamp) is unaffected.

## Rejected Or Not Applicable

The 69-commit range is dominated by removed-surface or TUI-only work. Key decisions:

### TUI-only (N/A — `crates/warp_tui/` absent)

`956ae6be4`, `49e9e2f3f`, `b462e0132`, `44f112cc0`, `dd793dfca`, `a95e6e541`, `eda008544`, `08ad6e8ab`, `d5b1e6998`, `53411ef0a`, `0fb1baea1`, `73529d1d6`, `79015984c`, `b8030efd1`, `b2f0b285d`, `014b46184`, `6cb30f3ec`, `4431b15ff`, `a1eeff960`, `9c6099477`, `7cbb22d5c` (TUI + Linux voice input), `cd45ebb6f`, `a3a06f234`, `bf56c3c18`, `f74bfe73a`, `620e8f388`, `5aaadb20e`.

Also N/A due to absent `warpui_core/src/runtime/` and `warpui_core/src/elements/tui/` subdirectories: `c1986e537` (shift+enter in runtime/mod.rs), `53411ef0a` (terminal_background + terminal_probe).

### Computer-use recording (Reject — removed `crates/computer_use` overlay/recording + agent SDK)

`52061d2ae`, `d84b4e3fc`, `306320a59`.

### Cloud / orchestration / agent-SDK / managed secrets / Grok OAuth / GEAP (Reject — removed)

`88d661573` (agent_sdk driver), `16ec6d4d7` (agent_sdk driver terminal), `f7a19b3e4` (orchestration tooltip), `8b7055e8b` (TUI provider cost footer + cloud agent), `dacff5e3d` (Grok OAuth — `crates/ai/src/grok_subscription/` absent), `ca1cd4303` (agent glyph model picker — fork `llms.rs` is a 1-line re-export), `41f91c6de` (server-authoritative AI credits), `89af53603` (TuiCostTransparency flag — absent in fork).

### Auth / onboarding / billing / credits / Teams (Reject — removed)

`2b4a66f81` (startup API key validation — `app/src/auth/` absent), `0d84c1280` (TUI logout — `app/src/auth/` absent), `89f0eaf63` (TUI out-of-credits gating), `a8941be03` (free-plan credit purchases), `53d770b55` (billing deeplink — `cloud_agent_capacity_modal` absent), `ddfe0b4cc` (channel_versions crate absent), `6de238814` (SSO-link onboarding blocker), `2ede88338` (TUI onboarding markers).

### MCP / skills (Reject — removed)

`4bff3ba0e` (TUI MCP catalog), `dc0c37a9a` (gui-settings-ui skill — absent), `867347a1e` (oz-platform skill — `resources/bundled/skills/` absent).

### Telemetry (N/A — infrastructure removed)

`8349ddb43` (TUI telemetry).

### Settings / cloud features (Reject — removed surfaces)

`fe8138bce` (third-party CLI agents settings — fork ai_page is ACP-only single-widget), `ddadceeaa` (Knowledge settings — Suggested Rules / Warp Drive context removed), `f79df8d9d` (cloud conversation storage setting — `settings/privacy.rs` absent).

### Linux / Windows / WASM platform / Windows installer CI (Reject — macOS-only)

`a24741b52`, `c917be01c`, `01f270960`, `d9ed47239`, `1535d3905` (Windows installer CI).

### Feature flags (N/A — flags absent in fork)

`fa70ad068` (ContextWindowUsageBreakdown absent), `3a7d18971` (OscHyperlinks absent).

### Skill docs (Deferred — optional, not code)

`59cbf2c4f` (testing guidelines for `gui-integration-test` [absent], `review-pr-local`, `rust-unit-tests`). Optional skill-document refinement; not ported as it is not a code fix.

## Verification

Commands run after porting:

- `cargo check -p warpui_core -p repo_metadata --all-targets --message-format short` — passed.
- `cargo check -p warp --all-targets --message-format short` — passed (includes `app/build.rs` build script).
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warpui_core -E 'test(transfer_view)'` — 12 passed.
- `cargo nextest run -p repo_metadata` — 49 passed, 3 skipped (includes 3 new `is_within_symlink` tests).
- `bash -n script/macos/bundle` — syntax OK.
- Deleted-surface scan for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — no restored surfaces (all hits are `Arc::upgrade()`/`WeakViewHandle::upgrade()` calls).
- Deleted-surface scan for `mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill` — no hits.
- Deleted-surface scan for `target_os = "linux"|target_os = "windows"|cfg(windows)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb` — only retained SSH `ForwardX11=no` config strings and a `ConPTY` explanatory comment in `zsh_body.sh`.
