# Localization specification delta

## ADDED Requirements

### Requirement: Deterministic language resolution

The application SHALL persist exactly one language preference from `system`, `zh-CN`, or `en`, with `system` as the default. The application SHALL resolve the effective locale to `zh-CN` only when the explicit preference is `zh-CN` or the primary system language has base language `zh`; all other cases SHALL resolve to `en`.

#### Scenario: Chinese system language

- **GIVEN** the saved preference is `system`
- **WHEN** the primary system locale is any valid `zh-*` locale
- **THEN** the effective locale is `zh-CN`

#### Scenario: Non-Chinese or unavailable system language

- **GIVEN** the saved preference is `system`
- **WHEN** the primary system locale is non-Chinese, missing, or invalid
- **THEN** the effective locale is `en`

#### Scenario: Explicit preference

- **GIVEN** the user saved `zh-CN` or `en`
- **WHEN** the application starts or the system language changes
- **THEN** the effective locale remains the explicitly selected locale

### Requirement: Complete localized application interface

The application SHALL source all application-owned visible and assistive text from complete typed `en` and `zh-CN` catalogs. It SHALL NOT retain hard-coded Chinese UI fallbacks or silently accept missing catalog entries.

#### Scenario: Rendering either supported locale

- **GIVEN** either supported effective locale
- **WHEN** the user navigates the history panel, settings, shortcuts, updater, About, confirmations, loading, empty, and error states
- **THEN** visible text and accessibility text are presented consistently in that locale

#### Scenario: User data is displayed

- **GIVEN** a clip, file path, source application name, version, keycap, or release-note body
- **WHEN** it is displayed inside a localized interface
- **THEN** the application preserves that data rather than translating it

### Requirement: Locale-aware presentation

The application SHALL set the document language to the effective locale and SHALL format user-facing dates and numbers with that locale while retaining the product's compact metadata layout.

#### Scenario: Effective locale changes

- **WHEN** the effective locale changes
- **THEN** translated UI, document language, dates, and numbers update without restarting the application

### Requirement: Persisted explicit selection

The settings interface SHALL allow `system`, `zh-CN`, and `en`. A selection SHALL preview immediately, SHALL persist only when settings are saved, and SHALL revert to the saved preference when the unsaved form is abandoned.

#### Scenario: Save language preference

- **WHEN** the user selects a language and saves settings
- **THEN** the preference survives application restart and controls subsequent locale resolution

#### Scenario: Abandon language change

- **WHEN** the user previews another language and leaves settings without saving
- **THEN** the previously saved preference and effective locale are restored

### Requirement: Locale-neutral backend errors

Rust commands SHALL expose stable semantic error codes and safe structured parameters. The frontend SHALL localize those codes, and raw backend diagnostic details SHALL NOT be displayed directly to users.

#### Scenario: Recoverable shortcut validation failure

- **WHEN** a shortcut is malformed, unsupported, duplicated, reserved, or unavailable
- **THEN** the UI displays the corresponding localized explanation without parsing backend prose

#### Scenario: Generic backend failure

- **WHEN** a storage, clipboard, or platform operation fails unexpectedly
- **THEN** the UI displays a localized category-level message while diagnostic detail remains outside the user interface

### Requirement: Breaking pre-release settings transition

The application SHALL adopt the new strict settings shape through the existing destructive development-schema policy and SHALL NOT carry permanent compatibility code for the old payload.

#### Scenario: Old development database

- **GIVEN** a database created with the previous development schema version
- **WHEN** the localized build opens it
- **THEN** the existing unsupported-schema behavior requires a rebuild rather than attempting legacy settings deserialization

