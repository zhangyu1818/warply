# Change Map

This map explains the large fork baseline change at a path level.

## Replaced AI Architecture

| Path | Change | Merge rule |
| --- | --- | --- |
| `app/src/ai/acp/` | Added ACP backend, config options, event mapping, model, terminal/file capability plumbing, permission handling, thread state, and tests. | Preserve as the only agent backend. Port upstream ideas here only if they fit ACP. |
| `app/src/ai/blocklist/` | Retained AgentView shell and adapted it to ACP request flow and ACP-native output rendering. | Port generic UI fixes. Reject old cloud-agent controls and server-backed action execution. |
| `FeatureFlag::AgentViewConversationListView`, `agent_view_conversation_list_view` Cargo feature | Rollout gate for the AgentView conversation side panel. | Removed. Conversation list/navigation is retained local ACP UI, not a backward-compatible optional path. |
| `app/src/ai/agent/` | Simplified conversation/task data toward local ACP history and local transcript representation. | Preserve local persistence semantics. Do not restore server transcript APIs. |
| `app/src/ai/execution_profiles/` | Execution profiles are retained for local permissions only. Old Warp model/profile selector UI, profile model fields, context-window overrides, and legacy model preference migration were removed. | Port permission/profile usability fixes only. Do not restore model selection; ACP config chooses the active model/backend. |
| `app/src/ai/terminal_suggestions/` | Added OpenAI-compatible suggestions client/provider/tests for Next Command and Prompt Suggestions. | Keep provider endpoint/model/API key configurable. |
| `app/src/ai/predict/terminal_input_suggestions.rs` | Replaced hosted AI input suggestion request path for Next Command. | Port context improvements, not hosted API calls. |
| `app/src/ai/predict/terminal_prompt_suggestions.rs` | Replaced hosted Prompt Suggestions request path with OpenAI-compatible requests. | Keep separate from ACP Agent and do not restore Warp-hosted suggestion APIs. |
| `app/src/settings/ai.rs`, `app/src/settings_view/ai_page.rs` | AI settings now focus on ACP and terminal suggestions. | Reject account/billing/cloud-agent/privacy telemetry sections. |

## 2026-05 No-Compatibility Cleanup

| Path | Change | Merge rule |
| --- | --- | --- |
| `app/src/terminal/model/ansi/dcs_hooks.rs`, `app/assets/bundled/ssh/**/warpify_ssh_session*.sh` | Removed legacy `os`/`pkg` aliases from `SystemDetails` and updated Warpify scripts to emit `operating_system`/`package_manager`. | Keep Warpify. Do not re-add old field aliases; update scripts and parser together. |
| `app/src/terminal/model/block/serialized_block.rs`, `app/src/terminal/history.rs` | Removed serde aliases for older block/history field names. | Current local schema is authoritative; do not add legacy field fallbacks. |
| `app/src/ai/llms.rs`, `app/src/util/file/external_editor/settings.rs` | Removed old server/model wire-format fallback and external-editor old-format deserialization. | Do not restore old Warp-hosted model metadata compatibility paths. |
| `crates/persistence/migrations/2025-01-15-174448_add-settings-panes/up.sql` | Replaced deleted `Account` settings pane default with current `AI` default. | Do not point persisted pane defaults at deleted account/product pages. |
| `app/src/lib.rs`, `app/src/terminal/model/session.rs`, `crates/warp_core/src/lib.rs`, `crates/repo_metadata/src/lib.rs` | Removed compatibility-only reexports and updated callers to import from the owning modules directly. | Prefer canonical module ownership over old import-path shims. |
| `app/src/remote_server/identity_context.rs` | Renamed the retained SSH remote-server identity context helper away from old `server_api` wording. | Keep SSH remote-server identity partitioning, but do not reintroduce Warp server API/client semantics. |
| `crates/local_object_model/src/ids.rs`, `app/src/object_ids.rs` | Removed lossy object ID normalization that generated replacement client IDs or truncated/padded invalid server-style IDs. | Retained local object IDs must parse strictly; do not add silent old-data or malformed-ID fallback. |
| `app/src/cloud_object/update_manager.rs`, `app/src/workflows/workflow_view.rs`, `app/src/env_vars/active_env_var_collection_data.rs`, `app/src/workspace/view.rs`, `app/src/ai/blocklist/` | Collapsed local-object update events to a single `SyncId` and removed client-id to server-id creation backfill listeners. | Object update routing should compare local `SyncId` values directly; do not restore server-id backfill or dual-id event matching. |
| `app/src/persistence/sqlite.rs` | Removed local-object permission fallback from missing `subject_id` to the current local user. | Persisted object permissions must contain an explicit owner subject; do not infer owners from app identity to keep old rows alive. |
| `app/src/ai/agent/` | Removed unused old `AIAgentApi` output metadata retention and model fallback flag from retained AgentView output state. | ACP transcript/output state should store only fields used by the ACP flow and local UI; do not add hidden old-service metadata fields for history round trips. |
| `app/src/ai/agent/`, `app/src/ai/blocklist/`, `app/src/terminal/view.rs` | Removed the old server-output request ID chain, no-op debug footer/copy action, and related empty logging hooks. | ACP/local AgentView output should use local conversation and exchange IDs where needed; do not restore server request IDs or debug UI whose only purpose is old hosted-service correlation. |
| `app/src/ai/blocklist/`, `crates/warp_core/src/ui/icons.rs`, `app/assets/bundled/svg/thumbs-*.svg` | Removed thumbs up/down response rating UI and assets. The retained ACP AgentView has no Warp-hosted feedback submission path. | Do not restore response-rating or feedback widgets unless they are wired to an explicit local-only feature owned by this fork. |

## Removed Cloud And Platform Product Code

