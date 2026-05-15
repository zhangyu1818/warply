# APP-4281: SSH Into Remote Hosts With Unsupported glibc

Linear: APP-4281

## Summary

SSH remote-server support is retained terminal functionality. When a user SSHes into a Linux remote host whose libc cannot run the fork's remote-server binary, Warp should avoid showing an install prompt, avoid launching the proxy, and continue the SSH session through the retained ControlMaster-backed terminal path.

This is a remote-host capability gate only. It must not restore Warp hosted downloads, Oz CLI naming, telemetry, account auth, or local Linux app packaging.

## Problem

The remote-server binary can fail to launch on old or non-glibc Linux hosts. Examples include RHEL/CentOS 7, Amazon Linux 2, Ubuntu 18.04, Debian 10, Alpine/musl, and Termux/bionic. If Warp offers installation or tries to launch the proxy on those hosts, the user sees a setup failure even though the SSH terminal itself is still usable.

The correct fork behavior is to treat unsupported remote-server setup as a normal SSH terminal outcome, not as a cloud product error.

## Behavior

1. Warp runs a remote-side preinstall check only for Linux remote hosts.
2. If the check positively identifies an unsupported libc, Warp does not show the SSH extension install choice block.
3. Warp does not invoke `install_remote_server.sh`, does not fetch any hosted artifact, and does not launch `remote-server-proxy` for that unsupported host.
4. The SSH session continues through the retained ControlMaster-backed terminal path. Remote-server-specific features are absent for that session, but the terminal remains usable.
5. If an incompatible remote-server binary is already present at the expected path, Warp may remove it on a best-effort basis so future sessions do not retry the same unusable binary.
6. macOS remote hosts are unaffected and do not run the libc probe.
7. Linux hosts with supported glibc continue through the normal binary check and proxy connect flow when a valid binary exists.
8. If the preinstall check is inconclusive, Warp should not block the SSH session. It follows the normal local SSH-extension setting path for that host.
9. Detection state is per SSH host/session, not a global app state.
10. Unsupported-host diagnostics stay local. Do not add telemetry or service reporting.

## Non-Goals

- Hosted remote-server downloads.
- Oz CLI artifact naming.
- Warp account, token, GraphQL, or release CDN dependencies.
- User-visible cloud setup or onboarding flows.
- Local Linux or Windows app support.
- Silent compatibility shims for old Warp product rollout paths.

## Validation

- Unsupported Linux host: no install choice block, no proxy launch, SSH terminal remains usable.
- Supported Linux host with a valid remote-server binary: proxy launches and initializes.
- macOS remote host: existing SSH remote-server behavior is unchanged.
- Inconclusive probe: SSH terminal remains usable and no hosted endpoint is contacted.
- Existing incompatible binary: removal is best-effort and failure does not block SSH.
