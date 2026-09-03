---
name: ClipClop
description: A quiet, keyboard-first clipboard history for macOS and Windows.
colors:
  bg-shell: "#17181a"
  bg-raised: "#1e2022"
  bg-hover: "#242628"
  bg-selected: "#2a2d30"
  hairline: "#2c2f32"
  text-1: "#ecedee"
  text-2: "#aeb3b7"
  text-3: "#8a9095"
  action: "#eceef0"
  action-hover: "#ffffff"
  action-on: "#17181a"
  danger: "#ff6b72"
  danger-fill: "#b4232f"
  danger-on: "#ffffff"
typography:
  heading:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, system-ui, sans-serif"
    fontSize: "18px"
    fontWeight: 680
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  body:
    fontFamily: "SF Mono, Cascadia Mono, ui-monospace, Menlo, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 600
    lineHeight: 1.4
  caption:
    fontFamily: "SF Mono, Cascadia Mono, ui-monospace, Menlo, monospace"
    fontSize: "10px"
    fontWeight: 400
    lineHeight: 1
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
  xl: "14px"
  pill: "999px"
spacing:
  "1": "2px"
  "2": "4px"
  "3": "6px"
  "4": "8px"
  "6": "12px"
  "8": "16px"
  "10": "20px"
  "12": "24px"
components:
  button-primary:
    backgroundColor: "{colors.action}"
    textColor: "{colors.action-on}"
    rounded: "{rounded.md}"
    padding: "7px 15px"
  button-primary-hover:
    backgroundColor: "{colors.action-hover}"
    textColor: "{colors.action-on}"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.text-2}"
    rounded: "{rounded.md}"
    padding: "7px 10px"
  button-ghost-hover:
    backgroundColor: "{colors.bg-hover}"
    textColor: "{colors.text-1}"
  button-destructive:
    backgroundColor: "{colors.danger-fill}"
    textColor: "{colors.danger-on}"
    rounded: "{rounded.md}"
    padding: "7px 10px"
  row:
    backgroundColor: "transparent"
    textColor: "{colors.text-1}"
    rounded: "{rounded.lg}"
    padding: "7px 8px"
  row-selected:
    backgroundColor: "{colors.bg-selected}"
    textColor: "{colors.text-1}"
  input-search:
    backgroundColor: "transparent"
    textColor: "{colors.text-1}"
    typography: "{typography.body}"
  menu:
    backgroundColor: "{colors.bg-raised}"
    textColor: "{colors.text-1}"
    rounded: "{rounded.lg}"
    padding: "6px"
---

# Design System: ClipClop

## 1. Overview

**Creative North Star: "The Native Utility Drawer"**

ClipClop looks and behaves like a mature, first-party desktop tool — the kind that ships inside the OS, not one bolted on top of it. It is quiet but never cold, compact but never cramped. Every surface earns its density: this is a keyboard-first tool where power users recall history with arrow keys, number keys, and Enter, so the interface prioritizes information rhythm and operational feedback over decoration.

The system is **dark-by-default, monospace-forward**. Content — clipboard snippets, file paths, metadata — is rendered in a monospaced stack because the data is code-adjacent and alignment matters; chrome and labels use the platform sans stack so the tool reads as native. Depth is conveyed almost entirely through **tonal layering** (four background steps), not shadows. Shadows appear only where an element genuinely floats above the plane: the app panel and popover menus.

This system explicitly rejects the consumer-app playbook. No marketing-style onboarding, no colored gradients, no glassmorphism, no tutorial carousels, no oversized illustrations, no gamified celebration, no nested cards, no coerced permissions. It must never read as a generic SaaS template, and it must never trade text contrast or keyboard reachability for a veneer of "premium."

**Key Characteristics:**
- Dark-first, with a hand-tuned light theme at equal parity (system-follow + manual override).
- Monospace for content, platform-sans for chrome.
- Depth via four tonal background steps; shadows reserved for true floats.
- High-density but rhythmic: a 2px spacing base and fixed px type scale.
- One near-white action color; danger red is the only other semantic accent.

## 2. Colors

