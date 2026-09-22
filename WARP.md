# Warply engineering guide

This repository is Warply, an independent macOS terminal fork of Warp. Start with [AGENTS.md](AGENTS.md) for the project contract and required checks. Architecture and upstream merge records live in [docs/agents-wiki/](docs/agents-wiki/README.md).

## Development

See [README.md](README.md#build-from-source) for setup and [CONTRIBUTING.md](CONTRIBUTING.md) for validation and pull requests. Use `./script/run` to build and launch the app. The binary is `warply`; its Cargo package is `warp`.

## Code conventions

- Prefer imports over long path qualifiers and avoid unnecessary type annotations.
- Name context parameters `ctx` and place them last, except when a trailing closure needs the final position.
- Remove unused parameters and update callers instead of prefixing them with `_`.
- Prefer inline format arguments and exhaustive matches.
- Do not add code comments unless explicitly requested.
- Create mouse-state handles during view construction and reuse them during rendering.
- Avoid acquiring a terminal model lock when a caller already holds it. Pass the locked model through the call chain and keep lock scopes short.

ACP execution belongs in `app/src/ai/acp/`. AgentView and terminal UI are host layers around that boundary. Keep terminal suggestions on their separate OpenAI-compatible provider path.
