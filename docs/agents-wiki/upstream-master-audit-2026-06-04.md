# Upstream Master Audit 2026-06-04

Range under review: `fc110333a..3497d1844`

Previous audited upstream tip: `fc110333a Tab group feature flag and entry points (#11486)`

Current upstream tip detected: `3497d1844 Stop watching gitignored directories in the repo file watcher (#12122)`

Total upstream commits in this incremental range: 166

Local sync status: local `master` was fast-forwarded from `be5b39ae7` to `3497d1844`; `master...upstream/master` is `0 0`.

Status: triage complete. No product code was ported in this pass. The commits below are the next merge candidates or explicit rejects for the ACP-only macOS fork.

## Port Candidates

These commits appear compatible with retained local terminal, editor, macOS, settings, logging, repo metadata, or code-review behavior. They should still be ported manually and tested against the fork before landing.

- `43e3f58cf` Force zsh bracketed-paste bootstrap behavior.
- `bc4ccd180` Carry ancestor gitignore status in repo metadata; port retained repo metadata parts only.
- `c7272dc44` Add in-session `warp.log` size rotation; omit onboarding-only callers.
- `2dde6964c` Fix working-directories cleanup.
- `bef008ca1` Improve code-review file-invalidation logs.
- `7940652d7` Bump command signatures for completions.
- `f60116d3e`, `5becb10b8`, `94d29fe20`, `6889a1d50`, `0e9232430` Migrate retained macOS/GPUI/computer-use code to `objc2`; omit deleted crash-reporting code.
- `c4946001f` Let tab-config focused picker own Space.
- `2330f8e70` Enable markdown table rendering by default for retained markdown surfaces.
- `2196ce9b3` Clarify create-file palette item.
- `ce73fe07b` Add configurable code-editor line numbers; reject bundled upstream specs.
- `a1a9315a6` Fix `script/macos/run --open_with_launchd` tail path.
- `e1be4179c` Support local `warp://action/open_file_editor` URIs.
- `e59e4aa18` Fix empty zsh `RPROMPT` handling.
- `294dbddcc` Harden settings-file migration tests.
- `186c8d1d2` Add rendered-markdown maximize-pane overflow action.
- `2e5ff6f42` Improve retained lightbox/image UX.
- `5767910b5` Add `script/format` if it can stay fork-local and not restore upstream process surfaces.
- `74fad4790` Bump completions for the dotnet spec.
- `d6c0c69b3` Prefer stateful mouse reporting command palette entry.
- `b8e86f34f` Add error log frequency mode.
- `654940eda` Fix `WeakModelHandle::upgrade` zombie handles.
- `18e85d520` Make `StandardizedPath::strip_prefix` component-aware.
- `6ab1c167c` Sync custom theme selections portably.
- `9f459842c` Fix symlinked gitignored paths in code review.
- `5c57b3850` Avoid repeated SVG rasterization.
- `388f5dc12` Fix terminal flat storage `RowIterator` underflow.
- `89c2193e5` Fix secret redaction after multibyte UTF-8 prefixes.
- `5bee7a759` Fix retained code-review "cannot detect diffs" path.
- `0f97ef18a` Allow partial repo metadata builds for large repos.
- `56e8617c3` Bump Mermaid renderer dependency.
- `3497d1844` Stop watching gitignored directories in the repo file watcher.

## Adapt Candidates

These touch retained UI or local data concepts but are entangled with upstream cloud Agent, app-side skills/MCP, feature flags, or telemetry. Port only fork-compatible behavior.

- `69ffea411` Fix raw image paste into CLI-agent terminal sessions; inspect against retained CLI-agent terminal integration.
- `ade38b082` Treat create/delete of the same path as file replacement in diff application.
- `bbdc5a2ea` Unify PR chip/button state for retained local code review and context chips.
- `530ca5229` Persist conversation IDs for local tasks; adapt only to ACP local conversation restore, not Agent SDK/shared-session plumbing.
- `98af7b654`, `eadc05e6e`, `fb8d00b07`, `c6b842fe7`, `1aa03f9c8`, `86a602b99`, `0aee45df2` Queue prompt UI improvements; strip cloud mode, telemetry, shared-session, and old Agent SDK dependencies.
- `00fa7aad1` Extract core search infra; keep only retained search/menu/editor pieces and omit telemetry/cloud preferences.
- `64862fe33` Fix grouped-image rendering crash in AI blocks; adapt to ACP AgentView state.
- `ba5dcd90e` Generic dropdown type cleanup; port helper changes only where retained callers exist.
- `82ec31fd5` Fix optimistic-root conversation restore; adapt to local ACP history/persistence.
- `040a7819f` Command palette entries for settings toggles; keep only retained local settings pages.
- `39dd121f3` Reorder tools panel; keep retained project explorer/conversation list behavior only.
- `a9daac2bf`, `f2cc205f3`, `edf83549f` AI-block find/code-diff focus fixes; adapt to ACP AgentView.
- `90f7a4c81` Add PR/repository info to input context; adapt to retained local git/context-chip paths.
- `89f61b63b` Limit apply-diff results to changed ranges; remove upstream feature-flag plumbing.
- `d86ad797c` Fix ask-question dropdown overflow; adapt to retained ACP permission/question UI.
- `14c8c8ded` Host-scoped requests; review carefully for retained SSH/remote-server host scoping, reject app-side skills and remote codebase-indexing dependencies.
- `a35bf3a47` Tighten dependencies; regenerate from retained manifests and keep cloud/API/reporting/platform removals.