A near-neutral, faintly cool graphite palette. There is no brand hue — restraint *is* the brand. The only saturated color in the entire system is danger red, and it appears only on destructive intent.

### Primary
- **Action Near-White** (`#eceef0` dark / `#1c1e20` light): The single action color, carrying the primary "Copy & Paste" button and active selection dot. In dark mode it is a near-white fill with dark ink on top; in light mode it inverts to near-black. Its scarcity is the point.

### Neutral
- **Shell** (`#17181a` dark / `#f2f3f4` light): The base panel background, furthest back in the tonal stack.
- **Raised** (`#1e2022` dark / `#fafbfb` light): Menus, cards, inputs — one step forward.
- **Hover** (`#242628` dark / `#eceeef` light): Transient row/button hover.
- **Selected** (`#2a2d30` dark / `#e2e5e8` light): Persistent selection state.
- **Hairline** (`#2c2f32` dark / `#d9dde0` light): All 1px dividers and borders. Never thicker than 1px.
- **Ink 1 / 2 / 3** (`#ecedee` / `#aeb3b7` / `#8a9095` dark): Three text levels — primary content, secondary labels, tertiary meta. All three clear 4.5:1 on Shell.

### Tertiary (semantic only)
- **Danger** (`#ff6b72` dark / `#b4232f` light) and **Danger Fill** (`#b4232f`): Destructive text and destructive button fill. Never decorative.

### Named Rules
**The One Action Rule.** Exactly one action color exists. If a second element competes with the Copy button for the eye, one of them is wrong.

**The Hairline Rule.** Borders and dividers are always 1px and always `hairline`. A heavier or colored border is never the answer — reach for a tonal background step instead.

**The Contrast Floor Rule.** Body text uses Ink 1 or Ink 2; tertiary meta (Ink 3) is reserved for genuinely secondary data. Never drop content below 4.5:1 for elegance.

## 3. Typography

**Content Font:** SF Mono / Cascadia Mono / ui-monospace (with Menlo fallback)
**Chrome Font:** system-ui stack — -apple-system, BlinkMacSystemFont, Segoe UI

**Character:** A deliberate two-family split by *role*, not by decoration. Monospace carries anything the user copied or that must align (snippets, paths, keycaps, counts, timestamps); the platform sans carries anything that is chrome (headings, nav, toggle labels). This is the opposite of a display/body pairing — both families are utilitarian, and the contrast axis is content-vs-chrome.

The scale is **fixed px, not fluid** — correct for a product UI viewed at consistent DPI. Steps are close at the bottom (10 / 11 / 12 / 13 px) because at small sizes a single pixel is a real hierarchy step in a dense tool; they open up only for the single heading role.

### Hierarchy
- **Heading** (sans, 680, 18px, lh 1.3, tracking −0.01em): The one heading size — settings section titles (h1) and the onboarding hero share it.
- **Emphasis** (sans, 600, 14px): Window and update-card titles — the one mid-weight step.
- **Body** (mono, 400, 13px, lh 1.5): Clipboard snippets, search input, preview text. Preview prose runs at lh 1.65.
- **UI Label** (sans, 600, 12px, lh 1.4): The workhorse — nav, buttons, toggle labels, menu items.
- **Meta** (mono, 400, 11px): Secondary meta, hints, file sub-rows.
- **Caption** (mono, 400, 10px, lh 1): Timestamps, metric labels, keycaps.

### Named Rules
**The Content-Is-Mono Rule.** If the text is something the user copied, or a path, count, or key, it is monospace. If it is a label the app authored, it is sans. No exceptions.

**The No-Middle-Dot Rule.** Do not use `·` as a visual separator in product UI. Separate metadata with spacing, alignment, line breaks, or distinct text roles instead.

**The Fixed-Scale Rule.** No `clamp()` type in the app UI. A heading that shrinks inside a narrow pane looks broken, not responsive.

## 4. Elevation

This system is **flat by tonal layering, not by shadow**. Depth on the working plane is expressed through four background steps (Shell → Raised → Hover → Selected). A surface changes its tonal step to signal state; it does not grow a shadow. Shadows are reserved exclusively for elements that genuinely leave the plane.

