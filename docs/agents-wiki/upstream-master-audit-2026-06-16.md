# Upstream Master Audit 2026-06-16

Range under review: `c0c6cead9..d4bb3f5b7`

Previous audited upstream tip: `c0c6cead9 Authenticate cloud-agent OTLP trace export (#12364)`

Current upstream tip detected: `d4bb3f5b7 Improve error message when LlmModelHost cannot be parsed (#12666)`

Total upstream commits in this incremental range: 28

Status: triage complete. Compatible retained terminal, markdown, code-review, language highlighting, prompt-suggestion, macOS UI, local file-reader, Mermaid, and dependency-security changes were ported or adapted manually. Upstream warpctrl/local-control, GraphQL/schema, cloud credential, Warp-hosted account/agent/telemetry, app-managed skills/MCP, tab grouping, broad remote search plumbing, and deleted spec/process surfaces were rejected or marked not applicable under the fork contract.

## Ported Or Adapted

- `1184de5b6` Ported Nix language grammar support for the retained editor/highlighting path by enabling `arborium`'s `lang-nix` feature, registering the language/extension/highlight query, and adding the Nix grammar queries.
- `802a881e5` Adapted the untracked-directory diff fix to this fork's single-file local `DiffStateModel`. Untracked directories now produce empty non-binary diffs, file baselines are only created for files, line-count failures are logged locally at debug, and focused tests cover directory and text-file behavior.
- `3aa6026c6` Ported the Mermaid renderer dependency bump for retained Markdown/Mermaid rendering by updating `mermaid_to_svg` to upstream rev `79aecc74187c29a027e85e18d070ff0dd884a0a7`.
- `09be9c1ff` Ported terminal startup inline image routing. Completed iTerm/Kitty image actions before preexec now route to output/background blocks when they create visible content, WarpInput bootstrap image completions remain dropped, image-only grids are visible, and tests cover script-execution, early-output, store-only, zero-sized, and non-moving Kitty placements.
- `881449d71` Ported the code-review file header corner-fill UI fix by applying the same inner corner radius to the opaque header background wrapper.
- `dcee061dc` Ported the OpenSSL security dependency bump, aligning the lockfile with `openssl 0.10.80` and `openssl-sys 0.9.116`.
- `11f6f4a91` Ported the `warp_files` truncated range clamp so a first-line truncation no longer produces a reversed range, with text-file-reader regression coverage.
- `dc2155193` Adapted CRLF command paste normalization for this macOS fork. POSIX shells normalize CRLF to LF before PTY writes, PowerShell still normalizes to CR, and command-byte tests cover bracketed POSIX, unbracketed POSIX, and PowerShell behavior without adding local Windows host branches.
- `4d8fbca79` Adapted the prompt-suggestions wrap revert to the fork's input layout. Prompt suggestion buttons no longer expand/wrap on hover, and the banner is constrained to a stable two-line height across Agent, classic, and terminal input renderers.
- `9077e2bf4` Ported the vertical-tabs traffic-light padding fix for retained macOS UI. Opening the tools/left panel no longer suppresses traffic-light padding, and tests cover theme chooser plus left/right toolbar placements available in this fork.
- `c682422f2` Ported the Markdown delimiter counter overflow fix by widening private delimiter run counters from `u8` to `usize` and adding long delimiter run regression coverage for `*`, `_`, and `~`.

## Already Present Or No-Op

- `ec29aefa6` The `pane_leaves` `save_app_state` deletion was already present in this fork's SQLite persistence code, so no change was needed.

## Rejected Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `aa0a2c210` | Reject | Implements `warpctrl`, local-control commands, bundled skill resources, and upstream specs. This fork has rejected local-control/warpctrl product surfaces and app-bundled skills. |
| `25a6a9e0e` | Not applicable | Bumps `shell-quote` only in deleted `crates/warp_graphql_schema`; GraphQL schema/client code remains removed. |
| `076260328` | Reject | Adds GEAP/cloud credential and provider plumbing tied to upstream cloud/workspace/account configuration. |
| `8c63aaf9b` | Reject / defer | Remote git chips depend on upstream remote diff-state, repo-status, app-side skills/specs, and broader remote code-review architecture absent from this fork. Retained SSH remote terminal behavior should be extended from current call sites if needed. |
| `1384dfac8` | Reject | Enables warpctrl dogfood rollout; warpctrl itself remains rejected. |
| `2735ae10a` | Reject / defer | Removes `WelcomePalette` through a compatibility migration path while this fork still uses `WelcomeTab`, `WelcomePane`, and `WelcomePalette` in retained new-tab/tests. |
| `3c953840a` | Not applicable | Touches `warp_search_core`, which is absent from this fork. |
| `30d8054e5` | Not applicable | The CLI rich-input attach-as-context helper path is absent; the fork currently has the simpler PTY-oriented CLI agent input path. |
| `169767c6d` | Not applicable | Removes an upstream free-tier telemetry requirement in privacy/account prompt-alert UI that this fork already removed with cloud/account/telemetry surfaces. |
| `d0d3d064d` | Reject | Continues upstream tab grouping/pinning invariants; this fork has not accepted the tab-grouping product line. |
| `f5c60e375` | Reject / defer | Adds remote ripgrep search protocol/server plumbing without a retained fork call site and overlaps broad remote-code-search architecture that should be designed from current SSH/ACP boundaries. |
| `08880b0b1` | Reject | Adds follow-ups for remote bundled skills. App-bundled skills and app-managed skill installation remain removed. |
| `a60d07502` | Not applicable | Adds `report_and_log` to upstream split local/remote diff-state files and telemetry/reporting flow. This fork uses a single local diff-state file and no external telemetry path. |
| `13fd05e0a` | Reject | Fixes `ReadSkill` display for app-side skills; `ReadSkill`, `InvokeSkill`, and app-managed skills are removed. |
| `d4bb3f5b7` | Not applicable | Improves `LlmModelHost` GraphQL parsing errors in deleted GraphQL/server API/workspace paths. |

## Verification

Commands run after porting:

- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo fmt -- --check`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- `git diff --check`
- Full deleted-surface scans from `AGENTS.md` for cloud/auth/billing/telemetry, MCP/skills, and local Linux/Windows/Web host patterns.

Results:

- Both cargo check commands passed.
- `cargo fmt -- --check` passed.
- `cargo nextest` ran 150 tests: 150 passed, 2372 skipped.
- `git diff --check` passed.
- Deleted-surface scans found only existing weak-handle `upgrade()` false positives, tokenizer vocabulary entries, retained SSH `ForwardX11`, and retained bootstrap `ConPTY` comments. The MCP/skills scan had no hits, and the added-line deleted-surface scan had no hits.
