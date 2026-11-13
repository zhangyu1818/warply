# Upstream Master Audit — 2026-11-13

Incremental audit for `0515627a4..upstream/master` (`8048d09e6`), 11 commits. The previous audit is `upstream-master-audit-2026-11-12.md`.

## Summary

Two adapted ports: the warpui_core spawn-path monomorphization fix (`783bc3b08`, compile-time: one non-generic `AppContext::spawn_abortable_erased` replaces per-entity/per-output copies of the oneshot relay) and the pending-command exact-submission fix (`8f4865adc`, `PendingCommand::Exact` stores the queued command verbatim so a leftover or concurrently edited input buffer can no longer concatenate with or replace it). Nine rejected/not-applicable commits live on fork-absent surfaces (billing/usage, agent_sdk git-credentials/prewarm/MCP harness, preview proxy, Factory flags). No removed product surface was reintroduced.

## Commit-by-Commit

### `0b2db6e0b` Standardize included personal usage labels (#16806) — rejected

Billing usage copy rename ("Base usage/credits" → "Included personal usage/credits") across `app/src/pricing/`, `app/src/settings_view/billing_and_usage_page*.rs`, and `crates/warp_tui/src/terminal_session_view/usage_menu*`. All paths fork-absent (verified: no `app/src/pricing/`, no `billing_and_usage_page*`, no `crates/warp_tui`). Billing/usage is a removed product area; nothing separable.

### `5603d2c1f` Display billing usage as credit-equivalent dollars (#16804) — rejected

Credit-equivalent dollar valuation in `app/src/settings_view/billing_and_usage/billing_cycle_usage_*` and its tests. The entire `billing_and_usage/` directory is fork-absent (verified). Billing/usage accounting is removed.

### `f11602bd1` [REMOTE-2960] Enable deferred Factory repositories by default (#16811) — not applicable

Promotes `FactoryDeferredRepositories` from dogfood to default-on (workspace manifest feature + `RELEASE_FLAGS`). Neither the flag nor the `factory_deferred_repositories` cargo feature exists in the fork (verified by search on `crates/warp_features/src/lib.rs` and `app/Cargo.toml`); the Factory cloud-run repository capability was rejected with the 2026-11-05 skill-reference stack.

### `073b7c4c5` Restore restricted Git credential retries and safe refresh recovery (#16813) — rejected

Retry/refresh recovery for restricted GitHub bot credentials in `app/src/ai/agent_sdk/driver/git_credentials{,/restricted.rs,/restricted_tests.rs}` plus `app/src/server/server_api/ai.rs`. The whole `git_credentials` stack was rejected with `651109041`/`619c82aa7` in the 2026-11-11 audit; all four files are fork-absent (verified).

### `d029c5922` feat: add configurable driver preview proxy with workload port cache (#16786) — rejected

New `crates/preview_proxy` crate (hyper-util/http-body-util workspace deps) plus `app/src/ai/agent_sdk/driver/preview_proxy*` wiring through `app/src/server/server_api/ai*`, proxying preview URLs for cloud driver workloads. Every consumer is fork-absent (`agent_sdk` driver, `server_api`); the two new workspace dependencies exist only for this feature, so the manifest hunks are omitted too.

### `3298829b5` Repair GitHub MCP credential startup and refresh (#16815) — rejected

GitHub MCP credential startup/refresh repairs spanning `app/src/ai/agent_sdk/**`, `app/src/ai/mcp/templatable_manager*`, and `app/src/ai/skills/resolve_skill_spec.rs`. App-managed MCP servers, app-resolved skill specs, and the agent_sdk driver are all removed surfaces in this fork (verified fork-absent paths); MCP configuration belongs to the ACP agent process.

### `63c8e7f7f` Use GitHub MCP credentials in prewarmed runtimes (#16816) — rejected

Threads restricted git credentials into prewarmed runtimes across `app/src/ai/agent_sdk/**` (driver, prewarmed, standby, mcp_startup), `app/src/server/server_api/prewarmed_sandbox*`, and `crates/graphql`/`crates/warp_graphql_schema`. All fork-absent (verified: no `agent_sdk`, no `server_api`, no graphql crates).

### `e0251e1ef` Fix managed MCP startup order for prewarmed runs (#16819) — rejected

Single-file startup-ordering fix in `app/src/ai/agent_sdk/driver/prewarmed.rs` — fork-absent (verified).

### `783bc3b08` Compile the warpui_core spawn path once instead of per entity and output type (#16789) — adapted, ported as `fbe15d7e7`