### Shadow Vocabulary
- **Panel** (`box-shadow: 0 2px 5px rgba(0,0,0,.18), 0 12px 24px rgba(0,0,0,.30)`): The floating app window against the desktop. The only ambient shadow.
- **Menu** (`box-shadow: 0 6px 8px rgba(0,0,0,.25)`): Popover and action menus. Tighter and shorter — these float just above the panel, not the desktop.

(Both soften in the light theme to `rgba(24,28,32,…)` at lower opacity.)

### Named Rules
**The Flat-Plane Rule.** Rows, chips, inputs, and buttons never cast shadows. State is a tonal step, not a lift. If you reach for `box-shadow` on an in-plane element, you are wrong — change the background token instead.

## 5. Components

Every interactive element is quiet at rest and responds through tonal shift. Corners follow the radius scale: `sm` (4px) for small controls and thumbnails, `md` (6px) for buttons/fields/menu-items, `lg` (8px) for rows and menus, `xl` (14px) for the app panel, `pill` for toggles.

### Buttons
- **Shape:** Softly rounded (`md`, 6px).
- **Primary:** `action` fill, `action-on` ink, weight 650, padding 7px 15px. The one filled button on any surface (the Copy action).
- **Ghost:** Transparent, `text-2` ink, padding 7px 10px. Hover fills `bg-hover` and lifts ink to `text-1`. Active drops to `bg-selected`.
- **Destructive:** `danger-fill` bg, `danger-on` ink, weight 600. Confined to delete actions.
- **Disabled:** opacity 0.45; no hover.

### Rows (signature component)
- **Shape:** `lg` (8px), full-width.
- **Structure:** number key · lead swatch/thumb (28px) · monospace snippet · optional disclosure chevron. Min-height 44px.
- **States:** hover → `bg-hover`; selected (list unfocused) → 55%-mixed `bg-selected`; selected (list focused) → solid `bg-selected`. Selection is tonal, never a border or stripe.

### Inputs / Fields
- **Search:** Borderless, transparent, mono 13px, with a leading search glyph and a `/` keycap hint. Lives in its own hairline-bottomed bar.
- **Select / recording field:** `bg-raised`, 1px hairline, `md` radius, focus ring 2px `text-1` at 2px offset.

### Toggle (switch)
- 36×20 `pill` track, 16px knob, transitions at `--dur-fast` (140ms) ease-out. Off = `bg-selected`; on = `action`. Honors `forced-colors` and `prefers-reduced-motion`.

### Menus
- `bg-raised`, 1px hairline, `lg` radius, 6px padding, `menu` shadow. Items are `md`-radius ghost rows with a right-aligned keycap. `z-index: var(--z-menu)`.

### Keyboard shortcuts
- Render every shortcut through `ShortcutHint`; do not style raw `<kbd>` elements in feature components.
- **Menu accelerator:** right-aligned, borderless `text-3`, system sans 12/500. macOS symbols use a 3px gap without `+`; Windows combinations use a low-contrast `+` separator.
- **Compact hint:** a single subtle hairline container for buttons, search hints, and inline teaching.
- **Teaching keycaps:** settings and onboarding render each key as its own 22px-high cap with a 4px gap and no `+` signs.
- Shortcut labels use the platform system font rather than the content mono stack so macOS modifier glyphs and Windows key names share stable metrics.
- Destructive command labels may use `danger`; their shortcut remains neutral `text-3` because it is supporting information, not the action's semantic label.

### Settings Row (the label/control contract)

Every row in Settings — toggles, selects, buttons, the update header, shortcut rows — follows **one horizontal contract** so the whole panel reads as a single ruled list, not a pile of ad-hoc rows. This is the layout law for `SettingsView`.

**Anatomy.** A row is a horizontal flex with two zones and a `24px` gap, divided by a 1px `hairline` bottom border:

```
┌──────────────────────────────────────────────┬────────────────┐
│  Text zone (flex:1 1 auto; min-width:0)        │  Action zone   │
│  ┌ strong: UI Label 12/600 (sans)              │  (flex:none)   │
│  └ small:  Meta 12 text-3 — wraps freely       │  right-aligned │
└──────────────────────────────────────────────┴────────────────┘
        grows / shrinks / wraps                    protected
```

