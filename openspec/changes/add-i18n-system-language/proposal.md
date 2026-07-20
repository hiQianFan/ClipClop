# Change: Add system-aware English and Simplified Chinese localization

## Why

ClipClop currently embeds Chinese user-facing text across Svelte views, presentation helpers, accessibility labels, shortcut speech, updater states, and several Rust errors. Continuing this pattern would make every new feature harder to translate and would preserve an IPC boundary that exposes backend prose instead of stable semantics. The application is still in its pre-release development phase, so this is the lowest-cost point to establish one localization path and remove the obsolete hard-coded path completely.

## What Changes

- Add `system | zh-CN | en` as the only persisted language preferences.
- Resolve `system` from the operating system's primary locale: any `zh-*` locale uses `zh-CN`; every other, missing, or invalid locale uses `en`.
- Keep `system` as the default preference and `en` as the fallback effective locale.
- Add a single typed frontend localization runtime and complete `en` and `zh-CN` catalogs.
- Move all application-owned visible and assistive text out of Svelte components and presentation helpers.
- Initialize language and theme before rendering the interactive application, and update the document language when the effective locale changes.
- Format dates and numbers through locale-aware presentation helpers while preserving the product's compact date shape.
- Replace user-visible Rust prose with stable, specific error codes and structured parameters; retain diagnostic details only for logging.
- Treat the settings shape as a breaking pre-release change: bump the development database schema and do not retain legacy settings deserialization or hard-coded-text fallbacks.

## Breaking Changes

- Existing development databases use the previous schema and must be rebuilt according to the repository's existing development-schema policy.
- `Settings` gains a required `language` field; callers that send the old settings payload are unsupported.
- UI code may no longer display backend `message` values directly.

## Impact

- Frontend: root bootstrap, settings UI, history panel, shortcut descriptions and speech, updater states, metadata formatting, accessibility text, and related tests.
- Rust: settings model, schema version, validation error representation, IPC error DTOs, and tests.
- Documentation: architecture must state the locale ownership and error-localization boundary.
- No clipboard storage, capture, search, copy, paste, or update transport behavior changes.

