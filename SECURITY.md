# Security Policy

[简体中文](SECURITY.zh-CN.md) | English

## Supported versions

ClipClop has not reached a stable release. Security reports are accepted for the latest code on the default branch. `0.1.x` builds are development previews and do not receive a long-term support commitment.

## Report a vulnerability

Do not disclose an unpatched vulnerability in a public issue, pull request, or discussion. This is especially important for issues that expose clipboard contents, bypass IPC or path validation, execute arbitrary programs, or leak local data.

The intended private channel is [GitHub Private Vulnerability Reporting](https://github.com/hiQianFan/ClipClop/security/advisories/new). Repository maintainers must enable and test this channel before switching the repository to Public. If the private reporting form is unavailable, do not publish vulnerability details; contact the repository owner without technical details to arrange a private channel.

Include the affected version or commit, platform, reproduction steps, impact, proof of concept, and suggested mitigation when possible. Use synthetic data and never attach real clipboard contents, credentials, tokens, or personal files.

## Response targets

Once the private channel is active, maintainers aim to acknowledge a report within seven days and provide a remediation plan after triage. Resolution time depends on severity and platform-validation requirements. Keep the report private until a fix is released or disclosure is coordinated.

## Security boundaries

ClipClop stores potentially sensitive clipboard data locally and does not currently encrypt its database at the application layer. Its WebView must not load remote pages. New permissions, network access, shell execution, arbitrary file access, or telemetry are security and privacy boundary changes and require dedicated review and documentation updates.
