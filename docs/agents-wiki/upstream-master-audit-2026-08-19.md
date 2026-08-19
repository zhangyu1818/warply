# Upstream Master Audit 2026-08-19

## Scope

- Current fork before this audit: `4fa4021fa` (`main`, `v2026.08.18`).
- Upstream source reviewed: `f466967f03..upstream/master` (12 commits, tip `8ba01aa1a8`).
- Result: 3 commits ported (`213c9b32e` SignatureCache bound, `e0d01fff4` excessive-memory confirmation, `8ba01aa1a` grid_renderer report throttle); 9 rejected or not applicable.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `569219106` | Remove 1px workspace border on web (wasm) (#15259) | **Not applicable** | Adds `#[cfg(target_family = "wasm")]` branches on `WORKSPACE_PADDING` in `app/src/workspace/view.rs`. The fork has no wasm target and removes wasm compile branches; the native value (`1.0`) is unchanged by the upstream patch. |
| `f7ca8506f` | Record cache mount errors in traces (#15234) | **Not applicable** | `crates/build_cache` does not exist in this fork. |
| `98b1f5af8` | Fix ⇧ glyph font-fallback mismatch in web keybinding chips (APP-5492) (#15261) | **Not applicable** | `app/src/font_fallback.rs` and `script/font_fallback/` are removed (wasm-only fallback-font table and its obsolete generator). Native macOS builds never consult the table. |
| `d13a30f4a` | TUI: add /usage command (#14968) | **Reject** | Billing/credits usage panel in `crates/warp_tui` (removed) plus `/usage` static-command registration in retained `commands.rs`; the command's runtime is account/billing data, a removed surface. |
| `9921300b7` | REMOTE-2597 Report "Cancelled by user" when Ctrl-C interrupts a third-party harness run (#15257) | **Reject (deferred)** | The only implemented observation hook is `write_viewer_bytes_to_pty` (shared-session viewer input path), which is removed in this fork; `FeatureFlag::CtrlCCancelsThirdPartyHarness`, `agent_management`, `agent_sdk` driver, and `local_agent_task_sync_model` anchors are all absent. The local-keystroke parity path is explicitly out of scope upstream ("left for a follow-up"), so porting would require writing wiring upstream deliberately did not ship. Revisit if upstream ships the local-parity follow-up; the fork's live `cli_agent_sessions` model could host it then. |
| `b870d25d7` | Make ephemeral MCP installation IDs deterministic across sandbox rebuild (#15283) | **Reject** | `app/src/ai/agent_sdk/driver.rs` (removed surface), app-managed ephemeral MCP installation, `uuid` v5 workspace feature, and `script/windows/*.ps1` changes — all removed areas. |
| `b90c734eb` | Add container/member doc-comment guideline to AGENTS.md (#15285) | **Not applicable** | Fork-owned `AGENTS.md` entry contract; upstream agent-guide process docs are not fork memory (kept in `docs/agents-wiki/`). |
| `213c9b32e` | Bound SignatureCache growth with a key-length cap and a bounded FIFO miss cache (APP-5431) (#15181) | **Accept (ported)** | Retained `warp_completer` completions fix for unbounded user-input-keyed cache growth. See provenance below. |
| `e0d01fff4` | Suppress excessive-memory reports for confirmed-transient spikes (#15165) | **Adapt (ported)** | Two-poll confirmation ported into the fork's local `check_for_excessive_memory_usage`; telemetry/Sentry paths omitted (no anchor). See provenance below. |
| `b1731dde0` | Throttle TUI frame-draw error reports to once per run (#15287) | **Not applicable** | `crates/warpui_core/src/runtime/mod.rs` and `spawn_tui_driver` do not exist in this fork (TUI runtime removed). |
| `d2af51aa3` | REMOTE-2931: carry HTTP status on public-API errors so deterministic 4xx stop retrying (#15297) | **Not applicable** | `app/src/server/server_api.rs` is removed in this fork (Warp server API). |
| `8ba01aa1a` | Throttle grid_renderer out-of-bounds row report to once per run (#15288) | **Adapt (ported)** | Retained terminal renderer log-flood fix, adapted from `report_error!`/`ReportErrorLogMode` (removed `warp_errors` crate) to throttled `log::error!`. See provenance below. |

## Provenance: `213c9b32e` port detail

Ported from the exact upstream patch:

- `crates/warp_completer/src/signatures/legacy/miss_cache.rs` and `miss_cache_tests.rs`: copied verbatim (bounded RwLock FIFO set, `MAX_CACHED_MISSES = 256`; 4 tests pass).
- `legacy/mod.rs`: upstream `mod miss_cache;` hunk applied exactly.
- `registry.rs`: upstream hunks applied via three-way patch — `MAX_CACHEABLE_COMMAND_LEN = 255`, `signatures: MemoMap<String, Signature>` retype (misses no longer stored as `None` entries), `misses: MissCache` field, the hit → miss-cache → lookup → insert ordering in `get`, the `insert`/`registered_commands`/`register_signature` doc-and-code updates.
- `registry_test.rs` (fork's singular-named twin of upstream `registry_tests.rs`): upstream test hunks applied — `signature_with_name`/`track_longest_name` helpers, `test_all_known_signature_names_are_within_the_length_cap` (passes against the pinned `ac69f9b0` corpus: longest known name is within the cap), and the 5 boundary tests (`test_oversized_command_is_not_cached_and_resolves_to_none`, `test_misses_are_never_cached_in_the_positive_cache`, `test_oversized_later_token_does_not_bypass_the_length_guard`, `test_ordinary_commands_still_resolve_and_are_cached`, `test_registered_signature_longer_than_the_cap_is_unresolvable`, `test_registered_commands_unaffected_by_oversized_lookups`).

Intentionally omitted or adapted upstream hunks:

- The `cfg!(windows)` `.exe` trim context in `SignatureCache::get`: the fork deleted those branches at baseline; the length cap and cache logic apply unchanged without them.
- `#[cfg(windows)] test_exe_suffix_is_trimmed_before_the_length_check`: omitted — the fork has no windows local host and no `.exe` trim to test.
- `warpui_core::r#async::block_on` in `test_oversized_later_token_does_not_bypass_the_length_guard`: adapted to `warpui::r#async::block_on`, matching the fork's `warp_completer` → `warpui` dependency (upstream depends on `warpui_core`) and the import style of the fork's existing tests in the same file.
- Import ordering kept in the fork's crate-first style.

## Provenance: `e0d01fff4` port detail

Ported from the exact upstream patch on `app/src/system/info.rs`, reduced to the retained local utility:

- Threshold renamed to `MEMORY_USAGE_WARNING_THRESHOLD_BYTES: u64` (`Byte::GIGABYTE.as_u64() * 10`).
- New `pending_excessive_memory_footprint_bytes: Option<u64>` field with the upstream doc comment, initialized in `SystemInfo::new`.
- `check_for_excessive_memory_usage` now records a threshold crossing as pending and only reports (jemalloc heap dump under `heap_usage_tracking`, `SystemInfoEvent::MemoryUsageHigh`, latch consumption) once the next 5s poll tick confirms the footprint is still excessive. The unconfirmed branch keeps the upstream `log::info!` skip message verbatim and does not consume the once-per-process latch.

Intentionally omitted upstream hunks:

- `app/src/server/telemetry/events.rs` `TransientMemorySpike` event and the `send_telemetry_sync_from_ctx!` call in the skip branch: telemetry is removed in this fork (no anchor).
- Upstream struct fields `stats`/`resource_usage_reporter`/`long_os_version` and their init lines: fork-removed resource-usage telemetry state.
- Doc references to Sentry/Rudderstack events adapted to the retained local heap-profile behavior; the upstream function doc's footprint rationale and confirmation semantics are kept.

## Provenance: `8ba01aa1a` port detail

Ported from the exact upstream patch on `app/src/terminal/grid_renderer.rs`:

- `used_displayed_output_rows: bool` threaded through `render_grid_without_ligatures` and `render_grid_with_ligatures` signatures and all four `render_grid` call sites (`visible_rows` paths pass `true`, `start_row..end_row` paths pass `false`), exactly as upstream.
- Both out-of-bounds-row reports now carry the upstream diagnostic context (`row_idx`, `total_rows`, `start_row`, `end_row`, `used_displayed_output_rows`, `use_ligature_rendering`) and report at most once per run per site; the non-ligature site's `#[cfg(debug_assertions)]` gate is removed so both paths report identically, per upstream.

Fork integration glue (handwritten, replacing the removed `warp_errors` crate):

- `report_error!(..., extra: {...}, ReportErrorLogMode::OncePerRun)` is expressed as a per-site `static REPORTED_OUT_OF_BOUNDS_ROW: AtomicBool` + `swap(true, Relaxed)` guard around `log::error!` with the same fields formatted into the message. The fork removed the `warp_errors` crate; `report_error!` sites were converted to `log::error!` at baseline.

## Verification

- `cargo check -p warp_completer --all-targets`: pass.
- `cargo nextest run -p warp_completer`: 163 passed, 4 skipped (includes the 5 new `miss_cache` tests and the 6 new registry tests; `test_all_known_signature_names_are_within_the_length_cap` passes against the pinned corpus).
- `cargo check -p warp --all-targets` / `cargo check --workspace --all-targets`: pass (pre-existing warnings only).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `cargo nextest run -p warp -E 'test(system)'`: 12 passed.
- `cargo fmt -- --check`: clean.
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` after the release push.
- Deletion-surface scans: MCP/skills scan 0 hits; broad removed-area and platform scan hit sets identical to the `v2026.08.18` baseline (differences are rg-vs-git-grep binary-reporting artifacts on the ONNX tokenizer vocabulary and test sqlite fixtures only; no new product-surface hits from the ported files).
