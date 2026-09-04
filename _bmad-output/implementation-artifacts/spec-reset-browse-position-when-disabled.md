---
title: 'Reset browse position when retention is disabled'
type: 'bugfix'
created: '2026-09-04'
status: 'done'
baseline_commit: '7a27b43b1713a2c01d4c13b79a476a3d3528c61e'
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** After a user browses to a later history page, disables “Restore last browse position,” saves, and returns to history, the current session still shows the old page and selection. This contradicts the disabled preference until the panel is summoned again.

**Approach:** Apply the already-established reset-to-latest behavior when leaving Settings whenever browse-position restoration is disabled, so the visible history immediately returns to page one and selects the latest item.

## Boundaries & Constraints

**Always:** Reuse `resetToLatest`; preserve search-condition behavior as an independent preference; preserve focus restoration and error handling.

**Ask First:** Any change to persistence semantics, settings storage, or behavior while browse-position restoration remains enabled.

**Never:** Add a second page-reset implementation, clear history data, alter pagination controls, or include unrelated `_bmad-output` artifacts in the release.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Disabled after later-page browsing | Current page > 1; setting saved off; Settings closes | History refreshes page 1 and selects the latest item | Existing localized refresh error remains visible |
| Still enabled | Current page > 1; setting remains on; Settings closes | Current page and selection remain unchanged | Existing settings-load error behavior remains |
| Already disabled on page 1 | Current page = 1; setting off; Settings closes | Page 1 refreshes safely with latest selection | Existing empty-history behavior remains |

</frozen-after-approval>

## Code Map

- `src/lib/history/HistoryWorkspace.svelte` -- owns Settings close, synchronized preferences, session page, selection, and the shared `resetToLatest` path.
- `src/lib/history/session.svelte.ts` -- owns page refresh and latest-item selection semantics used by `resetToLatest`.
- `src/lib/history/session.test.ts` -- existing focused tests for page and selection refresh behavior.

## Tasks & Acceptance

**Execution:**
- [x] `src/lib/history/HistoryWorkspace.svelte` -- route Settings close through `resetToLatest` when the synchronized preference is disabled; otherwise retain the existing resume behavior.
- [x] `src/lib/history/HistoryWorkspace.test.ts` -- cover disabled reset and enabled preservation without introducing a new state abstraction.

**Acceptance Criteria:**
- Given the user is on a later history page, when they disable browse-position restoration, save, and leave Settings, then page one and its latest item are shown.
- Given browse-position restoration remains enabled, when Settings closes, then the current page and selection are preserved.
- Search text and filters continue to follow only the separate “Preserve last search conditions” preference.

## Spec Change Log

## Verification

**Commands:**
- `pnpm test -- <focused-test>` -- expected: regression test passes.
- `pnpm check` -- expected: zero Svelte/TypeScript diagnostics.
- `git diff --check` -- expected: no whitespace errors.

## Suggested Review Order

**Reset behavior**

- Route disabled preference through the existing latest-item reset.
  [`HistoryWorkspace.svelte:401`](../../../src/lib/history/HistoryWorkspace.svelte#L401)

- Avoid resetting from stale preferences when settings refresh fails.
  [`HistoryWorkspace.svelte:116`](../../../src/lib/history/HistoryWorkspace.svelte#L116)

**Regression coverage**

- Verify disabled reset, enabled preservation, and refresh-failure safety.
  [`HistoryWorkspace.test.ts:66`](../../../src/lib/history/HistoryWorkspace.test.ts#L66)
