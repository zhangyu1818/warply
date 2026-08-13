# Upstream Master Audit 2026-08-13 — Settings Surface Boundary Correction

## Scope

- Current fork: `1f5bfdd099` (`main`, `v2026.08.13`).
- Upstream source reviewed: `upstream/master` at `5fb3144db9638c6c43371b566e1d0a89ae69236c`.
- Relevant upstream settings source: `fe8138bce8e67216d1349834c8f5384b8603e41f` and its current `app/src/settings_view/ai_page.rs` / `app/src/settings_view/code_page.rs` implementation.
- Relevant fork ports: `048e052751` (local CLI-agent Rich Input foundation) and `f7605db59e` (Rich Input Ctrl+Enter setting).
- This is a documentation-only audit. No application code was changed.

## Corrected decision rule

The earlier rejection of the Third party CLI agents settings page used the wrong unit of review: it treated the upstream page as an indivisible Warp product surface because the fork's `ai_page.rs` was then a small ACP-only page. That reason is superseded.

Settings pages are containers, not product boundaries. Review each widget, action, setting, persistence path, and runtime call site. Keep the local or retained-provider behavior and remove only the incompatible dependency. In this fork, a strong Warp-service dependency means that the core behavior requires Warp account/auth, Warp cloud/server APIs, billing or usage state, hosted agents/cloud handoff, cloud sync/sharing, app-managed MCP/skills, telemetry, or an unsupported native client platform. Being located under an AI or Code page is not such a dependency.

For a mixed page, apply the exact upstream source for the retained widgets or hunks first, then make the smallest ACP/local/provider-boundary adaptations. Do not bulk-port removed sibling sections merely because they share a Rust module or settings umbrella.

## Third-party CLI agents

The upstream `fe8138bce8` source splits the Third party CLI agents page into seven focused widgets:

| Upstream widget | Local state or behavior | Current fork status | Decision |
| --- | --- | --- | --- |
| `CLIAgentWidget` | `AISettings::should_render_cli_agent_footer`; show or hide the CLI-agent toolbar | Setting and runtime exist; no Settings row | Retain and expose |
| `CLIAgentAutoToggleRichInputWidget` | `AISettings::auto_toggle_rich_input`; open/close Rich Input around a local CLI/plugin listener's blocked state | Setting and runtime exist; no Settings row | Retain and expose |
| `CLIAgentAutoOpenRichInputWidget` | `AISettings::auto_open_rich_input_on_cli_agent_start` | Setting and runtime exist; no Settings row | Retain and expose |
| `CLIAgentAutoDismissRichInputWidget` | `AISettings::auto_dismiss_rich_input_after_submit` | Setting and runtime exist; no Settings row | Retain and expose |
| `CLIAgentSubmitRichInputWidget` | `AISettings::submit_on_ctrl_enter` | Already exposed on the current AI page | Retain; its current placement is valid local behavior |
| `CLIAgentCommandsWidget` | Local regex command list and per-CLI-agent mapping in `cli_agent_footer_enabled_commands` | State, mutation helpers, and runtime matching exist; no Settings row | Retain and expose |
| `CLIAgentToolbarLayoutWidget` | Local `CLIAgentToolbarChipSelection` and `AgentToolbarEditorMode::CLIAgent` layout editor | CLI toolbar editor/runtime exists; Settings entry is missing; prompt context-menu editing is only a workaround | Retain and expose |

The current CLI footer already has local `FileExplorer`, `RichInput`, and `FileAttach` toolbar items, and renders the configured selection through the terminal footer. Their default visibility is therefore not evidence that the controls are Warp-service functionality; it is evidence that the local settings UI is incomplete. The upstream layout editor is the authoritative source for allowing those items and local context chips to be removed or rearranged.

The local plugin/listener wording in the Rich Input settings describes an external CLI integration. It does not authorize restoring Warp plugin installation, marketplace, hosted-agent, cloud-handoff, account, or telemetry surfaces.

## Other settings surfaces that need widget-level review

The same correction applies to the upstream Code settings page. Keep local editor, code review, project explorer, external editor, and LSP controls when their state and execution remain local or SSH-backed. In particular, the upstream `EditorAndCodeReview` widgets for auto-opening the local review pane, code-review button/diff statistics, project explorer/global search/hidden files, format-on-save, auto-save, external editor, and local LSP management are retainable candidates. Omit or split away full-source/cloud codebase indexing, Warp account/workspace controls, telemetry, and unsupported-platform branches.

The correction does not reopen removed pages or sections whose core behavior remains service-owned. Knowledge widgets tied to Suggested Rules/Warp Drive context, old Warp Agent model/profile/credit flows, app-managed MCP/skills, cloud environments, billing, Teams, account/auth, and cloud sharing remain rejected. Local rules/AI facts, ACP configuration, and local terminal settings may still be retained where their call sites prove that ownership.

The same call-site test applies to legacy settings that look agent-related. `memory_enabled` is not an ACP request switch: it only changes the legacy rules-pane state, while `app/src/ai/blocklist/context_model.rs` adds project rules discovered by `ProjectContextModel` to ACP request context independently. Do not restore a "saved rules" or "memory" row as an ACP setting unless a verified ACP context path gives it real effect.

## Superseded historical decision

`upstream-master-audit-2026-08-03.md` classified `fe8138bce8` as rejected because the fork's `ai_page.rs` was an ACP-only single-widget page. That classification was too coarse after `048e052751` restored the local CLI-agent Rich Input foundation and `f7605db59e` added the local Ctrl+Enter setting. The commit is now a selective local-settings port candidate, not a request to restore the full upstream AI settings page.

The source-first requirement remains unchanged: if these rows are implemented, start from the exact upstream widget/action/page source and port only the retained hunks, with focused verification and an explicit record of omitted service-dependent paths.
