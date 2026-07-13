---
target: ClipClop Quick Panel layout
total_score: 27
p0_count: 0
p1_count: 2
timestamp: 2026-07-13T15-00-28Z
slug: src-routes-page-svelte
---
## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|---|---:|---|
| 1 | Visibility of system status | 3 | Loading and copy feedback exist, but the drag affordance has no visible structural cue. |
| 2 | Match between system and real world | 3 | List-to-preview model is familiar; the top right blank header has no natural semantic role. |
| 3 | User control and freedom | 2 | Escape/blur behavior is intended, but the panel's transient vs. persistent window role remains visually ambiguous. |
| 4 | Consistency and standards | 2 | The documented radius scale is not consistently applied; the outer elevation contract changed without a clear platform fallback. |
| 5 | Error prevention | 3 | Destructive actions are confirmed and disabled states exist. |
| 6 | Recognition rather than recall | 3 | Shortcut labels and a visible action bar help; source/type slots remain terse for first-time users. |
| 7 | Flexibility and efficiency | 4 | Keyboard navigation, number selection, search and copy shortcuts support the primary power-user path. |
| 8 | Aesthetic and minimalist design | 2 | The top row spends 420px on an empty drag zone and the absent shadow makes the whole panel read as an overlay. |
| 9 | Error recovery | 3 | Retry and inline error text exist; error messages can still be terse. |
| 10 | Help and documentation | 2 | Repository documentation exists, but the panel offers no first-use cue for its main keyboard workflow. |
| **Total** | | **27/40** | **Acceptable — significant refinement needed** |

## Anti-Patterns Verdict

The surface is not generically AI-styled: it avoids gradients, colored side rails, repeated cards, oversized radii, and decorative glass. Its failure is product-UI ambiguity rather than visual excess. The header is a structural dead zone: it looks like a missing title bar rather than intentional chrome.

The deterministic scan found five advisory design-system mismatches in `src/routes/+page.svelte`: undocumented radii of 5px (two), 10px (two), and the menu shadow color `rgba(0,0,0,.25)`. The 10px menu radius is a meaningful token-drift finding; the 5px app icon/fallback corners are a minor but real mismatch. The shadow finding is legitimate because the current elevation spec delegates external elevation to the native window and leaves this menu shadow undocumented.

## Overall Impression

The core interaction is strong: scan recent clips on the left, verify on the right, copy from the bottom. The single largest opportunity is to make the horizontal and vertical structure tell that story without creating a false, empty title bar.

## What's Working

- The fixed two-column list/preview model maps directly to the retrieval task and supports rapid keyboard use.
- The bottom action bar separates paging from reuse actions without adding a navigation system.
- Restrained gray surfaces let copied content remain the visual focus.

## Priority Issues

### [P1] Header breaks the two-column hierarchy

**Why it matters:** The search belongs to the left-side retrieval task, but its current top-row framing and the blank 420px right segment make the divider feel interrupted. Users read an unfinished title bar rather than two clearly owned workspaces.

**Fix:** Move the search visually into the left workspace and extend the vertical divider through the header. Keep a compact, explicitly structured window chrome above both panes only if it has a real role; otherwise remove it. Do not introduce navigation, tabs, or filters without new product scope.

**Suggested command:** `$impeccable layout`

### [P1] Floating-panel elevation is unreliable

**Why it matters:** In both light and dark environments, a same-tone borderless panel without a dependable shadow is indistinguishable from a modal scrim or a full-screen mask. This weakens discoverability and makes the rounded silhouette feel accidental.

**Fix:** Establish one cross-platform elevation contract: native shadow where reliable, with a transparent window inset and a single panel shadow fallback where it is not. Do not pair that fallback with an outer Web border.

**Suggested command:** `$impeccable polish`

### [P2] Geometry tokens drift from the documented system

**Why it matters:** The app uses 4/6/8/14px as its declared vocabulary, while app icons and the action menu use undocumented 5/10px corners. Small inconsistency is especially noticeable in a compact neutral UI.

**Fix:** Use 4px for app icon/fallback corners and 8px for the action menu, or explicitly add a semantic token if a new value is genuinely needed. Document any popup elevation token.

**Suggested command:** `$impeccable polish`

### [P2] First-use comprehension has no lightweight cue

**Why it matters:** A first-time user may see `RTF`, `IMG`, and the bottom shortcuts without understanding that Enter copies and the panel hides. The power-user model is efficient but not self-explanatory.

**Fix:** In the empty state and initial selection state, show one concise instruction such as “选择记录后按 Enter 复制”。 Keep this absent once history exists if density is the priority.

**Suggested command:** `$impeccable onboard`

## Persona Red Flags

### Alex (Power User)

Alex can complete the primary retrieval flow quickly using search, arrow keys, number jumps and Enter. The header ambiguity is the red flag: a blank right-side drag zone consumes valuable visual attention without accelerating the core flow.

### Jordan (First-Timer)

Jordan can recognize a search field and list, but has no immediate explanation of `RTF`, `IMG`, source metadata, or the Enter-to-copy behavior. The top chrome looks like an incomplete area rather than a window affordance.

### Sam (Accessibility-Dependent User)

Keyboard support and visible focus provide a useful foundation. The non-interactive drag zone has an `aria-label` but no semantic role, so it should be hidden from assistive technology. The small metadata and keycap text should be rechecked against AA at their actual rendering size.

## Minor Observations

- The right preview has no top anchor; a subtle structural header or a clean column start would improve scan rhythm.
- The source icon gives useful provenance, but the app fallback glyph should not become the only type cue.
- The action menu uses a broad CSS shadow even though the outer panel relies on native elevation; document the exception or simplify it.

## Questions to Consider

- Should the panel read primarily as a transient command surface, or as a compact persistent utility window?
- Is source/type metadata needed while scanning, or only after the user has selected a clip?
- Would a left-owned search plus a continuous divider better express “find, then verify” than a shared top bar?
