# Upstream Master Audit 2026-05-14

Range audited: `19659d12b79a6f03b9931eca93622e34d8665d3e..master`

Upstream tip audited: `1ca5496d8 docs: clarify bug readiness wording (#10866)`

Total upstream commits reviewed: 249

This audit supersedes the 2026-05-10 audit for future merge decisions. The 2026-05-10 file is still useful history, but later user requirements tightened the fork contract:

- Do not add backward compatibility shims for deleted Warp cloud/account/agent data or deleted host platforms.
- Treat macOS as the only native packaged platform.
- Remove or reject Linux/Windows native host, package, WSL, MSYS2, winit/Wayland, installer, registry, and CI platform changes.
- Keep SSH and remote terminal behavior because it is part of the terminal product, not the AI/cloud product.
- Keep ACP and OpenAI-compatible terminal suggestions as the AI path.
- Do not keep Warp app-managed MCP configuration, capability probing, startup, execution, persistence, or permissions. MCP belongs to the ACP agent process; Warp only renders ACP protocol events.

All commits in the range were inspected with path-level diffs using `git show --name-status --find-renames`. Compatible changes were manually ported or adapted; no upstream commit was cherry-picked directly.

## Ported Or Adapted

Feature flag cleanup and no-backward-compatibility behavior:

- `451da341f` RemoveAltScreenPadding cleanup: accepted as unconditional alt-screen behavior.
- `a7fd4a5d0` LessHorizontalTerminalPadding cleanup: accepted as fixed macOS terminal padding.
- `a7e795db9` SettingsImport cleanup: accepted as unconditional local settings import.
- `0d92811d6` BlockToolbeltSaveAsWorkflow cleanup: accepted as unconditional local workflow behavior.
- `2c1f2042d` NLD flag cleanup: accepted by removing fasttext/user-preference runtime gates and keeping ONNX plus heuristic fallback.

Terminal, editor, completion, workflow, and local UI:

- `b88ae2cec` tmux statusline color leak: ported to terminal grid rendering.
- `c23106005` `.command` run support: ported to local file type handling.
- `dc372ce5d` Vim `zz`: ported to the code editor Vim handler.
- `83340951b` Vim `ctrl-d` / `ctrl-u`: ported to code editor Vim actions.
- `4aa545819` bash HISTSIZE sentinel: ported for host shell and Docker sandbox launches.
- `0ed366385` `$CDPATH` `cd` completion: ported through bootstrap payload, session state, and completer engines.
- `568ed6208` live running-command duration: ported with a local `LiveElement` repaint wrapper.
- `69d8b47b` jq syntax highlighting: ported through arborium feature, grammar config, language registry, and text-file detection.
- `d72cee89c` command palette zero-state performance: ported by making zero-state derived from the editor/search state instead of cached state.
- `6df2cc08b` workflow editor close button: ported for retained local workflows.

ACP-adjacent local context and AgentView shell:

- `df4c8d2a6` context chip periodic refresh: ported with retry behavior preserved for error/timed-out refreshes.
- `ce56553d3` code review repo switcher theme fix: ported.
- `f3072231e` remember selected code review repo per pane group: ported.
- `21e70d566` watcher creation failure panic: ported as fail-soft watcher setup.
- `a1b76c288` multiline partial-line suffix preservation: ported to diff validation.

SSH and remote terminal behavior retained:

- `91b4f0971` run command executor race: ported by notifying the remote manager before initializing bootstrapped sessions.
- `2c5ba1af3` remote binary `--version` verification: ported.
- `d9dee18e1` daemon message too big: ported by truncating command output bytes under protocol max size.
- `363d1d6e9` SSH disconnect downgrade: ported so expected disconnect read/write/flush errors log as warnings.
- `cde06f9c0` version-aware daemon socket: ported for socket and pid naming plus stale cleanup.

## Rejected Or Reduced

Cloud/account/product surfaces were rejected:

- Auth, anonymous users, access tokens, SSO, paste-token login.
- Billing, credits, addon credits, referrals, invite/team/workspace discovery.
- Warp Drive cloud sync/sharing/import/export.
- Cloud GraphQL schema/client changes, managed secrets, hosted isolation, and cloud environments.
- Old Warp Agent SDK, third-party harness/cloud agents, ambient/scheduled agents, orchestration, handoff, child-agent management, and cloud remote-control semantics.
- Voice/transcription, onboarding, Oz/cloud-agent marketing/assets, telemetry/crash reporting/Sentry/event queues.
- Old `/model` and `/profile` selector flows.

Native non-macOS host and package changes were rejected:

- Windows registry, Inno updater, PowerShell host workarounds, WSL/MSYS2 host shell behavior, Windows titlebar/control changes.
- Linux/Wayland/winit/deb/package/Nix/AppImage/platform CI work.
- Platform owner/process metadata and platform-specific docs that do not apply to the local macOS fork.

Remote terminal changes were reduced instead of restoring Warp-hosted services:

- `9c162bca`, `654402fb`, `844dc2ce`, `4714a6149`, and related remote installer/download/SCP fallback changes were rejected because this fork intentionally disables remote server auto-install from Warp infrastructure.
- SSH connection, bootstrap, socket naming, daemon protocol, command execution, and disconnect handling were retained and ported where compatible.
- Remote codebase indexing, remote-backed global buffer, and remote editor sync commits were rejected unless they could be reduced to current retained local code review/editor behavior without cloud indexing.

ACP-related upstream agent features were not accepted as-is:

- Orchestration pills, cloud handoff, child agents, harness selection, ambient/cloud session restore, and cloud-mode settings gates depend on deleted Warp Agent SDK/cloud concepts.
- Global rulefile support in `b5a0d89bd` was inspected but not ported in this pass because the upstream implementation couples new home-directory watcher state, rule UI changes, and cloud/global fact terminology. If needed later, reimplement narrowly against ACP project-context packing without restoring cloud rule semantics.
- `6289aec15` MCP tools/resources capability probing was removed after the MCP ownership cleanup because app-side MCP capability queries are old Warp Agent residue in this fork.
- MCP OAuth token persistence in `edfd4149d` was rejected because this fork no longer has the upstream OAuth module and no backward compatibility shim should be added for it.

## Follow-Up Notes

- Treat this branch as an adapted port, not a merge base with upstream ancestry for these 249 commits.
- For future upstream pulls, start from this file plus `fork-contract.md`; do not use the old 2026-05-10 audit alone because it predates the macOS-only and no-backward-compatibility requirements.
- SSH should remain in scope even when the remote host is Linux or Windows. The removed platform scope is native host/platform/packaging code, not remote terminal interoperability.
