# Upstream Master Audit 2026-08-10

Range under review: `7d93fa4688..upstream/master` (3 commits)

Previous audited upstream tip: `7d93fa4688 [QUALITY-1333] Prevent background TUI agents from stealing focus (#14829)`

Current upstream tip detected: `b076027de Remove exclamation mark from model discount chip copy (#14876)`

Total upstream commits in this incremental range: 3

Status: triage complete. One retained macOS packaging fix was ported (`2655ec7a8`, #14873). The remaining two commits touch removed surfaces only: the model-discount chip lives in the removed `/model` selector (`b076027de`), and the Factory-definition clone lives in the removed `agent_sdk` driver (`4f15a21ba`).

## Per-Commit Triage

### `b076027de` — Remove exclamation mark from model discount chip copy (#14876)

Decision: **not applicable** (removed model-selector / model-discount surface).

The only touched file is `app/src/terminal/input/models/data_source.rs`, which upstream edits inside `impl SearchItem for ModelSearchItem` to change `"{}% off!"` to `"{}% off"` in the `discount_percentage` chip. Verified in the fork: `app/src/terminal/input/models/data_source.rs` does not exist, and `rg "ModelSearchItem|discount_percentage|% off"` under `app/` returns no hits. The chip is part of the removed Warp AI `/model` selector flow (old model catalog / model preference UI). Restoring it would re-introduce the model-picker surface the fork contract removes.

### `2655ec7a8` — Skip the Frameworks rpath add when it is already present (#14873)

Decision: **adapt (accept)** — retained macOS packaging tooling; idempotency fix ported to the two fork call sites.

Upstream unconditionally ran `install_name_tool -add_rpath "@executable_path/../Frameworks"` on the bundled binary, and `install_name_tool` hard-errors when an `LC_RPATH` for that path already exists. cargo-bundle 0.11.0 (this fork already pins `CARGO_BUNDLE_VERSION="0.11.0"` in `script/install_cargo_bundle`) adds this rpath itself whenever it copies at least one declared framework, so a re-run over an already-patched binary breaks the bundle step under `set -e`.

The upstream fix introduces `script/macos/add_framework_rpath`, which reads `LC_RPATH` load commands via `otool -l` and only calls `install_name_tool -add_rpath` when the path is absent, then routes the single-arch, universal, and `./script/run` call sites through it.

Adaptation for this fork:

- **Added `script/macos/add_framework_rpath`** verbatim in logic (same `otool -l` / `awk` `LC_RPATH` tracking, same `grep -Fxq` skip, same missing-binary hard error). The example framework in the log line is `Sparkle` (this fork's retained updater framework) instead of upstream's `Sentry`, matching the fork's Sparkle 2 updater path.
- **`script/macos/bundle`** — the single-arch call site (`install_name_tool -add_rpath ... "$BUNDLE_DIR/$WARPLY_APP_NAME.app/Contents/MacOS/$WARPLY_BIN"`) now routes through `"$WORKSPACE_ROOT_DIR/script/macos/add_framework_rpath"`, reusing the existing `WORKSPACE_ROOT_DIR` variable.
- **`script/macos/run`** — the local `./script/run` call site (`install_name_tool -add_rpath ... "$WARPLY_APP_PATH/Contents/MacOS/$WARPLY_BIN_NAME"`) now routes through `"${REPO_ROOT}/script/macos/add_framework_rpath"`, reusing the existing `REPO_ROOT` variable.
- **Universal-binary call site not ported.** Upstream edits a third call site that adds the rpath to both per-architecture thin binaries before `lipo -create`. This fork's `script/macos/bundle` has no `lipo` / universal-binary path (it builds only the ARM single-arch target: `DEFAULT_TARGET="$ARM_TARGET"`), so that hunk has no anchor here.

Note on trigger: this fork's `[package.metadata.bundle.bin.warply]` in `app/Cargo.toml` declares no `osx_frameworks` (Sparkle.framework is copied in by `script/macos/{bundle,run}` after `cargo bundle`, not declared as a bundle framework). cargo-bundle 0.11.0 therefore does not auto-add the rpath for the `warply` bin today, so the fork is not currently failing on a duplicate rpath. The port is still a retained-macOS-packaging robustness improvement: it makes re-running a bundle step over an already-patched binary idempotent instead of a hard `install_name_tool` error, and it future-proofs the path if `osx_frameworks` is ever declared.

### `4f15a21ba` — Drop the existence guard from the Factory definition checkout clone (#14852)

Decision: **not applicable** (removed `agent_sdk` driver surface).

The commit edits `app/src/ai/agent_sdk/driver.rs`, `app/src/ai/agent_sdk/driver/environment.rs`, and `app/src/ai/agent_sdk/driver/environment_tests.rs` to drop a `[ -e "$WARP_FACTORY_REPO_DIR" ] ||` existence guard from a Factory-definition clone command (`environment::prepend_factory_definition_clone`). Verified in the fork: `app/src/ai/agent_sdk/` does not exist (the entire Old Warp Agent SDK / cloud driver was removed at the fork baseline). `rg "prepend_factory_definition_clone|WARP_FACTORY_REPO|FACTORY_REPO_DIR"` returns no hits. Restoring this would re-introduce the cloud agent SDK / Factory-definition dispatch surface the fork contract removes.

## Verification

- `bash -n` on `script/macos/add_framework_rpath`, `script/macos/bundle`, `script/macos/run` — clean.
- `shellcheck` (0.10.x) on the same three scripts — no new findings introduced by this change; the remaining `SC2016` / `SC2086` / `SC2115` / `SC2129` reports are pre-existing on unmodified lines.
- Deleted-surface scans re-run to confirm no drift:
  - `rg "access token|AuthState|billing|credits|referral|upgrade|Teams|Warp Drive|GraphQL|Sentry|telemetry|crash reporting|agent_sdk|ambient_agents|managed secret|cloud environment"` — no new hits; the `Sentry` reference removed from the ported log line and replaced with `Sparkle`.
  - `rg "mcp.*capab|capab.*mcp|mcp_server|mcpServers|bundled skills|channel-gated-skills|ReadSkill|InvokeSkill"` — no hits.
  - `rg "target_os = \"linux\"|target_os = \"windows\"|cfg\(windows\)|WSL|MSYS2|ConPTY|Wayland|X11|winreg|x11rb"` — only the retained `ConPTY` explanatory comment in `zsh_body.sh` and retained SSH `ForwardX11=no` config strings (all allowed).

## Notes

- No Rust code changed in this cycle, so the workspace build state carries over from the 2026-08-08 merge. `cargo check` / `cargo fmt --check` were re-run as a sanity pass.
- No deferred ports this cycle. The `da4da09f8` Agent Mode Cmd-Up/Cmd-Down prompt navigation deferred from the 2026-08-08 audit remains deferred pending either a follow-up retained change on those navigation-cursor types or resolution of the keymap-context `Terminal` gate.
