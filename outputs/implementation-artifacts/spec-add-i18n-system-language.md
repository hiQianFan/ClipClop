---
title: 'Add system-aware English and Simplified Chinese localization'
type: 'feature'
created: '2026-07-20'
status: 'done'
baseline_commit: '03e38b495f4a1dfea39cf3da9e0f8c73cc4d1cab'
context:
  - 'openspec/changes/add-i18n-system-language/SPEC.md'
  - 'openspec/changes/add-i18n-system-language/design.md'
  - 'openspec/changes/add-i18n-system-language/specs/localization/spec.md'
  - 'docs/architecture.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** ClipClop hard-codes Chinese application text in Svelte views, presentation helpers, shortcut accessibility output, updater states, and Rust validation errors. Extending the product without one localization boundary would accumulate incompatible text and error paths.

**Approach:** Replace the old path in one breaking pre-release migration. Persist `system | zh-CN | en`, resolve `system` to `zh-CN` only for a primary `zh-*` locale and to `en` otherwise, render all application-owned prose from complete typed frontend catalogs, and localize stable backend error codes rather than backend messages.

## Boundaries & Constraints

**Always:** Default the persisted preference to `system` and the unsupported/missing/invalid locale fallback to `en`; treat the first WebView language preference as the primary system locale; map every `zh-*` variant to `zh-CN`; initialize language, theme, and `document.lang` before interactive UI; preview an unsaved language selection immediately, persist only on Save, and restore the saved preference on abandon; keep clipboard content, paths, app names, versions, keycaps, and release-note bodies unchanged; preserve compact date and byte presentation; use stable semantic IPC error codes and keep raw diagnostics out of the UI; update tests and architecture documentation.

**Ask First:** Any need to add a third effective locale, preserve an old database/settings payload, expose raw backend detail to users, or change clipboard/update transport behavior.

**Never:** Retain hard-coded Chinese UI fallbacks, optional or aliased legacy language fields, dual settings keys, permanent settings migrations, remote catalogs, runtime language downloads, or translation of user/system-provided data.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| System Chinese | preference `system`, primary locale any valid `zh-*` | effective locale `zh-CN` | none |
| System fallback | preference `system`, locale non-Chinese, missing, or invalid | effective locale `en` | never leak parse failure |
| Explicit override | preference `zh-CN` or `en` | effective locale equals preference across restart/system changes | reject unknown settings value |
| Unsaved preview | change language in Settings, then abandon | UI previews selection, then restores saved language | no persistence |
| Backend failure | known code with safe parameters or generic category | localized catalog message | raw diagnostic logged, never rendered |
| Incompatible dev data | database schema older than new version | existing unsupported-schema/reset behavior | no legacy deserialization |

</frozen-after-approval>

## Code Map

