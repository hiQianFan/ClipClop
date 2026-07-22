## Problem and solution

<!-- What problem does this solve, and why is this approach appropriate? -->

## Platforms affected

- [ ] macOS Apple Silicon
- [ ] macOS Intel
- [ ] Windows x64
- [ ] Platform-independent

## Verification

- [ ] `pnpm test`
- [ ] `pnpm check`
- [ ] `pnpm build`
- [ ] `cargo fmt --check --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] Relevant manual checks are described below

<!-- Record platform versions and non-sensitive results. Explain any skipped check. -->

## Privacy, permissions, and security

- [ ] No real clipboard contents, credentials, private URLs, personal paths, or other sensitive data are included
- [ ] Privacy, permission, network, and security impact is described below, including when there is no impact
- [ ] Relevant policy and user documentation is updated when a boundary changes

<!-- Describe any data-handling, permission, updater, or security impact. Report vulnerabilities privately instead of opening a PR. -->

## UI and accessibility

<!-- Add sanitized screenshots for visible changes and describe keyboard/screen-reader checks. Write "Not applicable" when appropriate. -->

## Remaining risks

<!-- List known limitations, follow-up work, and migration or rollback concerns. -->
