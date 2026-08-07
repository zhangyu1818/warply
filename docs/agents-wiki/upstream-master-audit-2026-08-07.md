# Upstream Master Audit 2026-08-07

Range under review: `5f8734329..upstream/master` (9 commits)

Previous audited upstream tip: `5f8734329 add /manage-billing and /upgrade slash commands (#14744)`

Current upstream tip detected: `06e4b74a4 [NLD] Classify \`warp\` as a shell command, not a prompt (#14783)`

Total upstream commits in this incremental range: 9

Status: triage complete. A focused set of retained NLD, macOS windowing, and completion-spec fixes were ported. The remainder is TUI-only, onboarding/billing, and pricing promotion (removed surfaces).

## Ported Or Adapted

- `06e4b74a4` Ported the NLD fix that classifies `warp` (and `warp …` invocations) as a shell command instead of a natural-language prompt. `ONE_OFF_SHELL_COMMAND_KEYWORDS` in `crates/input_classifier/src/util.rs` now includes `"warp"`, matching the existing `claude` / `codex` / `gemini` precedent. Adapted: the fork's keyword set was missing `agy` (Antigravity CLI, upstream `84b11bb1`) and `omp` (oh-my-pi CLI, upstream `8a30a37ff`) from earlier un-ported upstream commits, so both were folded into this single addition to bring the set current with upstream. Both NLD code paths (the ONNX pre-model gate via `is_one_off_shell_command_keyword`, and the heuristic fallback via `is_likely_shell_command`) read this shared set. Added a `warp agent run` case to the existing `test_is_likely_shell_command_one_off_keyword_short_circuits` test in `util_tests.rs`, matching the fork's single-function test convention.
- `64e3cd474` Ported the macOS Option-click green-button zoom fix verbatim. In `crates/warpui/src/platform/mac/objc/window.m`, `WarpWindow -sendEvent:` now marks standard-window-button mouse-downs as native chrome (`_leftMouseDownStartedInNativeWindowChrome = YES`) and routes them through `[super sendEvent:event]` instead of calling `[windowButton mouseDown:event]` directly, so Option-click on the green traffic-light button performs zoom instead of entering fullscreen.
- `7a6044bd5` Ported the Quake mode (dedicated hotkey) window multi-monitor focus fix verbatim. Extracted the post-activation re-key logic into a shared `activate_app_and_focus_window(window)` static helper in `window.m`, and called it from both `positionPinnedPanel` (first-open path) and `show_window_and_focus_app` (re-show path), so the hotkey panel reliably becomes key regardless of which screen the main Warp window is on. `positionPinnedPanel` now also orders the panel front before activating. The old inline observer block in `show_window_and_focus_app` was removed in favor of the shared helper.
- `733546102` + `c11b6b98f` + `c553a671a` Collapsed the three sequential command-signatures bumps (`4990fa1d` → `4094b657` → `5e08807c` → `fe352669`) into a single bump from the fork's previous pin `4990fa1d` straight to `fe3526693fe4ea3dc208ee5ef892b3aad2679af6`. Picks up the `vagrant`, OpenSSL, `yc` (Yandex Cloud CLI), `tcpdump`, bun package.json script, git worktree name, and Justfile recipe completion specs. The fork's `Cargo.lock` was updated by editing only the two `source` lines for `warp-command-signatures` and `warp-completion-metadata`; a `cargo update -p`-driven re-resolve was avoided because it incidentally re-resolved unrelated transitive crates (`windows-sys 0.52.0` → `0.59.0`). Verified that `4990fa1d..fe352669` has an empty `Cargo.toml` diff in the command-signatures repo, so no other lockfile entry legitimately changes.

## Rejected Or Not Applicable

### TUI-only (N/A — `crates/warp_tui/` absent)

`7c80cd5a3` (reuse streaming transcript height measurements in `crates/warp_tui/` — TUI-only, absent in this fork).

### Onboarding / billing / credits (Reject — removed)

`6b8ffecd0` (gate onboarding credit-pack option on a server experiment arm — touches `app/src/auth/login_slide.rs`, `app/src/server/experiments/`, `crates/onboarding/`, `crates/warp_graphql_schema/` — all absent in the fork; onboarding/billing/experiment surface, removed).

### Pricing promotion (Reject — removed)

`6e31fe434` (server-authored Fable and Opus promotion — touches `app/src/ai/pricing_promotion.rs`, `app/src/pricing/`, `crates/graphql/src/api/billing.rs`, `crates/onboarding/` — all absent in the fork; billing/pricing promotion surface, removed).

## Verification

Commands run after porting:

- `cargo fmt -- --check` — passed.
- `cargo check -p input_classifier -p warpui --all-targets --message-format short` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p input_classifier --all-features` — 7 passed (including the extended `test_is_likely_shell_command_one_off_keyword_short_circuits` covering the `warp` keyword).
- `cargo nextest run -p warp -E 'test(completer)' -p warp_completer` — 126 passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- Deleted-surface scan for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` over the changed files — only `bert_tiny_tokenizer.json` vocabulary hits (allowed).
- Deleted-surface scan for `mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill` over the changed files — no hits.
- Deleted-surface scan for `target_os = "linux"|target_os = "windows"|cfg(windows)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb` over the changed files — no hits.
- `Cargo.lock` diff verified to contain only the two command-signatures `source` lines; no unrelated transitive dependency churn.