Framework compile-time fix: `ModelContext::spawn`/`spawn_abortable` and `ViewContext::spawn`/`spawn_abortable` now box the future output as `ErasedOutput` and route through one non-generic `AppContext::spawn_abortable_erased` in the new `core/spawn.rs` (channel, abort handle, background task, main-thread relay compiled once). Cherry-picked exactly, then adapted:

- `report_error!` → `log::error!` (fork removed the `warp_errors` crate; established adaptation).
- View callbacks typed over `&mut dyn AnyView` (the fork's `StoredView`-removal lineage) with a `downcast_view` helper going through `as_any_mut()`; the model path keeps upstream's `downcast_entity` verbatim.
- The wasm `ErasedOutput = Box<dyn Any>` cfg omitted; the fork keeps only `Box<dyn Any + Send>` (macOS-only target, no wasm).
- Import hunks restructured around the fork's surrounding import blocks; `ModelContext::spawn_local` kept `#[cfg(test)]` as upstream.
- Nothing else omitted; `cargo nextest run -p warpui_core`: 297 passed, 7 skipped.

### `8f4865adc` Fix setup command submission racing and timing out (#16808) — adapted, ported as `47557639f`

Terminal input fix on the retained pending-command path (fork callers: launch-config tab commands, URI deeplinks, workspace editor/tail commands, terminal-view shell commands, synced-input submission). `has_pending_command: bool` becomes `pending_command: Option<PendingCommand>` with `InputBuffer`/`Exact(String)` variants; `set_pending_command` stores the command verbatim via `set_buffer_text` instead of `system_insert`; `execute_pending_command` restores the exact text before executing and only clears the pending state when `try_execute_command` returns `true`; `run_command_in_synced_terminal_input` renamed `submit_current_input`. Cherry-picked exactly (rename detection mapped `input_tests.rs` → the fork's `input_test.rs`), then adapted:

- The two `debounce_ai_query_prediction_tx` context lines dropped (fork-absent field).
- `finish_bootstrap_precmd` helper ported in the fork's fixture style (`command_finished`/`precmd` with `..Default::default()`, no completion metadata), replacing the inline block in `pending_command_waits_for_final_bootstrap_precmd`.
- Three of the four new tests ported verbatim (`exact_pending_command_does_not_concatenate_leftover_input`, `exact_pending_command_ignores_editor_changes_while_waiting`, `submit_current_input_executes_editor_contents_instead_of_pending_payload`).
- `exact_pending_command_retries_with_shared_session_attribution` omitted — its machinery (`SharedSessionStatus::ActiveSharer`, `PresenceManager::new_for_sharer`, `CommandExecutionSource::SharedSession`) is the removed shared-session surface.
- Upstream pre-existing tests that the rename-detection conflict block pulled in (`unshared_terminal_executes_command_locally`, the shared-session setup/attribution series) are not part of this commit and were not ported.
- Focused tests: 5/5 pass including the three new ones.

### `8048d09e6` Gate credit-equivalent billing usage to local and staging (#16812) — rejected

Gates #16804's credit-equivalent displays behind a new `CreditEquivalentBillingUsage` flag in `crates/warp_features/src/lib.rs` plus `billing_and_usage/` UI. The billing files are fork-absent and the flag does not exist in the fork's `warp_features` (verified); billing/usage remains removed.

## Verification

All commands run with `SDKROOT` pointed at the full Xcode SDK and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo check -p warpui_core --all-targets --message-format short` — passed after the spawn port.
- `cargo nextest run -p warpui_core` — 297 passed, 7 skipped.
- `cargo nextest run -p warp -E 'test(pending_command) | test(submit_current_input) | test(exact_pending)'` — 5 passed.
- `cargo check -p warp --all-targets --message-format short` — passed.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo fmt -- --check` — passed repo-wide.
- `cargo check --workspace --all-targets --message-format short` — passed.
- `cargo build -p warp --all-targets --message-format short` — passed, followed immediately by `cargo clean` (13.3 GiB removed).
- Restored-surface scans re-run: the MCP/skills pattern scan returns zero hits, the platform scan returns the same known retained occurrences (SSH `ForwardX11=no`, retained ConPTY comments in `zsh_body.sh`, `#[cfg(windows)]`-gated `warp_util` tests), and the cloud/auth/billing/telemetry hit list is byte-identical to `main` (verified by `rg -c` diff) — unchanged from the 2026-11-12 baseline.

## Tag

`v2026.11.13` on `main` via `merge/upstream-2026-11-13` merged with `--no-ff`.
