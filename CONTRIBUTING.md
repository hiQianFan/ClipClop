# Contributing to ClipClop

[简体中文](CONTRIBUTING.zh-CN.md) | English

Thank you for helping improve ClipClop. This is the single starting point for setting up the project, proposing changes, and submitting a pull request.

## Before you start

1. Bug fixes, documentation corrections, and small reversible changes can go directly to a pull request.
2. Open an issue first for new features, data migrations, dependency upgrades, or changes to permissions or privacy boundaries. Describe the motivation, user impact, and alternatives.
3. Never include real clipboard contents, tokens, full private URLs, or personal file paths in an issue, test fixture, screenshot, log, or pull request.

## Local development

You need Node.js from `.nvmrc`, pnpm `9.15.3`, Rust stable, and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform: Xcode Command Line Tools on macOS, or Microsoft C++ Build Tools and WebView2 on Windows.

```bash
nvm use
corepack enable
pnpm install --frozen-lockfile
pnpm tauri dev
```

The Svelte interface lives in `src/`; the Rust/Tauri application lives in `src-tauri/`.

Before submitting a pull request, run:

```bash
pnpm test
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Changes involving keyboard behavior, focus, the system clipboard, permissions, or window lifecycle also require manual verification. Record the platform and non-sensitive results in the pull request.

## Change conventions

- Keep Rust business rules in their feature modules and Tauri commands thin. Svelte code should call IPC through feature `api.ts` modules.
- Until a migration system is introduced, database schema changes must increment `SCHEMA_VERSION`; development databases with an older version must be reset.
- Update relevant documentation in the same pull request when user behavior, permissions, data handling, or release behavior changes.
- Keep changes focused. Suggested commit format: `type(scope): summary`, using types such as `feat`, `fix`, `docs`, `test`, `refactor`, `build`, or `ci`.

## Branches and versions

- `main` is the only long-lived branch and should always remain releasable.
- Make changes on a short-lived feature or fix branch, merge through a pull request, and delete the branch after merging. Release branches are not used.
- Versions follow Semantic Versioning. A version exists publicly when its `vX.Y.Z` tag and GitHub Release are published; branches do not represent versions.
- Dependency updates are reviewed deliberately. GitHub vulnerability alerts remain enabled, while scheduled dependency-update pull requests are not used.

## Pull requests

Describe the problem and solution, affected platforms, verification evidence, privacy/permission impact, UI screenshots when applicable, and unresolved risks. Use synthetic data in all evidence.

## Security reports

Do not disclose an unpatched vulnerability in a public issue or pull request. Use [GitHub Private Vulnerability Reporting](https://github.com/hiQianFan/ClipClop/security/advisories/new) and include the affected version, platform, reproduction steps, and impact using synthetic data only.

## Maintainer release

1. Update the matching version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. After the required `main` checks pass, manually run the **Release** workflow from `main` with that version. A version change or tag alone does not start a release.
3. The workflow creates a Draft Release and verifies the macOS DMG, Windows installer, updater signatures, and `latest.json`; it never publishes automatically.
4. Install the packages on both platforms when possible and review the generated notes. Publish the Draft as a normal Release only when verification is complete; publishing creates the canonical `vX.Y.Z` tag and makes the update visible to clients.
