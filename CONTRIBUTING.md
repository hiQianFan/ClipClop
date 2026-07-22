# Contributing to ClipClop

[简体中文](CONTRIBUTING.zh-CN.md) | English

Thank you for helping improve ClipClop. The project is a `0.1.0` development preview. Before proposing a larger change, check the v1 boundaries in [PRODUCT.md](PRODUCT.md) and [DESIGN.md](DESIGN.md).

## Before you start

1. Bug fixes, documentation corrections, and small reversible changes can go directly to a pull request.
2. Open an issue first for new features, data migrations, dependency upgrades, or changes to permissions or privacy boundaries. Describe the motivation, user impact, and alternatives.
3. Do not disclose security vulnerabilities publicly. Follow the [security policy](SECURITY.md).
4. Never include real clipboard contents, tokens, full private URLs, or personal file paths in an issue, test fixture, screenshot, log, or pull request.

## Local development

You need Node.js from `.nvmrc`, pnpm `9.15.3`, Rust stable, and the Tauri prerequisites for your platform.

```bash
nvm use
corepack enable
pnpm install --frozen-lockfile
pnpm tauri dev
```

Before submitting a pull request, run:

```bash
pnpm test
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Changes involving keyboard behavior, focus, the system clipboard, permissions, or window lifecycle also require manual verification. Record the platform and non-sensitive results in the pull request; see [docs/testing.md](docs/testing.md).

## Change conventions

- Keep Rust business rules in their feature modules and Tauri commands thin. Svelte code should call IPC through feature `api.ts` modules.
- Do not modify a released migration. Add an ordered migration and document upgrade and rollback verification.
- Update relevant documentation in the same pull request when user behavior, permissions, data handling, or release behavior changes.
- Keep changes focused. Suggested commit format: `type(scope): summary`, using types such as `feat`, `fix`, `docs`, `test`, `refactor`, `build`, or `ci`.

## Pull requests

Describe the problem and solution, affected platforms, verification evidence, privacy/permission impact, UI screenshots when applicable, and unresolved risks. Use synthetic data in all evidence.
