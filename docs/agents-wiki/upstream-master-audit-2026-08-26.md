# Upstream Master Audit 2026-08-26

## Scope

- Current fork before this audit: `e5e546f6c` (`main`, `v2026.08.25`).
- Upstream source reviewed: `c5e4a02e3..upstream/master` (11 commits, tip `1846f3000`).
- Result: 1 commit ported (`1e4b86a81` release-cli codegen-units bump, exact hunk), 1 partial widget-level port from `e054075b8` (settings-search term), 1 deferred with a recorded structural blocker (`21f413b79` crate split), 4 rejected (multi-team stack ×2, Teams/billing members, agent environment snapshot reporting), 5 not applicable (fork-absent destination page/wasm/orchestration/STAKEHOLDERS/shared-session tests).

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `e054075b8` | Move Code editor line numbers setting to Editor and Code Review page (#15458) | **Adapt (partial port)** | Pure UI relocation of the unchanged `code_editor_line_number_mode` dropdown from the Features page to upstream's `EditorAndCodeReviewPageView`. The fork's settings UI never had the Code umbrella / `code_editor_review_page.rs` (fork pages: about/ai/appearance/features/keybindings/pane_manager/warpify), so the relocation has no destination and the fork keeps its working dropdown on the Features page. Ported only the widget-level improvement that survives the move: `CodeEditorLineNumberModeWidget::search_terms` gains `numbers` (exact upstream string `"line number numbers relative line vim gutter code editor"`) so settings search finds the setting under its own label. |
| `e9a3350be` | [multi-team P5] Scope remote-session AI permissions across teams (#15444) | **Reject** | Team-scoped AI permissions threading through `agent_sdk`, `workspaces/user_workspaces`, and `terminal/view`; every anchor surface is removed in this fork. Consistent with the standing multi-team stack rejections (2026-08-24 `8e5bb1fad`, 2026-08-25 P1/P7a). |
| `21f413b79` | Move a lot of terminal grid code to the `warp_terminal` crate. (#14875) | **Defer (structural blocker recorded)** | See the deferral record below. Behavior-neutral compile-time refactor; base of an in-flight upstream stack (#15462/#15464/#15468/#15473). |
| `8da7d34f5` | Gray out disabled team and workspace members in Settings (#15258) | **Reject** | Teams members UI + GraphQL schema, `crates/graphql`/`warp_graphql_schema` migrations, teams-page tests, persistence columns for disabled members — all removed surfaces (Teams/workspace discovery). |
| `1b4b13964` | Report agent environment snapshots (#15381) | **Reject** | Reports client-observed environment metadata to `POST /api/v1/agent/runs/{runId}/environment-snapshot` through `server_api`; core lives in `agent_sdk/driver/environment.rs` + new `environment_snapshot.rs` (both fork-absent). The `terminal/view/docker_sandbox/mod.rs` hunk only wires the `EnvironmentSnapshotReporter` into the agent_sdk environment-preparation path; the fork's docker_sandbox view has no agent_sdk references. Warp server reporting is a removed service dependency, not a local utility. |
| `fb594d2c8` | Fix release compilation errors. (#15529) | **Not applicable** | Fixes compile errors introduced by `21f413b79` by adding an optional `sentry` dependency to `warp_terminal`'s `crash_reporting` feature. The split is deferred and Sentry is removed in this fork. |
| `0a7d5380e` | Remove unnecessary FTUE chrome from the wasm session view (#15515) | **Not applicable** | Every shared-file hunk is a `cfg!(target_family = "wasm")` / `#[cfg(target_family = "wasm")]` gate (buy-credits banner viewer skip, notifications banner skip, free-AI-removal modal guards, `WasmNUXDialog` intent check). The fork has no wasm target, no `wasm_nux_dialog.rs`, and the gated surfaces (credits banner, free-AI-removal modal) are themselves removed. Desktop behavior unchanged by construction. |
| `1e4b86a81` | Bump release-cli codegen-units from 1 to 4 (#15517) | **Accept (ported)** | Build-profile migration on a retained manifest path: the fork's root `Cargo.toml` still carries `[profile.release-cli]` (inheriting `release-lto`, `opt-level = "s"`) exactly as upstream's pre-change state. Applied the exact upstream hunk (3-line tradeoff comment + `codegen-units = 4`) via three-way patch. The fork's release workflow builds the GUI app with `release-lto`/`rlto`, so shipped artifacts are unaffected; the profile stays aligned with upstream. |
| `b0a638117` | Change owner of vertical tabs to @peicodes (#15541) | **Not applicable** | Only touches `.github/STAKEHOLDERS`, which does not exist in this fork. |
| `60d602df6` | Don't start MAA from buffered child event after teardown begins (QUALITY-1801) (#15432) | **Not applicable** | Ambient-run orchestration race: the fix's core is `ExitCommitHandle`/`OrchestrationEventService` (in `blocklist/orchestration_events.rs`, backed by `warp_multi_agent_api`) plus `agent_sdk/driver.rs` idle-window plumbing — none of these symbols or files exist in the fork (`rg` for `OrchestrationEventService`/`warp_multi_agent_api` is empty; `orchestration_events.rs` absent). The `blocklist/controller.rs` hunk only adds an `is_exiting` guard read from that service, and the remaining blocklist hunks are `#[cfg(test)]` helpers for agent_sdk driver tests. ACP is the sole agent backend; there is no ambient run/child-event injection path to fix. |
| `1846f3000` | Fix Windows CI flake in shared-session sharer network tests (#15532) | **Not applicable** | Only touches `terminal/view/shared_session/sharer/network_tests.rs`; the fork's `shared_session` directory is empty (sharer removed with cloud session sharing). |

## Deferral record: `21f413b79` (terminal grid crate split)

Upstream moves ~100 files (`model/ansi/*`, `model/grid/*`, `blockgrid`, `find`, `selection`, `secrets`, `session` ids, `local_tty/*` minus platform splits, `writeable_pty/message`, `focus_env`, `bootstrap`, `event`/`event_listener` splits, `runtime.rs`, `block_filter` data, `TrimStringExt`, `AsciiDebug`/executable-path helpers into `warp_util`) from `app` into `crates/warp_terminal`, switching `warp_terminal` from `warpui` to `warpui_core` and adding a large dependency set (`mio`, `nix`, `image`, `warp_isolation_platform`, `session-sharing-protocol`, wasm/Windows gating).

This fork defers the port with the following concrete structural blockers, to revisit when the upstream crate-split stack (#15462/#15464/#15468/#15473 and successors) settles:

1. The authoritative patch cannot be applied to this tree. `git diff` between the fork and `21f413b79^` over `app/src/terminal` + `crates/warp_terminal` shows 465 differing files (~117k upstream-only lines): the fork deleted WSL (`terminal/wsl/*`), Windows `local_tty/windows/*`, ambient/shared-session/telemetry code, and `focus_env.rs` outright, and adapted ~260 retained files (`warp_errors` removal, `report_error!` → `log::error!`, edition-2024 let-chains, `pty_controller` signature divergence). Every hunk of the upstream move conflicts; porting means hand-rebuilding the refactor against a different tree rather than applying upstream's patch, which the source-fidelity rules treat as a last resort for a behavior-neutral refactor.
2. The moved seam depends on fork-removed infrastructure: upstream's post-split `warp_terminal` requires `warp_isolation_platform` (removed), `session-sharing-protocol` (removed), optional `sentry` (removed), wasm target gating (removed), and `warp_errors/crash_reporting` (removed). The fork's current `warp_terminal` is a small crate (~20 deps, `test-util` feature only) whose `model/ansi/mod.rs` (3-line `control_sequence_parameters` module) collides with the incoming moved `ansi/mod.rs`.
3. The split is the base of an in-flight stack; sibling PRs continue reshaping `warp_terminal`'s boundaries. Porting the base alone would be re-done by the next audit's ports.

Cost of deferral, tracked as an accepted structural divergence: future upstream terminal fixes landing under `crates/warp_terminal/src/**` must be path-mapped back to `app/src/terminal/**` during merges (the same handling already applied for `warp_tui`, singular test filenames, and `Box<dyn AnyView>`). Future audits must keep checking moved-file paths against both locations.

## Provenance

- `1e4b86a81`: `git diff 1e4b86a81^ 1e4b86a81 -- Cargo.toml | git apply --3way` — the fork's `[profile.release-cli]` block matched upstream's parent exactly, so the hunk applied byte-for-byte (comment + `codegen-units = 4`). Nothing omitted.
- `e054075b8` (partial): the upstream destination-page `CodeEditorLineNumberModeWidget::search_terms` string (`"line number numbers relative line vim gutter code editor"`) copied verbatim into the fork's surviving widget in `app/src/settings_view/features_page.rs:4850`. Omitted: the entire Features→Editor-and-Code-Review relocation (destination page fork-absent), the `ViewHandle`/action/subscription move, and the telemetry event carry-over (`TelemetryEvent::FeaturesPageAction` — telemetry removed in this fork).

## Verification

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: pass.
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: pass (156/156, same suite size as the 2026-08-25 audit).
- Deletion-surface scans (`rg` for removed product surfaces, MCP/skills symbols, Linux/Windows/WASM platform branches) over `main...HEAD`: zero added hits beyond pre-existing allowed ones.
- Final `cargo build -p warp --all-targets --message-format short`: pass; `cargo clean` run after the release push.
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
