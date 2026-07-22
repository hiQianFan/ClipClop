---
title: 'Prepare ClipClop for a public GitHub source release'
type: 'chore'
created: '2026-07-22'
status: 'done'
baseline_commit: 'ea6702dda8ccaf518f206b84733fbd567982cf54'
context:
  - '{project-root}/docs/open-source-readiness.md'
  - '{project-root}/docs/release-checklist.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** ClipClop is ready to merge at the code-quality level, but its public documentation is Chinese-only, several readiness statements are stale, GitHub contribution automation is missing, and the release workflow can create unsigned preview artifacts without making the trust boundary sufficiently explicit.

**Approach:** Establish a bilingual public repository baseline around the fixed brand `ClipClop` and the slogans “A lightweight, offline-first, cross-platform clipboard history tool.” / “轻量、离线优先的跨平台剪贴板历史工具。”; add standard GitHub contribution automation; and harden the draft-release path while keeping Apple/Windows platform signing optional for the preview.

## Boundaries & Constraints

**Always:** Keep `README.md` as the English default and provide complete Simplified Chinese counterparts with reciprocal language links. Explain platform-signing limitations once in the README installation section and corresponding Release notes so users can interpret operating-system warnings; do not turn the limitation into headline marketing or repeat it across unrelated pages. Preserve MIT licensing, local-first privacy claims, current platform limitations, `releaseDraft: true`, and exact product naming. Never place clipboard data, personal paths, credentials, updater private-key material, or passwords in tracked files.

**Ask First:** Obtain human approval before changing repository visibility to Public, committing or pushing, publishing a Release, choosing a private security contact, adding the updater-key password secret, or enabling a paid/identity-verified platform-signing service.

**Never:** Do not publish the local updater private key, fabricate Windows/macOS test evidence, claim platform code signing or notarization exists, automatically publish a final Release, translate internal historical artifacts solely for completeness, or modify application behavior in this documentation/governance change.

</frozen-after-approval>

## Code Map

- `README.md` and `README.zh-CN.md` -- bilingual public entry, brand, preview status, installation warnings, privacy, development, and community links.
- `CONTRIBUTING.md` and `CONTRIBUTING.zh-CN.md` -- bilingual contributor workflow with the real `pnpm test` quality gate.
- `SECURITY.md` and `SECURITY.zh-CN.md` -- bilingual vulnerability policy using GitHub Private Vulnerability Reporting as the planned private channel.
- `CODE_OF_CONDUCT.md` and `CODE_OF_CONDUCT.zh-CN.md` -- bilingual community expectations and private-reporting routing.
- `.github/ISSUE_TEMPLATE/*`, `.github/pull_request_template.md` -- structured bug, feature, and PR intake with privacy/security prompts.
- `.github/dependabot.yml` -- scheduled npm, Cargo, and Actions dependency update coverage.
- `.github/workflows/bundle.yml` -- draft-only unsigned preview release with version/quality checks and explicit updater-secret expectations.
- `package.json`, `src-tauri/Cargo.toml` -- public repository metadata and consistent English package description.
- `docs/open-source-readiness.md`, `docs/release-checklist.md`, `docs/index.md` -- current readiness truth and bilingual navigation.

## Tasks & Acceptance

**Execution:**
- [x] Public docs -- create English defaults and complete Chinese counterparts; synchronize brand, slogan, preview warnings, links, commands, security route, and actual test state.
- [x] GitHub community files -- add issue/PR templates and Dependabot configuration without collecting sensitive clipboard examples.
- [x] Release workflow -- keep Releases as drafts, make version validation apply to tag and manual runs, run required quality gates before packaging, and document that updater signing does not equal platform signing.
- [x] Package metadata/readiness docs -- add repository fields, remove stale “no frontend tests” claims, and distinguish source-public, unsigned-preview, and signed-stable completion criteria.
- [x] Repository settings -- retain the private staging repository and configured `production-release` updater-key secret; report the unresolved password/security-contact/Public-switch items without guessing.

**Acceptance Criteria:**
- Given an English- or Chinese-speaking visitor, when they open the repository, then they can reach a complete README in their language and see the same product name, slogan, privacy model, supported platforms, development commands, community links, and a concise installation note explaining any operating-system warning.
- Given an external contributor, when they open an issue or PR, then the template requests reproducible non-sensitive evidence and routes vulnerabilities away from public issues.
- Given a `vX.Y.Z` tag or manual release run, when the workflow packages ClipClop, then versions and quality gates are checked and only a Draft Release is created.
- Given the repository files and GitHub secret listing, when reviewed, then public updater material is tracked while private-key contents and passwords are absent.

## Spec Change Log

## Design Notes

The public source release and binary trust level are deliberately independent. `0.1.0` may remain an unsigned preview. Keep the distinction concise and contextual: installation docs and Release notes explain that updater artifacts are integrity-signed with the existing Tauri key while macOS and Windows installers are not yet publisher-signed; general product messaging stays focused on ClipClop's value.

## Verification

**Commands:**
- `pnpm test && pnpm check && pnpm build` -- all frontend tests, diagnostics, and production build pass.
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml` -- Rust quality gates pass.
- `git diff --check` -- no whitespace errors.
- Markdown link scan and YAML parse -- all relative links resolve and GitHub configuration is syntactically valid.
- `gh secret list --env production-release --repo hiQianFan/ClipClop` -- updater private-key secret exists without revealing its value.

## Suggested Review Order

**Public entry and bilingual truth**

- Start with the English product promise, status, and complete public navigation.
  [`README.md:1`](../../README.md#L1)

- Compare the complete Simplified Chinese counterpart and reciprocal language link.
  [`README.zh-CN.md:1`](../../README.zh-CN.md#L1)

- Verify the single contextual installer-warning location in each README.
  [`README.md:22`](../../README.md#L22)

**Release safety boundary**

- Review trigger restrictions, stable-version validation, and duplicated quality gates.
  [`bundle.yml:55`](../../.github/workflows/bundle.yml#L55)

- Confirm published assets cannot be replaced and Releases remain drafts.
  [`bundle.yml:91`](../../.github/workflows/bundle.yml#L91)

- Check source, Preview, and stable-release requirements remain explicitly distinct.
  [`release-checklist.md:1`](../../docs/release-checklist.md#L1)

**Security and contribution intake**

- Verify vulnerabilities route privately without inventing a maintainer contact.
  [`SECURITY.md:9`](../../SECURITY.md#L9)

- Inspect synthetic-data requirements and platform-specific bug intake.
  [`bug_report.yml:6`](../../.github/ISSUE_TEMPLATE/bug_report.yml#L6)

- Confirm feature proposals must declare privacy, permission, and updater impacts.
  [`feature_request.yml:38`](../../.github/ISSUE_TEMPLATE/feature_request.yml#L38)

- Review PR attestations for verification and boundary-impact documentation.
  [`pull_request_template.md:12`](../../.github/pull_request_template.md#L12)

**Repository metadata and maintenance**

- Check repository URLs, English description, and issue metadata.
  [`package.json:1`](../../package.json#L1)

- Confirm Cargo license and repository metadata match the MIT project.
  [`Cargo.toml:1`](../../src-tauri/Cargo.toml#L1)

- Review npm, Cargo, and Actions update cadence.
  [`dependabot.yml:1`](../../.github/dependabot.yml#L1)

- Finish with the remaining human-controlled GitHub and platform-test gates.
  [`open-source-readiness.md:30`](../../docs/open-source-readiness.md#L30)