| Path or area | Removed purpose | Upstream merge decision |
| --- | --- | --- |
| `app/src/auth/` | Account auth, user identity, access tokens, login UI. | Reject. |
| `app/src/billing/` | Billing, usage, upgrade, referral gates. | Reject. |
| Cloud/billing/team/referral SVGs and icon variants | `cloud-01.svg`, `cloud-filled.svg`, `cloud-off.svg`, `create-team.svg`, `credits.svg`, `referral-*`, `Icon::Cloud`, `Icon::CloudFilled`, `Icon::CoinsStacked`. | Removed when unused. Keep generic publish/upload icons that are used by retained local code review flows. |
| `app/src/autoupdate/` | App update/changelog infrastructure. | Reject unless this fork intentionally restores updater behavior. |
| `app/src/crash_reporting/` | Crash reporter/Sentry integration. | Reject. |
| `app/src/ai/agent_sdk/` | Old Warp Agent SDK, harnesses, cloud environment, scheduling, cloud tool execution. | Reject; reimplement useful behavior in ACP if needed. |
| `app/src/ai/agent_management/` | Cloud agent management UI. | Reject. |
| `app/src/ai/ambient_agents/` | Ambient/scheduled/cloud agents. | Reject. |
| `Icon::AmbientAgentMode`, `app/assets/bundled/svg/ambient-agent-mode.svg` | Ambient/cloud agent icon surface. | Removed. Keep `agentmode.svg` for the retained ACP AgentView entrypoint. |
| `app/src/ai/cloud_agent_*`, `app/src/ai/cloud_environments/` | Cloud agent settings/environments. | Reject. |
| `crates/graphql/`, `crates/warp_graphql_schema/` | Warp cloud GraphQL API and schema. | Reject. |
| `crates/managed_secrets/`, `crates/managed_secrets_wasm/` | Cloud managed secrets. | Reject. |
| `app/src/external_secrets/`, `app/src/search/external_secrets/` | Local environment-variable references to user-installed secret-manager CLIs such as 1Password and LastPass. | Retain only as local shell/EVC integration. Do not connect it to Warp managed secrets, cloud secret storage, account auth, or cloud credential discovery. |
| `crates/isolation_platform/` | Hosted/cloud isolation. | Reject. |
| `crates/onboarding/` | Product onboarding and marketing flows. | Reject. |
| `app/assets/async/png/onboarding/` | Onboarding, agent intention, third-party toolbar/notification, and Warp Drive marketing screenshots. | Removed. Do not restore unless a retained local terminal UI directly owns a new asset. |
| `crates/warp_web_event_bus` and wasm host events | Old Web host event bridge for logged-out/session-joined/open-native/theme events tied to the cloud web client. | Removed. This fork packages macOS and should not restore Web host account/session event buses. |
| `crates/serve-wasm`, `app/assets/remote`, and `asset_macro::remote_asset` | Web/WASM asset server and hashed remote asset loading path. | Removed. macOS bundles include async theme assets locally via `bundled_async_asset!`; do not restore remote/Web asset fallback. |
| `aws-sdk-sts`, `aws-types` app dependencies | Old AWS STS credential loading dependency surface for BYO/cloud LLM paths. | Removed when unused. OpenAI-compatible terminal suggestions use explicit provider config and must not restore cloud credential discovery or STS fallback paths. |
| `crates/voice_input/` | Voice input/transcription. | Reject. |
| Codex modal/deeplink and old model selector state | Upstream Codex marketing modal, `codex` URI host, root open-codex actions, per-terminal LLM override snapshots, and `llm_model_override` persistence migration. | Reject. ACP/session settings are the only model/backend configuration surface. |
| Bundled/channel-gated skills, MCP skills, and app-side skill management | App-distributed agent instructions, MCP helper skills, Claude API/managed-agent docs, tab/settings/PR helper skills, channel-gated verification skills, local skill scanners, `/skills` and `/open-skill` UI, `ReadSkill`/`InvokeSkill` actions, and CLI skill spec parsing. | Reject. Skills and MCP tool instructions belong to the ACP agent process, not the Warp app bundle or Warp runtime. |
| Agent shared-session/viewer action sync | Old shared-session viewer state, remote action execution mirroring, and view-only action-result replay. | Reject. ACP AgentView owns the active agent flow; local transcript viewing may remain read-only history, but Warp should not restore cloud session sharing or action replication. |
| Shared-block cloud sharing surface | `ViewSharedBlocks`, shared-block title generation setting/getter/keymap flag, block-sharing comments, and shared-block tooltip copy. | Removed. Do not restore cloud sharing actions or AI gates for shared-block dialogs. |
| `Workspace_CloudConversationWebViewer` keymap context | WASM-only old cloud conversation web-viewer shortcut gate. | Removed. Local conversation transcript viewing may remain, but it must not carry cloud web-viewer context or behavior. |
| `crates/warp_core/src/telemetry.rs`, `crates/warpui_core/src/telemetry/`, app focus telemetry | External telemetry/event queues. | Reject. |
| `script/sentry_create_release.sh`, `script/sentry_upload_dif.sh` | Sentry release/upload. | Reject. |
| `script/font_fallback/` | Obsolete fallback-font generator that read `gs://warp-static-assets` via `gcloud`; its referenced generated app file is no longer present. | Reject; do not require Warp cloud buckets for local packaging. |
| Linux/Windows host platform paths under `app/src/`, `crates/warpui/`, `crates/computer_use/`, `crates/warpui_extras/`, `crates/command/`, `crates/warp_util/`, and `docker/` | Native host implementation and packaging/build support for non-macOS targets. | Reject; this fork packages macOS only. Preserve SSH/remote terminal code separately. |

## Retained Generic Warp Code

| Area | Why retained | Merge rule |
| --- | --- | --- |
| Terminal emulator, blocks, shell integration | Core terminal product. | Accept compatible upstream fixes. |
| Input editor, slash commands, completions, NLD | Needed for terminal and ACP entry. Slash commands are the current local menu for static commands, saved prompts, and ACP commands, not a legacy Warp Agent path. | Accept or adapt. |
| AgentView shell and shortcuts | GUI shell for ACP conversations. | Adapt to ACP; keep generic conversation/code-review/help behavior. |
| Code review side panel | Generic Git diff UI, not old Warp Agent backend. | Accept local UI fixes. |
| Long-running command control transfer / CLI subagent UI | Can be routed through AgentView and ACP as terminal control state. | Keep when it integrates with ACP flow. Reject only cloud handoff, orchestration, or remote-control services that bypass ACP. |
| ACP tool-call rendering | Protocol-native ACP UI surface. | Keep generic rendering; reject app-side MCP config/start/execute/capability management. |
| Local object/persistence model | Needed by workflows, prompts, AI facts, conversation history. | Inspect carefully before accepting or removing. |
| `crates/remote_server/` | Remote terminal support. | Keep SSH/local behavior; reject Warp-account auth requirements. |
| OS launch-at-login | Operating-system setting, not account login. | Keep unless product scope changes. |
| macOS platform integration | Only maintained native host platform. | Accept compatible AppKit/windowing/signing/secure-storage/preferences fixes. |

## 2026-05 macOS-Only Host Cleanup

