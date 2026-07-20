# Implementation tasks

## 1. Establish the language contract

- [x] Add strict Rust and TypeScript types for `system | zh-CN | en` and make `system` the settings default.
- [x] Increment the development database schema version and update settings tests for the intentionally breaking shape.
- [x] Add pure locale resolution with `zh-* -> zh-CN` and all other cases `-> en`.

## 2. Add the frontend localization foundation

- [x] Add typed, key-complete `en` and `zh-CN` catalogs plus interpolation support.
- [x] Add reactive preference/effective-locale state and localized number/date helpers.
- [x] Bootstrap settings, theme, effective locale, and document language in the root layout before rendering the interactive UI.

## 3. Remove hard-coded application text

- [x] Migrate history-panel visible text, status text, menus, confirmations, placeholders, titles, and accessibility attributes.
- [x] Migrate settings, shortcuts, updater, About, and data-management text.
- [x] Localize shortcut spoken labels and presentation-helper metadata without translating keycap glyphs or user data.
- [x] Remove superseded hard-coded text helpers and duplicated settings initialization.

## 4. Make IPC errors locale-neutral

- [x] Replace expected Rust validation prose with specific stable error codes and safe parameters.
- [x] Prevent raw backend diagnostic strings from being shown directly in the UI.
- [x] Map error codes and existing typed outcomes to catalog messages.

## 5. Document and verify

- [x] Update architecture documentation with language ownership, resolution, bootstrap, and IPC error rules.
- [x] Add locale-resolution, catalog-parity, formatting, persistence, and error-mapping tests.
- [x] Run `pnpm check`, `pnpm test`, `pnpm build`, Rust formatting, Clippy, and Rust tests.
- [ ] Perform English and Chinese UI smoke checks for overflow, keyboard use, and assistive text on macOS and Windows.
