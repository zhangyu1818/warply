# Upstream Master Audit — 2026-10-12

Incremental audit for `c1ec372cb..upstream/master` (`25e82210a`), 2 commits, completing the currently fetched upstream master tip. The previous audit is `upstream-master-audit-2026-10-11.md`.

## Summary

One adapted port and one rejection. `7c3148819` adds first-class Muse Code (`muse`) CLI-agent support and was ported under the established Grok (`2718b6658`) / Kiro (`41b83dc7c`) CLI-agent port pattern: registry entry, Meta-blue brand color, `muse.svg` logomark + `Icon::MuseLogo`, default OSC 777 session listener, bash mode, and `BracketedPaste` rich-input submit. `25e82210a` propagates the parent's OTLP tracing token into the agent-sdk `environment-checkout` child process; every hunk anchors on fork-absent surfaces (`agent_sdk` driver, `EnvironmentCheckout` CLI command, OTLP/cloud-agent-auth tracing stack) and was rejected with zero anchors.

## Commit-by-Commit

### `7c3148819` [GH-15768] Add first-class Muse Code CLI agent support (#15999) — adapted port

Recognizes Meta's Muse Code as a first-class third-party CLI agent. Local-only behavior throughout: detection, branding, icon, session listener, rich-input submit strategy. Ported via `git cherry-pick --no-commit` with conflicts resolved toward the fork's CLI-agent structure. Ported paths:

- `app/assets/bundled/svg/muse.svg` — exact upstream copy (24×24 Meta loop stencil).
- `app/src/terminal/cli_agent.rs` — `MUSE_COLOR` (#0081FB), `CLIAgent::Muse` variant, `command_prefix() => "muse"`, display name "Muse Code", `Icon::MuseLogo`, `supports_bash_mode()`, `brand_color()`, module doc updated to upstream text. Muse has a single command prefix, so the fork's singular `command_prefix()` needs no `detect()` second-prefix special case.
- `app/src/terminal/cli_agent_sessions/listener/mod.rs` — `is_agent_supported` admits Muse; `create_handler` routes Muse to `DefaultSessionListener` (structured OSC 777 with `"agent":"muse"`, no Warp-installed plugin). Upstream `muse_is_supported_via_default_listener` test ported into the fork's inline test module, adapted to the fork's 2-arg `try_parse` (upstream gained a third parameter in an unported commit).
- `app/src/terminal/view/cli_agent_footer.rs` — Muse joins the `BracketedPaste` rich-input submit arm. Upstream's hunk extends the `Hermes` arm; the fork has no `Hermes` variant, so Muse joins the `Codex` arm, preserving upstream's Muse behavior.
- `app/src/terminal/cli_agent_tests.rs` — `("muse", CLIAgent::Muse)` detect entry, `muse exec "…"`/`npx muse` detect asserts, and `test_muse_public_configuration` following the fork's Grok-test shape.
- `app/src/terminal/view_test.rs` — `submit_cli_agent_rich_input_muse_uses_bracketed_paste` modeled on the fork's Codex test.
- `crates/warp_core/src/ui/icons.rs` — `MuseLogo` variant + `bundled/svg/muse.svg` mapping, exact upstream hunks.

Intentionally omitted paths and hunks:

- `app/src/terminal/cli_agent_sessions/plugin_manager/mod.rs` + `mod_tests.rs` — the `plugin_manager` module is fork-absent (this fork ships no Warp-managed plugin installers). The upstream hunk makes `plugin_manager_for(CLIAgent::Muse)` return `None`, which is trivially satisfied here; Warp writes nothing into Muse config directories either way.
- `supported_skill_providers()` Muse arm and `skill_command_prefix()` — fork-absent methods (app-side skills are removed; skills belong to the ACP agent process). The corresponding `test_muse_public_configuration` asserts were dropped.
- `telemetry_name()` / `command_prefixes()` conflict sides — fork-divergent structure from unported upstream commits (fork keeps the singular `command_prefix()` and `detect()` special cases).
- `CLIAgent::WarpTui` / `OhMyPi` / `Hermes` / `Antigravity` rows — fork-absent variants; the enum doc list was kept to the fork's actual variants (upstream's added comment names `Hermes`).
- `app/src/terminal/view/use_agent_footer/mod_tests.rs` submit-strategy unit assert — fork-absent file; the behavior is covered by the ported `view_test.rs` integration test.
- `specs/GH15768/**` (`PRODUCT.md`, `TECH.md`, `warp-hooks-sample.md`) — upstream specs are rejected per the fork contract.
- `attach_path_as_context_routes_to_open_cli_agent_rich_input` — upstream-side conflict context from a different commit, gated on removed flags (`AgentView`/`CLIAgentRichInput`/`HoaCodeReview`); not part of this delta.

### `25e82210a` Trace environment checkout with the parent's current OTLP token (#16386) — rejected (removed surface, anchor absent)

Propagates the parent process's current OTLP tracing token into the `environment-checkout` child process via a new `tracing::init_child_process` (span-context extraction from a `tracing_native` OTLP exporter plus token refresh through `cloud_agent_auth`), reworking `agent_sdk/driver/environment_checkout.rs` to pass the parent context and threading a `CliCommand::EnvironmentCheckout` arm in `app/src/lib.rs`. Every touched surface is fork-absent:

- `app/src/ai/agent_sdk/driver/environment.rs`, `environment_checkout.rs`, `environment_checkout_tests.rs` — the `agent_sdk` driver was removed at the fork baseline; the environment-checkout executable itself was rejected on 2026-10-11 as the Factory cloud-execution stack (`crates/warp_cli/src/environment_checkout.rs` fork-absent).
- `app/src/lib.rs` — the hunk edits the `CliCommand::EnvironmentCheckout` arm; the fork's CLI has no such command (verified: `EnvironmentCheckout|environment_checkout|init_child_process|cloud_agent_auth` has zero hits across `app/src/lib.rs`, `crates/warp_cli/src`, and `app/src/tracing.rs`).
- `app/src/tracing.rs`, `tracing/child_process.rs`, `tracing/child_process_tests.rs`, `tracing/cloud_agent_auth.rs` (+tests), `tracing/native.rs` — the fork's `app/src/tracing.rs` is the 12-line `NoSubscriber` stub; external telemetry/OTLP export and cloud-agent auth token refresh are removed surfaces (telemetry + account-coupled tracing).

## Verification

- `cargo check -p warp --all-targets --message-format short`: clean (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short`: clean.
- `cargo fmt -- --check`: clean.
- `cargo nextest run -p warp -E 'test(muse)'`: 3/3 passed.
- `cargo nextest run -p warp -E 'test(cli_agent) | test(slash_command) | test(acp) | test(terminal_suggestions)'`: 255/255 passed.
- `cargo build -p warp --all-targets --message-format short`: succeeded; `cargo clean` follows after push per the release workflow.
- Restored-deleted-surface scans (`rg` over `app crates script Cargo.toml`): 232/0/6 hits — identical to the previously documented allowances (no new hits; zero hits in the Muse-touched files; the six platform hits remain the retained SSH `ForwardX11` options, zsh bootstrap ConPTY comments, and `local_or_remote_path_tests.rs` `cfg(windows)` remote-path tests).
