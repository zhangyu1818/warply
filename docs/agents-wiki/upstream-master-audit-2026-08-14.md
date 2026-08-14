# Upstream Master Audit 2026-08-14 — Incremental Terminal/AI/Perf Fixes + Settings Surface Port

## Scope

- Current fork before this audit: `f4462424b` (`main`, `v2026.08.13`).
- Upstream source reviewed: `5fb3144db9..upstream/master` (9 commits, tip `c9e562294`).
- Working-tree settings surface port carried from the `2026-08-13` correction (`fe8138bce8` selective port) was verified source-faithful and committed alongside this audit.

## Settings surface port (carried from 2026-08-13 correction)

Implements the widget-level rule recorded in `upstream-master-audit-2026-08-13.md`. The fork's `ai_page.rs` and `features_page.rs` now expose local CLI-agent, external-editor, and code-editor/review settings whose runtime ownership is already local or ACP-backed.

- `app/src/settings/ai.rs`: added `ai_autodetection_enabled_internal` / `nld_in_terminal_enabled_internal` (default `false`, matching upstream `agents.warp_agent.input.*` semantics, adapted to `ai.input.*` toml paths). Wired `add/remove/set_cli_agent_footer_enabled_command` to persist via `set_value` + `log_setting_result` (previously no-ops).
- `app/src/settings_view/ai_page.rs`: inline CLI-agent toolbar widgets (show/hide toolbar, auto-toggle/open/dismiss Rich Input, submit-on-Ctrl-Enter, command regex list, toolbar chip layout) rendered through the fork's existing `AISettingsPageView` pattern rather than upstream's separate `CLIAgentWidget` structs.
- `app/src/settings_view/features_page.rs` + `features/external_editor.rs`: Code Editor and Review category with `CodeAsDefaultEditor`, `AutoOpenCodeReviewPane`, `CodeReviewPanel`, `CodeReviewDiffStats`, and `ExternalEditorView` (editor/layout/conversation-layout dropdowns, tabbed-editor/markdown-viewer toggles).
- `app/src/settings/ai_tests.rs` / `terminal/input_test.rs`: test coverage for the new NL-detection defaults and CLI-agent toolbar persistence.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `7795e6728` | Enable vim mode on more multi-line editors (sweep) | **Accept (5/6 files)** | One-line `supports_vim_mode: true` on compact_agent_input, git commit dialog, workflow enum dialog, env-var command dialog, queued-prompt editor. `suggested_rule_modal.rs` skipped — Suggested Rules is a removed Warp Drive/Agent surface; file absent in fork. |
| `bc0f17ce7` | Improve EditFiles diff match failure errors | **Accept (adapted)** | `crates/ai/src/diff_validation/mod.rs` ported exactly (`DiffMatchFailure` struct, `fuzzy_match_failure_details` tracking, `Copy`→`Clone`). Error-message enrichment adapted from upstream `request_file_edits/diff_application.rs` (absent in fork) to fork's `edit_documents.rs` inline error path, appending block numbers. `mod_tests.rs`→`mod_test.rs` path adapted. |
| `8ba89e110` | QUALITY-928: Orchestration unified stack (M1) | **Reject** | Orchestration is a removed cloud-agent area. All anchor symbols absent (`orchestration_child_tracker`, `orchestration_event_streamer`, `child_agent/hydration`, `server_api/ai.rs`). |
| `2861a6e43` | Bump command-signatures to 77c4a9a7 | **Accept (superseded)** | Intermediate bump; final state applied via `3535362d7`. |
| `3535362d7` | Bump command-signatures to 32a7fd56 | **Accept** | Cargo.toml rev `fe352669`→`32a7fd56`; Cargo.lock updated. Upstream `winit`/`x11rb` lines NOT added (Linux/X11 deps removed in macOS-only fork). |
| `6e192572e` | Arc-wrap FileTreeState.gitignores | **Accept** | `crates/repo_metadata/src/file_tree_store.rs` + `local_model.rs` applied cleanly. Pure O(1) refcount-bump perf fix. |
| `63a17a50a` | Cache is_passive on AIBlock | **Accept (adapted)** | `is_passive: bool` field, constructor cache, `reset_conversation_id` recompute, `is_passive_conversation()` signature change. Avatar fields (`profile_image_path`, `user_display_name`, `user_avatar_info_for_ai_block`) omitted — removed with account surfaces. `pending_unit_test_suggestion` branch in `editor/view/mod.rs` omitted — method absent in fork. `view_tests.rs` regression tests skipped — file absent in fork. |
| `eaf70a6af` | Bound code review diff memory for huge untracked files | **Accept** | `app/src/code_review/diff_state/local.rs` early `MAX_DIFF_SIZE` guard applied. Import conflict resolved (fork's split `diff_size_limits` imports merged). |
| `c9e562294` | Settings: split the Code page | **Not applicable** | Fork has no `code_page.rs` to split; Code widgets live in `features_page.rs`. AI-page subpage refactor (`Option<AISubpage>`→`AISubpage`) targets upstream's multiplexed page model, which the fork doesn't use. Integration-test infrastructure depends on fork-absent settings structure. |

## Verification

- `cargo fmt -- --check`: clean.
- `cargo build -p warp --all-targets --message-format short`: succeeded (pre-existing warnings only).
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions) | test(diff_validation) | test(natural_language_detection) | test(cli_agent_toolbar)'`: 158 passed.
- `cargo nextest run -p ai -E 'test(diff_validation)'`: 32 passed.
- Deletion-surface scans: all hits are false positives (`Weak::upgrade()`, tokenizer vocabulary), retained SSH behavior (`ForwardX11=no`), shell-bootstrap comments (`ConPTY`), or test cfg guards (`#[cfg(windows)]` in remote-path tests).

## Omitted paths and concrete reasons

- `suggested_rule_modal.rs` (vim sweep): removed Suggested Rules surface.
- `winit`/`x11rb` (command-signatures bump context): Linux/X11 deps, removed in macOS-only fork.
- Avatar fields + `pending_unit_test_suggestion` (is_passive cache): removed account/unit-test-suggestion surfaces.
- `view_tests.rs` (is_passive cache): file absent in fork.
- `diff_application.rs` / `diff_application_tests.rs` (EditFiles diff): module absent in fork; error enrichment adapted to `edit_documents.rs`.
- Orchestration commit (`8ba89e110`): entirely removed cloud-agent area.
- Code-page split (`c9e562294`): fork uses a different settings-page organization.