- Local WSL/MSYS2 shell discovery, shell launch, startup-directory conversion, path conversion, shell indicators, and bootstrap compatibility branches were removed.
- Shell bootstrap assets now use the POSIX DCS JSON path only; ConPTY, MSYS2, and Windows OSC/reset-grid branches are no longer retained.
- Local PTY spawning, terminal-manager startup, local command execution, history reads, persisted path encoding, Node runtime installation, and LSP command spawning were reduced to macOS/POSIX host behavior.
- Host UI/keybinding/platform abstractions were folded to macOS: AppKit windowing, native modals, macOS global shortcuts, macOS keybindings, macOS log/font/LSP binary conventions, and Unix path encoding are the maintained paths.
- Linux/Windows-only settings, keybinding alternatives, integration-test gates, window-instance metadata, and host build aliases were removed instead of kept as compatibility no-ops.
- App build-script platform aliases were reduced to macOS/native features; the app no longer declares Linux/Windows crash-recovery gates.
- Removed unused Windows bundled asset constants for ConPTY/OpenConsole/DX DLL files from `crates/warp_util/src/assets.rs`.
- Shell bootstrap assets no longer define Warp app update hooks or Linux apt-source repair helpers such as `FinishUpdate` and `warp_handle_dist_upgrade`; package updater behavior is outside this macOS-only local fork.
- SSH remains retained terminal functionality. Remote SSH command execution was kept and simplified around direct `ssh` usage; do not remove SSH, remote-server, or remote terminal code merely because the connected host may be Linux or Windows.
- SSH Warpify no longer offers the Linux `~/.warp` portable tmux installer that downloaded Warp-owned binaries from `github.com/warpdotdev/portable-tmux`. Keep SSH/Warpify itself, but use the remote host's package-manager tmux install scripts when available instead of restoring Warp-hosted binary download paths.
- Removed remaining app/CLI/remote-server Web/WASM modules, compile gates, no-op implementations, and WASM-only UI/data-source branches. Search, code review, AgentView, terminal, user config, plugin host, prompt-chip logging, file/code context menus, remote-server client/manager/transport, and tests now use the retained native macOS host paths directly.
- Removed app-level WASM target dependencies and root `release-wasm`/`dev-wasm` profiles. The app manifest should describe the macOS package path, not keep Web/WASM release profiles or dependency sections for deleted app targets.
- Removed the `warp_logging` WASM logger implementation and target-specific web dependencies. Logging now uses the retained native file/stderr implementation directly.
- Removed the `ipc` crate's WASM placeholder transport and target-specific dependency gate. IPC now uses the retained native local-socket transport directly, with no WebWorker/wasm compatibility stub.
- Removed `warp_ripgrep` WASM dependency gates and search API cfgs. Ripgrep search now keeps the retained native helper-process path directly, including parent-process monitoring for the macOS host.
- Removed the `websocket` crate's WASM backend and target dependency sections. Remote terminal websocket traffic keeps the native `async-tungstenite` TLS/proxy implementation directly.
- Removed the `asset_cache` WASM fetch branch and `cfg-if` dependency. URL asset loading now uses the retained native `reqwest` path with `async-compat`, while local cache persistence remains unchanged.
- Removed the editor image asset resolver's WASM branch. Markdown image paths now use the retained local path canonicalization and URL asset loading behavior directly.
- Removed `warpui_extras` Web Storage user-preferences backend and wasm secure-storage selector. User preferences and secure storage now use the retained macOS/file/no-op test backends directly.
- Removed `repo_metadata`'s wasm-only `is_in_repo` fallback, target dependency section, and build-script platform detector. Local repo detection now uses the retained repository model directly; `local_fs` defaults on for the macOS package and remains a feature boundary for local watcher behavior, not Web/WASM compatibility.
- Removed `node_runtime`'s not-wasm dependency section and build-script platform detector. Node/npm runtime management now defaults to the retained local filesystem implementation for LSP setup on macOS.
- Removed `local_object_model`'s not-wasm persistence gate. Local object persistence is part of the retained macOS fork and is now compiled directly.
- Removed `warp_js` wasm cfg wrappers and target dependency gate. Plugin JavaScript interop now uses the retained native `rquickjs` path directly.
- Removed the integration test crate's not-wasm/unix dependency sections and macOS test gates. Local integration tooling now treats SQLite/sysinfo/nix support and macOS-specific tests as part of the retained macOS test path.
- Removed `persistence` and `ai` build-script platform detectors and made their local filesystem features default. AI code indexing and file outline generation now use the retained native local filesystem implementation directly, without wasm stubs.
- Removed `input_classifier` wasm `?Send` async-trait branches. Input classification now uses the retained native Send/Sync async trait implementations directly.
- Removed the editor render model's wasm/mobile viewport autoscroll branch. The retained macOS editor resize path emits layout resize events without web keyboard special handling.
- Removed the terminal flat-grid storage wasm size assertion branch. The retained macOS terminal grid uses the native 64-bit `Entry` size assertion directly.
- Removed the local `TargetOS::current()` wasm branch. Linux/Windows/Web enum values remain only for retained remote SSH/Warpify or serialized override semantics, not local host packaging.
- Removed `warp_completer` wasm target dependency sections, source cfgs, and stale wasm signature-loading TODOs. Command signatures now use the retained native embedded-signatures path directly for local terminal completions.
- Remote-server still parses remote host OS/architecture for SSH extension setup and fallback decisions. That is retained SSH behavior, not local Linux/Windows host support.
- Removed the `SSHTmuxWrapper` rollout flag, `ssh_tmux_wrapper` Cargo feature, private override, Features-page SSH wrapper switch, `SshSettings`, and legacy SSH wrapper integration tests. SSH Warpify is retained under `WarpifySettings`, and tmux-backed Warpification is the current default path.
- The shell-level ControlMaster SSH wrapper remains as a Warpify setting-controlled fallback only when tmux Warpification is disabled. It is retained terminal functionality, not an old AI/backend compatibility path.
- No backward-compatibility shims should be added for deleted host platforms or removed Warp cloud/account/agent data.

## 2026-05 MCP Ownership Cleanup

- Removed Warp app-side MCP file configuration, server panes, slash/CLI entrypoints, persistence tables/migrations, feature flags, capability probing, server startup, tool/resource execution, and MCP permission allow/deny lists.
- Removed the bundled `add-mcp-server` skill because it instructed agents to edit Warp-owned `.mcp.json` config and referenced the deleted Settings MCP surface.
- ACP agents may still use MCP internally, but that configuration is owned by the ACP agent process. Warp only renders ACP protocol events such as assistant text, reasoning, permissions, commands, diffs, and tool-call updates.
- `rmcp` may still appear as a transitive dependency of `agent-client-protocol`; that is part of the retained ACP protocol implementation, not Warp app-side MCP configuration, probing, or tool execution.
- Future upstream MCP changes should be rejected unless they are purely generic ACP event rendering and do not reintroduce app-side MCP management.

## 2026-05 App-Side Skills Cleanup

- Removed all app-bundled skills under `resources/bundled/skills/`, including Claude API/Managed Agents docs, tab/settings helpers, PR comment helpers, skill authoring helpers, and the MCP setup helper.
- Removed all app-bundled MCP skills under `resources/bundled/mcp_skills/`, including Figma MCP instruction bundles.
- Removed channel-gated skills under `resources/channel-gated-skills/` and the channel-gated skill copy script. Warp no longer ships dogfood/preview/stable skill resources in the app bundle.
- These resource directories are physically deleted from the repo; future merges should not keep them as inert, unreferenced files.
- `script/prepare_bundled_resources` now generates bundle metadata, licenses, and settings schema only; it no longer copies skill directories or appends skill license files.
- Removed `BundledSkills`, `PRCommentsSkill`, `ListSkills`, `PlatformSkills`, and `SkillArguments` feature flags and app feature declarations.
- Removed app-side local skill parsing, file watching, manager singleton, `/skills` and `/open-skill` commands, AI context skill insertion, inline skill menu, `ReadSkill` executor/action/result, `InvokeSkill` input/output types, skill-specific file/diff open buttons, tab-config update skill CTA, and `warp_cli` skill spec parsing.
- ACP agents may still use their own skills or MCP servers internally. Warp does not manage, distribute, discover, invoke, or inject those skills into ACP prompts, and OpenAI-compatible Next Command/Prompt Suggestions do not use skill bundles.
- ACP client capabilities remain in `app/src/ai/acp/` only to advertise implemented Warp host handlers, such as terminal and file read/write, to the ACP agent. They must not be treated as app-side MCP capability probing.
- Retained local context references such as `<plan:...>`, `<block:...>`, and `<change:...>` remain Warp UI attachment syntax. They are parsed into ACP prompt context, not into app-managed skills or MCP calls.
- Stale ACP implementation plans were updated to use generic file-read tool-call examples instead of old skill-file examples, and to reject any skill-specific read renderer. ACP tool calls render protocol fields generically; skill semantics are external to Warp.

## 2026-05 UI Source Metadata Cleanup

