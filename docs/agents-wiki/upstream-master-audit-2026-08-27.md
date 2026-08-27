# Upstream Master Audit 2026-08-27

## Scope

- Current fork before this audit: `78b022ecf` (`main`, `v2026.08.26`).
- Upstream source reviewed: `1846f3000..upstream/master` (25 commits, tip `511b952c2`).
- Result: 4 commits ported (`1c925e333` editor rayon fan-out bound, `607be8c26` bash bootstrap `shell_plugins`, `e36efd068` theme background animation, `5e7030db7` warping-row model naming as a partial adapt), 21 rejected or not applicable (multi-team stack ×8, Teams UI ×2, Windows host ×2, voice ×2, GraphQL/server-API team scoping, TUI ×2, skills lock, onboarding deep link, custom inference endpoints, cosmic-text pin, `create_file` allow_overwrite).

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `1c925e333` | Bound rayon fan-out in EditDelta::layout_delta (APP-5392) (#15128) | **Accept (ported)** | Retained `crates/editor` memory fix: chunked parallel layout (`MAX_LAYOUT_TASKS_PER_PARALLEL_CHUNK = 64`, `MAX_LAYOUT_CONTENT_CHARS_PER_PARALLEL_CHUNK = 64 KiB`) plus the per-line shaping cap (`MAX_LAYOUT_LINE_CHARS = 2M`) and `truncate_text_for_layout`/`clamp_style_runs_for_layout` in `render/layout.rs`. Applied via three-way patch; both `report_error!` sites keep the fork's `log::error!` adaptation, tests import from `warpui` (not `warpui_core`) and pass `RenderLayoutOptions` by value matching the fork's `layout_delta` signature. New `render/layout_tests.rs` created as upstream did. |
| `afd2aecd4` | [multi-team] Scope multi-agent requests to a team (#15355) | **Reject** | Team-scoping through `warp_multi_agent_client`/`warp_server_client`/`app/src/server/team_scope.rs` and `agent/api` — all fork-absent. Consistent with the standing multi-team rejections. |
| `a5fa4da82` | Scope link-sharing settings to the window's team (P6) (#15440) | **Not applicable** | Drive link-sharing dialog and `workspaces/user_workspaces` are removed surfaces. |
| `9c257115f` | Scope /ai/predict_am_queries to the window's team (#15555) | **Not applicable** | `app/src/server/` does not exist in the fork. |
| `711a44e3e` | Scope /ai/generate_input_suggestions to the window's team (#15558) | **Not applicable** | Pure team-scope threading of `server_api` calls; the fork's `next_command_model.rs` uses the OpenAI-compatible `terminal_suggestions` provider with no server API. |
| `874e7cf7e` | Scope /ai/generate_am_query_suggestions to the window's team (#15556) | **Not applicable** | `passive_suggestions/legacy.rs` and `server_api` are fork-absent. |
| `8cc4e2fdf` | Scope /ai/transcribe to the window's team (#15554) | **Not applicable** | Voice input/transcription is a removed surface. |
| `4111d08f9` | Teach the GraphQL transport to optionally carry a team scope (#15557) | **Not applicable** | `crates/warp_server_client` is removed in this fork. |
| `a45efa093` | Scope /ai/relevant_files requests to the window's team (#15559) | **Reject** | Team-scope plumbing for the removed server API; the fork's `get_relevant_files` controller has no `server_api` dependency and no team concept. |
| `5e7030db7` | Name the model in use in the warping row (APP-5532) (#15323) | **Adapt (partial port)** | Ported `status_message_naming_model`, `STATUS_MESSAGE_ELLIPSIS`, `WarpingProps.model_in_use_name`, the `naming_model` wrapper on the five model-phase status messages, and `ModelInUse`/`WarpingModelMessage`/`warping_model_message` fed from the active exchange's ACP-populated `model_info.display_name`. Omitted: the `WarpingModelName` rollout flag and Cargo feature (fork runs retained AgentView behavior ungated), and all fallback-model machinery (`OutputModelInfo.is_fallback`, `FallbackModelLoadOutputMessaging`, previous-exchange lookback, fallback explanation element, unnamed-fallback copy) — server-side fallback routing has no ACP counterpart and the fork's `OutputModelInfo` never had `is_fallback`. Default row copy stays the fork's `"Working..."`; the naming test's default-copy assertion was adapted accordingly. |
| `9589305a2` | [Windows] Fix login-shell commands in PowerShell 5.1 (#15544) | **Reject** | Windows-host fix (`#[cfg(windows)]` PowerShell 5.1 without `-Login`). The fork is macOS-only and keeps `-Login`, which is correct for macOS pwsh 7. |
| `ece537be1` | Center the Teams settings confirmation dialogs on the window (#15542) | **Not applicable** | `settings_view/teams_page.rs` is fork-absent; the `mod.rs` hunks only route Teams page events/modals. |
| `ad28e590f` | Recover the renderer after Windows RDP device loss (#15566) | **Not applicable** | Destination `crates/warpui/src/rendering/wgpu/renderer.rs` does not exist: the fork kept the macOS Metal renderer and removed the cross-platform wgpu backend. |
| `542683634` | Pin cosmic-text to the fix that forbids Hack as a fallback donor (APP-5492) (#15569) | **Not applicable** | The fork's `crates/warpui/Cargo.toml` has no cosmic-text dependency (macOS uses CoreText's FontDB; upstream's own description states macOS is unaffected). |
| `607be8c26` | Fix bash bootstrap dropping shell_plugins from the Bootstrapped payload (#15518) | **Accept (ported)** | Retained shell-integration fix: join the `shell_plugins` array into a newline-separated list before escaping and add the missing `shell_plugins` key to the primary `Bootstrapped` JSON payload. Omitted the MSYS2 kv-pairs branch (fork keeps the POSIX DCS JSON path only) and `wsl_name` (removed with the WSL cleanup). `bash -n` verified. |
| `d6a389d6d` | [multi-team P3a] Parse and store the team catalog into UserWorkspaces (#15463) | **Reject** | Team catalog parsing into removed `workspaces/user_workspaces`; the `llms.rs` hunk only widens `info_for_id` visibility for code the fork does not have (fork's `llms.rs` has no `info_for_id`). |
| `43de7e46b` | Fix warp_tui compile error: thread RequestTeamScope through TUI voice transcription (#15570) | **Not applicable** | `crates/warp_tui` is fork-absent. |
| `6afb6c884` | Update common skills lock for warpdotdev/common-skills#77 (#15577) | **Not applicable** | `skills-lock.json` is fork-absent; bundled skills are a rejected surface. |
| `7c84048ca` | Don't let onboarding interrupt a workspace opened via a content deep link (#15550) | **Not applicable** | Onboarding and content deep links (shared session / cloud conversation) are both removed; the fork's `root_view.rs`/`workspace/view.rs` have no onboarding or `is_content_deep_link` code. |
| `1e45ef773` | [APP-5380] Share custom inference endpoints across GUI and TUI (#15574) | **Reject** | Custom inference endpoints are the removed BYOK multi-provider surface: the fork has no `ApiKeyManager`, no `custom_endpoints` in `LLMPreferences` (reduced to static model metadata), no `api_keys.rs` in `crates/ai`, no `custom_inference_modal.rs`/`warp_agent_page.rs`, no `cloud_preferences_syncer.rs`, and no TUI. ACP adapter configuration is the only model/backend surface. `specs/APP-5380/**` rejected per the upstream-specs rule. |
| `dcfad88c0` | [multi-team P2] Scope LLM host settings to the window's team (#15447) | **Reject** | Team-scoped LLM host/BYOK credentials across removed surfaces (geap credentials, buy-credits banner, warp agent page). |
| `e36efd068` | fix(workspace): Enable animation for theme background images (#14618) | **Accept (ported)** | Retained local UI fix: store a construction-time `Instant` on `Workspace` and call `enable_animation_with_start_time` on the background image so animated GIF theme backgrounds advance past their first frame; ports the `warpui_core` `image_cache_tests` animation coverage. Adaptation: kept the fork's `if let` render branch and import header (the three-way conflict came from upstream tree shape we do not share). |
| `511b952c2` | Add allow_overwrite flag to create_file tool (#15380) | **Reject** | No live runtime destination in the fork: the producer (`warp_multi_agent_api` proto → `action/convert.rs`) and the consumer (`request_file_edits/diff_application.rs` executor) are both absent — the fork's request-file-edits execution is the CodeDiffView accept-and-save flow, where overwriting an existing file is user-mediated. Porting only the `FileEdit::Create.allow_overwrite` field would add dead schema with no owner. |

## Pre-existing test failures fixed alongside

The localization commit `3dfa8418f` rewrote `https://warp.dev` → `https://example.com` in `warp_editor` tests without updating dependent expectations, leaving 6 failures on `main`:

- `test_inline_markdown_roundtrips`: both adjacent links ended up with the same URL and merged on export; restored distinct-URL intent with `example.org`.
- `test_highlight_urls`, `test_highlight_urls_unicode`, `test_highlight_url_before_link`, `test_links_not_auto_highlighted`: five stale expected `url_range`s (16-char vs 19-char URL length); updated to the highlighter's correct arithmetic.

`cargo nextest run -p warp_editor` now passes 434/434 (420 pre-port + 14 new from `1c925e333`).

## Provenance

- `1c925e333`: `git diff 1c925e333^ 1c925e333 -- crates/editor/... | git apply --3way`; conflicts at the two error-reporting sites resolved to upstream's chunked structure with the fork's `log::error!`; test-import and by-value call-site adaptations listed above.
- `607be8c26`: three-way patch on `bash_body.sh`; MSYS2 branch and `wsl_name` omitted as recorded.
- `e36efd068`: three-way patch on `workspace/view.rs` (import/field/init hunks) + clean apply of the `image_cache_tests.rs` tests; the render-branch conflict resolved to the fork's `if let` shape plus upstream's `enable_animation_with_start_time` call.
- `5e7030db7` (partial): copied upstream's `status_message_naming_model`, `STATUS_MESSAGE_ELLIPSIS`, the `model_in_use_name` prop + doc comment, and the `naming_model` closure verbatim (comment adapted off server wording); `ModelInUse`/`WarpingModelMessage`/`warping_model_message` ported reduced to the non-fallback core (display-name-only `ModelInUse`, no lookback, no flag); `status_bar.rs` call-site recomputes `default_warping_text` from the named message as upstream does. Omitted upstream files/hunks: `app/Cargo.toml` feature, `app/src/features.rs`, `crates/warp_features/src/lib.rs` (rollout flag), fallback branches of `warping_model_message`/`resolve_warping_model_message`, `latest_model_used_before_exchange`, `render_fallback_explanation`, `fallback_warping_text`, `UNNAMED_FALLBACK_MODEL_WARPING_TEXT`, and `status_bar_tests.rs` (exercises the fallback matrix the fork does not have).

## Verification

- `bash -n app/assets/bundled/bootstrap/bash_body.sh`: clean.
- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: pass.
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo nextest run -p warp_editor`: 434/434 pass (includes the 6 pre-existing failures fixed here and 14 new tests).
- `cargo nextest run -p warpui_core image_cache` + `-p warp workspace::tests`/`workspace::view`: pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: pass (156/156, same suite size as the 2026-08-26 audit).
- Deletion-surface scans (`rg` over removed product surfaces, MCP/skills symbols, Linux/Windows/WASM platform branches): zero added hits in `main...HEAD` beyond pre-existing allowed ones.
- Final `cargo build -p warp --all-targets --message-format short`: pass; `cargo clean` run after the release push.
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
