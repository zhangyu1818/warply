# Upstream Master Audit 2026-08-08

Range under review: `06e4b74a4..upstream/master` (14 commits)

Previous audited upstream tip: `06e4b74a4 [NLD] Classify \`warp\` as a shell command, not a prompt (#14783)`

Current upstream tip detected: `d78ced530 Avoid Git probes for filesystem-only directory watchers (#14830)`

Total upstream commits in this incremental range: 14

Status: triage complete. Two retained macOS-windowing and bootstrap fixes were ported. The `d78ced530` repo_metadata change was triaged as not applicable (the bug it fixes is absent in this fork). The remainder is TUI-only, cloud/billing/Teams/agent-SDK/orchestration, MCP/skills, Grok/model-picker (removed `/model` surface), WASM, and Docker CI infra.

## Ported Or Adapted

- `e5d0e004a` Ported the macOS fullscreen window-corner fix verbatim. Added `WindowManager::window_corner_radius_for_window(window_id)` in `crates/warpui_core/src/windowing/state.rs` that returns square corners (`Radius::Pixels(0.)`) when the given window is fullscreen, falling back to the existing `window_corner_radius()`. The root workspace container in `app/src/workspace/view.rs::Workspace::render` now calls the per-window variant instead of the global one, so a fullscreen window no longer draws rounded notches that a square background pokes through. The ~15 modal-scrim call sites intentionally keep the global method, matching upstream scope. Per-window (not `is_active_window_fullscreen()`) so a windowed Warp window keeps rounded corners while another window is fullscreen on a multi-display setup.
- `f3b156487` Ported the bootstrap prebuilt-binary speedup. `script/install_cargo_binstall` now installs cargo-binstall `v1.18.1` from its GitHub release archive, verified against a SHA-256 pinned in the repo (four archives: macOS universal, x86_64/aarch64 linux-musl, x86_64 windows-msvc) before extraction/execution; mismatches and unpinned platforms abort. `script/install_cargo_release_deps` installs `cargo-about@0.8.4` via binstall and explicitly installs cargo-binstall first (needed under `--no-build-deps`). `script/install_cargo_bundle` now installs `cargo-bundle@0.11.0` via binstall (0.11.0 contains the `ae4c76e` `--profile` + `CARGO_BUNDLE_SKIP_BUILD` fixes `script/macos/bundle` depends on). `script/macos/bootstrap` calls `install_cargo_bundle` instead of a raw `cargo install --git --rev`. Adapted: the fork's `install_cargo_bundle` already used the git-rev install (not the GCS zip that upstream's parent carried), so the fork was moved straight to the binstall target body rather than diffing against the GCS-zip version.

## Not Applicable (bug absent in fork)