- `src/lib/i18n/` -- new typed catalogs, reactive locale state, resolution, formatting, and error mapping.
- `src/routes/+layout.svelte`, `src/app.html` -- preference bootstrap, render gate, update scheduling, document defaults.
- `src/routes/+page.svelte` -- history panel text, status, menus, confirmations, and accessibility attributes.
- `src/lib/settings/SettingsView.svelte` -- language selector preview/save/abandon and all settings/updater text.
- `src/lib/settings/api.ts`, `src/lib/settings/shortcuts.ts` -- shared settings contract and locale-neutral shortcut results/key speech.
- `src/lib/clips/view.ts`, `src/lib/updater/api.ts` -- translated outcomes/errors and locale-aware presentation without UI prose in transport helpers.
- `src-tauri/src/settings.rs`, `src-tauri/src/error.rs`, `src-tauri/src/commands/settings.rs` -- strict preference model and semantic IPC errors.
- `src-tauri/src/clipboard/system.rs` -- remove the persisted Chinese filename fallback so the frontend can localize unnamed files.
- `src-tauri/src/storage/database.rs`, `docs/architecture.md` -- breaking development schema and documented ownership boundary.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/settings.rs`, `src/lib/settings/api.ts`, `src-tauri/src/storage/database.rs` -- add strict language preference and destructive schema bump with tests.
- [x] `src/lib/i18n/**`, `src/routes/+layout.svelte`, `src/app.html` -- add typed complete catalogs, deterministic resolution, reactive formatting, and startup bootstrap.
- [x] `src/routes/+page.svelte`, `src/lib/settings/SettingsView.svelte`, `src/lib/settings/shortcuts.ts`, `src/lib/clips/view.ts`, `src/lib/updater/api.ts` -- migrate all application-owned visible and assistive prose and remove superseded initialization/helpers.
- [x] `src-tauri/src/error.rs`, Rust command/validation call sites, `src-tauri/src/clipboard/system.rs`, frontend error mapping -- replace displayed backend prose with stable safe codes and remove persisted localized fallback data.
- [x] Frontend/Rust tests, `docs/architecture.md`, OpenSpec tasks -- verify both locales and record completion.

**Acceptance Criteria:**
- Given either effective locale, when every history/settings/update/loading/empty/error/accessibility state renders, then all application-owned text uses the selected complete catalog.
- Given a saved explicit choice, when the application restarts, then it remains effective; given an abandoned preview, the prior saved locale is restored.
- Given a locale change, when metadata rerenders, then `document.lang`, compact dates, and numbers use that effective locale.
- Given any IPC error, when it reaches the UI, then a localized semantic/category message is shown and no raw diagnostic text is displayed.
- Given the approved breaking policy, when old development data is opened, then the current unsupported-schema flow applies with no legacy branch.

## Spec Change Log

## Design Notes

English defines the static catalog shape; `zh-CN` must satisfy the same TypeScript type. Interpolated entries use typed functions. Unsupported locales resolve to English before lookup, while missing keys fail type-check/development tests rather than falling through to old literals. Rust owns persisted preference and semantic failure data; the frontend exclusively owns prose and locale formatting.

## Verification

**Commands:**
- `pnpm check` -- Svelte and TypeScript pass, including catalog parity.
- `pnpm test` -- locale, formatting, shortcuts, outcomes, and existing frontend tests pass.
- `pnpm build` -- static application build succeeds.
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml` -- Rust formatting passes.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` -- Rust lint passes.
- `cargo test --manifest-path src-tauri/Cargo.toml` -- settings, error, storage, and existing backend tests pass.
- `rg -n '[\\p{Han}]' src src-tauri/src` -- remaining Han text is limited to catalog content, comments, fixtures, or user-data test values.

**Manual checks:**
- Inspect English and Chinese layouts for overflow and verify keyboard/assistive labels on macOS and Windows.

## Suggested Review Order

**Localization contract**

- Start with deterministic locale resolution, typed interpolation, and formatting ownership.
  [`index.svelte.ts:17`](../../src/lib/i18n/index.svelte.ts#L17)

- Review English shape authority and complete Simplified Chinese parity.
  [`catalogs.ts:1`](../../src/lib/i18n/catalogs.ts#L1)

**Startup and settings lifecycle**

- Confirm settings bootstrap gates rendering without leaving failure paths blank.
  [`+layout.svelte:10`](../../src/routes/+layout.svelte#L10)

- Trace preview, asynchronous save, abandon restoration, and translated state refresh.
  [`SettingsView.svelte:76`](../../src/lib/settings/SettingsView.svelte#L76)

- Check all history states, accessibility text, counters, and unnamed-file fallbacks.
  [`+page.svelte:57`](../../src/routes/+page.svelte#L57)

**Backend boundary**

- Verify strict language values and semantic shortcut validation errors.
  [`settings.rs:13`](../../src-tauri/src/settings.rs#L13)

- Confirm IPC serialization exposes codes while retaining safe diagnostics server-side.
  [`error.rs:23`](../../src-tauri/src/error.rs#L23)

- Review the intentional development schema break from version 3 to 4.
  [`database.rs:17`](../../src-tauri/src/storage/database.rs#L17)

**Update behavior and documentation**

- Confirm automatic checks read current settings and updater errors remain semantic.
  [`api.ts:100`](../../src/lib/updater/api.ts#L100)

- Review the documented locale and IPC ownership rules.
  [`architecture.md:259`](../../docs/architecture.md#L259)

**Verification**

- Inspect locale resolution, catalog parity, typed interpolation, formatting, and errors.
  [`index.test.ts:5`](../../src/lib/i18n/index.test.ts#L5)

- Inspect strict persisted-language and breaking payload coverage.
  [`settings.rs:173`](../../src-tauri/src/settings.rs#L173)
