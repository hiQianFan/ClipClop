# Clip-Clop Design Spec

Operationalizes the first-principles direction in `addendum.md` into a single committed theme, a converged feature set, and the overall layout. This is the design source of truth; where it conflicts with earlier prototype notes, this wins.

## 1. Theme — "Neutral Graphite"

One theme, two modes (follows system). Dark is the hero. Achromatic neutral-graphite shell (no warm/brown tint). A single amber accent used ONLY on the primary Paste button — nowhere else, including selection. There is no type-icon color system; each content type represents itself in the lead slot (swatch / thumbnail / file-type icon / favicon), and text/code carry no lead mark. Clipboard content is monospace.

### 1.1 Color — Dark (hero) — "Neutral Graphite" (chosen)

| Token | Value | Use |
|---|---|---|
| `--bg-shell` | `#17181A` | panel base / shell |
| `--bg-raised` | `#1E2022` | row + preview surface |
| `--bg-hover` | `#242628` | row hover |
| `--bg-selected` | `#2A2D30` | selected row (neutral fill, one step above hover) |
| `--border-hairline` | `#2C2F32` | dividers |
| `--text-primary` | `#ECEDEE` | content, primary |
| `--text-secondary` | `#9BA1A6` | source app |
| `--text-tertiary` | `#6B7075` | time / meta |
| `--accent` | `#E8A33D` | **Paste button ONLY** — appears nowhere else |
| `--accent-on` | `#17181A` | text on amber fill |

Content-type representation (NOT a type-icon color system — dropped as noise per Tonsky "icons should differentiate"). Types represent themselves in the lead slot: `color = actual swatch` · `image = thumbnail` · `file = OS file-type icon + filename` · `link = favicon (or empty)` · `text/code = no lead mark, plain text`. No desaturated tint icons anywhere.

### 1.2 Color — Light (neutral graphite)

| Token | Value |
|---|---|
| `--bg-shell` | `#F2F3F4` |
| `--bg-raised` | `#FAFBFB` |
| `--bg-hover` | `#ECEEEF` |
| `--bg-selected` | `#E2E5E8` (neutral fill, one step above hover) |
| `--border-hairline` | `#E0E3E5` |
| `--text-primary` | `#1C1E20` |
| `--text-secondary` | `#5D6367` |
| `--text-tertiary` | `#8A9095` |
| `--accent` | `#C77D1A` (Paste button only; darker amber for AA on light) |
| `--accent-on` | `#FFFFFF` |

### 1.3 Type, spacing, form

- **Font:** system UI (SF Pro / Segoe UI) for chrome (search, meta, buttons). **Clipboard content in the list + preview is monospace** (SF Mono / Cascadia Mono) — the chosen instrument identity. Use a readable mono at generous line-height; CJK falls back to the system CJK face (true monospace CJK is impractically wide).
- **Scale:** search 15px · row content 13px/1.5 (looser leading to offset mono width) · source 12px · meta 11px.
- **Spacing:** 4px base grid (4/8/12/16/24). Row height ~44px. Row padding 12px h / 8px v.
- **Radius:** panel 14px · rows 8px · chips/buttons 6px. Panel more rounded than its children.
- **Depth:** one soft shadow + 1px inner hairline on the shell only. No per-row cards/outlines — selection carries separation.
- **Material:** shell may use subtle vibrancy at the outer edge (macOS NSVisualEffectView / Windows Acrylic); default is solid `--bg-shell`, glass is progressive enhancement with solid fallback.
- **Motion:** panel fade+scale ~120ms; selection move instant; respect Reduced Motion.
- **Selection (the hero pixel):** neutral `--bg-selected` fill only, set one clear step above `--bg-hover` so keyboard nav reads unambiguously; primary text brightens slightly. NO amber bar, NO colored block, NO `⏎` glyph on the row. The right-hand preview reflecting the selection is the second confirmation. Amber is reserved exclusively for the Paste button.
- **Lead slot (per Tonsky "icons break scanning"):** every row reserves a fixed ~28px left slot so all content text aligns on one x-edge. The slot is EMPTY for text/code, and holds the type's own representation for others (swatch / thumbnail / file-type icon / favicon). No generic/uniform type icons — a globe, a "document" glyph, etc. are noise and are omitted (leave empty instead).

## 2. Feature Convergence Audit

Reviewed FR-1..FR-14 against the invariants. The set is close to right. Findings:

### 2.1 Gaps to add (genuinely missing, low cost, high trust)

- **G-1 — Ignore sensitive/concealed pasteboard types (NEW, must-fix).** FR-13 only ignores whole *apps*. Password managers mark clipboard data as concealed/transient (macOS `org.nspasteboard.ConcealedType` / `TransientType`; Windows `ExcludeClipboardContentFromMonitorProcessing` / `CanIncludeInClipboardHistory`). Clip-Clop must skip these by default. This is table-stakes privacy every serious competitor honors; without it "private by default" is false. → add as FR-15.
- **G-2 — Keyboard navigation contract (spec gap, not new feature).** Define explicitly: ↑/↓ move selection, Enter = paste, Esc = dismiss, and reuse moves the item to top. This is implied by FR-4/5/9 but must be pinned so build is unambiguous.
- **G-3 — Paste as plain text (optional, v1.1).** A single modifier (e.g. ⌘/Ctrl+Enter) pastes stripped of formatting. High value for the keyboard-heavy target user, low cost. Not MVP-blocking; note as fast-follow.

