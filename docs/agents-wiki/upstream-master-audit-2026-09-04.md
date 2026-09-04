# Upstream Master Audit 2026-09-04

## Scope

- Current fork before this audit: `9ec2cc1d1` (`main`, `v2026.09.03`).
- Upstream source reviewed: `b9c21aa01f..upstream/master` (10 commits, tip `83ddbefff7`).
- Result: four direct ports (fish ctrl-r fzf.fish handoff, two command-signatures pins, smart_select reversed-glyph guard), two adapted ports (in-band command reset warning, Attach file command palette action), and four rejected/not-applicable commits.

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `e69555270` | Keep WASM conversation-viewer chrome after owned HandoffCloudCloud restore (APP-5753) | **N/A** | Entire change lives in `#[cfg(target_family = "wasm")]` code (`get_simplified_wasm_tab_bar_content`, `SimplifiedWasmTabBarContent`, `Workspace_CloudConversationWebViewer`, `opened_from_content_deep_link`). No anchor symbol exists in this fork; the WASM/web compile branch and cloud conversation viewer were removed. |
| `7c360f772` | Hand fish ctrl-r to PatrickF1/fzf.fish history search (GH 4125) | **Accept (ported)** | Purely local shell-integration follow-up to the ported `bf2364bc9` handoff. Exact upstream patch to `fish.sh` applied cleanly: allowlist `_fzf_search_history`, invoke the tagged widget instead of hardcoding `fzf-history-widget`, and guard tagging with `functions -q`. |
| `51a74992a` | Pass confirmation dialogs in 3p harnesses, and ensure driver exits | **N/A** | Targets `app/src/ai/agent_sdk/driver/**` (`AgentDriver::run_harness`, claude/codex harnesses, `use_agent_footer`), all removed with the agent SDK. No `run_harness`/`AgentDriver` anchor exists in this fork; the ACP/`CLIAgentSessionsModel` hosts have no equivalent wait-forever exit loop. The `nix` `process` feature was for its kill ladder only. |
| `1ad7f191c` | Update common skills lock for warpdotdev/common-skills#92 | **Reject** | `skills-lock.json` distributes bundled skills; the file does not exist in this fork. |
| `81744c2dc` | Pin command-signatures so git ls-remote X-Ray describes --branches | **Accept (ported)** | Exact `d3725aa` → `1776eb16` pin of the retained `warp-command-signatures` dependency; fork was on the exact upstream baseline rev. Conflict resolution dropped upstream's `winit`/`x11rb` workspace lines (absent in this fork). |
| `c3c397d6c` | Switch WASM team picker in place instead of opening a window (REV-2280) | **Reject** | Core is the Teams `OpenNewWindowForTeam` action (`UserWorkspaces::team_uid_for_window`, `NewWorkspaceSource::TeamSwitched`) plus a WASM branch; both surfaces are removed. The separable warpui_core test-only helper `WindowManager::last_window_shown_and_focused_for_test` (default trait method + test-delegate tracking + `test-util` accessor) was reviewed and not ported: its only consumers are the team-window focus tests on the removed Teams surface, so it would be dead test-util API here. |
| `58d94a53b` | Guard reversed glyph bounds in smart_select | **Accept (ported)** | Local crash fix in `warpui_core` `FormattedTextElement::smart_select` (`return None` when a line's visual first glyph has a higher logical index than its visual last glyph, which RTL layout can produce) plus the upstream regression test. Path remapped to the fork's pre-split `elements/formatted_text_element*.rs` (no `gui/` segment); test-file imports kept on the fork's set with only the new `FontId`/`Glyph`/`Line`/`Run`/`TextAlignment`/`TextStyle`/`vec1` imports added (`StringRange` already re-exported via `super`). |
| `5cd24ed1b` | Silence expected in-band command reset warning (CORE-3810) | **Adapt (ported)** | Exact upstream hunk: the `end_in_band_command_output` no-op warning now fires only for unexpected end OSC sequences (`IsReceivingInBandCommandOutput::No if from_osc_sequence`), expected path silent. Fork adaptation: drop the `_` prefix on the now-used `from_osc_sequence` parameter. |
| `142b87102` | Add unbound Attach file command palette action | **Adapt (ported)** | See port record below. |
| `83ddbefff` | Complete git merge-base | **Accept (ported)** | Exact `1776eb16` → `6a39d620` pin adding `git merge-base` completions; same `winit`/`x11rb` conflict-resolution note as `81744c2dc`. |

## Port record: `142b87102` Attach file command palette action

### Runtime-ownership review

Local ACP/CLI-agent behavior: a new unbound, editable `terminal:attach_file`
binding dispatches `TerminalAction::AttachFile`, which routes through
`Input::attach_file` → `AgentInputFooter::select_file` — the same
CLI-session/agent-view branch the plus button already uses (`select_cli_file`
file picker for active CLI agent sessions, `AgentInputFooterEvent::SelectFile`
for the agent-view attachment flow). The gate is the agent view (fullscreen or
inline) or an active CLI agent session. No Warp service dependency.

### Applied from the exact upstream source

- `view_components/action_button.rs`: `tooltip_keybinding` field,
  `with_tooltip_keybinding`, and the render-time tooltip sublabel via
  `KeystrokeSource::Binding(name).displayed(app)` (the fork already had
  `KeystrokeSource` with this API) — verbatim.
- `workspace/view/right_panel.rs`: both maximize-button hunks refactored onto
  `with_tooltip_keybinding("workspace:toggle_maximize_code_review_panel")` —
  verbatim (fork pre-image was identical).
- `agent_input_footer/mod.rs`: `ATTACH_FILE_KEYBINDING` import, plus-button
  `.with_tooltip_keybinding(ATTACH_FILE_KEYBINDING)`, extracted
  `pub(crate) fn select_file`, and the simplified
  `AgentInputFooterAction::SelectFile` arm — verbatim.
- `view/action.rs`: `AttachFile` variant + `Debug` arm, anchored between
  `DeleteAttachment` and `ToggleAutoexecuteMode` (upstream neighbors
  `OpenAttachmentLightbox`/`WriteCodebaseIndex` are absent in this fork).
- `view/init.rs`: `ATTACH_FILE_KEYBINDING`/`CAN_ATTACH_FILE_KEY` consts and the
  `EditableBinding` with the upstream context predicate
  `(Input | Terminal) & (ACTIVE_AGENT_VIEW | ACTIVE_INLINE_AGENT_VIEW |
  CLI_AGENT_SESSION_ACTIVE_KEY) & CAN_ATTACH_FILE_KEY`.
- `view.rs`: `is_in_agent_or_cli_attach_context` (verbatim), `can_attach_file`,
  the a11y no-content arm, the dispatch arm with the `can_attach_file` guard,
  and the `CAN_ATTACH_FILE_KEY` keymap-context insertion.
- `input.rs`: `attach_file` method (verbatim), the
  `CLI_AGENT_SESSION_ACTIVE_KEY` insertion in `Input`'s view context before
  `EMPTY_INPUT_BUFFER` (verbatim), and a `CAN_ATTACH_FILE_KEY` insertion.

### Fork adaptations

- `file_attach_allowed_for_shared_session` and every
  `shared_session_status()`/`available_to_session_viewer`/`CloudModeImageContext`
  /ambient-agent-view-model gate is omitted: the fork has no
  `SharedSessionStatus`, no `ambient_agent` module, no `CloudModeImageContext`
  flag, and no `available_to_session_viewer` helper. Attach availability is
  gated only by the agent/CLI attach context:
  `can_attach_file = is_in_agent_or_cli_attach_context`, and the TerminalView
  context inserts `CAN_ATTACH_FILE_KEY` under that predicate while the Input
  context inserts it unconditionally (upstream's shared-session computation
  is vacuously true here). The upstream doc comment on `CAN_ATTACH_FILE_KEY`
  ("Shared-session availability … cloud-viewer exception") was dropped because
  it describes semantics this key does not have in the fork.
- `bindings::BindingGroup::WarpAi` → `bindings::BindingGroup::Ai` (the fork's
  `BindingGroup` predates upstream's WarpAi rename).
- Enum/Debug/a11y/dispatch arm anchors use `DeleteAttachment` as the
  predecessor since `OpenAttachmentLightbox` and `WriteCodebaseIndex` do not
  exist in this fork.

## Addendum: late-arriving commit `a7326f8fe`

After the `v2026.09.04` tag was pushed, upstream master advanced by one
commit: `a7326f8fe` "Fix crash when promoting an end-of-line cell to a wide
char" (#15763, fixes #15753). It was reviewed and ported as an addendum
commit on `main` (after the tag; it ships in the next release tag).

**Accept (ported).** Purely local terminal-grid fix: a variation selector that
promotes an end-of-row grapheme from one to two cells wide could create
inconsistent wide-char metadata that later crashed `FlatStorage` row
reconstruction. `push_zerowidth` now reports whether the append was accepted
and gains a reversible `pop_zerowidth`; the zero-width path applies wide-char
layout only when the grapheme width actually changes; normal wide writes and
width promotions share one `write_wide_char` helper (wrapping, spacer
creation, hyperlink propagation, cursor advancement); an end-of-row promoted
grapheme moves to the next row when line wrapping is enabled and the selector
is rolled back when it is disabled. All upstream regression tests ported
(cell accepted/rejected/rollback appends; end-of-row promotion, disabled line
wrapping, already-wide graphemes, hyperlink propagation in
`grid_handler_test.rs`).

Path remaps: `crates/warp_terminal/src/model/grid/ansi_handler.rs` →
`app/src/terminal/model/grid/ansi_handler.rs` and
`grid_handler_tests.rs` → `app/src/terminal/model/grid/grid_handler_test.rs`
(the fork moved the handler layer into the app at creation);
`cell_tests.rs` → `cell_test.rs` (fork singular test filename; matches the
existing `#[path = "cell_test.rs"]` module attribute). Every hunk applied
cleanly; remaining whole-file differences vs upstream are pre-existing fork
divergences (edition 2024 let-chains, import ordering).

Verified: `cargo check -p warp_terminal -p warp --all-targets` and
`cargo check --workspace --all-targets` pass; upstream-specified suites pass
(cell zerowidth 3, warp grid/ref 174 incl. 19 grid-handler wide-char tests,
156 slash_command/acp/terminal_suggestions); `cargo fmt -- --check` clean;
`cargo build -p warp --all-targets` pass.

## Verification

- `cargo fmt -- --check`: clean.
- `cargo check -p warp --all-targets --message-format short`: pass (8
  pre-existing lib warnings: warpui mac unsafe-block set, remote_server
  `client_event_kind`, `test-util` cfg note — identical on `main`).
- `cargo check --workspace --all-targets --message-format short`: pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`: 156 passed.
- `cargo nextest run -p warpui_core -E 'test(smart_select)'`: 2 passed (incl. new `smart_select_returns_none_when_line_glyph_indices_are_reversed`).
- `cargo nextest run -p warp -E 'test(terminal_model) + test(keymap)'`: 28 passed; `test(terminal::input) + test(action_button)`-area suites 266 passed.
- `cargo check -p warp_completer` after each signatures pin: pass.
- `cargo build -p warp --all-targets --message-format short`: pass; `cargo clean` after the release push.
- Deletion-surface scans: no new hits; every flagged file is byte-identical to `main` (pre-existing allowed set: retained SSH `ForwardX11=no`/remote-path platform tests, bootstrap ConPTY comments, ONNX tokenizer vocabulary, `WeakHandle::upgrade`).
- `fish -n` could not run locally (fish not installed); the applied `fish.sh` hunks are byte-identical to the upstream-verified patch.
- `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
