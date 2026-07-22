# ClipClop

[简体中文](README.zh-CN.md) | English

**A lightweight, offline-first, cross-platform clipboard history tool.**

ClipClop is built with Tauri 2, Rust, Svelte 5, TypeScript, and Vite.

> Status: `0.1.0` development preview. Local macOS builds have been verified. The Windows build workflow is configured, but Windows device testing is still pending.

## Features

- Captures plain text and its existing HTML/RTF flavors, images, and file references. The interface renders only safe plain text.
- Press Enter to paste with available formatting, or Shift+Enter to paste plain text. If direct paste fails, the content remains on the system clipboard for manual paste.
- Open ClipClop with `⌃⌘C` on macOS or `Ctrl+Alt+C` on Windows. The shortcut can be changed under Settings → Shortcuts.
- Runs in the tray and supports Light/Dark themes, retention settings, and launch at login. Quitting ClipClop stops capture.
- Checks GitHub Releases for Tauri-updater-signed updates. Automatic checks run at most once per day; download and installation require confirmation.
- No account, cloud sync, telemetry, advertising, or network enrichment of copied links.

On macOS, direct paste requests Accessibility/Post Event permission. If permission is denied, ClipClop still copies the selected item for manual paste. On Windows, a normal process cannot inject input into an elevated application and uses the same fallback.

## Install

Preview installers are published from [GitHub Releases](https://github.com/hiQianFan/ClipClop/releases) when available: a Universal DMG for macOS and an x64 setup EXE for Windows.

The preview installers are not currently signed with Apple Developer ID or Windows Authenticode. macOS may require you to allow the app in System Settings → Privacy & Security, and Windows may show an Unknown Publisher or SmartScreen warning. Tauri updater signatures verify update integrity, but they do not replace operating-system publisher signing. If you do not want to bypass an operating-system warning, build ClipClop from source or wait for a signed release.

## Privacy

ClipClop stores clipboard contents, source-application metadata, file-path references, and settings on the current device. It does not upload this data, and copied URLs do not trigger network requests. ClipClop does not try to identify or filter sensitive content; use item deletion, Clear History, retention settings, or quit the app to control capture.

The local database is not encrypted by ClipClop and relies on operating-system account permissions and disk encryption such as FileVault or BitLocker. See the [privacy notice (Simplified Chinese)](docs/privacy.md) for additional details.

## Platform status

| Platform | Package | Status |
| --- | --- | --- |
| macOS (Apple Silicon and Intel) | Universal DMG | Local build verified; device smoke testing remains pending |
| Windows x64 | NSIS setup EXE | CI configured; device smoke testing remains pending |

## Develop from source

Requirements:

- Node.js version from `.nvmrc`
- pnpm `9.15.3`
- Rust stable via rustup
- Tauri's platform prerequisites: Xcode Command Line Tools on macOS, or Microsoft C++ Build Tools and WebView2 on Windows

```bash
nvm use
corepack enable
pnpm install --frozen-lockfile
pnpm tauri dev
```

Run all local quality checks before submitting a change:

```bash
pnpm test
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Vitest covers frontend logic such as update throttling, list-state behavior, shortcut formatting, and paste fallback. Complete desktop interaction still requires macOS and Windows smoke testing.

## Contributing and support

- [Documentation](docs/index.md)
- [Contributing guide](CONTRIBUTING.md)
- [Security policy](SECURITY.md) — do not disclose vulnerabilities in public issues
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Changelog (Simplified Chinese)](CHANGELOG.md)

ClipClop is available under the [MIT License](LICENSE).
