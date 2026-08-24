---
title: 'Deduplicate and polish clipboard metadata'
type: 'feature'
created: '2026-08-24'
status: 'done'
baseline_commit: '99c3be7'
context:
  - '{project-root}/PRODUCT.md'
  - '{project-root}/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Exact clipboard items can still be stored repeatedly, manual paste does not record usage when promotion is disabled, metadata columns shift with value width, link domains are not actionable, and Universal Clipboard captures are misattributed to the receiving Mac's foreground app.

**Approach:** Perform indexed, transactional SHA-256 deduplication in SQLite; separate immutable creation, actual use, and list position with `created_at`, `last_used_at`, and internal `sort_at`; stabilize the two-slot metadata rail; open a link's validated origin from its domain fact; and recognize macOS's remote-clipboard marker before local app inference.

## Boundaries & Constraints

**Always:** Preserve the canonical row ID and `created_at`; update `last_used_at` after every successful external capture, copy, or paste; update `sort_at` for every external capture and only for manual use when “move used to top” is enabled; keep all database decisions atomic; reuse the existing SHA-256 and hash index; keep the information-bar height unchanged; make the domain control keyboard accessible; validate/open URLs by clip ID in Rust; display remote captures as “Universal Clipboard” without claiming a device or originating app.

**Ask First:** Removing duplicate rows already present before this release, changing what constitutes exact equality, increasing the information-bar footprint, or adopting a private macOS signal beyond presence-only fallback attribution.

**Never:** Normalize whitespace, use fuzzy matching, change `created_at` to simulate recency, accept arbitrary external URLs from the frontend, fetch link metadata, infer iPhone/iPad, add a new dependency, or expose `sort_at` in the UI.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| New external copy | Unseen exact hash | Insert one row; initialize all three timestamps | Existing capture error path |
| Repeated external copy | Same complete flavor hash at any later time | Keep ID/created time, update use/order time and latest source, emit refresh | Transaction rolls back together |
| Slightly different copy | One whitespace or flavor byte differs | Store a separate row | N/A |
| Manual Enter/copy | Promotion disabled | Update only `last_used_at`; position remains stable | Clipboard write failure does not mark use |
| Manual Enter/copy | Promotion enabled | Update `last_used_at` and `sort_at`; move to top | Existing warning/error path |
| Metadata facts | Values vary greatly in width | Labels stay in two fixed slots; long values ellipsize | Missing second fact leaves its slot empty |
| Link domain | Valid HTTP(S) link | Domain button opens scheme + authority only | Invalid/non-link record is rejected in Rust |
| Remote clipboard | Exact `com.apple.is-remote-clipboard` format exists | Source is “Universal Clipboard” with device glyph | Marker absence safely falls back to current inference |

</frozen-after-approval>

## Code Map

- `src-tauri/schema.sql`, `src-tauri/src/storage/migrations.rs` -- add/backfill indexed `sort_at` in schema v6.
- `src-tauri/src/storage/database.rs`, `src-tauri/src/history/service.rs` -- atomically insert-or-promote exact hashes and split use from ordering.
- `src-tauri/src/workflows/{capture,clip_actions,paste_clip}.rs` -- apply capture/manual-use semantics and emit refreshes.
- `src-tauri/src/clipboard/source.rs` -- prioritize the macOS remote-clipboard presence marker.
- `src-tauri/src/preview/mod.rs`, `src-tauri/src/commands/preview.rs` -- open a validated clip URL's origin.
- `src/lib/history/{presentation.ts,ClipPreview.svelte,HistoryWorkspace.svelte,api.ts}` -- fixed fact grid, actionable domain, and refreshed timestamps.
- Relevant Rust/Vitest files -- migration, deduplication, ordering, attribution, origin, interaction, and layout regressions.

## Tasks & Acceptance

**Execution:**
- [x] Storage and history files -- migrate to v6 and replace the two-second precheck with one transactional indexed capture operation.
- [x] Clipboard workflows -- always record successful use while independently controlling promotion.
- [x] Source attribution -- recognize Universal Clipboard before any local source candidate and provide a neutral device icon.
- [x] Preview backend/API -- add origin-only opening without weakening the existing URL trust boundary.
- [x] Metadata presentation -- use a fixed two-column rail and accessible domain button without changing bar height.
- [x] Tests -- cover the I/O matrix and preserve existing behavior.

**Acceptance Criteria:**
- Given future exact repeats, when captured at any interval, then retained history contains one canonical item which becomes newest without recomputing or scanning payloads.
- Given promotion is disabled, when Enter successfully uses an item, then its displayed last-use time updates while its list position does not.
- Given metadata values change across selected items, when the detail view rerenders, then both label anchors remain stationary.
- Given a valid link or Universal Clipboard capture, when metadata renders, then the domain is safely actionable and the source is not misattributed to a local foreground app.

## Spec Change Log

## Design Notes

The right rail is a fixed 200px, two-equal-slot grid. Values remain right-aligned and tabular; overflow ellipsizes within its own slot. The domain looks like existing metadata at rest, gaining underline/ink only on hover or keyboard focus. `com.apple.is-remote-clipboard` is a widely observed private marker, so its absence must remain a harmless fallback rather than a capture failure.

## Verification

**Commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml` -- all Rust tests pass, including v4→v5→v6 migration.
- `npm test -- --run` -- all frontend behavior and interaction tests pass.
- `npm run check` -- no Svelte or TypeScript errors/warnings.
- `git diff --check` -- no whitespace errors.

**Manual checks:**
- Switch among image, text, long-domain, and one-fact items; metadata label anchors do not move.
- Copy from Universal Clipboard on macOS; source reads “Universal Clipboard” rather than the foreground Mac app.

## Suggested Review Order

**Identity and ordering**

- Start here: one transaction inserts or promotes the canonical exact-hash record.
  [`database.rs:50`](../../src-tauri/src/storage/database.rs#L50)

- Manual use updates real recency while promotion independently controls list position.
  [`database.rs:209`](../../src-tauri/src/storage/database.rs#L209)

- Schema v6 gives ordering its own indexed timestamp.
  [`schema.sql:12`](../../src-tauri/schema.sql#L12)

- Existing v4/v5 databases backfill ordering without changing stored history.
  [`migrations.rs:68`](../../src-tauri/src/storage/migrations.rs#L68)

**Attribution and safe navigation**

- Remote pasteboard presence wins before misleading local foreground attribution.
  [`source.rs:46`](../../src-tauri/src/clipboard/source.rs#L46)

- Rust resolves the stored clip and strips links to validated origins.
  [`preview/mod.rs:68`](../../src-tauri/src/preview/mod.rs#L68)

**Metadata interaction**

- Fixed slots, truncation, tabular values, and domain action prevent layout drift.
  [`ClipPreview.svelte:88`](../../src/lib/history/ClipPreview.svelte#L88)

- Copy refreshes use time without forcing promotion; domains request origin-only opening.
  [`HistoryWorkspace.svelte:193`](../../src/lib/history/HistoryWorkspace.svelte#L193)

**Verification**

- Database regressions cover canonical promotion and non-promoting use.
  [`database.rs:430`](../../src-tauri/src/storage/database.rs#L430)

- Component regression covers keyboard domain action and remote-source icon.
  [`ClipPreview.test.ts:60`](../../src/lib/history/ClipPreview.test.ts#L60)

- IPC regression keeps full-link and origin-only operations distinct.
  [`api.test.ts:23`](../../src/lib/history/api.test.ts#L23)
