# ClipClop documentation

[简体中文](index.zh-CN.md) | English

The current code and tests are the source of truth for implementation status. Product and design documents describe intent; implementation and release documents describe what has actually been verified.

## Public project entry points

- [README](../README.md): product overview, platform status, privacy model, installation, and development.
- [Contributing guide](../CONTRIBUTING.md): contribution workflow and quality gates.
- [Security policy](../SECURITY.md): private vulnerability-reporting route and supported versions.
- [Code of Conduct](../CODE_OF_CONDUCT.md): community expectations and enforcement.
- [Changelog](../CHANGELOG.md): user-visible changes.

## User and maintainer references

The detailed references below currently use Simplified Chinese; the English README contains the complete public baseline needed to evaluate and build ClipClop.

- [Privacy notice](privacy.md)
- [Build and distribution](distribution.md)
- [Troubleshooting](troubleshooting.md)
- [Testing guide](testing.md)
- [Release checklist](release-checklist.md)
- [Open-source readiness audit](open-source-readiness.md)
- [Architecture](architecture.md)

## Current facts

- Version: `0.1.0` development preview.
- Platforms: local macOS build verified; Windows CI configured, with device validation pending.
- Data: local SQLite/FTS5 storage; no account, cloud sync, or telemetry.
- Tests: Rust and frontend logic tests, static diagnostics, and production builds are in CI; full desktop end-to-end coverage remains pending.
