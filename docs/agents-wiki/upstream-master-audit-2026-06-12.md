# Upstream Master Audit 2026-06-12

Range under review: `d3757291a..a30cc7a33`

Previous audited upstream tip: `d3757291a Add basic tab group rendering for horizontal tabs (#12089)`

Current upstream tip detected: `a30cc7a33 Enhance writing logic for ai_queries sqlite db table (#12484)`

Total upstream commits in this incremental range: 97

Status: triage complete. Compatible retained security, terminal, shell, local UI, file-search, and build fixes were ported manually. Deleted cloud agent, Oz/orchestration, app-side MCP/skills, account/subscription, telemetry, native Linux/Windows/Web host, process-doc, and tab-grouping surfaces were rejected or deferred according to the fork contract.

## Ported Or Adapted

- `f3b9ce1c8` Disabled iTerm file download writes while retaining inline image display. The fork now ignores non-inline iTerm file payloads and tests that CWD files are not overwritten.
- `0c1e24329` Stripped leading environment assignments before command denylist matching while keeping allowlist matching on the original command text.
- `b1a41d0b1` Added a local `terminal.osc52_clipboard_access` setting and gated OSC 52 read/write behavior. The settings UI/banner from `164e60e42` was not ported in this batch.
- `43f4f483e`, `b6caa9576` Quoted model-controlled grep, file-glob, `is_file_path`, and `is_git_repository` command arguments with shell-aware single-quote handling.
- `88c344e2d` Escaped remote SSH session history/current-directory command arguments using the same shell-aware helper.
- `c697c8f5f` Escaped the conversation-restoration `cd` path. The non-local-conversation branch is not applicable because the fork lacks the upstream cloud/nonlocal metadata path.
- `4295ec08d` Reworked DisplayChip click actions to carry structured `PromptChipShellCommand` values and render them with active-session shell quoting at execution time.
- `0902e9730` Hardened interactive shell `PATH` capture with sentinels so noisy shell startup output cannot corrupt the captured path.
- `1df6ff130` Fixed Shift+Backspace to emit DEL instead of Ctrl-H.
- `7a58b59c8` Pinned Metal AIR bytecode compilation to `MACOSX_DEPLOYMENT_TARGET`.
- `0446a507a` Adapted `script/macos/run` to resolve Cargo's real target directory for the fork's `WARPLY_*` app naming.
- `e59c7a491` Queued terminal response sequences through the PTY write list when read-side parsing or sync-output finishing emits responses.
- `ae832ff68` Adapted the zsh prompt-width stripping fix for retained Warp prompt rendering.
- `ab0815281` Refreshed active IME cursor position after scene rebuilds when the cursor position changes.
- `cc1ee636f` Updated `tar` from `0.4.45` to `0.4.46`.
- `b34f4ecbd` Let prompt suggestion banners/buttons grow and wrap in narrow panes instead of enforcing a fixed two-line height.
- `723b445a6` Pushed command-palette and `@` file-search queries into repo metadata traversal for query-specific repo contents.
- `262a66961`, `01778efe7`, `e1ae7cb3c` Fixed DirectoryFetcher lifecycle and chip reload behavior: abort in-flight fetches on drop, refresh directory entries through the shared `SessionContext` cache, compare full chip click values, and avoid reset when focus/repo path updates do not change chip values.

## Deferred Retained-Adjacent Work

| Commit | Decision | Reason |
| --- | --- | --- |
| `d2391bad1`, `a44fbf163`, `2d799049a`, `83c11f155`, `08487819f`, `4b5c94d43`, `19018bf4a`, `3ae6f0821`, `e367c9de8`, `ae7f6574a`, `a90be740b`, `d9c4c1a70`, `9c4c656d2`, `26e81f9da`, `16ab97297`, `912e4540f`, `a18da9590`, `0d24d2cff`, `4815c8250`, `65381be1f`, `5bc232d81`, `a30cc7a33` | Defer | These are potentially retained local UI, code-review, AgentView, SSH/remote-server, prompt queue, markdown, tab-pinning, or persistence changes, but each is broad enough to need a focused pass. Some patches include telemetry, settings UI, remote git architecture, or upstream-only tab/agent assumptions that should not be half-ported. |
| `9093f116f` | Defer | NLD rollout behavior is retained-adjacent, but upstream channel/feature-flag semantics should be reviewed separately against this fork's current classifier wiring. |
| `7f0c4dd23` | Not applicable | The fork's notebook link opening path does not contain the upstream `SystemGeneric`/OS-default file opener that was patched. |
| `32d21d15c`, `ca745b402`, `51bd32678` | Defer | DCS integrity/session-id work is security-relevant but spans bootstrap scripts, terminal-model session tracking, remote TTY, session viewers, and removed platform branches. Port only as a complete macOS/SSH security pass. |

