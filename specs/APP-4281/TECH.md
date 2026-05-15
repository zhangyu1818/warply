# APP-4281: Remote-Server Preinstall Check

Linear: APP-4281

## Fork Contract

Keep SSH remote-server support and Warpify. Remove Warp hosted install assumptions, Oz CLI naming, telemetry, and any account/token service dependency. Remote Linux host detection is retained only because SSH sessions may target Linux hosts; it is not local Linux app support.

## Current Code

- `crates/remote_server/src/preinstall_check.sh` emits structured libc capability data.
- `crates/remote_server/src/setup.rs` parses `PreinstallCheckResult`, `PreinstallStatus`, `UnsupportedReason`, `RemoteLibc`, and `RemotePlatform`.
- `app/src/remote_server/ssh_transport.rs` runs the preinstall script over the existing SSH ControlMaster socket.
- `crates/remote_server/src/manager.rs` includes the preinstall result in `BinaryCheckComplete` and records `RemoteServerSetupState::Unsupported`.
- `app/src/terminal/writeable_pty/remote_server_controller.rs` decides whether to connect, install, show the choice block, or continue SSH without the extension.
- `crates/remote_server/src/install_remote_server.sh` currently reports that remote-server auto-install is unavailable in this build.

## Required Behavior

### Preinstall Script

`preinstall_check.sh` runs before any user-visible install affordance on Linux remote hosts. It emits `key=value` lines:

```text
status=supported|unsupported|unknown
reason=<short identifier when unsupported>
libc_family=glibc|musl|bionic|uclibc|unknown
libc_version=<major.minor when known>
required_glibc=<major.minor>
```

The script is the source of truth for remote libc capability. The Rust side parses the script output and does not duplicate the glibc floor in a separate constant.

### Parse Rules

- `status=supported` becomes `PreinstallStatus::Supported`.
- `status=unsupported` plus `reason=glibc_too_old` becomes `UnsupportedReason::GlibcTooOld`.
- `status=unsupported` plus `reason=non_glibc` becomes `UnsupportedReason::NonGlibc`.
- Missing, malformed, or unknown status becomes `PreinstallStatus::Unknown`.
- Unknown is treated as inconclusive, not as unsupported.

### Controller Gate

When `RemoteServerController::on_binary_check_complete` receives an unsupported result, it should:

1. Mark the setup state as `RemoteServerSetupState::Unsupported`.
2. Skip the choice block.
3. Skip install and proxy launch.
4. Optionally schedule best-effort removal of the expected remote-server binary.
5. Flush the stashed bootstrap so the SSH terminal continues through the retained ControlMaster-backed path.

This is not an old product recovery path. It is the retained terminal path for a host where the remote-server extension cannot run.

### Inconclusive Results

If the script cannot classify the host, the controller should not block the SSH terminal. It may proceed through the normal local `SshExtensionInstallMode` path. Because hosted auto-install is disabled in this fork, this path must not contact Warp service endpoints.

### Local Diagnostics

Unsupported and inconclusive results may be logged locally. Do not add telemetry events, Rudderstack schemas, Sentry reporting, or service-side diagnostics.

## Rules For Future Changes

- Do not reintroduce Oz CLI names.
- Do not call `/download/cli`, release CDN URLs, GraphQL, or token-backed APIs.
- Do not add a cloud setup/onboarding surface for unsupported hosts.
- Do not treat remote Linux detection as permission to restore native Linux app code.
- Do not remove Warpify, ControlMaster bootstrap, or `remote-server-proxy`.
- If local packaging later provides remote-server artifacts, keep the preinstall check as the gate before local deployment.

## Validation

- Unit tests cover `PreinstallCheckResult::parse` for supported glibc, old glibc, non-glibc, unknown, and malformed output.
- Unit tests cover `RemoteServerSetupState::Unsupported` as a terminal setup state.
- Manual SSH validation on supported Linux verifies proxy initialization.
- Manual SSH validation on old/non-glibc Linux verifies no choice block and a usable SSH terminal.
- Manual macOS remote validation verifies the preinstall check is skipped.
- Endpoint scan verifies this spec and implementation do not require hosted remote-server downloads.
