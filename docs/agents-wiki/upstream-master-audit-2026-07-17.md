# Upstream Master Audit 2026-07-17

Range under review: `62da4ee72..upstream/master` (82 commits)

Previous audited upstream tip: `62da4ee72 [CODE-1829] Render agent task lists in the TUI transcript (#13570)`

Current upstream tip detected: `f1547fefc Add bottom padding below TUI footer/prompt (CODE-1878) (#13850)`

Total upstream commits in this incremental range: 82

Status: triage complete. Retained terminal/renderer/markdown/AgentView/TaskStore fixes and a new box-drawing renderer were ported or adapted manually. The bulk of the range is TUI-only (`crates/warp_tui/`, absent in this fork), cloud/agent-sdk/orchestration/computer-use recording work (removed surfaces), or feature-flag promote/telemetry commits that touch flags the fork does not carry.

## Ported Or Adapted

- `9444f94db` Ported the WarpUI table body click-through fix. The table element's decorative body paint layer (`start_layer(ClipBounds::BoundedByActiveLayerAnd(body_clip_rect))`) intercepted `LeftMouseDown`, so a selection that originated inside a rendered Markdown table never started. Added `ctx.scene.set_active_layer_click_through()` immediately after the body clip layer starts, mirroring the editor/notebook table renderer. Fork path is `crates/warpui_core/src/elements/table/mod.rs` (no `gui/` segment); the single-line patch applied cleanly.
- `086dade63` Ported the GFM autolink trailing-punctuation fix. `parse_url` in `crates/markdown_parser` only stripped trailing formatting delimiters (`* _ ~`), so an emphasized autolink followed by other punctuation (e.g. `**https://example.com**.`) absorbed the closing `**` into the URL and broke emphasis matching. Added the `AUTOLINK_TRAILING_PUNCTUATION` constant (`?!.,:*_~`) and a trim loop that strips the full GFM set while respecting backslash escapes. The only conflict was a stale doc-comment block (resolved by keeping the upstream wording plus the fork's pre-existing `backslash escapes` line).
- `868f9b18d` Adapted the agent tool-call banner border-gap fix. The requested-action/command banner chrome used 8px corner radii that left a 1px gap against the 7px inline-action container. Aligned the header/body/footer corner radii to 7px and squared the pinned-header top corners. Adapted: the upstream MCP JSON tree-view corner change (`should_render_mcp_content` / `FeatureFlag::McpJsonTreeView` block in `requested_command.rs`) is absent in this fork and was dropped; the five non-MCP 8→7 sites were applied manually after the auto-merge left the MCP block as a conflict.
- `07956a677` Ported the `TaskStore::exchange_by_id` not-yet-linked-subtask crash fix. `exchange_by_id` only consulted the linearized (root-reachable) index, so an optimistically-created subtask whose parent sub-agent call hadn't streamed in yet could not resolve its own exchanges, crashing `AIBlockModelImpl::new`. Added an all-tasks fallback scan when the linearized lookup misses. The regression test was ported unchanged; all referenced helpers (`create_test_subtask_with_exchanges`, `TaskStore::insert`, `all_exchanges`) exist in the fork.
- `9d563b01c` Ported the fullwidth/CJK punctuation link-detection fix. URL and file-path detection treated adjacent fullwidth/CJK punctuation as part of the clickable link. Added `is_url_link_separator` (extends the ASCII separator set with non-ASCII whitespace and Unicode punctuation categories via the already-present `unicode_general_category` crate) and trimmed fullwidth trailing punctuation from file-path hover candidates with a fallback to the original candidate so real filenames ending in fullwidth punctuation stay openable. Clean auto-merge across `grid_handler`, `link_detection`, and `util/link_detection`.
- `39b09fad0` Adapted the procedural box-drawing glyph renderer. Solid box-drawing characters (`U+2500..=U+257F`) now render as cell-filling, device-pixel-snapped rectangles instead of font glyphs, eliminating seams between adjacent box-drawing cells. Added the `box_drawing` module under `app/src/terminal/grid_renderer/`, the `NativeGlyphType::BoxDrawing` variant, and the dispatch in `native_glyph_for_cell`/`render_native_glyph`. Adapted: added only the `BoxDrawingGlyphs` feature flag to the fork's `warp_features` enum and `DOGFOOD_FLAGS` list — the many other upstream flags in the same commit (`CloudModeInputV2`, `HandoffCloudCloud`, `BillingAndUsagePageV2`, `PinnedTabs`, `SuperGrok`, `GeminiEnterprise`, `CustomModelRouters`, `TerminalLifecycleRecovery`, `CloudRunners`, `McpJsonTreeView`, etc.) are for removed cloud/billing/MCP/skills surfaces and were deliberately not introduced.
- `88cf89531` Adapted the headless local-HTTP-server gating fix. The fixed-port loopback HTTP server (`crates/http_server`) was registered for every launch mode, so co-located headless remote-server processes contended for the fixed port and logged spurious bind errors. Added `LaunchMode::should_start_local_http_server` (`!is_headless()`) and gated the existing `HttpServer` singleton registration on it. Adapted: the fork's HTTP server registers only the `profiling::make_router()` router (the upstream `app_installation_detection::make_router()` and the `local_control`/`WarpControlCli` block are absent in this fork and were not restored); the upstream `lib_tests.rs` harness does not exist in this fork.
- `79e5873cc` Ported the warp_cli global-flags-before-subcommand parse fix. Replaced clap's `args_conflicts_with_subcommands` with `subcommand_precedence_over_arg` on `Args` in `crates/warp_cli/src/lib.rs` so global flags such as `--api-key`/`--debug` may precede the subcommand instead of being rejected as a parse error. The upstream regression test file (`crates/warp_cli/src/lib_tests.rs`) references removed cloud/agent CLI modules (`agent`, `artifact`, `environment`, `harness_support`, `integration`) and was not restored.

## Deferred (Retained But Requires Dedicated Port)

- `4b39aa316` OSC 8 hyperlink support was deferred here because of its breadth and structural divergence. It has since been ported source-faithfully: the per-grid registry, parser, flat storage, model/view routing, interaction behavior, integration registration, and source tests are present; only the rollout flag, upstream `specs/GH6393/`, and unrelated metadata were excluded.
- `51dae19e9` tab_config new-session menu scrollable + window-height cap is a retained local UI improvement, deferred because `app/src/workspace/view.rs` diverged substantially (3 conflict regions including window-height math) and the upstream `app/src/workspace/view_tests.rs` was deleted in the fork (modify/delete conflict). Port manually in a follow-up: promote the local `NEW_SESSION_MENU_WIDTH` constant, add the `MenuVariant::scrollable()` calls, and recompute the max height against the window bounds.

## Rejected Or Not Applicable

| Commit | Decision | Reason |
| --- | --- | --- |
| `64ddc5f5c` | Reject | Gate NLD `InputBufferSubmitted` telemetry to dogfood; `app/src/server/telemetry/` absent. |
| `27af3ab75` | Reject | Preserve custom-endpoint model selections across device sync; `reconcile_disabled_model_preferences` and cloud device-sync flow absent. |
| `1be2b6b00` | N/A | Shared keychain storage; entire change is a `LaunchMode::Tui` namespace split + Linux/Windows secure-storage backends, all absent. |
| `69d57eea7` | N/A | Generalize TUI inline menu routing; `crates/warp_tui/` absent. |
| `a554fbe99` | N/A | Bump `taiki-e/install-action` in `.github/workflows/ci.yml`; fork-owned CI. |
| `0b5adc053` | Reject | Surface-agnostic conversation list policy; bulk is `crates/warp_tui/` + `app/src/tui_export.rs` (absent) and a `specs/` TECH doc. |
| `41e9fe4a6` | N/A | TUI conversation management; `crates/warp_tui/` absent. |
| `129860cfe` | N/A | TUI word wrapping; bulk is `crates/editor` char-cell display + `crates/warp_tui/` consumer. |
| `ab0aff6d2` | Reject | Remove low-value `InputBufferSubmitted` enablement test; telemetry absent. |
| `7998f4cbf` | N/A | TUI model selector; `crates/warp_tui/` absent. |
| `e08ab1de7` | Reject | Remove `ImageReceived`/`CommandXRayTriggered`/`TabSingleResultAutocompletion` telemetry; telemetry absent. |
| `b755d16a8` | N/A | Replace dispatch-time area passing with paint-retained size/origin; `crates/warp_tui/` + `crates/warpui_core/src/elements/tui/` consumers absent. |
| `f459e8588` | N/A | Synthetic mouse build scene pass; `crates/warpui_core/src/elements/tui/` consumers absent + `specs/` doc. |
| `799727885` | Reject | TUI skills browser; skills rejected and `crates/warp_tui/` absent. |
| `0f2407aef` | N/A | Restore TUI inline menu styling; `crates/warp_tui/` absent. |
| `4d9f61ce1` | N/A | STAKEHOLDERS co-owner edit; `.github/` is fork-owned. |
| `a35d4125b` | N/A | Prevent overlapping TUI inline menus; `crates/warp_tui/` absent. |
| `230a9f379` | Defer | Basic LRC support; the `app/src/ai/blocklist/block/cli_controller.rs` shared-controller additions (`CLISubagentTarget`, `set_latest_instruction`, `active_target`) are retained GUI-relevant state, but the bulk is TUI rendering (`crates/warp_tui/`, `app/src/tui_export.rs`) and the GUI blocklist does not yet consume the new target API. Defer until a GUI LRC surface is needed. |
| `5f52cf1da` | N/A | Render alt-screen apps in TUI; `crates/warp_tui/` absent. The `crates/warp_terminal/src/model/escape_sequences.rs` alt-screen size handling is shared but only consumed by the TUI. |
| `cbe8a7535` | Reject | Gemini Enterprise (GEAP) credential recovery UX; `app/src/ai/geap_credentials.rs`, `crates/ai/src/geap_credentials.rs`, `app/src/settings_view/main_page.rs`, and `app/src/workspaces/user_workspaces.rs` are absent (cloud/BYOLLM credential surfaces removed). |
| `9bc9919b9` | Reject | Extract orchestration edit state into frontend-neutral module; `app/src/ai/orchestration/` and `run_agents` orchestration removed. |
| `72ca3d3c6` | Reject | `tui-verify-change` skill docs; skills rejected. |
| `16a4726dc` | N/A | Resume TUI session after login; `app/src/tui/` and `crates/warp_tui/` absent. |
| `28f25535f` | N/A | Long-running command rendering; bulk is `crates/warp_tui/` terminal content element. |
| `c394918af` | N/A | Fix TUI copy inserting newlines; `crates/warp_tui/` absent. |
| `c3ba49419` | Reject | `specs/GH11134/` tab-config boolean parameters; upstream specs rejected. |
| `be547674a` | Defer | Refresh changed local Markdown images; requires porting the `AssetSource::LocalFile.content_version` field and `with_local_file_content_version` method (a cross-commit type migration on `crates/warpui_core/src/assets/asset_cache.rs`) that the fork does not carry. Defer until the content-version plumbing is ported alongside. |
| `9c594f729` | N/A | Promote async find to Stable; the fork's `warp_features` enum has no `AsyncFind` flag. |
| `6df6a3173` | N/A | Fix flaky `tui_selection_reconciles_split_and_removed_selection`; `crates/warp_tui/` absent. |
| `42921f38d` | Reject | Add You.com product icon for the MCP gallery; app-managed MCP gallery removed. |
| `21e2df7eb` | N/A | Remove the `RunAgentsTool` feature flag; the fork's `warp_features` enum has no `RunAgentsTool` flag. |
| `da447b488` | N/A | Reusable Markdown rendering for TUI; `crates/warp_tui/` absent. |
| `19ebec9da` | N/A | Enable terminal lifecycle recovery in prod; the fork's `warp_features` enum has no `TerminalLifecycleRecovery` flag. |
| `8c7b514f7` | N/A | Enable Kitty keyboard protocol for TUI Shift+Enter; `crates/warpui_core/src/runtime/` change serves the TUI only. |
| `1e2d1a313` | N/A | Disable `nld_prompt_history_match` on all channels; the fork's `warp_features` enum has no `NldPromptHistoryMatch` flag. |
| `b44b230a6` | Reject | Burn keyboard action overlays into computer-use recordings; depends on removed `crates/computer_use` recording + agent SDK flow + `specs/` docs. |
| `c63c0dce9` | Reject | Proper TUI initialization for Sentry and telemetry flushing; telemetry/Sentry removed. |
| `960c3ec86` | N/A | Editor-backed code blocks for TUI; `crates/warp_tui/` absent. |
| `dcd7494aa` | Reject | V0 MCP implementation; app-managed MCP file-based manager, watcher, templatable manager, and `app/src/tui/mcp.rs` are removed MCP config surfaces. |
| `4783ab7cd` | N/A | Promote pinned tabs to Stable; the fork's `warp_features` enum has no `PinnedTabs` flag. |
| `93f8712c6` | Reject | Windows bootstrap build dependencies; `script/windows/` rejected platform. |
| `a5242e603` | Reject | Handle claude code `StopFailure` hook; depends on removed `app/src/ai/agent_sdk/`, `agent_management`, and `cli_agent_sessions` cloud-agent paths. |
| `3bf0899d5` | Defer | Exclude directories from command palette file search; `app/src/search/files/model.rs` diverged (fork uses `canonical_repo_path: PathBuf`, upstream uses a `relative_path` closure) producing 7 conflict regions. Retained local UX improvement; defer to a manual port. |
| `696837a25` | Reject | Terminate ConPTY root process on Windows PTY kill; `app/src/terminal/local_tty/windows/` rejected platform. |
| `87dafc769` | Reject | Fix asymmetric right padding on MCP tool chips; `app/src/settings_view/mcp_servers/server_card.rs` (MCP settings pane) removed. |
| `6f5d21145` | N/A | TUI alt-screen mouse interactions; `crates/warp_tui/` absent. |
| `fd210f13f` | N/A | Single chevron for TUI input prompt marker; `crates/warp_tui/` absent + `specs/` doc. |
| `73b2955ab` | Reject | Option snapshots + GUI orchestration pickers; `app/src/ai/orchestration/` and `run_agents` orchestration removed. |
| `0993c3161` | N/A | Route TUI editor input by owning-view focus; `crates/warp_tui/` absent. |
| `effa8ef9f` | Reject | Hide child (orchestrated) agent conversations; depends on removed `agent_management`, orchestration event streamer, and cloud-agent conversation model. |
| `7ffccbb82` | Reject | `oz runner` CRUD CLI commands; depends on removed `app/src/ai/agent_sdk/`, `crates/graphql/`, and `server_api` cloud paths. |
| `330337190` | N/A | Bump `ws` in `crates/warp_graphql_schema` yarn.lock; GraphQL schema crate removed. |
| `187916fe8` | N/A | Bump `namespacelabs/nscloud-cache-action` in `.github/workflows/feature_flag_cleanup.yml`; fork-owned CI. |
| `9fd88b6fe` | Reject | Mint IAP tokens for Oz runners via WIF; depends on removed `agent_sdk`, `warp_server_client` IAP, and cloud OIDC paths. |
| `a50b9dd04` | Reject | Fix flaky Windows PTY read test; `app/src/terminal/shared_session/sharer/` cloud session sharing removed. |
| `1c6ef6800` | Reject | PowerShell history read path embedding; entirely `#[cfg(windows)]`, fork is macOS-only. |
| `0cbc2563b` | N/A | Render semantic Markdown in TUI agent output; `crates/warp_tui/` absent. |
| `7e0ff783e` | N/A | Open TUI conversation list with left arrow; `crates/warp_tui/` absent. |
| `b63471799` | N/A | Hide outer agent bar when TUI runs in a pane; `app/src/terminal/cli_agent.rs` TUI-branch changes serve the absent TUI. |
| `8ca818387` | Reject | Fix clippy linting for TUI; bulk touches removed MCP file-based manager + `.github/workflows/ci.yml` TUI clippy gate. |
| `74a0d6758` | Defer | `files:` filter exclude directories; depends on the deferred `3bf0899d5` `get_repo_file_contents`/`exclude_folders` plumbing. |
| `ea24a1a56` | Reject | VA cursor visibility + 4x playback speed; depends on removed `crates/computer_use` recording + agent SDK + skills conversion. |
| `7c8362b27` | N/A | Add NLD support to TUI; `crates/warp_tui/` + `app/src/tui_export.rs` absent. |
| `cf1d88ae0` | N/A | Use `×` for TUI failed-tool-call glyph; `crates/warp_tui/` absent. |
| `0346faadb` | N/A | Render inline plans in TUI agent output; bulk is `crates/warp_tui/`. The shared `crates/ai/src/agent/document_action_presentation.rs` extraction is only consumed by the TUI. |
| `eb1c691a2` | Reject | Record targeted Linux windows with native x11grab; removed `crates/computer_use` Linux recording + agent SDK + skills. |
| `20dcd2905` | Defer | Fix pane header showing cloud icon for shared local conversations; the `tab.rs`/`terminal_model.rs`/`pane_impl.rs` shared-icon logic is retained, but the change is entangled with cloud shared-conversation metadata. Defer until the local/shared pane-icon boundary is revisited. |
| `336af3051` | N/A | Ctrl+Shift+P plan toggle for TUI; `crates/warp_tui/` absent. |
| `659cf3747` | N/A | Reusable TUI editor view; `crates/warp_tui/` absent. |
| `e7c409bad` | N/A | Reusable TUI option selector; `crates/warp_tui/` absent + `specs/` doc. |
| `f1547fefc` | N/A | Bottom padding below TUI footer/prompt; `crates/warp_tui/` absent. |

## Verification

Commands run after porting:

- `cargo check -p warp --all-targets --message-format short` — passed (no errors).
- `cargo fmt -- --check` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed, 2434 skipped.
- `cargo nextest run -p markdown_parser` — 141 passed, 0 skipped (covers the new `AUTOLINK_TRAILING_PUNCTUATION` autolink tests).
- `cargo nextest run -p warp -E 'test(box_drawing)'` — 10 passed (new procedural box-drawing tests).
- `cargo nextest run -p warp -E 'test(link_detection) | test(task_store) | test(exchange_by_id) | test(parse_url) | test(autolink)'` — all passed.
- `cargo nextest run -p warp --no-fail-fast` — 2587 run, 2578 passed, 9 failed, 3 skipped. The 9 failures are identical to the pre-merge `main` baseline (`test_plan_markdown_content_preserves_copyable_structure`, `test_focused_pane_is_synchronized_with_application_focus`, `test_tokenizer_warp_special_chars`, `test_smart_selection_override/in_multiple_blocks/in_single_block`, `test_find_url_omits_trailing_periods`, `test_secrets_serialization`, `inline_agent_view_persists_across_transfer_takeover_for_monitored_long_running_command`); confirmed pre-existing on `main` and unrelated to these ports.
- Deleted-surface scan of `v2026.07.14..HEAD` for `access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment` — no restored surfaces.
- Deleted-surface scan of `v2026.07.14..HEAD` for `mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill` — no restored surfaces.
- Deleted-surface scan of `v2026.07.14..HEAD` for `target_os = "linux"|target_os = "windows"|cfg(windows)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb` — no restored surfaces.
- No new Cargo dependencies were added.