## Rejected Or Not Applicable

| Commits | Decision | Reason |
| --- | --- | --- |
| `981cb1c7d`, `b24fce3db`, `e0535ca2c`, `f6d8167f4`, `8fd3d8a75`, `665f0f657`, `984a88962`, `ebaef155b`, `f658c30b5`, `7076885b3`, `4598f4fb4`, `011d9da70` | Reject | Continued the upstream tab-grouping line that this fork has not accepted. |
| `284443c15`, `855aa8993`, `2c38e1fd6`, `d098332da`, `895016351`, `13d7c78f`, `b693bc8e0`, `30a788873`, `b5a6ea9e5`, `168f95ee3`, `3d8235443`, `a93f5c75e` | Reject | Oz, orchestration, cloud/shared-session, harness, ambient-agent, or cloud-agent behavior from deleted product areas. |
| `ff6c2a455`, `362f17a58`, `a30c03cbc`, `163380dc2`, `9a9439ee0`, `2522c8760` | Reject | App-managed skills or MCP behavior. Skills and MCP remain owned by the ACP agent process, not the Warp app. |
| `6d4201ba9`, `1ef622ea4`, `e566a6ced`, `861dacea2`, `c66cff48a`, `f6b28f5e9` | Reject / not applicable | Native Linux, Windows, WSL, or Web/WASM host fixes outside the macOS-only app target. Retained remote-host metadata remains separate. |
| `9157e3e5`, `c2954dcbc`, `a7f668eaa`, `38703bca7`, `41b276b0a`, `31a4f1202`, `5d833febe`, `7056eac00`, `fde172a29` | Reject | Account/subscription/cloud model/profile/Grok/SuperGrok/GraphQL/telemetry product surfaces are removed. |
| `b473fedbf`, `d87691981`, `d9305e13c` | Reject | Upstream automation/spec/process docs are not fork memory. Keep durable fork decisions in `docs/agents-wiki/`. |

## Verification

Commands run after porting:

- `cargo fmt`
- `zsh -n app/assets/bundled/bootstrap/zsh_body.sh`
- `bash -n script/macos/run`
- `cargo check -p warp --all-targets --message-format short`
- `cargo nextest run -p warp_terminal test_shift_backspace_emits_del_sequence`
- `cargo nextest run -p warp_completer test_command_without_leading_env_vars`
- `cargo nextest run -p warp osc52`
- `cargo nextest run -p warp iterm_file`
- `cargo nextest run -p warp handles_inline_iterm_image_payload`
- `cargo nextest run -p warp single_quotes`
- `cargo nextest run -p warp env_prefixed`
- `cargo nextest run -p warp prompt_chip_command`
- `cargo nextest run -p warp path_shell_quoting`
- `cargo nextest run -p warp escapes_single_quote`
- `cargo nextest run -p warp terminal::local_shell::tests`
- `cargo nextest run -p warp no_quotes_returns_input_unchanged`
- `cargo nextest run -p warp create_git_branch`
- `cargo nextest run -p warp allowlist_precedence`
- `cargo nextest run -p warp search::files::model`
- `cargo fmt -- --check`
- `git diff --check`
- `cargo check --workspace --all-targets --message-format short`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
- Added-line deleted-surface scans for cloud/auth/billing/telemetry, MCP/skills, and local Linux/Windows/Web host patterns. All returned no matches.
- Full deleted-surface scans from `AGENTS.md`. Hits were existing weak-handle `upgrade()` false positives, tokenizer vocabulary, retained SSH `ForwardX11`, and existing bootstrap `ConPTY` comments.