- `d78ced530` Avoid Git probes for filesystem-only directory watchers (#14830). Triaged as **not applicable** rather than ported. The PR fixes a startup process storm where filesystem-only `DirectoryWatcher` consumers launched `git rev-parse --symbolic-full-name @{u}` (upstream-ref probes) for every watched directory. The entire storm mechanism is **absent in this fork**: `Repository.tracked_remote_ref`, `Repository::refresh_tracked_remote_ref`, `Repository::resolve_tracked_remote_ref`, the `RepositoryUpdate::remote_ref_updated` field, and every `git rev-parse`/`symbolic-full-name` process spawn were removed when the cloud/git-upstream metadata surface was deleted (verified: no `Command::new("git")`, `rev-parse`, `symbolic-full-name`, or `@{u}` hits exist anywhere in `crates/repo_metadata/` or its consumers). The PR's second component — the `RepositoryWatchMode::{FilesystemOnly, GitRepository}` API refactor that splits filesystem vs git watch paths and filters git-only events from filesystem-only subscribers — is a broad refactor of every `start_watching` call site plus a large test suite carried only to gain a minor noise-filtering benefit, since the primary performance bug does not exist here. Per "add complexity only when truly necessary", the refactor was not ported. The fork's `watcher.rs` still routes `.git` internal events (`.git/HEAD`, `.git/index.lock`, commits) via `find_repos_for_git_event` to all subscribers regardless of mode; if a future retained change builds on these types, revisit this decision then.

## Rejected Or Not Applicable

### TUI-only (N/A — `crates/warp_tui/` absent)

`7c80cd5a3`-class work in this range: `da449ab22` (make TUI startup non-blocking on staging IAP — touches `LaunchMode::Tui` + `IapManager`/`warp_server_auth` auth-state, both removed-surface; the `startup_auth_is_non_blocking` gate is TUI-only and the IAP/auth machinery is absent), `bf508d24e` (`/connect-grok` TUI slash command — `crates/warp_tui/` portions N/A; the app-side `static_commands` additions restore a Grok-OAuth surface absent in the fork).

### Cloud / billing / Teams / agent-SDK / orchestration (Reject — removed)

`e62ec07c6` (Billing & Usage split Team vs Workspace bonus-credit cards — `billing_and_usage_page*`, `gql_convert`, `teams_page`, GraphQL billing schema, removed), `c34f05b6e` (team switcher membership filtering — `user_workspaces`, `gql_convert`, `ai_assistant` transcript — Teams/GraphQL, removed), `5688e06d9` (cloud-agent attachment `upload_target` — `server_api/ai`, cloud-agent upload, removed), `da449ab22` (see TUI-only above; also auth/agent-SDK surface), `bf508d24e` (Grok-OAuth connect surface).

### MCP / skills (Reject — removed)

`144568278` (factory-mcp bundled skill — `ai/skills/bundled.rs`, `skill_manager_tests`, `resources/bundled/skills/factory-mcp/` — app-bundled skills, removed surface). `d78ced530`'s MCP/file-watcher and skill-watcher call-site edits (`app/src/ai/mcp/file_mcp_watcher.rs`, `app/src/ai/skills/file_watchers/skill_watcher.rs`) are absent in the fork.

### Pricing promotion / onboarding (Reject — removed)

None standalone in this range beyond the billing entries above.

### Grok logo / model picker (Reject — removed `/model` surface)

`4e09c695f` (Grok logo for xAI models — `LLMProvider::Xai.icon()`, `Icon::GrokLogo`, `grok.svg`, `model_leading_icon`). The fork removed the old Warp AI `/model` and `/profile` selector flows: `crates/ai/src/llm_provider.rs` does not exist, `LLMProvider` is absent, `Icon::GrokLogo`/`XLogo` are absent, and there is no model picker. The icon asset and provider→icon mapping have no consumer in the fork.

### Agent Mode Cmd-Up/Cmd-Down prompt navigation (Deferred)

`da4da09f8` (navigate Agent Mode prompts with Cmd-Up/Cmd-Down, 1135 lines across `terminal/view.rs`, `terminal/model/blocks.rs`, `ai/blocklist/block.rs`, `terminal/view_tests.rs`). This is a retained AgentView feature, but it is a large, UI-dense port that adds a per-prompt navigation cursor + accent-ring render path, and the PR itself reports the navigation is inert inside the full-screen agent view on its test platform because the rich input holds focus and its keymap context lacks `Terminal` (the same `SELECT_PREVIOUS_BLOCK_ACTION_NAME` context gate exists in the fork at `app/src/terminal/view/init.rs`). Given the self-reported UI instability and the size/risk, this is deferred rather than ported this cycle; revisit when a follow-up retained change lands on these navigation-cursor types or the keymap-context gate is addressed.

### WASM / web (Reject — macOS-only)

`07c3effc2` (preserve WASM bundle defaults for feature flags — `script/wasm/bundle`, wasm target, removed).

### CI / Docker infra (Reject — absent in fork)

`2a8d62079` (agent-dev Dockerfile: golangci-lint v2.12.2 + libtokenizers — `docker/agent-dev/Dockerfile`, absent in fork).

## Verification

Commands run after porting:

- `bash -n` on `script/install_cargo_binstall`, `script/install_cargo_bundle`, `script/install_cargo_release_deps`, `script/macos/bootstrap` — all OK.
- `shellcheck -S warning` on the four touched scripts — clean.
- `cargo fmt -- --check` — passed.
- `cargo check -p warpui_core -p warp --all-targets --message-format short` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed, 2453 skipped.
- Deleted-surface scan for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — only `Arc::upgrade()`/`WeakViewHandle::upgrade()` calls, `repository.upgrade()` weak-handle calls, and a `toolchain upgrades` comment (all allowed).
- Deleted-surface scan for `mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill` — no hits.
- Deleted-surface scan for `target_os = "linux"|target_os = "windows"|cfg(windows)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb` — only the retained `ConPTY` explanatory comment in `zsh_body.sh` and retained SSH `ForwardX11=no` config strings (all allowed).
