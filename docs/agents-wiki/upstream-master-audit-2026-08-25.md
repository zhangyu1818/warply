# Upstream Master Audit 2026-08-25

## Scope

- Current fork before this audit: `271542385` (`main`, `v2026.08.24`).
- Upstream source reviewed: `79a9cb721a..upstream/master` (22 commits, tip `c5e4a02e3`).
- Result: 6 commits accepted or adapted (ConPTY doc citation, two command-signatures bumps, editor `EditDelta` Arc-wrap, last-tab shortcut hint precedence, serde Content→JSON deserializer rewrite, contributor-docs `/warp-agent-review` rename), 7 rejected (multi-team stack, skills lock, Windows-arm64/Sentry release CI, onboarding removals), 9 not applicable (fork-absent `warp_tui`/warpui-core TUI runtime, MCP templatable manager, agent_sdk driver/task-sync/attachments, `CtrlCCancelsThirdPartyHarness` promotion, fork-owned release workflow).

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `996babeea` | Cite the ConPTY source for the PowerShell virtual key code claim (#15482) | **Accept (ported)** | Comment-only: pinned ConPTY permalink on `input_reporting_sequence` and `kill_buffer_bytes` doc comments in retained `crates/warp_terminal/src/shell/mod.rs`; cherry-picked cleanly. |
| `8cbb01d45` | [multi-team P1 1/3] Split user_workspaces.rs into a module (#15485) | **Reject** | Pure refactor of `app/src/workspaces/user_workspaces.rs` into team/billing settings modules; the fork has no `app/src/workspaces/` (Teams/workspace discovery removed). No anchor symbol exists in the fork. |
| `e326a774a` | [Completions] Bump command-signatures to 564724fe (#15480) | **Accept (ported)** | `ln` spec fix (`-s`/`-F`/`-f`/`-i` boolean flags). Rev `ac69f9b` → `564724fe`; conflict resolution dropped upstream-only `winit`/`x11rb` workspace deps (Linux windowing, removed). |
| `359400445` | [multi-team P1 2/3] Add CLI --team flag with team selection (#15486) | **Reject** | Threads `TeamScope` through `warp_cli` scope plus `agent_sdk` (removed); no `TeamScope`/`user_workspaces` in the fork. |
| `84d3e332a` | [multi-team P1 3/3] Scope member BYO key/endpoint policy to the window's team (#15446) | **Reject** | Team-scoped BYO policy across `agent_sdk`, billing pages, custom model router, buy-credits banner — all removed surfaces; consistent with the 2026-08-24 rejection of `8e5bb1fad`. |
| `ee351a0e7` | Terminate TUI when host terminal disconnects (#15421) | **Not applicable** | Touches `crates/warp_tui` (absent) and `crates/warpui_core/src/runtime/mod.rs`; the fork's `warpui_core` has no `tui` feature, no `runtime/` module, and no `spawn_tui_driver` anywhere. |
| `378b74f3b` | Fix Windows arm64 release: use forked setup-sentry-cli with win32/arm64 mapping (#15491) | **Reject** | Windows arm64 + Sentry CLI release infra; the fork's `create_release.yml` is macOS-only and Sentry is removed. |
| `4b894db80` | Compile time: replace serde Content buffering with JSON-value deserializers (#15455) | **Adapt (ported)** | See provenance below. Stacked on `6a96a72d8` (#15454), already ported in the 2026-08-24 audit. |
| `e83d07d8b` | [Completions] Bump command-signatures to d3725aa (#15487) | **Accept (ported)** | Second bump `564724fe` → `d3725aa` on the same day; same `winit`/`x11rb` conflict resolution. |
| `40e397170` | Fix last-tab shortcut hint precedence (#15496) | **Adapt (ported)** | See provenance below. Builds on the 2026-08-21 switch-to-tab shortcut hints port. |
| `1704db4cf` | Update common skills lock for warpdotdev/common-skills#79 (#15497) | **Reject** | `skills-lock.json` for app-bundled skills; the fork has no skills bundle and no lock file. |
| `d89e78385` | fix: stop cloning the whole file's styled blocks when loading a large file in the code editor (APP-4844) (#13508) | **Adapt (ported)** | See provenance below. `EditDelta::new_lines` becomes `Arc<Vec<StyledBufferBlock>>` and `layout_delta` borrows. |
| `d2cb17abb` | Throttle 'No template UUID found' report_error! to once per run (#15498) | **Not applicable** | Only touches `app/src/ai/mcp/templatable_manager/native.rs`; app-managed MCP is removed and the file does not exist in the fork. |
| `9c08cd31d` | [multi-team P7a] Scope default host slug and agent attribution to the window's team (#15445) | **Reject** | Orchestration/handoff/ambient host-selector + `warp_tui` orchestration blocks + team workspace threading; every anchor surface is removed. |
| `401029754` | Scope the sandboxed-agent denylist to the window's team (#15488) | **Reject** | Replaces `UserWorkspaces::sandboxed_agent_settings()` with a team-scoped getter; `UserWorkspaces` does not exist in the fork, so there is no local path to adapt. |
| `2da797f60` | ci: fix release workflow warnings (gcloudignore, Node 20, go cache) (#15502) | **Not applicable** | Only touches `.github/workflows/create_release.yml`; the fork's file is the fork-owned Warply release workflow with none of the gcloud/node/go sections. |
| `4ab7ef99c` | Use logical filename instead of file_id for attachment downloads (#15505) | **Not applicable** | Entirely inside `app/src/ai/agent_sdk/driver/attachments.rs` (removed backend). The fork's attachments are ACP-side `AIAgentAttachment`s with no agent_sdk download path. |
| `6b7a743a7` | Rename /oz-review to /warp-agent-review in contributor docs (#15407) | **Adapt (ported)** | The fork's `CONTRIBUTING.md` (2 lines) and `FAQ.md` (1 line) match upstream's renamed lines; the upstream "If a maintainer requests changes…" hunk and its surrounding manual-testing/PRs-without-issue block were omitted because the fork's docs never contained them. |
| `c5b7d0860` | Guarantee a terminal task state when the agent client exits, including failed for sandbox deadline reached (#15387) | **Not applicable** | Core changes live in `app/src/ai/agent_sdk/driver.rs` and `app/src/ai/blocklist/local_agent_task_sync_model.rs`; neither file exists in the fork (agent execution is ACP-only). |
| `7feb88b5e` | Remove the concluded REV-1939 onboarding "Choose how to start" experiment (#15501) | **Reject** | Onboarding experiment teardown across `crates/onboarding/`, auth login slides, pricing, experiments infra — all removed surfaces. |
| `6696954c6` | Promote CtrlCCancelsThirdPartyHarness to stable (#15506) | **Not applicable** | The flag and its feature (#15257/`9921300b7`) were never ported (deferred 2026-08-19). Verified upstream's only read site is still `write_viewer_bytes_to_pty` — the shared-session viewer input path this fork removed — so the deferred port's trigger (local-keystroke parity wiring) remains unmet. |
| `c5e4a02e3` | Delete the unreachable project onboarding step, and retire OpenWarpNewSettingsModes (#15481) | **Reject / N.A.** | Onboarding step deletion plus `OpenWarpNewSettingsModes` retirement; `rg` finds no `OpenWarpNewSettingsModes` in the fork, and the touched settings pages diverge without the flag. |

## Provenance: `4b894db80` port detail

Core copied from the exact upstream commit (envelope/twin-struct `Deserialize` pattern, `parse_hook_value`/`parse_artifact_data` helpers, `RawBootstrappedField` missing-vs-present semantics, doc comments, round-trip tests), adapted to fork surfaces:

- `app/src/terminal/model/ansi/dcs_hooks.rs`: `RawDProtoHook` envelope + hand-written `DProtoHook` deserialize with the fork's 16 arms — upstream's `FinishUpdate` arm and `DPROTO_HOOK_VARIANTS` entry replaced by the fork's retained `InitSsh`, `RemoteWarpificationIsUnavailable`, `SshTmuxInstaller`, `TmuxInstallFailed` arms in enum order. `BootstrappedValue`/`RawBootstrappedValue` omit `wsl_name` (WSL field, fork-absent). `trim_null_byte_deserializer`/`empty_string_is_none`/`parse_shell_options_list` remain for `InitShellValue` and other structs still using `deserialize_with`, exactly as upstream keeps them.
- `app/src/terminal/model/ansi/dcs_hooks_tests.rs` (new): upstream tests copied with `wsl_name`/`FinishUpdate` references removed, the `every_hook_tag_dispatches_to_the_matching_variant` case list extended with the four SSH-warpification payloads, and the two `PrecmdHookValue`-classification tests collapsed into one `precmd_hook_parses_payload_fields` asserting the fork's flat `PrecmdValue` (the upstream `WithCompletionMetadata`/`PromptOnly` dispatch enum does not exist in this fork).
- `app/src/ai/artifacts/mod.rs`: `ArtifactEnvelope` + per-variant data structs replace the `ArtifactHelper` mirror enum, with `EXTERNAL_REFERENCE` and `PlanData::notebook_uid` omitted (fork's `Artifact` has neither). `ARTIFACT_TYPES` lists the fork's four tags.
- `app/src/ai/artifacts/mod_tests.rs`: the four new round-trip/rejection tests ported (round-trip list uses the fork's variant set). Upstream-preexisting download/lightbox/file-button tests were not ported: they exercise `ArtifactDownloadResponse`/`screenshot_lightbox_image_from_download_result`, which do not exist in the fork.
- `app/src/ai/agent/mod.rs`: `AIAgentContextTagged`/`AIAgentAttachmentTagged` twins + `From` impls + buffered-fallback `Deserialize` impls copied from upstream, trimmed to the fork's variant sets — no `Repository`, `PullRequest`, `Skills` (context) and no `DriveObject` (attachment); `ExecutionEnvironment` uses the fork's `AiExecutionContext` instead of upstream's `WarpAiExecutionContext`; `deserialize_pull_request_number` omitted with the `PullRequest` variant.
- `app/src/ai/agent/mod_tests.rs` (new): the fork never had this file (upstream's preexisting contents use `warp_multi_agent_api`, `server_api`, `DriveObjectPayload`, `WarpAiExecutionContext` — all removed), so the file is created containing only the six new tests (sample `BlockContext`, context/attachment tagged round-trips, untagged `Block` round-trips with the `block_id` assertion, unknown-variant rejection), adapted to the fork types; registered via `#[cfg(test)] #[path = "mod_tests.rs"] mod tests;` as upstream does.

## Provenance: `40e397170` port detail

- `app/src/tab.rs`: `TAB_ACTIVATE_LAST_BINDING_NAME`, `tab_activate_binding_name(tab_index, tab_count)` helper, the `.chain(once(TAB_ACTIVATE_LAST_BINDING_NAME))` modifier-kind collection in `reveals_tab_shortcut_hints`, and the `TabComponent` hint computation switched to the helper — all applied cleanly.
- `app/src/tab_tests.rs`: the three new `tab_activate_binding_name_*` tests applied cleanly.
- `app/src/workspace/view/vertical_tabs.rs`: `PaneProps.shortcut_hint_tab_index: Option<usize>` → `shortcut_hint_binding_name: Option<&'static str>` (struct, constructor, filter call-site `None`, and the two render call-sites now pass `tab_activate_binding_name(tab_index, workspace.tabs.len())`); `shortcut_hint` reads `props.shortcut_hint_binding_name?` directly. Conflict resolution kept the fork's import layout (upstream's import block pulls modules the fork has removed); the fork's `use crate::tab::{...}` mirrors upstream's `TAB_ACTIVATE_BINDING_NAMES` → `tab_activate_binding_name` swap.
- `app/src/workspace/view/vertical_tabs_tests.rs`: upstream deleted the four `shows_shortcut_hint`/`TAB_ACTIVATE_BINDING_NAMES` tests (the `shows_shortcut_hint` helper itself was removed upstream and does not return in the fork). Conflict resolution kept the fork's import lists minus the two removed symbols; upstream-only import items (`sort_summary_primary_labels_status_first`, re-added `shows_synced_inputs_indicator`) not present in the fork's test file were not introduced.

## Provenance: `d89e78385` port detail

- `crates/editor/src/content/edit.rs`: `EditDelta::new_lines: Arc<Vec<StyledBufferBlock>>` (derives restored, O(1) clone), `layout_delta(&self, ...)` borrowing `new_lines` via `.iter()`, `LayoutTask<'a>` with `Text(&'a StyledTextBlock)` / `MermaidDiagram { text_block: &'a StyledTextBlock, .. }`, `from_styled_block(content: &'a StyledBufferBlock, ...)`, `layout_text_block`/`layout_mermaid_diagram_block`/`layout_table_block` taking `&StyledTextBlock`, and the `match &text_block.style` borrow refactor with per-arm `*` copies — all copied from upstream.
- `crates/editor/src/content/buffer.rs`, `core.rs`: the `Arc::new(self.styled_blocks_in_range(...))` construction sites applied; fork's grouped imports kept with `sync::Arc` added.
- `crates/editor/src/content/buffer_test.rs` (fork's singular-name file, auto-merged): `(*delta.new_lines)` derefs.
- Omitted (fork-absent upstream context, not this commit's core): `layout_mermaid_block_for_test` helper, the `MermaidCodeFallback` `LayoutTask` variant, the `pending_mermaid_asset` field on `BlockItem::RunnableCodeBlock`, and the `replace_first_n_characters_handles_incremental_unicode_prefix` test — all belong to the upstream mermaid-fallback feature the fork never ported.
- Kept fork divergence: `layout_delta` takes `layout_options: RenderLayoutOptions` by value (the fork's pre-existing signature; upstream's `&` is parent context, not this commit's change), so the new test passes it by value.
- `app/src/code/editor/model_tests.rs`: `mock_model_with_buffer` helper + `test_two_editors_sharing_a_buffer_both_lay_out_a_large_content_replace` regression test copied verbatim.

## Provenance: command-signatures bumps

- `e326a774a` then `e83d07d8b`: workspace `Cargo.toml` rev `ac69f9b00c…` → `564724feb…` → `d3725aa423…` plus the two matching `Cargo.lock` entries each. Both conflict resolutions omitted upstream's neighboring `winit` git dep and `x11rb = "0.13.0"` (Linux windowing dependencies removed from this fork).

## Verification

- `cargo fmt -- --check`: clean (one `cargo fmt` pass folded into the `4b894db80` port commit, also re-wrapping the prior tab-hint port's test imports).
- `cargo check -p warp --all-targets --message-format short`: pass.
- `cargo check --workspace --all-targets --message-format short`: pass.
- Focused tests: new dcs_hooks/artifacts/agent deserializer suites 24/24; `tab_activate_binding_name`/shortcut tests 6/6; `warp_editor` 420 tests — 415 pass, 5 failures (`test_inline_markdown_roundtrips`, four `test_highlight_url*`) verified pre-existing on clean `main` via a temp worktree; standard suite `test(slash_command) | test(acp) | test(terminal_suggestions)` 156/156.
- Pre-existing warnings only (notably the unused-import warnings in `app/src/ai/agent/conversation.rs`/`task.rs` left by the 2026-08-24 `ai_types` re-export port; verified present on `main`).
- Deletion-surface scans over `main...HEAD`: zero added hits for removed product surfaces, MCP/skills symbols, `DriveObject`/`Skills`/`wsl_name`/`FinishUpdate`, or Linux/Windows/WASM platform branches.
- Final `cargo build -p warp --all-targets --message-format short`: pass; `cargo clean` to run after the release push (see below).
- Disk note: `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
