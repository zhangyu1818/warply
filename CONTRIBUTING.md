# Contributing to Warply

Warply is an independent macOS fork of Warp. Please open issues and pull requests in [zhangyu1818/warply](https://github.com/zhangyu1818/warply).

## Report a problem

Search existing issues first. Include the Warply version, macOS version, CPU architecture, reproduction steps, and expected versus actual behavior. Remove credentials and private information from logs and screenshots.

For feature requests, describe the problem and the desired user experience. Discuss substantial changes in an issue before implementing them.

## Development

Follow the [build instructions](README.md#build-from-source), then read [AGENTS.md](AGENTS.md) and the [fork contract](docs/agents-wiki/fork-contract.md). Keep changes focused and preserve the macOS-only terminal, ACP agent boundary, and OpenAI-compatible terminal suggestions.

Fork architecture and upstream merge decisions live in [docs/agents-wiki/](docs/agents-wiki/README.md). Upstream ports must start from the actual upstream implementation and follow the [merge guide](docs/agents-wiki/upstream-merge-guide.md).

## Validation

Run focused checks for the affected behavior. The current CI checks are:

```sh
cargo fmt -- --check
cargo check -p warp --all-targets --locked --message-format short
cargo check --workspace --all-targets --locked --message-format short
cargo nextest run -p warp -E 'test(slash_command) | test(acp) | test(terminal_suggestions)'
```

Add regression coverage for behavior changes where appropriate. Follow the additional build and cleanup requirements in [AGENTS.md](AGENTS.md) for upstream ports and workspace migrations. `./script/presubmit` runs the broader formatting, lint, and test suite.

## Pull requests

Target `main`. Explain the problem, the resulting behavior, and the checks you ran, including any limitations. Keep documentation consistent with the final implementation and preserve applicable licenses and copyright notices.

Be respectful and constructive in issues and reviews.