## Rejected Or Not Applicable

Default reject categories in this range:

- Cloud Agent, orchestration, ambient agents, handoff/cloud run, cloud mode, shared sessions, and viewer streaming.
- Auth, billing, Teams, credits, custom model/BYOK UI, Warp server APIs, GraphQL, cloud object crate restructuring, IAP, and managed secrets.
- App-side MCP and skills, bundled skills, remote project skills, skill watchers, skill output rendering, MCP logs/settings/install UI, and Codex plugin packaging.
- Sentry/crash reporting, telemetry-only commits, Slack changelog handoff, upstream process docs/specs, and release-process docs that are not fork memory.
- Native Windows/Linux/Web/WASM implementation or packaging changes, except retained macOS-to-remote SSH behavior after inspection.
- Tab grouping feature work remains rejected for now because the previous upstream entrypoint/feature-flag commit was rejected and this fork has not adopted tab groups as a retained product surface.

Specific rejects or no-ops include:

- `f2d23a151`, `557897d20`, `856c74b04`, `fe0aee14c` telemetry, Sentry, Slack, GraphQL, or observability surfaces.
- `f365f9672`, `6984bc390`, `f6e6f78a9`, `836b73c88`, `a79721a5c`, `e9decdbb3`, `74d256646`, `debe6d810`, `92069590d`, `b48ece2e1`, `ac4225c18`, `483318700`, `63fe72858` app-managed skill, MCP, or plugin surfaces.
- `2fe5cd414`, `c99b9546b`, `edef7f83f`, `c37c1cd6e`, `17183d99b`, `37df9ef20`, `3092698d6`, `e41bf4f74`, `f0e3128b4`, `a4d19abdc`, `abc70eba1`, `3f8cbb782`, `f6bf91cea`, `f240e8042`, `62b40f40b`, `42cb22bb9`, `385b2a90e`, `06ba1bb36`, `45b0c6740`, `52a708bdf`, `6fe675601`, `b96a60457`, `d9b50a20b`, `944e5b4be`, `c74d16e37`, `3298ddcf6`, `67569a760`, `c425ef1d0`, `a572a2d9d`, `27ff15b50`, `d14ab25d3`, `9727a08a7`, `b3be049d7` cloud Agent/orchestration/cloud-mode/shared-session surfaces.
- `e6d8aee3c`, `d37e7a8cc`, `a44b70306`, `eec8d4c1a`, `8901a2fc5`, `fc2dfe971`, `086150b87`, `1a3bdee4a`, `2249469e5`, `42e583a97`, `1bdae67c3`, `c85bf84f8`, `8f8ff4a86`, `1b6642f2c` auth, billing, Teams, API-key, custom model, BYOK, IAP, or usage-credit surfaces.
- `37f104a1c`, `38f7e2893`, `15f8435a0`, `a6d9b93ae`, `feff4718e`, `7f72eaf8c`, `af64c3107`, `978412316`, `3cbaef9d5` Warp server/client/auth/cloud-object restructuring.
- `ebedb9fdc`, `2992d02e3`, `af886f7ce`, `808a54d8d`, `4ca690bee`, `21334d424`, `876b840c7`, `8d5f3b318`, `463df3629` Windows/Web/WASM/Nix/dev-container/voice-input changes not needed for retained macOS app behavior.
- `f3bfb750b`, `4f5d0d6f8`, `98dbf7831`, `910d0fc46`, `9e23bd22f`, `662bd7376`, `2e5a3e6e4` tab-grouping feature work.
- `a7d65440f`, `51ce7a4eb`, `6d15163ea`, `6b848990e`, upstream `specs/**` additions, and upstream `WARP.md`/`SECURITY.md` changes as non-fork process/spec docs.
- `175faadce`, `2566f54af`, `a3d10ce67`, `3dc094132`, `c4b082909`, `9de6d4dc6`, `81d317424`, `405c83cba`, `6616a42eb` rollout, eval, test-layout, deleted-helper, or feature-flag commits with no direct retained behavior to port.
- `0632bd1b4`, `6307c9867`, `df02914a6` remote codebase-indexing or remote diff-state architecture previously rejected for this fork.
- `530ca5229`, `82ec31fd5`, and other AgentView/history commits must not be merged wholesale even when useful local restore behavior exists, because they also touch removed Agent SDK or shared-session state.

## Verification

Commands run during this triage:

- `git fetch upstream master --prune`
- `git fetch upstream master:master`
- `git rev-list --left-right --count master...upstream/master`
- `git rev-list --count fc110333a..master`
- `git log --oneline --reverse fc110333a..master`
- Path-specific `git log` and `git show --stat --name-only` inspections for terminal, repo metadata, code review, macOS UI, editor, settings, and AgentView candidates.
- `cargo fmt -- --check`
- `cargo check -p warp --all-targets --message-format short`
- `cargo check --workspace --all-targets --message-format short`
- `cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'`
