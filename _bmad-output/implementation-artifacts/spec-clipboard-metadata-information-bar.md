---
title: 'Clipboard metadata information bar'
type: 'feature'
created: '2026-08-24'
status: 'done'
baseline_commit: 'c7bd3c41a1cba3acf1f2a8c5535234f49d749808'
context:
  - '{project-root}/PRODUCT.md'
  - '{project-root}/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The preview information bar exposes only the initial capture time and generic facts, so promoted/reused clips can show a timestamp inconsistent with their position and non-text content lacks the most useful identifying metadata.

**Approach:** Return the already-persisted `last_used_at`, show both first-copy and last-used times, and derive a compact maximum of two type-specific facts locally from existing clip data. Keep content formats persisted but invisible.

## Boundaries & Constraints

**Always:** Preserve the existing source-app block and information-bar footprint; label both timestamps explicitly; use local-only derivation; preserve text/code character count and clipboard byte size; preserve image dimensions and size; make missing file sizes an omitted fact rather than `0 B`; keep Chinese and English catalogs aligned and keyboard/screen-reader semantics intact.

**Ask First:** Any schema migration, source-file probing beyond the existing explicit preview path, information-bar height increase, new network request, or new metadata setting.

**Never:** Add line count, clipboard flavor/MIME UI, copy/use count, hashes, database IDs, remote link titles/favicons, or silently treat partially known multi-file sizes as a total.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Text/code | Plain text with character metadata | Characters and clipboard size | Fall back to plain-text character count |
| Link | Valid HTTP(S) text | Hostname and characters | If parsing fails, omit hostname |
| Image | Dimensions present | Dimensions and clipboard size | Omit dimensions if unavailable |
| Color | Recognized hex color | Format `HEX`; no characters/size | N/A |
| Single file | Path with or without extension | Type and available size | Extensionless path uses localized generic file type |
| Multiple files | Two or more paths | File count and total only when every size is known; otherwise available selected-file size may be shown without claiming a total | Unknown sizes are omitted |
| Time | Newly captured or later touched clip | First-copy and last-used timestamps both remain visible | Invalid persisted timestamps continue through existing storage error path |

</frozen-after-approval>

## Code Map

- `src-tauri/src/history/model.rs` -- add `last_used_at` to serialized clip summaries.
- `src-tauri/src/storage/database.rs` -- select and decode `last_used_at` in list/detail queries and cover it in database tests.
- `src/lib/history/types.ts` -- mirror the backend summary field.
- `src/lib/history/presentation.ts` -- derive link hostname, file type/count/size, color format, and existing text/image facts.
- `src/lib/history/ClipPreview.svelte` -- render labeled first-copy and last-used timestamps and pass localized metadata labels.
- `src/lib/i18n/catalogs.ts` -- add aligned English/Chinese labels.
- `src/lib/history/{presentation,ClipPreview,session,preview-session,HistoryList}.test.ts` -- update fixtures and test type-specific facts and dual timestamps.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/history/model.rs`, `src-tauri/src/storage/database.rs` -- expose persisted last-use time without a schema change.
- [x] `src/lib/history/types.ts`, `src/lib/i18n/catalogs.ts` -- align the frontend contract and labels.
- [x] `src/lib/history/presentation.ts`, `src/lib/history/ClipPreview.svelte` -- implement compact, local type-specific metadata and dual-time UI.
- [x] Relevant Rust and Vitest files -- cover query mapping, timestamp rendering, every content type, invalid links, extensionless files, and incomplete multi-file sizes.

**Acceptance Criteria:**
- Given any selected clip, when its information bar renders, then both localized first-copy and last-used timestamps are visible and the latter matches persisted ordering state.
- Given each supported content type, when metadata renders, then only its confirmed facts appear and no forbidden technical field is shown.
- Given unavailable or partial metadata, when the item renders, then the UI remains accurate, compact, and free of fabricated zero/total values.
- Given the implementation, when Rust and frontend test suites run, then all relevant tests pass.

## Spec Change Log

## Design Notes

The source block becomes three compact text lines beside the existing 22px icon: application name, first-copy time, last-used time. The right rail remains a two-fact definition list; content preview keeps visual priority.

## Verification

**Commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml` -- expected: all Rust tests pass.
- `npm test -- --run` -- expected: all frontend tests pass.
- `npm run check` -- expected: Svelte/TypeScript checks pass.
- `git diff --check` -- expected: no whitespace errors.

## Suggested Review Order

**UI behavior**

- Start here: dual labeled timestamps and compact per-type facts share the existing bar.
  [`ClipPreview.svelte:91`](../../src/lib/history/ClipPreview.svelte#L91)

- Local derivation keeps metadata type-specific, capped, and network-free.
  [`presentation.ts:24`](../../src/lib/history/presentation.ts#L24)

- Refetch stale cached details when persisted recency changes.
  [`session.svelte.ts:59`](../../src/lib/history/session.svelte.ts#L59)

**Persistence contract**

- Existing last-use state now crosses the Rust serialization boundary.
  [`model.rs:105`](../../src-tauri/src/history/model.rs#L105)

- List and detail queries decode the same persisted ordering timestamp.
  [`database.rs:137`](../../src-tauri/src/storage/database.rs#L137)

- TypeScript mirrors the backend summary contract.
  [`types.ts:25`](../../src/lib/history/types.ts#L25)

**Language and verification**

- English and Chinese labels remain aligned.
  [`catalogs.ts:123`](../../src/lib/i18n/catalogs.ts#L123)

- Content-type cases cover missing and partial metadata.
  [`presentation.test.ts:63`](../../src/lib/history/presentation.test.ts#L63)

- Cache regression proves refreshed last-use timestamps reach the detail view.
  [`session.test.ts:42`](../../src/lib/history/session.test.ts#L42)

- Component coverage verifies both time labels remain visible.
  [`ClipPreview.test.ts:47`](../../src/lib/history/ClipPreview.test.ts#L47)
