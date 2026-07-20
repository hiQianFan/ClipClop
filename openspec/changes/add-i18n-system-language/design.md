# Localization design

## Language model

The persisted preference and resolved locale are separate types:

```text
LanguagePreference = system | zh-CN | en
EffectiveLocale     = zh-CN | en
```

Resolution is deterministic:

```text
preference = zh-CN  -> zh-CN
preference = en     -> en
preference = system and primary system language is zh -> zh-CN
preference = system otherwise                           -> en
```

The primary system language is the first available WebView language preference. Locale parsing compares the base language subtag, so `zh`, `zh-CN`, `zh-TW`, and other `zh-*` tags resolve to `zh-CN` while the preference remains `system`. Invalid or unavailable system locale data resolves to `en`.

## Ownership and startup

Rust persists `LanguagePreference` with the rest of `Settings`; the frontend owns locale resolution and all user-facing translations. The root layout loads settings once, resolves language, applies language and theme, sets `document.documentElement.lang`, and only then renders the interactive application and schedules automatic update checks. Feature components consume the initialized localization runtime and do not independently fetch preferences for initialization.

Changing the language selector updates the preview immediately. Saving persists the preference. Cancelling or leaving without saving restores the previously persisted preference, matching the existing settings form semantics.

## Catalog and API

Use a first-party typed module under `src/lib/i18n/` with complete static catalogs for `en` and `zh-CN`. English defines the key shape and TypeScript requires the Chinese catalog to match it exactly. Keys are semantic, for example `settings.general.language`, rather than source-language sentences.

The runtime exposes only the capabilities the current product needs:

```text
t(key, parameters?)
formatNumber(value)
formatDateTime(value)
setLanguagePreference(preference)
```

Catalog entries that interpolate values are functions with typed parameters. There is no runtime per-key fallback to old literals: missing or mismatched keys fail type-checking, and invalid dynamic access fails loudly in development and tests. Unsupported locales are resolved to the complete English catalog before lookup.

Application-owned text includes visible labels, status and error text, placeholders, `title`, `alt`, `aria-label`, shortcut spoken labels, metadata labels, and empty/loading states. Brand names, user clipboard content, file paths, application names, version strings, keycap glyphs, and release-note bodies are data and remain untranslated.

## Formatting

Use `Intl.NumberFormat` for counts. Use `Intl.DateTimeFormat` parts to preserve the existing compact product format (`MM-DD HH:mm`, with `YYYY-` only outside the current year) while applying locale-correct digits and separators. Byte units remain compact technical units (`B`, `KB`, `MB`); their surrounding labels are translated.

## IPC errors

Rust errors crossing IPC contain a stable code and optional safe parameters. Expected user-recoverable failures receive specific codes such as:

```text
HOTKEY_INVALID_FORMAT
HOTKEY_MISSING_MODIFIER
HOTKEY_UNSUPPORTED_KEY
HOTKEY_DUPLICATE_MODIFIER
HOTKEY_RESERVED
HOTKEY_UNAVAILABLE
NOT_FOUND
STORAGE_ERROR
CLIPBOARD_ERROR
PLATFORM_ERROR
```

The frontend maps codes to catalog entries. Generic failures use a localized category message. Raw database, clipboard, operating-system, and plugin error strings are diagnostic data and must not be rendered to users; Rust logs them where useful. Existing typed outcomes such as `PasteOutcome` remain semantic and are translated in the frontend.

## Breaking migration

This proposal follows the current pre-release database policy: increment the development schema version and require incompatible databases to be rebuilt. The new `Settings` shape is strict and required. Do not add aliases, optional legacy language fields, dual settings keys, translation wrappers around old hard-coded strings, or permanent migration branches.

## Verification strategy

- Unit-test preference resolution, including Chinese variants, non-Chinese locales, empty input, and invalid tags.
- Type-check catalog parity and interpolation parameters.
- Exercise both catalogs in component-level or presentation tests, including accessibility labels and error-code mapping.
- Verify preference preview, save, cancel, restart persistence, and system resolution manually on macOS and Windows.
- Run existing frontend and Rust checks to ensure clipboard and shortcut behavior remains unchanged.

