# Remote Server SSH: Initialization

## Summary

Keep the SSH remote-server flow as a retained Warpify terminal feature. When an SSH wrapper session starts, Warp may connect to a remote-server binary that already exists on the remote host and use its protobuf channel for richer remote capabilities such as file tree support, reliable command execution, and session-scoped SSH enhancements.

This fork must not download remote-server artifacts from Warp services, releases CDN, GraphQL, or any hosted product endpoint. The current install script reports that auto-install is unavailable in this build. Future install work must use local packaging or an explicit local deployment path, not Warp cloud infrastructure.

## Problem

The SSH wrapper can work through a ControlMaster-backed SSH channel, but richer remote features need a persistent process on the remote host. The remote-server binary provides that process without making Warp a cloud product client.

The stale upstream version of this spec described downloading an Oz CLI tarball from Warp service endpoints and hiding the new flow behind a rollout flag. That is not valid for this fork.

## Goals

- Preserve SSH and Warpify as full terminal features, including subshell and remote SSH sessions.
- Use an already-deployed remote-server binary when available on Linux or macOS remote hosts.
- Detect remote OS and architecture only for the remote-host setup path.
- Keep initialization local to the SSH connection: binary check, optional local install prompt, proxy launch, and protobuf handshake.
- Surface clear local status and error UI when remote-server setup is unavailable.
- Keep the SSH terminal usable when the remote-server extension is not installed or cannot run.

## Non-goals

- Warp service or CDN downloads for remote-server artifacts.
- Oz CLI naming, old Agent SDK harnesses, cloud agents, or hosted agent setup.
- Feature-flagged compatibility paths that preserve an old product rollout.
- Windows remote-server support in this spec.
- Local Linux or Windows app packaging support.
- Silent installation from an external Warp endpoint.

## User Experience

### Existing Binary

If the expected remote-server binary exists on the remote host and responds to `--version`, Warp launches `remote-server-proxy` over the existing SSH ControlMaster channel and initializes the protobuf session.

### Missing Binary

If the binary is missing, Warp may show the retained Warpify SSH extension choice UI. The user can install through an explicit local deployment path when one exists, or continue the SSH session without the remote-server extension. Continuing without the extension is terminal error handling, not a restored cloud product path.

### Unsupported Remote Host

If the preinstall check classifies the remote host as unsupported, Warp skips remote-server setup and continues the SSH session without showing a misleading install prompt. Remote Linux and macOS host checks remain in scope because SSH remote hosts are part of the terminal product.

### Status

The UI should distinguish these local states:

1. Checking the remote-server binary.
2. Installing or updating only when a local install path is explicitly available.
3. Connecting to `remote-server-proxy`.
4. Connected and ready.
5. Unavailable or unsupported, with local diagnostics.

## Success Criteria

1. SSH sessions remain usable with or without the remote-server extension.
2. Existing remote-server binaries can be launched and initialized over SSH.
3. Missing binaries do not trigger Warp service, CDN, GraphQL, or token-backed downloads.
4. Remote Linux/macOS detection is used only for SSH remote-server setup, not local app packaging.
5. No old Agent SDK, Oz, Cloud Agent, orchestration, handoff, or Warp service client path is introduced.
6. The fork wiki records that SSH remote server is retained but hosted auto-install is not.

## Validation

- SSH into a Linux remote host that already has the remote-server binary deployed and verify the proxy launches and initializes.
- SSH into a macOS remote host that already has the remote-server binary deployed and verify the proxy launches and initializes.
- SSH into a host without the binary and verify Warp does not call Warp service download endpoints.
- Verify the session remains a functional SSH terminal when remote-server setup is unavailable.
- Verify the Warpify settings control only local prompt/install behavior.