- Removed stale UI source-only metadata enums and no-op action/function parameters for link opening, notification agent variant, saved-workflow modal source, block-filter toggle source, prompt-suggestion interaction source, CLI-agent telemetry type, and rewind entrypoint. Retained actions now carry only the data required to perform local behavior.
- Removed `AgentModeEntrypoint` source metadata from AgentView tab/pane actions. The retained actions now route directly to local ACP AgentView creation instead of carrying old analytics entrypoint labels.
- These source labels were old telemetry/product analytics residue. Future upstream merges should not restore them unless a retained local behavior actually branches on the value.

## 2026-05 macOS-Only Platform Cleanup

- Removed Linux/Windows fallback branches from default custom keybinding resolution and binding display tests. The app now treats default shortcuts as macOS-native only; future upstream merges should not restore cross-platform keybinding fallbacks or cmd-binding rejection for non-macOS hosts.

## 2026-05 Test Harness Notes

- Workspace and pane-group tests that can construct `WelcomePane` must register `ProjectManagementModel`, matching production app initialization.
- Tests that need a terminal-backed initial workspace should keep `WelcomeTab` disabled while creating the workspace, then enable it only around explicit welcome-pane snapshot construction.

## 2026-05 Old Warp Model Surface Cleanup

- Removed the Codex integration modal, `codex` custom URI host, related root/workspace actions, and the unused Codex integration image asset.
- Removed debug actions that wrote `opencode-warp` plugin entries into the user's global OpenCode config, including the `github:warpdotdev/opencode-warp-internal` installer path.
- Removed execution-profile model selection UI: base model, coding model, full terminal use model, computer use model, and configurable context-window controls.
- Removed profile-stored model fields and old `PreferredAgentModeLLMId` inheritance. Execution profiles now represent local permissions and profile naming, not model routing.
- Removed the inert `warply model list` CLI parser, old CLI output-format plumbing, `LaunchMode::CommandLine`, and the non-interactive CLI default AI execution profile. Retained CLI parsing is limited to app launch arguments, worker subprocesses, and shell completions; do not restore old headless model/agent command surfaces.
- Removed per-terminal LLM override snapshot/restore paths and the `llm_model_override` persistence migration/schema field.
- ACP request flow still records the active ACP model on outbound request data; model/backend choice comes from ACP adapter configuration, not old Warp `/model` or profile selectors.
- Removed old request-entrypoint metadata plumbing (`EntrypointType`, `RequestMetadata`, `query_metadata`, and unused resume-on-error request flags). ACP requests now flow through typed `AIAgentInput`/conversation state only; Warp does not attach old hosted-service analytics metadata to ACP sends.
- Removed the unused `ApiKeyManager` and `AiApiKeys` secure-storage slot for old multi-provider BYO key management. OpenAI-compatible Next Command and Prompt Suggestions keep their dedicated terminal-suggestions endpoint/API-key settings under `AISettings`; do not restore generic Warp-provider key storage.
- Reduced `LLMPreferences` to local static model metadata for retained AgentView rendering and image-context gating. Removed old server-fetched model catalog types, provider/host routing metadata, context-window config, and wire-format compatibility tests; ACP adapter/model choice comes from ACP settings and session config.
- Removed the deprecated `planning_model_id` AI-query schema column from the fork's final local database shape and stopped mapping it into app runtime structs. Retained AI history should record ACP model metadata only, not old Warp planning-model selector state.
- Reworded retained AI settings comments around speedbumps, setup banners, and default tab config paths to local persistence. These settings should not imply telemetry, cross-device tracking, cloud sync, or server-backed model/API configuration.
- Removed the old account identity payload chain from local identity and the remote-server initialize handshake. Remote terminal setup keeps only the non-secret identity key needed for daemon/socket partitioning; do not send user id/email fields in the remote-server protocol.
- Removed unreachable old provider/BYOK error variants from AgentView rendering, including invalid API key and context-window specific branches plus the dedicated "Edit API Keys" failed-output UI. ACP and terminal suggestions should surface current errors through retained ACP/suggestions error paths instead of restoring old Warp provider key-management UX.
- Renamed the retained 429 AI error path from server-overloaded to provider-overloaded and removed the unused no-context-found API error variant. Remaining terminal suggestion errors are provider/local transport errors, not Warp server search failures.
- Reworded stale AgentView comments that described ACP/local flow as server formatting, server task creation, or server streaming. Use agent/backend/local wording for retained ACP code and reserve `remote_server` for SSH remote terminal behavior.
- Reworded retained Agent Mode and SSH remote-terminal coordinator comments away from old service/orchestration wording. Local ACP request handling and SSH init coordination are not cloud orchestration.
- Removed explicit `BlockContext` serde defaults used to accept older context payloads. Current block context payloads must explicitly carry auto-attachment state, while optional environment fields remain ordinary nullable current payload fields.
- Removed the `AgentModeAutoReadFiles` private-setting migration from Agent Mode permissions. Retained permissions read the current `AgentModeCodingPermissionsType` settings only.
- Removed the deprecated `AIMemory.is_autogenerated` rule field and stopped preserving it across local AI fact edits. Rule behavior is represented by current content/name only.
- Removed unused conversation-list environment and harness filters/display fields. Local ACP conversations are `LocalInteractive`; CLI-agent status icons remain terminal-session state, not cloud run/harness metadata.
- Removed the unused `warp_cli::agent::Harness` type and terminal conversion helper. Terminal CLI-agent detection remains command-based; Warp no longer keeps a separate legacy agent-run harness model.
- Removed the unused `installation_detection_server_subcommand()` helper. The fork no longer keeps dead CLI entrypoint helpers for worker services that are not declared by the current Warply CLI.
- Removed the `AgentViewBlockContext` rollout flag and `agent_view_block_context` Cargo feature. AgentView block auto-attachment is now the current ACP AgentView behavior rather than a compatibility branch with the old non-AgentView context reset path.
- Removed the disabled `AgentViewPromptChip` rollout flag and `agent_view_prompt_chip` Cargo feature. Terminal mode keeps the retained AgentView message-bar entrypoint instead of a dormant alternate prompt-chip path.
- Collapsed AgentView block chrome/rendering gates in AI status bars, plan/todo context chips, passive code diff accept keybindings, block background painting, and block-list viewport traversal. Old non-AgentView AI stripes, plan chips, and status padding paths were deleted; the retained UI now follows the ACP AgentView behavior directly.
- Removed the remaining `FeatureFlag::AgentView` runtime checks, app feature registration, and test overrides. Terminal view creation, CLI subagent handoff, conversation restore/resume/fork, inline code review, context selection, fullscreen rendering, key contexts, and AgentView zero-state handling now use the retained ACP AgentView path directly.
- Removed the final unused `FeatureFlag::AgentView` enum shell and deleted the old AgentView feature-availability keybinding context. AgentView is retained local ACP UI state, not a rollout flag or disabled compatibility path.
- Removed the `ActiveConversationRequiresInteraction` rollout flag and Cargo feature. The AgentView conversation list now always treats a conversation as active only after retained local interaction state marks it active, instead of falling back to the old "all open AgentViews are active" path.
- Removed the `AIRules` rollout flag and Cargo feature. Rules/AI facts remain retained local ACP prompt context and rules panes restore directly without an old disabled-feature error branch.
- Removed the `AgentToolbarEditor` and `CLIAgentRichInput` rollout flags plus their Cargo features and test overrides. The agent/CLI toolbelt editor and CLI rich input editor remain retained local AgentView/terminal features and now run directly without fallback-disabled branches.
- Removed the old automatic suggested-rules output path: `FeatureFlag::SuggestedRules`, the `suggested_rules` Cargo feature, `AIAgentOutput.suggestions`, `SuggestedRule`/`SuggestedLoggingId`, rule suggestion chip/modal/footer UI, the rule suggestion AI setting, and `AIMemory.suggested_logging_id`. Rules/AI facts and project rules remain retained local ACP prompt context and are still managed through the Rules pane.
- Removed local non-mac platform branches from default-terminal registration, meta-key compose handling, local interactive PATH capture, native window-button visibility, restored-window fullscreen bounds, login-item labels, bundle warning checks, UI shortcut hints, file reveal labels, prompt/dynamic-enum keybindings, inline-menu defaults, editor/block-selection modifier handling, shell-script openability, mac-only test guards, and Windows-only remote-server unsupported fallbacks. The app target is macOS-only; SSH/remote platform support remains separate and retained.
- Removed mac-only `cfg` wrappers around the terminal-server worker, app menus, preview config migration, app icon setup, Services menu entry, and Sample Process action. These are now unconditional local macOS app behavior.
- Removed mac-only `cfg` wrappers around appearance app-icon handling, login-item registration, app Services integration, and terminal platform initialization.
- Removed mac-only `cfg` wrappers around settings import iTerm support, UserDefaults-backed preferences, notification sound settings, app-icon restart warning, resource limits, pane hover focus, integration preview migration tests, Homebrew shell discovery, AgentView default shortcut selection, and Sample Process imports/dispatch.
- Removed the final local macOS target gates from heap-profile helper lookup, embedded plist generation, and audible bell implementation selection. The audible bell now uses the macOS implementation directly; the WASM/noop implementation was deleted.
- Removed the remaining macOS target check around unsaved-tab native modal display; close confirmation now directly uses the local native modal path.
- Removed Web/WASM branches from `warp_core` operating-system info, error registration, sync-queue futures, and `http_client` request/SSE/response handling. `warp_core` now enables local filesystem support through default features instead of target-detection build scripts, and HTTP OS headers no longer carry Linux-specific kernel metadata.
- Removed Web/WASM and disabled-local-filesystem branches from `lsp`. LSP service startup, command building, file URI conversion, server logging, repo watching, language-server installation, and supported-server candidates now use the retained local macOS implementation directly, and clangd auto-install resolves the macOS asset only.
- Removed Web/WASM support branches from `warpui_core`, including the WASM platform module, WASM async executor, clipboard save stub, target-specific dependency sections, native alias build script, and native-gated font/test-driver paths. `warpui_core` now exposes the retained local macOS/native UI core behavior directly.
- Removed root Web/WASM workspace dependencies and Web platform setting branches. `warpui` now depends on its macOS backend directly instead of target-specific dependency sections or a native-vs-Web build alias.
- Removed the remaining `warpui` text-layout test/example platform branches and the `cfg_aliases` build dependency. Text layout tests now assert the retained CoreText/macOS behavior directly instead of carrying non-macOS expected values or ignore gates.
- Removed the final source-level `target_os` checks from retained macOS host code. Rectangular selection now directly uses the macOS Cmd+Option modifier rule, and rust-analyzer auto-install selects only between macOS architecture assets.
- Collapsed remaining local macOS dependency gates in `app`, `command`, `prevent_sleep`, and `computer_use` into direct dependencies/modules. Test-only noop behavior remains feature-driven, not platform fallback-driven.

