# Upstream Master Audit 2026-08-21

## Scope

- Current fork before this audit: `8157471a2` (`main`, `v2026.08.20`).
- Upstream source reviewed: `4e49d04f5a..upstream/master` (15 commits, tip `8936686f2`).
- Result: 1 commit accepted via adapted full port, 10 commits partially or fully ported as adaptations, 3 rejected or not applicable (Teams foundation, bundled factory-files skill, billing pricing breakdown).

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `8b88df987` | Show switch-to-tab shortcut hints while their modifier is held (#15221) | **Accept (ported)** | Fully local tab UI. `TabShortcutModifierState` singleton, `WorkspaceAction::SetTabShortcutModifierKey` forwarding via the workspace-root `on_modifier_state_changed`, binding-aware hint labels in the horizontal tab bar and vertical tabs sidebar, focus-loss clearing, and the 5 new tests. Adaptations: the vertical-tabs hint wiring uses the fork's `render_row_title_line` badge-arg form instead of upstream's unread-activity refactor; summary rows wrap their title in `render_row_title_line`; upstream's Windows/Linux ctrl+scroll-zoom branch omitted (removed platform). |
| `58464a291` | [multi-team PR 0] Context foundation: TeamContext and TeamRenderContext (#15348) | **Not applicable** | Entirely anchored on `app/src/workspaces/user_workspaces.rs` and multi-team scoping; `app/src/workspaces/` is removed in this fork (Teams/workspace discovery surface). |
| `f42c4ab6c` | Compute UserBlockCompleted's expensive fields lazily (#15162) | **Adapt (ported)** | See provenance below. |
| `ff16a0b2a` | Used faster hash maps and precomputed hashes in hot paths (#15279) | **Adapt (ported)** | New `warp_util::hashed::Hashed` + tests; workspace gains `hashbrown 0.17.1`, app gains `hashbrown` + `rustc-hash`. TaskStore: hashbrown map + `Hashed<TaskId>` root with `raw_entry().from_hash()` lookup; BlockList `block_id_to_block_index` → `FxHashMap`; AIBlock `requested_action_ids` → `FxHashSet` with `new_action_ids` hoisted out of the actions loop; AppContext window maps → `FxHashMap`. Omitted: `conversation.rs` `tasks_by_id` (fork conversation impl diverged) and the task-store pruning reachability/queue hunks (fork has no task-pruning feature). |
| `36dd2cc2e` | fix: coalesce AsyncSearcher full-index rebuilds to bound memory (APP-5389) (#15119) | **Adapt (ported)** | Ported to the fork's `app/src/search/searcher.rs` (upstream `crates/warp_search_core`): `QueuedItem` channel, `SearcherProducerState` pending-rebuild coalescing, `rebuild_index_async`, `merge_with_rebuild` chunked commits, and all 4 new async-rebuild tests. `launch_config`/`new_session` palette data sources switched to `rebuild_index_async`; the `warp_drive` data source hunk omitted (cloud sync surface removed). `report_error!` sites kept as fork `log::error!` (with `{e:#}`); let-chains rewritten as nested if. Drive-by: fixed the fork-baseline stale expected offset in `test_tokenizer_warp_special_chars` (`local_object-0` rename missed the offset; the test failed identically before this port). |
| `092c1dce9` | Preserve scroll position when toggling markdown Rendered/Raw views (#13967) | **Adapt (ported)** | `ScrollPosition::Fraction` + editor `RenderState` `scroll_fraction`/`scroll_to_fraction` (buffer-version-guarded), CodeView/LocalCodeEditorView passthroughs, `FileNotebookView::pending_scroll_fraction` consumed on `set_content`, pane `Replace*` events carrying ordered-float fractions, and `replace_code_pane_with_file_pane` constructing the pane empty then seeding scroll before opening. Omitted: `RenderState::new_tui` (TUI mode removed), the remote-file `OpenInEditor` branch and `file_state.path()`-based rewrites (fork FileNotebookView is local-path only; fork `open_as_code` now carries the fraction), `header_title_mouse_state` (absent tooltip feature). |
| `1eae40994` | factory-files: fix validator 401s in Oz staging sandboxes (#15378) | **Reject** | Bundled skill under `resources/bundled/skills/factory-files/` — app-bundled skills are a removed surface; the directory does not exist in the fork. |
| `0a0fd3ae1` | Add Paste to the block list right-click context menu (#15346) | **Adapt (ported)** | `paste_menu_item` helper (binding-aware, disabled on empty clipboard); Paste appended to the link menu and to the block list menu for right-click sources only. Upstream's share-block menu item and `share_block_label` omitted (shared-session cloud surface removed). |
| `3a7a4a5b3` | Don't draw empty category headers before any search pass (APP-5559, PR0) (#15376) | **Adapt (ported)** | `categories_with_visible_content` filter applied before the categorized settings page render. Upstream's `mod_tests.rs` additions omitted (fork has no `settings_view/mod_tests.rs`). |
| `1c6708dde` | Wire live per-category pricing breakdown into GUI usage surfaces (#15148) | **Not applicable** | Billing/usage pricing plumbing: `usage_totals`/`charged_usage` anchors, the usage rollup/views, `warp-proto-apis` pin, `workspaces/gql_convert.rs`, and `crates/graphql` are all absent from the fork. The only shared-file hunk (`terminal/view.rs`) is anchored on `conversation.usage_totals()` which does not exist in the fork. |
| `94daf47f3` | Add setting to disable '#' trigger for AI Command Search (#15340) | **Adapt (ported)** | See provenance below. |
| `0140af045` | Fix zsh compadd shim dropping descriptions from _describe's clustered -ld (#15313) | **Accept (ported)** | Applied verbatim to `app/assets/bundled/bootstrap/zsh_body.sh`. |
| `c25ac4070` | Add a right-click behavior setting: context menu (default) or paste (#15365) | **Adapt (ported)** | See provenance below. |
| `8936686f2` | Restore report_error! for flex infinite-constraint, guarded to report once (#15390) | **Adapt (ported)** | The fork has no `report_error!`; the flexible-children infinite-constraint site in `crates/warpui_core/src/elements/flex/mod.rs` now reports once per run via a static `AtomicBool` guard around `log::error!`, expressing upstream's `ReportErrorLogMode::OncePerRun` semantics. The debug-assert sibling site is unchanged, as upstream. |

## Provenance: `f42c4ab6c` port detail

- `crates/warp_util/src/lazy.rs` + `lazy_tests.rs` copied verbatim; `parking_lot.workspace = true` added to `warp_util`.
- `app/src/terminal/event.rs`: `UserBlockCompleted`'s five expensive fields become `Lazy<_, BlockList>`; `new` (pub(super)) and `new_for_test` constructors added verbatim.
- `app/src/terminal/model/block.rs`: `lazy_block_field!` macro resolves by stable `BlockId`; upstream's `report_error!` in the missing-block arm is the fork's `log::error!` with the block id. `From<&Block>` builds deferred fields; `Block` gains the three `compute_*` methods without upstream's `is_ai_ugc_telemetry_enabled` branches (telemetry field removed at baseline). The unused `command_and_output_with_secret_obfuscated` is replaced as upstream.
- Consumers adapted with `get_with`/`get` resolution: terminal `view.rs` (honor_ps1, pending-command success, block duration, workflow/env-var object-action exit data, history marking, `Event::BlockCompleted`, notification, alias-expansion, open-in-warp, command-corrections pre-spawn), `input.rs` (empty-command gate, `BlockContext::from_completed_block(&_, &self.model)`, rich-history pre-spawn resolution), `next_command_model.rs` (`get_next_command_context` resolves fields; `get_similar_history_context` takes resolved `command`/`pwd`/`exit_code`/`shell_host`), `block_context.rs` (`from_completed_block` takes the model), `passive_suggestions/terminal.rs` (fork equivalent of upstream's `legacy.rs` pattern), `current_prompt.rs` (`new_with_model_events` bundles `(model_events, terminal_model)`; `handle_model_event` resolves via a `Weak` model ref).
- `persistence/commands.rs`: `get_same_commands_from_history` takes resolved fields (upstream verbatim; the fork's version had no double-reverse bug).
- Tests: `blocks_test.rs` assertion updates plus the BlockId-reindex regression test adapted to the fork's `remove_block_at_index` and direct `block_list` test utils.
- Omitted upstream paths (no fork anchor): `ai/aws_credentials.rs` (BYOK removed), `passive_suggestions/legacy.rs`/`maa.rs` (old agent backends removed), `warp_tui/terminal_session_view.rs` (crate removed), the ApiKeyManager subscription move in `terminal_manager.rs` (manager removed), the input placeholder hunk (fork zero-state hint has no `'#'` advertisement branch), and the predicted-queries hunk (flow absent in fork).

## Provenance: `94daf47f3` port detail

- `InputSettings.enable_ai_command_search_hash_trigger` (default on, fork setting format without `sync_to_cloud`/`surface`).
- The `'`#`'` typed-character trigger gate in `terminal/input.rs` plus the `InputSettingsChangedEvent` handler refreshing zero-state hints; the hotkey action stays ungated.
- The toggle renders on the fork's Features page as `AiCommandSearchHashTriggerWidget`, following the existing `AtContextMenuInTerminalMode` widget/action pattern (upstream's `warp_agent_page.rs` `AIInputWidget`, telemetry `send_telemetry_from_ctx!`, and `ToggleSettingActionPair` binding registration omitted — page removed in fork; the fork's Features-page widgets do not use binding pairs).
- All three upstream tests ported to `input_test.rs`.

## Provenance: `c25ac4070` port detail

- `warpui_core` `EventHandler::on_right_mouse_down` handlers receive `&ModifiersState` (`HandlerWithModifiers`); `RightMouseDown` dispatch forwards cmd/shift with hit-testing. Fork keeps the nested-if edition-2021 form; the shift-reporting test ported to `event_handler_test.rs`.
- `SelectionSettings.right_click_behavior` (`ContextMenu` default | `Paste`, fork format) + `right_click_pastes()`.
- `BlockListElement::right_mouse_down` rework verbatim: raw right-click forwarded to a mouse-reporting long-running app, paste when enabled, else block/text/outside-block menu; `should_right_click_paste` helper in `terminal/mod.rs`.
- Right-click paste honored in alt-screen, prompt render helper, CLI-agent rich input, terminal input, terminal view, `settings_view/mod.rs`, and the AI blocklist/env-var/notebook-file right-click closures.
- Features-page dropdown mirrors the fork's `CtrlTabBehavior` pattern (`log_setting_result`/`log::error!` instead of upstream telemetry/`report_error!`/`report_if_error!`; no `sync_to_cloud`, no `LocalOnlyIconState` cloud icon).
- Five upstream right-click tests ported to `view_test.rs` (`EntityIdSet::default()` → `HashSet::new()` per the fork's test harness).
- Omitted: `notebooks/notebook.rs` (file absent), the file-notebook rich-editor context-menu right-click (`show_rich_editor_context_menu` absent), and `linux_selection_clipboard` (removed platform setting).

## Verification

- `cargo fmt -- --check`: clean after every port.
- `cargo check -p warp --all-targets --message-format short`: pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short`: pass.
- Focused tests: `warp_util` 72/72 (incl. 7 lazy + 8 hashed); `test(deferred_fields_resolve…)`/`test(test_background_blocks_finished)` pass; consumer suites (`terminal::input`, next-command/current-prompt/passive-suggestions/block-context/open-in-warp) 186/186; `test(task_store)` 28/28; `test(searcher)` 19/19 (incl. 4 new rebuild-coalescing tests); `test(right_click)` 6/6; `test(hash_trigger)`/`test(hotkey_opens…)` 3/3; `warp_editor` viewport tests 7/7; `warpui_core` event-handler tests 6/6.
- Standard suite `test(slash_command) | test(acp) | test(terminal_suggestions)`: 156/156.
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` after the release push.
- Pre-existing failures verified against unmodified HEAD (via stash): `warp_editor` `test_inline_markdown_roundtrips` and `terminal::model::blocks::selection` smart-selection tests — unrelated to these ports.
- Deletion-surface scans: MCP/skills scan 0 hits; broad removed-area scan hits in ported files are `Weak::upgrade` method calls and upstream-verbatim comments only; platform scan hits are retained SSH/remote paths (`ForwardX11=no`, zsh bootstrap ConPTY comments for remote hosts, pre-existing `#[cfg(windows)]` tests in `warp_util`).
- Disk note: local free space repeatedly exhausted during full-debug test builds; verification used `CARGO_PROFILE_DEV_DEBUG=0` for check/test/build runs, with `cargo clean` between phases. Final release build/clean follows the standard workflow.
