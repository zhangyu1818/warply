# Agents Wiki

This wiki is the project-level memory for the local ACP fork. Its purpose is to make future upstream merges from original Warp possible without accidentally restoring removed cloud product code.

The wiki is organized for quick agent lookup:

- What this fork changed.
- What this fork deleted.
- Which upstream changes should be accepted, adapted, or rejected.
- Which legacy names are still present but do not mean the old product surface should return.

## Baseline

```text
19659d12 refactor: create local ACP-only Warp fork
```

Treat that commit as the fork baseline. Future merge work should compare upstream changes against the code after this commit.

## Product Shape

The fork keeps:

- Warp terminal GUI.
- Local terminal sessions and retained remote terminal support.
- ACP agent conversations displayed through Warp AgentView.
- OpenAI-compatible Next Command and Prompt Suggestions.
- Local settings and local persistence.

The fork removes:

- Warp account login and access tokens.
- Billing, usage credits, referrals, Teams, cloud workspace discovery.
- Warp Drive cloud sync/sharing UI.
- Warp-hosted Agent SDK/cloud/ambient/scheduled agents.
- Cloud GraphQL APIs, managed secrets, hosted isolation.
- Telemetry, crash reporting, Sentry release upload.
- Onboarding, marketing surfaces, voice input/transcription.

## Wiki Files

- `fork-contract.md`: Detailed product and architecture contract.
- `upstream-merge-guide.md`: Decision process for pulling from original Warp.
- `change-map.md`: Path-level map of added/replaced/removed/retained code.

## Quick Merge Principle

When an upstream commit improves generic terminal behavior, port it.

When an upstream commit improves AI UI or local data structures, adapt it to ACP and local suggestions.

When an upstream commit restores cloud product behavior, reject it or reduce it to a local utility.

Do not resolve conflicts by bringing back deleted modules just because upstream still depends on them.