## 2026-05 Local Object Cloud Gate Cleanup

- Removed cloud online/offline gates from local AI facts/rules and workflow modal deletion. Local persisted objects should remain editable and deletable without Warp account, cloud sync, or network status.
- Removed AI facts cloud-offline read-only UI and rule sync-status badges that represented old cloud sync state rather than local ACP behavior.
- Removed the app-level `NetworkStatus` singleton, offline cloud toolbar indicator, debug network-status toggle, and Next Command online gate. OpenAI-compatible terminal suggestions should issue requests directly and handle provider errors locally; Warp no longer maintains a cloud-product online/offline mode.
- Removed shared-block cloud sharing leftovers: `ViewSharedBlocks`, shared-block title generation setting/getter/keymap context flag, stale block-sharing comments, and shared-block tooltip copy.
- Removed Drive/local-object sync-status badge rendering (`Saving` / `Failed to save`) from retained local object items. Pending status fields may still support local persistence and unsaved-at-quit accounting, but they should not be surfaced as Warp cloud sync UX.
- Removed dead local-object conflict/error sync states and unused visible-error counting. Retained local object pending state is limited to no-pending-change vs in-flight local persistence for SQLite bookkeeping and quit-warning behavior.
- Removed online-only pending metadata, permission, untrash, and delete status fields from local object metadata. Local object updates now persist metadata and permissions directly to SQLite; only content pending state remains for local save and quit-warning behavior.
- Removed stale cloud-sync/server wording from retained local object model comments and test assertions, including local-object persistence helpers, settings value serialization notes, env-var collection state, and legacy `SyncId`/`ServerId` descriptions.
- Cleaned remaining retained `CloudObject` comments to describe local object model behavior instead of shareable/collaborative cloud objects or server APIs. Type names may remain, but new explanations should use local persistence wording.
- Renamed the SQLite `object_metadata.shareable_object_id` column and Rust fields to `local_object_id` in the current fork schema/migrations without compatibility aliases. The value is a local content-row id, not a Warp Drive sharing identifier.
- Renamed the SQLite `object_metadata.server_id` column and Rust fields to `stable_object_id` in the current fork schema/migrations without compatibility aliases. This stores the stable server-style local object id form and is not a Warp service id.
- Removed stale auth-state/cloud-sync wording from retained toolbar and keymap comments; these retained UI paths should not imply account auth or cloud synchronization.
- Rewrote the EVC README to describe local GenericStringObject persistence instead of deleted Warp server APIs, cloud object update managers, or multi-user edit collision flows.
- Renamed retained workflow and environment-variable collection runtime types/events from cloud wording to saved/local wording (`SavedWorkflow`, `SavedWorkflowModel`, `WorkflowType::Saved`, `WorkflowSource::Saved`, `SavedEnvVarCollection`, saved workflow command metadata, and saved workflow pane snapshots). The SQLite command-history column is now `saved_workflow_id` for fresh local schema generation; do not add compatibility aliases for the old `cloud_workflow_id` column.
- Reworded retained workflow, EVC, transcript, and prompt-suggestion comments away from server/cloud phrasing. Saved workflows and EVCs are local object model updates, transcript loading reads local history, and terminal prompt suggestions refer to the configured suggestion provider rather than a Warp server.
- Removed dead local-object cloud-sync writer events (`MarkObjectAsSynced` and `IncrementRetryCount`) and the unused workflow stable-id helper. Retained `ServerId` names are legacy local-object identifiers only; `stable_object_id` is the current SQLite column for server-style local object ids.
- Collapsed local-object update events from `client_id`/`stable_object_id` dual fields to a single `SyncId`. Object creation no longer waits for or backfills a server-assigned id, and UI listeners should compare the current local object id directly.
- Removed the unused workflow-alias id migration helper. Workflow aliases should be saved against the current local workflow `SyncId`; do not add old-id to new-id alias backfill paths for initial save or cloud-style id replacement.
- Removed server-style hash fallback when restoring saved workflow panes, saved environment-variable panes, and command-history `saved_workflow_id` links. These local UI references should parse the current client-id form only.
- Updated local object metadata writes to match the current object hash in either local id column rather than assuming only the `stable_object_id` column. This keeps client-id objects created by the fork persisting metadata directly.
- Replaced folder parent-id fallback parsing with prefix-directed parsing of the retained local `SyncId` forms. Do not restore ambiguous "try server id, then client id" parsing for local object references.
- Removed workflow argument import/deserialization fallbacks that silently converted missing or invalid `arg_type` data to text arguments. Workflow JSON/YAML import should reject malformed current-format arguments instead of accepting old formats.
- Removed local settings compatibility paths for old shell and prompt-chip settings: new-session shell selection no longer migrates from `startup_shell_override`, and the private `GitPromptDirtyIndicator` setting is no longer retained.
- Removed tab selected-color restoration fallback for the old bare `AnsiColorIdentifier` YAML format. Persisted tabs should use the current `SelectedTabColor` format.
- Removed stale `warp-server`, GCP load balancer, and `warp-internal` wording from retained local asset, diff-validation, command-signature, and workflow comments/errors.
- Removed stale onboarding, Warp Drive, cloud sync/status, and terminal input sync wording from retained prompt-preview, local object, scroll-performance, and terminal input mirroring comments.
- Removed the command-palette `ItemSummary::CloudObject` dummy variant because it only represented unsupported old cloud-object zero-state UI.
- Removed the old Warp Packs folder marker: `FeatureFlag::WarpPacks`, `warp_packs` Cargo feature, `folders.is_warp_pack` schema/migration/model fields, and the unused package-check icon asset. Retained local folders are plain local object folders, not cloud template/marketplace packs.
- Removed the old Agent Mode workflow suggestion output path: `FeatureFlag::SuggestedAgentModeWorkflows`, the `suggested_agent_mode_workflows` Cargo feature, `SuggestedAgentModeWorkflow` output data, suggestion chip/modal events, and the suggested workflow modal. Saved workflows and Agent Mode workflow execution remain retained local functionality; only the old agent-output suggestion chain was removed because ACP does not produce it.
- Removed the orphaned `/cloud-agent` default slash-command binding and the unused `triggers_server_subagent` action-result helper. `/agent` remains the retained ACP AgentView entrypoint, and long-running command control transfer remains modeled as local CLI subagent state rather than server-agent routing.
- Removed the old `SuggestNewConversation` action/result/executor/render/persistence chain and the AIBlock accept/reject button plumbing that only served it. ACP does not emit this server-agent output action; `/agent`, `/new`, `/fork`, and accepted OpenAI-compatible terminal prompt suggestions remain the current local conversation entrypoints.
- Removed the old `AgentPromptSuggestionsEnabled` / `ai.active.prompt_suggestions_enabled` setting path. Prompt Suggestions are controlled by the retained `terminal_suggestions.prompt_suggestions_enabled` OpenAI-compatible terminal-suggestions setting; do not restore a separate old Agent Mode prompt-suggestion gate.
- Removed the old `SuggestPrompt` action/result/executor/render/persistence chain, the Unit Tests suggestion inline view, and the dead `TriggerPassiveSuggestion` / `PassiveSuggestionResult` AI input paths. ACP does not emit these legacy Agent actions or inputs, and OpenAI-compatible Next Command / Prompt Suggestions continue through `app/src/ai/terminal_suggestions/` and terminal banner entrypoints without bundled skills or app-managed MCP.
- Removed the old `LLMPreferences` singleton and model-info stub that only preserved legacy Warp AI model/profile selector shape. Request history keeps `LLMId` fields for display/persistence, but ACP request dispatch overwrites them from the active ACP backend/config; do not restore `/model` or `/profile` selector plumbing.
- Removed unused `loading-agents-01..08.svg` assets and the corresponding `Icon::LoadingAgents*` enum variants. ACP AgentView uses retained generic AgentMode and Warpify icons; do not restore old agent-loading asset sets unless they are required by a retained local UI.
- Removed automatic Warp product headers from the shared `http_client` wrapper. Local fork HTTP calls, including OpenAI-compatible terminal suggestions and LSP downloads, should not attach Warp client id, app version, OS, or integration-test extra headers by default; add endpoint-specific headers only at the actual retained caller.
- Removed unused request/response hook plumbing from the shared `http_client` wrapper, including mirrored request-body serialization for hook consumers. Retained HTTP usage should be direct endpoint code for local features such as OpenAI-compatible suggestions and LSP downloads, not a global service instrumentation layer.
- Removed the global execution-mode client-id reporting path and collapsed the old command-line SDK execution mode into `ExecutionMode::Headless`. Execution mode now only gates local desktop-vs-headless behavior; it should not be used to identify a Warp cloud/API client or old headless agent/model CLI.
- Removed autonomous headless-agent action execution branches. Non-user-initiated ACP actions that cannot auto-execute now consistently require confirmation instead of using old SDK/headless denylisted-command result shortcuts.
- Removed `warp_core::errors` actionability registration and its `inventory`-collected error adapters. That machinery only modeled old central error triage/crash-reporting decisions and had no retained caller after local ACP conversion; provider errors now stay as local `AIApiError` values rendered by the terminal/AgentView paths.
- Made local identity mandatory in the fork. `LocalIdentity` and `UserWorkspaces::current_user_owner` now return concrete local values instead of optional account-style fallbacks; callers should not skip workflow, environment-variable, rule, conversation, or remote-terminal setup because a Warp account identity is unavailable.
- Future upstream changes should not reintroduce online-only edit/delete checks, cloud sharing permissions, or sync-status UX for retained local workflows, prompts, AI facts, and environment-variable collections unless they are backed by a local-only persistence rule.
- Removed dead `RetryTruncatedCodeResponses` and `SummarizationViaMessageReplacement` feature flags. They had no retained callers after ACP conversion, and the latter only represented old server-side conversation summarization behavior rather than local ACP AgentView or OpenAI-compatible terminal suggestions.
- Removed dead `LazySceneBuilding` and `MarkdownImages` rollout flags from `warp_features`. They only remained in the DOGFOOD list with no runtime or Cargo callers; retained AgentView markdown image behavior now runs directly, while Mermaid rendering remains controlled by the retained `MarkdownMermaid` feature.
- Removed dead terminal rollout flags `CommandCorrectionsHistoryRule`, `NativeShellCompletions`, and `MaximizeFlatStorage`. They had no Cargo/channel/runtime enable path in the fork. Current behavior is preserved: command correction keeps the history rule ignored, native shell completions still follow the existing `ForceNativeShellCompletions` private preference, and grid storage no longer carries an unreachable max-storage branch.
- Removed the dead `RememberFastForwardState` rollout flag. Fast-forward autoexecute override is retained local conversation state and restored directly from local persistence instead of being hidden behind an unenabled rollout gate.
- Removed the old `AgentMode` rollout flag plus the `agent_mode` and `agent_mode_debug` Cargo features. Agent Mode is now part of the retained ACP AgentView surface whenever local AI is enabled; do not restore disabled Agent Mode keybinding, icon, prompt-history, or natural-language fallback branches.
- Collapsed obvious `AgentView`-always-on branches in retained settings, slash-command registration, and conversation restore entrypoints. These surfaces now assume the local ACP AgentView shell exists; do not restore non-AgentView settings, prompt icon, or active-pane restore fallbacks.
- Removed the `agent_view` Cargo feature and made `FeatureFlag::AgentView` unconditionally enabled during app initialization. Removed old non-AgentView slash-command modality branches from `SlashCommandModel`, the terminal input message-bar helper, and the disabled new-conversation keybinding. Slash commands now follow the retained AgentView/ACP availability model directly, including terminal-mode slash command settings and command-level auto-enter behavior.
- Removed remaining non-AgentView slash-command execution/data-source fallbacks. Slash command availability now marks either AgentView or terminal view from the retained ACP AgentView controller state, and `/conversations` plus `/prompts` always open the AgentView inline menus instead of falling back to old palette paths.
- Removed dead non-AgentView branches from classic terminal input rendering. Classic input remains available for retained terminal presentation behavior, but no longer carries old AgentView-disabled attachment, Vim-status, or divider fallback code.
- Collapsed shallow AgentView gates in terminal input initialization. Slash-command placeholders, AgentView shortcut binding, editor keymap context, and inline conversation/user-query menu subscriptions are now registered directly because ACP AgentView is part of the retained local input surface.
- Collapsed AgentView-always-on prompt and decoration branches. Same-line prompt rendering, editor decorator prompts, slash-command input highlighting, and AI cursor color now use the retained ACP AgentView behavior directly instead of preserving old AgentView-disabled styling paths.
- Removed the dead `should_override_shell_lock` parameter from agent-mode entry helpers. AI-feature entrypoints now use the retained ACP AgentView behavior directly and always preserve an explicit locked shell mode instead of carrying an old override/fallback switch.
- Removed the old AgentView-disabled `* ` AI input prefix handler. Terminal `!` prefix handling remains retained behavior, scoped to active AgentView or CLI bash-mode input instead of falling back to the removed non-AgentView modality.
- Collapsed `BlocklistAIInputModel` AgentView feature checks. Input state now subscribes to AgentView events unconditionally, initializes terminal mode from terminal NLD settings, uses AgentView fullscreen state to choose ACP AI autodetection, and rejects locked AI mode outside active AgentView or CLI rich input without an extra feature gate.
- Collapsed terminal input-mode keybinding gates around AgentView. Prompt suggestion accept uses the retained ACP AgentView binding directly, and terminal/agent mode switching predicates no longer carry the removed AgentView-disabled branch.
- Removed the old `AgentModeWorkflows` rollout flag and `am_workflows` Cargo feature. Saved prompts/workflows are retained local data and now appear in command search whenever the local AI/ACP surface is enabled, instead of carrying a separate old Agent Mode gate.
- Removed the `AIResumeButton` rollout flag and `ai_resume_button` Cargo feature. Resume remains retained local ACP AgentView behavior for cancelled or errored conversations, not a channel-gated old Agent feature.
- Removed the old `/compact`, `/compact-and`, and `/fork-and-compact` summarization/compaction path, including `AIAgentInput::SummarizeConversation`, `WorkspaceAction::SummarizeAIConversation`, the `SummarizationConversationCommand` rollout flag, the `summarize_conversation_command` Cargo feature, and the compact-only SVG assets. The removed path did not implement ACP-native context compaction; it degraded to sending the legacy `SummarizeConversation` input name as a prompt. Reintroduce compaction only when ACP exposes a real protocol-backed flow.
- Removed the `FastForwardAutoexecuteButton` rollout flag and `fast_forward_autoexecute_button` Cargo feature. The auto-execute toggle remains retained local ACP AgentView behavior whenever the AI/ACP terminal context is active; it is not tied to bundled skills, app-managed MCP, or the old Warp Agent rollout path.
- Removed the `QueueSlashCommand` and `PendingUserQueryIndicator` rollout flags plus their Cargo features. `/queue`, queue-next-prompt, and the pending queued-prompt block remain retained local ACP AgentView behavior, independent of old channel-gated Agent rollout state and unrelated to bundled skills or app-managed MCP.
- Removed the `AgentDecidesCommandExecution` rollout flag and `agent_decides_command_execution` Cargo feature. Command permission logic still honors retained `is_risky == Some(false)` action metadata as an Agent-decided allow decision, but no longer depends on an old Warp Agent feature gate.
- Removed the `AskUserQuestion` rollout flag and `ask_user_question` Cargo feature. The clarifying-question executor, permissions, inline UI, and result rendering remain retained local AgentView behavior that can be driven by ACP-compatible action flow; it is not an app-bundled skill or Warp cloud service.
- Removed the `SearchCodebaseUI` rollout flag and `search_codebase_ui` Cargo feature. Search-codebase execution and inline rendering remain retained local AgentView behavior; the old fallback renderer and gate should not be restored as a compatibility path.
- Removed the `LocalComputerUse` rollout flag and `local_computer_use` Cargo feature. Computer-use permissions remain local execution-profile settings; normal profiles still default to disabled and no old CLI sandbox profile or CLI override remains.
- Removed the `AgentTips` rollout flag and `agent_tips` Cargo feature. Agent tips remain local AgentView UI controlled only by the retained `ai.input.show_agent_tips` user setting, not by old channel rollout state.
- Removed the `CycleNextCommandSuggestion` and `PartialNextCommandSuggestions` rollout flags plus their Cargo features. Next Command cycling and prefix-based OpenAI-compatible terminal suggestions remain retained terminal-suggestions behavior controlled by current terminal suggestion settings, not old Warp rollout gates.
- Removed the `ConversationsAsContext` rollout flag and `conversations_as_context` Cargo feature. Local conversation history remains attachable through the AgentView context menu as ACP prompt context; do not restore a separate rollout gate or cloud conversation viewer path.
- Removed the `LSPAsATool` rollout flag and `lsp_as_a_tool` Cargo feature. LSP repo watcher lifecycle is retained as local code-intelligence plumbing and no longer depends on old Agent rollout state.
- Removed the `RevertToCheckpoints` and `RewindSlashCommand` rollout flags plus their Cargo features. Local `/rewind` and rewind-to-before-exchange UI remain retained AgentView behavior, with restored blocks still excluded because their full diff state is not restored.
- Removed the `AIContextMenuCommands` and `AIContextMenuCode` rollout flags plus their Cargo features. Command context remains available to ACP prompt context, and code-symbol context is controlled by the retained code-outline setting plus repository availability rather than old rollout gates.
- Removed the `AtMenuOutsideOfAIMode` rollout flag plus its Cargo feature. Terminal-mode `@` context menu behavior is controlled by the retained `terminal.input.at_context_menu_in_terminal_mode` setting and ACP-compatible context validation rather than old rollout state.
- Removed the `AIContextMenuEnabled` rollout flag and `ai_context_menu` Cargo feature. The AgentView/terminal `@` context menu remains retained ACP prompt-context UI and now runs directly through the existing AI input and terminal-mode settings without an old disabled-feature branch.
- Removed the `BlocklistMarkdownImages` rollout flag and `blocklist_markdown_images` Cargo feature. Inline markdown image/lightbox handling remains retained AgentView output rendering and no longer falls back to plain markdown because of an old disabled-feature branch.
- Removed the `AllowIgnoringInputSuggestions` rollout flag and `allow_ignoring_input_suggestions` Cargo feature. Ignoring local terminal input/history suggestions remains retained terminal UI behavior and is controlled directly by the existing setting/widget path instead of an old rollout gate.
- Removed the `GithubPrPromptChip` rollout flag and `github_pr_prompt_chip` Cargo feature. The GitHub PR chip remains retained local prompt/context UI for terminal and AgentView footers, not a Warp cloud product surface, and no longer has an old rollout-disabled branch.
- Removed the `ImageAsContext` rollout flag and `image_as_context` Cargo feature. Image attachments remain retained local AgentView/CLI prompt context and rendering behavior, gated only by the current AI input mode where applicable, not by an old rollout-disabled branch.
- Collapsed AgentView feature gates in terminal block visibility. Block and block-list filtering now always use `AgentViewState` and `agent_view_visibility` directly; terminal blocks and ACP AgentView blocks remain scoped by conversation visibility without a disabled-AgentView bypass.
- Collapsed AgentView feature gates in the ACP AgentView context/controller flow. Slash-command requests and auto-enter slash commands now enter AgentView to create ACP conversations, slash command parsing no longer mutates AI input state directly, explicit slash-command AgentView entry keeps AI input locked, pending-query helpers always route through `AgentViewController`, autoexecute toggles target the active AgentView conversation, and the old classic follow-up fallback parameter was removed from request sending.
- Collapsed AgentView feature gates in terminal input rendering and submission. Natural-language/AI input submission now always emits `EnterAgentView`, terminal input keymap context always advertises AgentView capability, image attach enters AgentView directly when needed, and the disabled-AgentView Universal input renderer plus classic AI follow-up icon helpers were removed.
- Collapsed AgentView feature gates in terminal pane header integration. Pane title/menu refresh, fullscreen AgentView header rendering, navigation back button, and conversation chrome title fallback now use `AgentViewController` state directly instead of an AgentView rollout fallback.
- Collapsed AgentView feature gates in context chip and editor control rendering. Prompt chips now keep AgentView and CLI footer chip state active when an `AgentInputFooter` consumer is present, render with the AgentView/UDI wrap layout directly, and the old non-AgentView editor image/@ control buttons were removed from the editor surface. ACP AgentView attachments and OpenAI-compatible terminal suggestions remain the retained entrypoints, without app-bundled skills or MCP-managed prompt helpers.
- Collapsed AgentView feature gates in AI conversation restore. Historical restores now use the live AgentView appearance path, startup restore inserts AgentView entry blocks directly, restored rich content is always conversation-scoped, and restoring a previously open AgentView depends on restored conversation state rather than a rollout flag.
- Collapsed AgentView feature gates in code-review context insertion. Adding a diff or diff hunk as context now enters AgentView directly when the target terminal is not already in AgentView, preserving ACP AgentView as the only agent backend.
- Collapsed AgentView feature gates in AI block rendering and passive-diff continuation. AI block headers, status padding, transparent block backgrounds, hidden-block visibility, fork handling, and passive-diff continuation now use AgentView behavior directly; the old non-AgentView stopped/continue block UI path and related action/event plumbing were removed.
- Removed remaining local-host `#[cfg(unix)]` branches in virtual filesystem test helpers and LSP process shutdown. Remote Linux metadata, bootstrap parsing, and SSH/Warpify assets remain retained because the macOS client still needs them for remote sessions.
- Removed the `WarpifyFooter` rollout flag plus its Cargo feature. Warpify now uses the retained footer path directly for subshell and SSH prompts; the old in-block warpification banner fallback actions should not be restored.

