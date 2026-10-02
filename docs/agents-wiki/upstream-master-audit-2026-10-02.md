# Upstream Master Audit 2026-10-02

Range under review: `40b791c35..upstream/master` (9 commits)

Previous audited upstream tip: `40b791c35 Add ChatGPT subscription support (#16216)` (merged as `30c2761a4`)

Current upstream tip: `d5d23e611 Bound MCP tool calls and environment setup commands (#16219)`

Total upstream commits in this incremental range: 9

Status: triage complete. One adapted port (`41b83dc7c`); eight commits rejected or not applicable.

## Per-Commit Triage

### `bbef0848f` / `d5410a107` / `a92d72352` — CI action bumps (rust-cache, paths-filter, install-action)

Decision: **not applicable**. All three bump actions in upstream `.github/workflows/ci.yml`. The fork's `ci.yml` uses only `actions/checkout` and the local `./.github/actions/prepare_environment` action; none of the three bumped actions appear in any fork workflow.

### `db1c7a50a` — Fix Claude and Codex follow-up progress reporting (#16217)

Decision: **reject / not applicable**. The fix threads a new `is_prompt_submit` flag through `CLIAgentSessionsModelEvent::SessionStatusChanged` so explicit prompt submissions bypass cached `INPROGRESS` deduplication in `app/src/ai/blocklist/local_agent_task_sync_model/**` (update queue + tests) and `app/src/ai/agent_management/agent_management_model.rs` — both fork-absent (the task-sync model was removed at fork creation with the ambient-agent stack per the 2026-08-29 audit; `agent_management/` is a Removed System). In the fork the corresponding event variant is `CLIAgentSessionsModelEvent::StatusChanged`, whose only consumers are the TerminalView rich-input auto-toggle and desktop-notification paths (verified in `app/src/terminal/view.rs`) plus workspace pane routing — none performs progress-report caching or deduplication, so the upstream bug surface does not exist here and a ported field would have zero consumers.

### `1cc4edf69` — Fix CVE-2026-102277 in GraphQL codegen dependencies (#16233)

Decision: **not applicable**. Both paths (`crates/warp_graphql_schema/package.json`, `yarn.lock`) belong to the removed Warp cloud GraphQL schema crate, absent from this fork.

### `19afe1e5e` — Fix git credential file location on Windows sandboxes (#16236)

Decision: **not applicable**. Entirely inside `app/src/ai/agent_sdk/driver/git_credentials.rs`(+tests), the removed agent-SDK surface, and Windows-sandbox-specific.

### `2a8604fb2` — Fix Factory definition clone setup command on Windows (PowerShell) (#16234)

Decision: **not applicable**. Entirely inside `app/src/ai/agent_sdk/driver*` (Factory cloud feature on the removed agent-SDK driver), Windows/PowerShell-specific.

### `41b83dc7c` — Support Kiro CLI as a first-class coding agent (#16243)

Decision: **adapted port**. Adds first-class Kiro recognition (`kiro-cli` and `kiro`) to the retained third-party CLI-agent surface: enum variant, brand color (Kiro purple `#9046FF`), display name "Kiro CLI", ghost logomark asset, `Icon::KiroLogo`, `!` bash-mode support, and Inline rich-input submit strategy. Kiro has no Warp notification plugin, so it lands in the no-session-handler branch.

- `app/assets/bundled/svg/kiro.svg`: new file, applied verbatim.
- `app/src/terminal/cli_agent.rs`: applied via `git diff <c>^ <c> -- <paths> | git apply --3way`, conflicts resolved against the fork's smaller variant set (no `Hermes`/`Antigravity`/`WarpTui`/`OhMyPi`): `KIRO_PURPLE` const, enum variant after `Grok`, display name, icon, `supports_bash_mode` arm, `brand_color` arm; the enum-level doc comment upstream deleted is deleted here too.
- `app/src/terminal/cli_agent_sessions/listener/mod.rs`: `CLIAgent::Kiro` added to the `create_handler` no-handler branch (upstream places it beside fork-absent `Hermes`).
- `app/src/terminal/view/cli_agent_footer.rs` (upstream `view/use_agent_footer/mod.rs`): `CLIAgent::Kiro` added to the `Inline` arm of `rich_input_submit_strategy`, matching upstream's placement in the `Amp|Droid|Pi|Kiro|Goose|Vibe|… => Inline` group.
- `crates/warp_core/src/ui/icons.rs`: `Icon::KiroLogo` variant and `From<Icon> for &str` mapping applied verbatim.

Intentionally omitted paths/hunks:

- `app/src/server/telemetry/events.rs` (`CLIAgentType::Kiro`) and the `From<CLIAgent> for CLIAgentType` impl: telemetry Removed System; `CLIAgentType` has zero references in the fork.
- `supported_skill_providers`/`skill_command_prefix` hunks: the fork's `cli_agent.rs` has no `SkillProvider` machinery (app-managed skills are a Removed System).
- `app/src/terminal/cli_agent_sessions/plugin_manager/mod.rs`: fork is pre-`plugin_manager` lineage; no Warp-managed plugin install exists.

Fork integration adaptations (recorded, both forced by the fork's pre-`command_prefixes` singular interface):

- `command_prefix()`: upstream's `&["kiro-cli", "kiro"]` plural form becomes `CLIAgent::Kiro => "kiro-cli"` (the official binary name) plus a `detect()` special case `resolved_first_word == "kiro"`, mirroring the existing `Vibe`/`"vibe-acp"` idiom. Detection, the tab-config `SessionType` prefix, code-review command construction, and the notification title fallback all consume the single primary prefix, as they already do for every other agent.

### `d5d23e611` — Bound MCP tool calls and environment setup commands (#16219)

Decision: **reject / not applicable**. Every runtime path is fork-absent: `app/src/ai/agent_sdk/driver*` (Removed System), `app/src/ai/mcp/**` (app-managed MCP, Removed System), and `crates/mcp/` (does not exist in this fork). The workspace `Cargo.toml` hunk bumps `rmcp` 2.2 → 3.5 under `[workspace.dependencies]` for `crates/mcp`; the fork has no workspace `rmcp` entry — its `rmcp` 1.6.0 is a transitive dependency of `agent-client-protocol` 0.11.1, whose version is chosen by the ACP crate itself and is unaffected by the upstream bump. No retained boundary uses rmcp directly.

## Verification

Run on `merge/upstream-2026-10-02` (with `CARGO_PROFILE_DEV_DEBUG=0`):

- `cargo fmt -- --check` — pass (after `cargo fmt` applied the listener arm line-wrap).
- `cargo check -p warp --all-targets --message-format short` — pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short` — pass.
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'` — 156 passed.
- `cargo nextest run -p warp -E 'test(cli_agent)'` — 94 passed.
- `cargo build -p warp --all-targets --message-format short` — pass.
- Deleted-surface scans — no new hits; the working diff touches only the five CLI-agent files listed above.

`cargo clean` run after merge/tag/push.
