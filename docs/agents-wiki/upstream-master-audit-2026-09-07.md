# Upstream Master Audit 2026-09-07

Range under review: `c388229de..upstream/master` (1 commit)

Previous audited upstream tip: `c388229de [REV-2380] Hide native-workspace teamless CTAs in Warp Drive (#15790)`

Current upstream tip detected: `4b7798fca feat(agent-sdk): authenticate Azure CLI with Entra token (#15840)`

Total upstream commits in this incremental range: 1

Status: triage complete. No commits ported. The single commit lives entirely inside the removed `app/src/ai/agent_sdk/` surface and depends on a Warp server companion change, so it is rejected as a whole with no separable local utility.

## Per-Commit Triage

### `4b7798fca` — feat(agent-sdk): authenticate Azure CLI with Entra token (#15840)

Decision: **reject** (removed Warp Agent SDK / Oz sandbox credential surface with a Warp server dependency).

The commit authenticates Azure CLI commands inside Oz sandboxes: a sandbox-local `az` wrapper re-exports `AZURE_DEVOPS_EXT_PAT` from an owner-only Entra token file, with `0600` temp-file writes, fsync, atomic rename, symlink refusal, and a PATH-prepend fallback warning. All three touched paths live under `app/src/ai/agent_sdk/`:

- `app/src/ai/agent_sdk/driver.rs` — adds the `git_credentials::prepend_azure_cli_wrapper_to_path` call into the `AgentDriver` sandbox env-var setup. The hunk's surrounding imports (`warp_graphql::ai`, `warp_managed_secrets`, `warp_errors::{report_error, register_error}`) are themselves removed surfaces.
- `app/src/ai/agent_sdk/driver/git_credentials.rs` — adds `AZURE_DEVOPS_*` constants, `write_azure_cli_token`, `write_azure_cli_auth`, `prepend_azure_cli_wrapper_to_path`, and helpers. These functions exist only to feed the sandbox credential path; the generic-looking helpers (`shell_single_quote`, `write_executable_file`, the atomic-rename write) are private to this removed module and no retained fork path needs them, so nothing was extracted.
- `app/src/ai/agent_sdk/driver/git_credentials_tests.rs` — tests for the above.

Verified in the fork: `app/src/ai/agent_sdk/` does not exist (`app/src/ai/` contains only the retained ACP/agent-view/predict/suggestion trees), and `rg "git_credentials|AZURE_DEVOPS_EXT_PAT" app crates` returns no hits, so the upstream hunks have no anchor symbol here. The commit description also declares a companion server PR (`warpdotdev/warp-server#16905`); the token refresh flow cannot run without the removed Warp server, which independently places this in the rejected auth/server-API category rather than a not-applicable shared-file case.

## Verification

No code was ported in this cycle, so the fork tree is unchanged and the build state from the 2026-09-06 merge (`cargo build -p warp --all-targets` clean, then `cargo clean`) carries over. `cargo fmt -- --check` passes. Deleted-surface scans were re-run to confirm no drift:

- `rg "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment"` — only allowed hits (`Arc::upgrade()`/`WeakViewHandle::upgrade()` weak-handle calls, doc-comment usages of "upgrade", the pre-existing `git_dialog` telemetry-worded comment).
- `rg "mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill"` — no hits.
- `rg "target_os = \"linux\"|target_os = \"windows\"|cfg\(windows\)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb"` — only retained occurrences (`ForwardX11=no` SSH config strings, the retained ConPTY explanatory comment in `zsh_body.sh`, and `#[cfg(windows)]`-gated tests in `warp_util`).

## Notes

- This is the third zero-port cycle after 2026-08-09 and 2026-08-29; the triage rule remains: check every shared-file hunk for a live anchor symbol before classifying, and reject rather than "not applicable" when the change's core requires a removed Warp service even if all its files are also absent.
- Deferred ports unchanged: `9921300b7` (Ctrl-C harness cancel, waiting on upstream local-keystroke wiring) and the mermaid toggle from `#10431` architecture work.