## 2026-05 Agent Shared-Session Viewer Cleanup

- Removed the old `BlocklistAIActionModel` shared-session/view-only state, remote action execution markers, and finished-action replay path. These paths had no callers after ACP conversion and belonged to cloud session sharing rather than the local ACP execution flow.
- Kept generic read-only code diff rendering for ACP tool-call diffs. Read-only rendering is UI display state, not a session-sharing execution path.
- Removed terminal shared-session resize reasons and input CRDT peer-edit plumbing whose only remaining consumer was the deleted session-sharing flow. Terminal input still tracks the latest active block for local input lifecycle behavior.
- Removed stale session-sharing protocol references from retained local CLI-agent metadata helpers.
- Future upstream changes should not restore shared-session viewer action mirroring, cloud session sharing, or remote action-result replay. Preserve local conversation transcript viewing only when it reads retained local history without cloud session services.

## 2026-05 Agent/Service Residual Scan

- Scanned source for old Agent SDK, cloud/hosted/ambient/scheduled agent, orchestration, remote-control, harness, Warp server, GraphQL, auth token, billing, telemetry, crash reporting, onboarding, marketplace, and template product surfaces.
- `handoff` source hits were local cross-window tab drag transfer or AgentView long-running-command control transfer. Keep these when they remain local terminal UI or ACP-routed AgentView behavior; do not reinterpret them as old cloud handoff/orchestration.
- `harness`, `remote control`, `GraphQL`, `billing`, `credits`, `marketplace`, and `template` hits were test harnesses, keyboard documentation, file-type fixtures, tokenizer vocabulary, retained tab/workflow templates, or docs. They are not live Warp cloud product paths.
- Reworded the retained remote terminal daemon model away from orchestration wording. `remote_server` remains SSH/remote terminal functionality, not Warp service-client behavior.