### 2.2 Overdesign to trim / constrain

- **O-1 — Type filter (FR-7): keep, but demote.** Do NOT build a chip row competing with search. One compact control (cycles All→Text→Link→Image→File→Color) in the search bar, plus type-ahead. Search and the self-representing lead slot (swatch/thumbnail/file-icon) already carry recognition; a heavy filter chip row would be noise (violates invariant 2/3).
- **O-2 — Formatted text (FR-2): store plain + a small "RTF" badge; preserve original for paste.** Do not build a rich-text renderer in the list. Preview shows plain; paste re-emits original formatting. Avoids a mini word-processor.
- **O-3 — Color as a type: keep (cheap + delightful).** Detect `#hex`/`rgb()` in copied text and render a swatch. It is functional recognition, not decoration. But no dedicated color *filter* beyond O-1.
- **O-4 — Settings (FR-14): keep to one flat window.** No tabs-heavy prefs. Guard against settings creep — every new toggle must justify itself against "useful without opening Settings" (SM-3).

### 2.3 Confirmed correct as-is

Capture + dedup (FR-1), multi-type (FR-2), source metadata (FR-3), global shortcut (FR-4), continuous recent-first list with time as row meta not section headers (FR-5), faithful preview with no AI enrichment (FR-6), local search (FR-8), Enter/double-click paste with copy-only fallback (FR-9/10), pin floats to top of same list (FR-11), delete/clear (FR-12). Non-goals (no AI/sync/accounts/plugins/tags) hold.

## 3. Layout

### 3.1 Surfaces

- **Home = menu bar (macOS) / system tray (Windows) icon.** Not a dock/taskbar app. Right-click menu: Open, Clear History, Settings, Quit.
- **Quick Panel** = the product. Centered, floating, summoned by global shortcut over the active display. ~720×480, panel radius 14px, soft shadow. No app sidebar, no window title bar emphasis — it reads as summoned, not as a document window.
- **Settings** = one small separate window, opened rarely.

### 3.2 Quick Panel anatomy (two-pane: list + preview)

Two-pane is the honest choice here (invariant 3: heterogeneous content — images/files/color/long text need a real preview). This is convergence with the proven pattern, not imitation. Left list is dominant; right preview is progressive detail.

Lead slot is empty for text/code (text aligns on one x-edge); filled only for color/image/file/link. Monospace content. Selection = neutral fill only.

```
┌──────────────────────────────────────────────────────────────┐
│ 🔎  Search your clipboard…                        [ All  ▾ ]  │  header ~52px, full width
├───────────────────────────────┬──────────────────────────────┤
│     📌 npm run build          │                              │  pinned float on top,
│        Terminal · 2h          │   ┌────────────────────────┐ │  hairline divider under
│ ─────────────────────────────│   │                        │ │
│     git commit -m "fix auth"  │   │   PREVIEW of selected  │ │  ← selected row:
│        Terminal · 3m          │   │   (faithful render:    │ │    neutral bg-selected
│     https://example.com/…     │   │    text/img/color/     │ │    fill only, no amber,
│        Safari · 12m           │   │    file/link)          │ │    no bar, no glyph
│ ▙  design-tokens.json         │   │                        │ │
│        VS Code · 20m          │   └────────────────────────┘ │
│ ▨  #E8A33D                    │   Terminal · Text · 42 · 3m  │  quiet meta as plain
│ ▤  screenshot.png             │                              │  tertiary text, no pills
│              (list ~60%)      │        (preview ~40%)        │
├───────────────────────────────┴──────────────────────────────┤
│                                        ⏎ Paste    ⌘K Actions  │  footer ~44px, minimal
└──────────────────────────────────────────────────────────────┘
```
(▙ file-type icon · ▨ actual color swatch · ▤ image thumbnail — the only things in the lead slot; text/code rows leave it empty and align.)

### 3.3 Rules

- **Header:** search glyph + borderless input (auto-focused on open) + one compact type control at right (O-1). No chip row.
- **List (~60%):** pinned items float at top under a hairline (FR-11, no separate view). Then recent newest-first, continuous — NO Today/Yesterday section headers (time is row meta). Row = `[fixed 28px lead slot: empty for text, else swatch/thumbnail/file-icon/favicon] [1–2 line monospace snippet] … [source app · relative time]`. No per-row type icons for text. Hover = `--bg-hover`; selected = neutral `--bg-selected` fill only. List scrolls.
- **Preview (~40%):** faithful render of the selected payload only; metadata as quiet plain tertiary text below (source · type · size/chars · time) — NOT filled capsule pills. Empty query still shows recent list + preview of the top item.
- **Footer (~44px):** primary `⏎ Paste` (amber) + `⌘K Actions`. Everything secondary (Copy, Copy-only, Pin/Unpin, Delete, Paste as text, Ignore source) lives behind the ⌘K action menu — this prevents footer/action-bar bloat (the earlier prototype's mistake).
- **Keyboard (G-2):** ↑/↓ select · Enter paste+dismiss · ⌘/Ctrl+Enter paste as text (v1.1) · ⌘K actions · Esc dismiss. Optional later: 1–9 quick-select.
- **Empty state:** centered quiet line with the shortcut hint and a touch of the clip-clop cadence in copy — no illustration.
- **Reduced-transparency / Windows-solid:** shell falls back to solid `--bg-shell`; layout is identical.
