---
id: SPEC-add-i18n-system-language
companions:
  - design.md
  - specs/localization/spec.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate.

# System-aware English and Simplified Chinese localization

## Why

ClipClop needs one localization boundary before additional hard-coded UI and backend prose create ongoing compatibility work. Pre-release status makes a clean, intentionally breaking transition cheaper than maintaining legacy language paths.

## Capabilities

- id: CAP-1
  intent: The application selects English or Simplified Chinese from a system-following or explicit user preference.
  success: Chinese primary system locales resolve to `zh-CN` under `system`; every other system locale resolves to `en`; explicit choices override the system.

- id: CAP-2
  intent: Users can operate the entire application in either supported language.
  success: All application-owned visible and assistive text renders from complete typed catalogs with no hard-coded Chinese fallback.

- id: CAP-3
  intent: Users can preview, save, abandon, and retain a language choice through the existing settings experience.
  success: Preview is immediate, abandon restores the saved locale, and save survives restart.

- id: CAP-4
  intent: Users receive localized, actionable errors without exposure to backend implementation prose.
  success: Expected failures map from stable semantic codes and raw diagnostic strings never render in the UI.

- id: CAP-5
  intent: Dates, numbers, and assistive document metadata follow the effective locale.
  success: Locale changes update formatting and `document.lang` without an application restart.

## Constraints

- Supported preferences are exactly `system`, `zh-CN`, and `en`; effective locales are exactly `zh-CN` and `en`.
- `system` is the persisted default; `en` is the fallback for every non-Chinese, missing, or invalid system locale.
- The transition is destructive under the existing development-schema policy; no permanent old-settings or old-text compatibility layer is allowed.
- Rust owns persisted preference and semantic error data; the frontend owns locale resolution, catalogs, formatting, and user-facing prose.
- Clipboard content, file paths, application names, versions, keycaps, and release-note bodies remain untranslated data.

## Non-goals

- Supporting Traditional Chinese or a third UI language.
- Translating repository documentation, release-note bodies, installers, operating-system permission dialogs, or external web content.
- Adding a translation management service, remote catalog loading, or runtime language downloads.
- Preserving old development databases or old Settings IPC payloads.

## Success signal

On a clean build, ClipClop starts in Chinese only for a Chinese primary system locale, starts in English everywhere else, allows an explicit saved override, and exposes no application-owned hard-coded Chinese UI or directly rendered backend diagnostic prose.

