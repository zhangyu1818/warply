# Upstream Master Audit 2026-08-24

## Scope

- Current fork before this audit: `7abf0db4c` (`main`, `v2026.08.23`).
- Upstream source reviewed: `dc1077845f..upstream/master` (10 commits, tip `79a9cb721a`).
- Result: 6 commits accepted or adapted (rust-analyzer toolchain component, completer option-argument resolution fix, Warpified pwsh PSReadLine vi-mode fix, settings registration compile-time refactor, `ai_types` crate partial port, settings schema generation moved into the main executable), 4 rejected or not applicable (multi-team AI autonomy scoping, three CI/release-workflow commits against fork-owned or removed pipelines).

## Commit-by-commit triage

| Commit | Title | Decision | Notes |
| --- | --- | --- | --- |
| `83b4c101e` | Optimize settings schema generation during release process (#13240) | **Adapt (ported)** | See provenance below. Moves schema generation into `warply dump-settings-schema`; retained app/CLI/macOS-script surface, omitted fork-absent pipeline paths. |
| `6a96a72d8` | Compile time: share settings registration code across settings (#15454) | **Adapt (ported)** | See provenance below. Retained local settings infrastructure; adapted to the fork's de-cloud-synced settings crate. |
| `efbf553ed` | Add the ai_types crate for pure AI model types (#15459) | **Adapt (partial port)** | See provenance below. Only the three types the fork retains; all ambient/entrypoint/execution-context/skills/tui hunks touch removed surfaces. |
| `8e5bb1fad` | [multi-team P3a-1] Scope the AI autonomy accessor to the window's team (#15443) | **Reject** | Every hunk threads `TeamScope`/`TeamContextForOperation`/`UserWorkspaces`/`AiAutonomySettings`; the fork has none of these (no `app/src/workspaces/`, no `ai_autonomy_settings`, no `is_ai_autonomy_allowed` anywhere). The permission flows the commit touches run through the fork's local execution-profile settings and ACP. Consistent with the 2026-08-23 rejections of `e2a080210`/`19548aec6`. |
| `b5194c627` | Add back some go setup commands to release creation workflow (#15471) | **Not applicable** | Only touches upstream `.github/workflows/create_release.yml`; the fork's file is the fork-owned "Create Warply Release" tag-push workflow with no go/pprof jobs. |
| `b1bcc3564` | Add rust-analyzer to rust-toolchain.toml components (#15477) | **Accept (ported)** | Fork's `rust-toolchain.toml` was byte-identical to the upstream parent; cherry-picked cleanly. |
| `8817f6aa9` | ci: don't let draft PRs skip the required Check CI results job (#15478) | **Not applicable** | Only touches upstream `ci.yml`'s `params`/`ci-result` job graph; the fork's 62-line `ci.yml` has neither job. |
| `da434eb6e` | Migrate Windows arm64 release build to native windows-arm-latest-large runner (#15437) | **Reject** | Windows arm64 release infra: `create_release.yml` is fork-owned and macOS-only; the `install_cargo_binstall` hunks (aarch64-pc-windows-msvc pin + `PROCESSOR_ARCHITEW6432` arch detection) exist solely to run the bootstrap under Git-for-Windows on ARM hosts, which this macOS-only fork never does. |
| `748b635cd` | Fix PSReadLine vi edit mode corrupting submitted commands in Warpified pwsh (#15476) | **Accept (ported)** | Retained shell integration: `Warp-Configure-PSReadLine` force-switches EditMode Vi to Emacs at bootstrap (recording `vi_mode_enabled` in the bootstrapped metadata) so fragmented ESC chords cannot strand the editor in vi command mode; regression integration test added. |
| `79a9cb721` | completer: resolve an option's argument by value position, not the last declared (#15475) | **Accept (ported)** | Retained `warp_completer` fix: `complete_option` resolves the argument by the option instance's value position (variadic-aware) instead of `.last()`; `NamedArgument` carries `name_span` so repeated single-argument options count per instance. 4 new suggest tests + `enum_then_path_option_signature` test signature copied verbatim. |

## Provenance: `83b4c101e` port detail

Ported from the exact upstream commit:

- `app/src/settings/schema_generation.rs` (+ tests): the moved generator — `dump_settings_schema` (atomic `NamedTempFile` persist or stdout), `settings_schema_json(is_flag_enabled)`, `ensure_hierarchy`, `strip_numeric_metadata`, `strip_empty_enum_entries`, `write_atomically`. The fork's three unit tests are the upstream tests minus the `x-warp-surfaces` test.
- `crates/warp_cli/src/lib.rs`: `DumpSettingsSchema { output_path }` variant + `prints_to_stdout` arm.
- `app/src/lib.rs`: dispatch arm in `run()` after `init_feature_flags()`.
- `app/src/settings/mod.rs`: `pub(crate) mod schema_generation;` + `pub use schema_generation::dump_settings_schema;`.
- Deletion of `app/src/bin/generate_settings_schema.rs` and its `[[bin]]` entry.
- `script/prepare_bundled_resources`: fail-closed `SETTINGS_SCHEMA_EXECUTABLE`/`SETTINGS_SCHEMA_SOURCE` inputs (mutually exclusive, existence/runnability checked, non-empty output verified); `cargo_profile` arg and `SETTINGS_SCHEMA_CACHE` removed with the standalone-binary path.
- `script/macos/bundle`: exports `SETTINGS_SCHEMA_EXECUTABLE` pointing at the just-bundled app binary before calling `prepare_bundled_resources` (2-arg form), with the `--skip-build` fail-closed error.
- `script/macos/run`: `GENERATE_SCHEMA=true` now exports `SETTINGS_SCHEMA_EXECUTABLE` at the bundled binary path.
- `script/test_prepare_bundled_resources`: copied verbatim; passes locally (executable path, source path, missing-input, conflicting-inputs, skip).

Fork adaptations (recorded omissions and adjustments):

- Omitted as fork-absent: `.github/workflows/create_release.yml` (fork-owned Warply release workflow), `script/linux/*`, `script/windows/*`, `flake.nix`, `script/run-tui`, `crates/warp_tui/src/session.rs`, `script/deploy_remote_server*`, and the `x-warp-surfaces`/`SettingSurfaces`/`SettingsMode` annotation (the fork's settings crate has no `surfaces_fn`; the fork has no TUI surface).
- The fork's `warp_cli::Command` has no `DumpDebugInfo`/`PrintTelemetryEvents` neighbors and no wasm cfg gates; the variant was added in the fork's enum layout without the `#[cfg(not(target_family = "wasm"))]` attributes.
- `script/macos/bundle` omits upstream's `HOST_ARCH`/`CAN_EXECUTE_DEFAULT_TARGET` guard: the fork's bundle is arm64-native only (`ARM_ARCH="aarch64"`, `DEFAULT_TARGET="$ARM_TARGET"`, no `UNIVERSAL_BINARY` path), so the cross-arch exec hazard cannot arise. The `--skip-build` error branch is kept.
- Title/description keep the fork's "Warply Settings" branding with `ChannelState::channel()` interpolated (upstream: "Warp Settings").

Verification against the removed generator (built side by side before deletion): outputs are byte-identical except (a) `appearance.tabs.directory_tab_colors` now appears — the old bin's hardcoded channel→flag-list mapping missed cargo-feature-backed flags (`directory_tab_colors` is an app default cargo feature), while the in-app path reflects the real compiled feature set, which is precisely the feature-state parity this upstream PR exists to restore; and (b) the description now reports the actual `oss` channel instead of the old CLI default `dev`.

## Provenance: `6a96a72d8` port detail

Ported from the exact upstream commit, adapted to the fork's settings crate (which removed cloud sync at the fork baseline):

- `crates/settings/src/registration.rs` (new): `parse_value`/`parse_value_strict`/`equals_serialized`, `SettingCallbacks` fn-pointer struct, `SettingMetadata`, `register_setting_events` (gathers `Setting` trait metadata) and the shared `register_setting_events_impl` body — all copied from upstream and trimmed to the fork's `register_setting` signature (no `sync_to_cloud`, no reset/is-syncable callbacks, no `SettingsEvent::LocalPreferencesUpdated` subscribe).
- `crates/settings/src/macros.rs`: the `define_setting!` and `implement_setting_for_enum!` emit sites now call `<Self as SettingChangeEvent>::change_event(reason)` for `Clear`/`LocalChange` (the fork has no `CloudSync` sites); each macro emits one `SettingChangeEvent` impl via `concat_idents!`; `register_settings_events!` builds `SettingCallbacks { apply_set, apply_load }` at the expansion site (no `from_cloud_sync` flag) and delegates to `registration::register_setting_events::<$setting, _>`; `generate_settings_event_fn!` is deleted (its only caller was `register_settings_events!`).
- `crates/settings/src/lib.rs`: `pub mod registration;` and the upstream `SettingChangeEvent` trait verbatim.
- `warpui::` paths where upstream uses `$crate::warpui_core::` (the fork's settings crate depends on `warpui`).

## Provenance: `efbf553ed` port detail

Ported subset (the three types the fork retains, definitions byte-identical to both the upstream crate and the fork's previous local definitions):

- `crates/ai_types` (new): `Cargo.toml` (deps trimmed to `anyhow`/`serde`/`uuid` — upstream also has `serde_json`/`thiserror` for the ambient/execution modules the fork omits), `src/lib.rs` (module list trimmed to `agent`), `src/agent.rs` (`AIConversationId`, `AIAgentActionId`, `TaskId`).
- Workspace `Cargo.toml` `ai_types` entry; `app/Cargo.toml` and `crates/persistence/Cargo.toml` dependencies.
- `app/src/ai/agent/conversation.rs`, `task.rs`, `mod.rs`: local definitions replaced with `pub use ai_types::{AIConversationId, TaskId, AIAgentActionId}` re-exports at the old paths; the mod.rs re-export comment updated per upstream.
- `crates/persistence/src/model.rs`: the orphan-rule `From` conversions between the persistence row `AIAgentActionId(pub String)` and `ai_types::AIAgentActionId` move into the persistence crate exactly as upstream; the app-side impls they replace are deleted with the local type.

Omitted (fork-absent surfaces): `AmbientAgentTaskId`/`ParseAmbientAgentTaskIdError`, `EntrypointType`/`PassiveSuggestionTriggerType`, `WarpAiExecutionContext`/`WarpAiOsContext` and `execution_context_for_session`, the `SkillDescriptor` move to `crates/ai/src/skills/`, and every hunk under `app/src/ai/ambient_agents/`, `app/src/ai/skills/`, `app/src/ai_assistant/`, `app/src/server/server_api/`, `crates/warp_tui/`, `app/src/tui_export.rs`, `app/src/terminal/input.rs`, `active_session.rs`, `app/src/workspace/view.rs` (all reference surfaces removed from the fork; `WarpAiExecutionContext` has zero hits in the fork).

## Provenance: `748b635cd` port detail

- `app/assets/bundled/bootstrap/pwsh.ps1`: all three hunks applied cleanly (`vi_mode_enabled` metadata field after `shell_plugins`; `$script:viEditModeOverridden = $false` before `Warp-Configure-PSReadLine` plus the EditMode Vi→Emacs switch at its head; the `Warp-Configure-PSReadLine` call at the top of `Warp-Finish-Bootstrap`). The `vi_mode_enabled` field is part of the local bootstrapped-message metadata payload and stays on-device.
- Integration tests: `test_pwsh_vi_edit_mode_does_not_corrupt_commands` (builder + registration + `integration_tests!` list entry) copied verbatim. Conflict resolution kept the fork's `test_tmux_ssh_into_bash`/`test_tmux_ssh_into_zsh` registrations (fork-owned tmux coverage) in place of upstream's neighboring `test_ssh_wrapper_into_*` context.

## Provenance: `79a9cb721` port detail

- `completer/engine/argument/legacy.rs`: `option_value_index` helper, the two `suggestions_for_last_argument` call-site additions, and the variadic-aware `complete_option` argument resolution are byte-identical to upstream post-change. Conflict resolution kept the fork's import layout (the fork groups the `crate::parsers` imports differently); `FlagType, NamedArgument` joined the fork's existing `hir::{...}` import.
- `parsers/hir/mod.rs`: `NamedArgument { name, name_span, parsed_token }` + the constructor hunk applied cleanly.
- `completer/suggest/test.rs` and `signatures/testing/legacy.rs`: the four new tests and `enum_then_path_option_signature` applied cleanly; remaining diff versus upstream is the fork's pre-existing layout (no windows/unix `TEST_WORK_DIR` split, `warpui` vs `warpui_core` block_on).

## Verification

- `cargo fmt -- --check`: clean across all touched crates.
- `cargo check -p warp_completer / -p integration / -p settings / -p ai_types -p persistence / -p warp -p warp_cli --all-targets`: pass (pre-existing warnings only).
- `cargo check --workspace --all-targets --message-format short`: pass.
- Focused tests: `warp_completer` 172/172 (was 168; includes the 4 new option-resolution tests); `settings` 65/65; `warp` schema_generation 3/3, agent/conversation 80/80, settings/theme 57/57; standard suite `test(slash_command) | test(acp) | test(terminal_suggestions)` 156/156.
- `./script/test_prepare_bundled_resources`: pass (all five cases).
- Schema equivalence: `warply dump-settings-schema` vs the removed standalone generator built side by side — byte-identical except the corrected `directory_tab_colors` entry and the real `oss` channel name (see `83b4c101e` provenance).
- `bash -n` on `script/prepare_bundled_resources`, `script/macos/bundle`, `script/macos/run`: pass.
- Deletion-surface scans over the branch diff (`main...HEAD`): removed-product, MCP/skills, and platform scans all return zero added hits; repo-wide scans show only the pre-existing allowed hits (weak-handle `upgrade()` methods, local comments).
- Disk note: `CARGO_PROFILE_DEV_DEBUG=0` used for check/test/build as in prior audits.
- Final `cargo build -p warp --all-targets --message-format short` and `cargo clean` recorded after the release push (see below).