- **Text zone** owns all slack: `flex:1 1 auto; min-width:0`. A long description wraps or ellipsizes *inside its own column* — it never steals width from the control.
- **Action zone** is inviolable: `flex:none`. It sizes to its content and never compresses. This is the rule that keeps a "管理" button from collapsing into stacked glyphs when the help text runs long.
- Rows are min-height `68px` (control rows) or `56px` (shortcut reference rows), vertically centered, with `padding-block:12px` so a wrapped description keeps breathing room off the hairline instead of hugging it.
- **Keep the text short enough to stay one line.** The two-zone contract survives wrapping, but a description that runs 3+ lines is a copy problem, not a layout one — trim it. Row labels pair with a terse ghost action ("管理"), not a restated sentence ("打开完整磁盘访问…").

**Action-zone vocabulary** — a row's action is exactly one of these, each `flex:none`:

| Control | When | Sizing |
|---|---|---|
| Ghost button | Navigate out / open system settings / one-shot action ("管理", "快速入门", "清空所有", "检查") | `min-height:32px; padding:0 12px; white-space:nowrap` |
| Toggle switch | Boolean app state saved with the panel | 36×20 pill (see Toggle) |
| Select | Choice from a fixed set | `min-width:116px` |
| Control group | Multiple related controls (label + switch + button in the update header; keycap + record + restore in shortcuts) | inline flex, `gap:8px`, right-aligned |

**Named Rules**

- **The Two-Zone Rule.** Text flexes, action is `flex:none`. Description length is the text zone's problem, never the control's. If a button wraps or shrinks, the row is missing this contract.
- **The One-Button-Size Rule.** Every action *button* across every Settings section is the same ghost button: `min-height:32px`, `padding:0 12px`, no border. There is no second button size and no bordered variant — the update-check button is not special. (The single filled `primary` lives only in the footer save action.)
- **The Right-Rail Rule.** Action zones align to the row's right edge, forming one continuous vertical rail down the panel. Don't center or left-float a control.
- **Density-Not-Ratio.** Do **not** hard-code a text/action width ratio (60/40, etc.). A fixed ratio strands whitespace beside short buttons. Elastic text + protected action gives correct alignment at every label length.

## 6. Do's and Don'ts

### Do:
- **Do** keep exactly one filled `action` button per surface; everything else is ghost. (The One Action Rule.)
- **Do** express state through the four tonal background steps — hover, selected, active are background changes.
- **Do** use monospace for any copied content, path, count, or key; system-sans for authored chrome.
- **Do** keep every divider and border at 1px `hairline`.
- **Do** teach the keyboard with the appropriate `ShortcutHint` variant: accelerator in menus, compact in controls, keycaps in instructional surfaces.
- **Do** honor `prefers-reduced-motion` and `forced-colors` on every animated or bordered control.
- **Do** keep motion in the 140–250ms range (`--dur-fast/mid/slow`), conveying state only.
- **Do** give every Settings row the two-zone contract: elastic text (`flex:1;min-width:0`), protected action (`flex:none`). (The Two-Zone Rule.)

### Don't:
- **Don't** add colored gradients, glassmorphism, or blur as decoration — the brand is restraint. (PRODUCT.md anti-reference.)
- **Don't** build marketing-style onboarding, tutorial carousels, oversized illustrations, or gamified celebration.
- **Don't** nest cards, or use a card where a tonal row would do.
- **Don't** use `border-left`/`border-right` > 1px as a colored accent stripe — never intentional here.
- **Don't** introduce a second accent hue; danger red is the only saturated color, and only for destructive intent.
- **Don't** drop body text below 4.5:1 contrast, or use `text-3` for primary content, for the sake of "elegance."
- **Don't** cast shadows on in-plane elements (rows, chips, inputs, buttons). (The Flat-Plane Rule.)
- **Don't** use `clamp()`/fluid type in the app UI, or introduce off-scale radius values (5/7/9/10px) — snap to the `rounded` scale.
- **Don't** let a Settings row's description compress its control, give an action button a border or a second size, or hard-code a text/action width ratio. (Settings Row rules.)
