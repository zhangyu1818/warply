# Upstream Master Audit — 2026-11-17

Incremental audit for `3741d2b74..upstream/master` (`9c9f0b579`), 8 commits. The previous audit is `upstream-master-audit-2026-11-16.md`.

## Summary

Three commits ported (menu compile-once refactor, agent-hooks fifth PR, deny.toml key removal); five rejected or not applicable. No removed product surface was restored.

## Commit-by-Commit

### `b65097e5d` Compile the menu logic once instead of once per action type (#16850) — adapted port

`Menu<A>`/`MenuItem<A>`/`MenuItemFields<A>` become thin typed wrappers over the non-generic `AnyMenu`/`AnySubMenu`/`AnyMenuItem`/`AnyMenuItemFields`, with `ItemAction`/`AnyItemAction` erasure, the `MenuViewContext` trait, and `MenuItem::into_any`/`from_any` conversions. Ported by copying the exact upstream post-commit `app/src/menu.rs` and `menu_tests.rs`, then re-applying the fork's existing removals on top (all traced to removed surfaces):

- `width_match_position_id` + `set_width_match_position_id` + the render-time width override (upstream `1c4b181e4`, orchestration picker).
- `MenuTooltipPosition::Above` + `with_tooltip_position` (execution-profiles model picker).
- `RightSideIconConfig`, `right_side_icon`, `with_right_side_icon*`, `render_right_side_icon` (upstream `fc2dfe971`, auth secret deletion), including the action-drop block in `with_different_on_select_action_type` and `with_defer_events_to_children` (same commit).
- `image_icon` + `with_image_icon` + the `Image`/`AssetSource`/`CacheOption` render branch (upstream `35cb40c31`, the still-unported #10431 mermaid toggle).
- The `Expanded` clipped-label fix in `label_row` (upstream `f5ef3f8f2`, cloud-mode Execution host dropdown).
- `Menu::select_first` (upstream `3c3fcf34a`, cloud mode fixes); `select_first_selectable` is pre-existing fork code and stays.
- Unused imports (`AssetSource`, `CacheOption`, `Image`, `Expanded`, `icon_button_with_color`) dropped with their features; `#[path]` restored to the fork's `menu_test.rs` naming.

Omitted fork-absent paths: `app/src/menu_tests.rs` content ported into the fork's `app/src/menu_test.rs` (import header kept upstream-flat; both styles pass fmt), `app/src/notebooks/context_menu.rs` (`item_names` helper) and `app/src/notebooks/file/mod_tests.rs` (assert) — the fork's notebooks module has neither file. Public API checks: `Menu::items()` now returns `Vec<MenuItem<A>>`; fork callers in `workspace/view.rs` already use owned-compatible patterns.

### `2f2dda747` hooks: load and freeze explicitly configured sources (#16588) — adapted port

Fifth PR of the agent-hooks Phase 1 stack. `HookRegistry` gains the one-time `finalize` step (`is_active` now requires a finalized, non-empty registry), `HookSourceKind` drops the never-shipped `RunDelivered`/`ProjectCursorFile` variants, `HookSourcePolicy`/`HookOnAsk` replace the trust model, `trust.rs`/`trust_tests.rs`/`runner/mod_tests.rs` are deleted, `sources.rs`/`sources_tests.rs` add `ExplicitHookSource` + `load_explicit_sources` (behind `local_fs`), and the Cursor adapter gains `source_layout`/`HookDirectory`. Applied as an exact three-way patch over `crates/ai/src/hooks/**` and `app/src/ai/blocklist/hooks/**` — every library file was byte-identical to the upstream parent, so the patch landed cleanly. `HookSourceKind::FactoryAgent`/`FactoryWide` are local enum tags naming which part of a configuration selected a source; `sources.rs` itself only loads files from disk, and no runtime producer exists in the fork (the `AgentHooks` flag stays off by default, matching upstream Phase 1).

Fork adaptations kept from previous ports (divergences vs the upstream result): `hooks/mod.rs` driver checks still reduce to `conversation_exists` (no `ConversationDriver` split), `test_utils.rs` keeps `AIConversation::new()` without `RecordingController`/`UserWorkspaces`/child-conversation helpers, and `execute_tests.rs` keeps the fork's earlier test-module shape (Warply naming, `path_shell_quoting` instead of `read_file_failures`). `specs/agent-hooks/**` hunks rejected (specs stay out of the fork).

### `a36f769ed` [MCP] Refresh managed MCP proxy credentials during Oz agent runs (#16865) — rejected, one hunk ported

The managed-MCP credential-lease refresh lives in `agent_sdk`/`crates/mcp`/`warp_server_client`/managed-MCP paths, all fork-absent. The commit's `deny.toml` hunk (drop the unsupported `advisories.git-fetch-with-cli` for cargo-deny 0.21.0) is ported verbatim; `specs/managed-mcp-cli-resolution/TECH.md` rejected with the rest.

### `4be2b18b1` Improve execution time of some tests (#16852) — not applicable

All four files target fork-absent anchors: `app/src/server/server_api.rs` (removed server API), `crates/cloud_object_models` (crate absent), and `crates/http_client` `new_external_for_test` (the fork's trimmed http_client has no `new_external` — only `new_for_test`).

### `5bacd2b61` Send a User-Agent on MCP HTTP requests (#16866) — not applicable

`crates/mcp` is fork-absent; MCP belongs to the ACP agent process.

### `999111330` Remove git-fetch-with-cli from deny.toml (#16869) — not applicable

Empty commit against its parent (`5bacd2b61`): the upstream tree already dropped the key in `a36f769ed`, so this PR carries no diff. The fork receives the change through the `a36f769ed` hunk above.

### `fb26f8a3a` Keep the JSON-RPC error data from ACP agents (#16868) — not applicable

The change lives in `app/src/ai/agent_sdk/driver/harness/acp/` (fork-absent harness ACP lineage). The fork's own ACP client surfaces JSON-RPC errors through `crates/jsonrpc` `service.rs`, which already renders the full raw error value (`error.get()`, including `data`) in its anyhow message, so the fork does not drop the detail this PR preserves.

### `9c9f0b579` Fix partial upgrades in the Arch package builder (#16875) — not applicable

`.github/actions/bundle_arch_package/Dockerfile` is fork-absent Linux packaging.

## Restored-Surface Scans

All three skill scans re-run. Hit files match the 2026-11-16 audit's documented inert set: `mcp_server_denylisted` permission-floor booleans and Cursor MCP encode/fixtures inside the ported hooks library, `#[cfg(windows)]` arms in `crates/command`/hooks runner that compile out on macOS, retained SSH/remote-terminal and Warpify bootstrap hits, and `Weak::upgrade` word matches. A scan limited to this merge's touched files returns no hits.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo fmt -- --check` — passed repo-wide.
- `cargo check -p ai --all-targets --message-format short` — passed.
- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo nextest run -p ai` — 262 passed (hooks library incl. sources/explicit-load tests).
- `cargo nextest run -p warp -E 'test(hooks) | test(real_process)'` — 57 passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 157 passed.
- `cargo nextest run -p warp -E 'test(menu)'` — 150 passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean`.

Pre-existing warnings confirmed unchanged: the three unused `Menu` setters (`set_border`, `set_flatten_bottom_corners`, `set_pinned_header_builder`) had no callers at the merge base either.

## Tag

`v2026.11.17` on `main` (menu, hooks, deny.toml port commits plus this audit, merged with `--no-ff`).