## Legacy Names Still Present

These names are not enough to decide merge behavior:

- `blocklist`: legacy name for AgentView and AI output code.
- `subagent`, `handoff`, and `control transfer`: inspect the data flow. Keep long-running command control handback/takeover paths when they are AgentView/ACP-integrated terminal behavior; remove only Warp cloud handoff/orchestration/remote-control services.
- `CloudObject`: can be local persisted object data after cloud sync removal.
- `CloudModel` and related object IDs can remain as legacy type names for retained local persistence, but new or updated docs/comments should describe them as local object model behavior, not cloud sync/server APIs.
- Retained workflows and environment-variable collections should use saved/local naming in runtime code. Do not restore `CloudWorkflow`, `WorkflowType::Cloud`, `WorkflowSource::PersonalCloud`, `CloudEnvVarCollection`, or `cloud_workflow_id` for new command-history data.
- Renamed workflow enum `is_shared` to `is_visible_to_other_workflows` without a serde alias or migration shim. Workflow enum visibility is local argument-selector behavior, not Warp Drive/cloud sharing.
- `ServerId`: legacy naming only when still present in retained local-object data; describe it as a server-style local identifier, and do not add new compatibility fallback around it. `stable_object_id` is the current SQLite column for that identifier form.
- `local_object_model`: contains retained shared DTOs/identity/object types after cloud API removal.
- `remote_server`: remote terminal, not necessarily cloud account auth.

Always inspect call sites and data flow before making a merge decision.

## Required Audit Queries

Before finishing a major upstream merge, run:

```bash
rg -n "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment" app crates script Cargo.toml
```

Every hit should be one of:

- Documentation.
- Local logging.
- Legacy local naming with no live cloud dependency.
- Retained remote-terminal code.
- Retained local object/persistence code.

Also audit macOS-only host scope:

```bash
rg -n "target_os = \"linux\"|target_os = \"windows\"|cfg\\(windows\\)|WSL|MSYS2|x11|wayland|winreg|windows-registry|x11rb" app crates Cargo.toml
```

Allowed hits should be terminal protocol/remote-path data that a macOS client still needs, documentation, or tests that intentionally exercise cross-platform parsing. Native Linux/Windows host code should be removed.

Anything else should be removed or adapted to the fork contract.
