# ClipClop PRD Addendum

> **历史补充记录。** 本文中的 glass/amber、类型筛选、自动敏感内容过滤、Pin 与 direct paste 仅代表早期探索。2026-07-13 之后以 `PRODUCT.md`、PRD 顶部修订、根目录 `DESIGN.md` 和 `docs/architecture.md` 为准。

## Technical Baseline

The current product direction assumes:

- Tauri as the cross-platform desktop application shell.
- Web UI inside Tauri for the main interface.
- Rust for local application logic: clipboard history, storage, search, settings, deduplication, and privacy filters.
- Thin platform adapters for macOS and Windows where system integration is required.

This belongs in architecture, not PRD requirements, unless a product requirement depends on it directly.

## Clipboard Privacy and Encryption Assessment

Open-source comparables generally emphasize local-first privacy, ignored sources, and sensitive pasteboard-type filtering more prominently than default encrypted-at-rest storage.

- Maccy documents "secure and private" positioning, ignores confidential/temporary pasteboard types by default, and supports ignoring copy events or custom pasteboard types.
- CopyQ documents storage for text, HTML, images, and custom formats, plus ignored windows/text rules; encryption is not a prominent default README feature.
- Ditto documents local-first behavior: no login, no cloud, no telemetry. Its repository includes an `EncryptDecrypt` library, but the README does not position encrypted local history as the primary default behavior.

Recommendation for MVP:

- Do not add custom full-database encryption as a v1 blocker.
- Do require local-only storage, no cloud, no account, no telemetry, retention limits, ignored apps, sensitive pasteboard-type ignoring, and explicit clear-history controls.
- Keep the storage layer narrow enough to add encryption later if user feedback or distribution positioning requires it.

## Frontend UI Stack Assessment

Tauri is frontend-agnostic and serves static HTML/CSS/JavaScript/WASM through a WebView. Its docs recommend Vite for SPA frameworks such as React, Vue, Svelte, Solid, and also plain JavaScript or TypeScript.

Recommendation for MVP:

- Use Svelte + TypeScript + Vite for the Tauri UI.
- Avoid SvelteKit/Next/Nuxt for MVP because ClipClop does not need SSR or routing-heavy app structure.
- Avoid React unless ecosystem/team familiarity matters more than bundle/runtime minimalism.
- Avoid plain Web UI unless the UI is intentionally kept extremely small; ClipClop's preview pane, filtering, grouped list, keyboard navigation, settings, and state transitions justify a small component framework.

Rationale:

- Svelte compiles declarative components into lean JavaScript, matching the "lightweight + modern" product promise.
- The UI is stateful but not large enough to justify React's ecosystem weight.
- The product needs careful keyboard and layout behavior; a component model will keep implementation simpler than hand-managed DOM.

## Permission Onboarding Assessment

Recommendation:

- Show a first-run permission onboarding flow on macOS.
- Do not show the same permission flow on Windows; show only a short capability check/status row if needed.

macOS:

- Clipboard read/write itself should not be framed as a user-granted permission.
- Clipboard monitoring can work through pasteboard APIs.
- Direct paste into another app requires synthetic keyboard/input behavior.
- Direct paste generally requires Accessibility permission so ClipClop can control the Mac by sending paste input to the previously active app.
- Global shortcuts should use a registration API/plugin, not a low-level keyboard event tap. If ClipClop ever records arbitrary keyboard events through an event tap, that becomes an Input Monitoring concern and should be avoided for MVP.
- Screen Recording is not required for MVP because ClipClop does not inspect screen pixels.
- Automation permission is not required if ClipClop uses accessibility/input APIs rather than Apple Events to control specific apps.
- The onboarding should explain this narrowly: "Allow ClipClop to paste the item you choose into the current app."
- If permission is not granted, ClipClop should still support copy-only fallback.

Windows:

- Clipboard read/write, clipboard monitoring, global shortcuts, and synthetic paste input are available through Win32 APIs without a macOS-style user privacy authorization prompt.
- Direct paste should use normal user-mode input injection and should not require administrator rights.
- Windows User Interface Privilege Isolation can block input injection from a normal app into elevated/admin apps. Treat that as an expected limitation, not an onboarding permission.

MVP onboarding:

- macOS first run: show permission card for Accessibility, with "Grant Permission", "I will do this later", and clear copy-only fallback.
- Windows first run: skip permission gate; show the app directly.
- Both platforms: include Settings status for direct paste availability.

## Raycast Technology and UI Assessment

Confirmed from public sources:

- Raycast itself is proprietary, so its main app implementation stack is not fully public.
- Raycast's extension platform is explicitly built around React, Node.js, and TypeScript.
- The public `raycast/extensions` repository is predominantly TypeScript and documents building extensions with React.
- Raycast's UI model is a command palette with incremental search, keyboard-first actions, list/grid/details layouts, and a consistent action system.

What this means for ClipClop:

- Do not copy Raycast's extension technology choice as proof that the core Raycast app is React-based.
- Do copy the interaction model: centered command surface, fast search focus, left list, right preview/details pane, type filter, action bar, and keyboard-first confirm/copy/delete/pin actions.
- Our Svelte + TypeScript + Vite recommendation remains valid for a Tauri app because the target UI is compact, stateful, and design-system driven.
- The important product lesson from Raycast is not "React"; it is "a constrained component system with strong keyboard defaults and very little visual noise."

## Product UI Direction

ClipClop should not copy Raycast Clipboard History's exact layout. The design should start from the user's actual clipboard-reuse flow:

1. Recall roughly what was copied.
2. Narrow by search, type, source, or time.
3. Confirm the right item from preview and metadata.
4. Paste it into the current cursor location.

Design baseline:

- Use a liquid-glass visual direction for the primary theme: translucent depth, warm blur, restrained contrast, and soft highlights.
- Keep the UI cross-platform rather than macOS-only glass. Use the glass effect as a material language, not as imitation of one OS.
- Treat the Quick Panel as a focused work surface, not a command launcher clone.
- Prefer progressive detail: the list should carry enough recognition signal; the details panel should expand value only when selection needs inspection.
- Avoid noisy top chrome. The active task is finding and pasting, not navigating an app.

Layout candidates to explore:

- **Source-first rail:** a narrow left rail groups by source app/type, middle list shows items, right preview shows details. Best when source recognition is the main retrieval cue.
- **Timeline stack:** a single primary list grouped by time, with inline source/type chips and a lower expandable inspector. Best for fast keyboard flow and small windows.
- **Canvas preview:** list on the left, large preview on the right, metadata as compact floating rows. Best for mixed content including images/files.
- **Spotlight strip:** search and filters on top, horizontal type/source chips under it, list and preview below. Best for reducing the Raycast-like two-pane command feel.

Recommended next prototype direction:

- Use a **continuous history stream + inline inspector** as the default layout.
- Do not hard-group the primary list by Today/Yesterday/This Week in MVP; copied time belongs in item metadata.
- Borrow **Canvas preview** only for image/file-heavy selections.
- Keep Source-first rail as a later mode if source-based retrieval proves important.

This gives ClipClop its own product logic: content recognition first, source/type/time metadata second, paste confirmation last.

## UI Prototype Critique and Revised Direction

The first prototype set had a calmer hierarchy. The later liquid-glass prototype moved in the right material direction, but introduced too much interface noise.

Noise sources to remove:

- Persistent left navigation rail: the Quick Panel is not a full app shell.
- Duplicated action bars: actions should appear once, close to the selected item or in a restrained footer.
- Too many visible filters at once: type/source filters should not compete with search.
- Expanded item too tall by default: inspection should be available, not always dominant.
- Strong outlines around every row: glass material should use depth and selection, not boxed cards everywhere.
- Window chrome emphasis: the panel should feel summoned, not like a normal document window.

What to keep from the first prototype set:

- Calm split of list and information.
- Quiet grayscale/warm-neutral base.
- Clear selected state.
- Readable metadata.
- Minimal bottom action affordance.
- Low visual novelty.

Revised layout recommendation:

- **Primary structure:** centered Quick Panel, no app sidebar.
- **Top zone:** single search field with one compact type filter control. Secondary filters live behind a small filter button or adaptive chips after search.
- **Main zone:** continuous clipboard history list remains the dominant surface. Rows show icon/type, content snippet, source app, and copied time.
- **Inspection zone:** selected item opens a compact inline preview drawer. It should be 2-4 lines for text by default, expandable only for long content.
- **Action zone:** one restrained footer/action strip with primary action first: `Paste`. Secondary actions: Copy, Pin, Delete, More.
- **Metadata:** source/type/characters/copied time should read as quiet capsules, not a table.

Visual direction:

- Liquid glass should be the container material, not every child component.
- Use one translucent parent panel and mostly solid/soft child surfaces.
- Use warm coral/burnt-orange only for focus, selected state, and primary action.
- Avoid frosted blur behind dense text areas if it reduces contrast.
- Keep component radii moderate; only the main panel can be more rounded.

Next image generation prompt should produce:

- One single Quick Panel mockup.
- No left rail.
- No permanent two-pane Raycast layout.
- No duplicated bottom actions.
- Search + compact filter.
- Dense continuous history list + compact inline inspector.
- No Today/Yesterday/This Week section headers.
- Liquid-glass parent shell with quiet solid inner rows.

## Clipboard Content Rendering Rule

ClipClop is a clipboard manager, not a content intelligence tool. A Clipboard Item is one captured payload. The preview should render that payload faithfully and add only capture metadata.

Do:

- Show exact copied text as copied.
- Preserve line breaks and formatting where supported.
- Show image/file/link/color payloads using type-appropriate preview.
- Show source app, content type, size/count, and copied time.

Do not:

- Generate explanations.
- Generate titles.
- Summarize copied content.
- Rewrite or translate copied content.
- Infer meaning or classify beyond basic Content Type.

For a copied text item like `pco是什么`, the preview should show exactly `pco是什么` as the content. Any richer explanation would be an AI feature and is out of scope.

## Design Direction — First-Principles Derivation (supersedes glass-vs-warm framing)

This section supersedes the earlier "Product UI Direction" and "UI Prototype Critique" reasoning where they argued *from avoiding Raycast* or *toward liquid glass as the theme*. Both were style-shelf choices. The direction below is derived from the product's invariants, so it should be treated as the governing UX intent.

### Product invariants (non-negotiable facts)

1. It is an interruption tool, not a destination. Summon → one action → dismiss. It is never the main task.
2. The core cognitive act is *recognition*, not reading. Users already roughly know what they copied; they scan to re-identify one row.
3. The content is the user's and heterogeneous (text, code, URL, color, image, file). The content is the interface; any chrome with color competes with it.
4. Speed is the entire value (~150ms felt). Every layer, blur, or animation is a tax on the only thing that matters.
5. Keyboard summons, operates, dismisses. The selection state is the single most important pixel, not any button.
6. It floats over an arbitrary background on two operating systems. It must feel summoned and self-sufficient without knowing what is behind it.

### What the invariants force

- Recognition (2) → high contrast, strong left alignment, stable row rhythm, type made visually distinct (icon / monospace / color chip). Rules out low-contrast and decorative surfaces.
- Content is the interface (3) → the shell must be near-achromatic; color is allowed only for system state and content-type identity, never theming.
- Speed (4) → few layers, minimal blur, minimal motion → surfaces are mostly solid. Real-time large-area blur is a GPU cost, worst on Windows.
- Keyboard-first (5) → the selection state is a filled, high-contrast block, not a faint tint.
- Floats over anything (6) → the shell needs its own ground and edge; translucency at most decorates the outer perimeter as a "summoned" signal and must degrade to solid.

### Resulting style: a quiet neutral instrument

Not glass, not a warm mascot aesthetic. The landing point is a **near-achromatic, high-contrast, high-density, content-first tool with a single functional accent** — reference class: Linear, Raycast, a pro camera viewfinder. The interface disappears; only content and state carry color and weight.

### Why convergence with Raycast is correct, not plagiarism

Raycast looks the way it does because it is also a summoned keyboard recognition tool, pushed to the same point by the same invariants. Convergence is evidence the reasoning is right. Plagiarism is copying the skin without understanding why; deriving the same point from first principles is understanding. Identity therefore does not live in picking a different macro-style — it lives in the free variables the invariants do not pin down (below).

### Committed free-variable decisions (ClipClop's identity)

The invariants lock ~80% of the design. These are the remaining degrees of freedom, committed to defensible defaults. Each is a decision, not a constraint — revisit with real content, but do not leave them open.

- **Color temperature:** warm-neutral gray (a faint warm cast in the grays), not cool/blue-gray. Keeps the "restrained but not cold" personality and quietly separates from Raycast's cooler shell at near-zero cost.
- **Functional accent:** a single warm amber/honey hue used ONLY for selection state and the primary Paste action. Avoids the danger-red / trust-blue clichés and reads as the product's signature. Everything else stays achromatic.
- **Content-type system (the primary expressive canvas):** each Content Type gets a quiet, desaturated color + icon so it is identifiable while scanning — e.g. text = neutral, code = indigo (monospace), link = blue, color = the swatch itself, image = green, file = amber-gray. This is where ClipClop's character lives, and it is functional, not decorative.
- **Density:** medium-tight — between Maccy (very dense) and Paste (loose). Target ~8–10 rows visible without scrolling in the default panel, with enough vertical breathing room to scan comfortably.
- **Edge material:** soft shadow + a thin edge to signal "floating," with optional subtle vibrancy at the outer shell only; must fall back to solid on Windows where Mica is unreliable.
- **Radius / rhythm:** moderate corner radius; the panel may be more rounded than its inner rows. Keep it quiet — radius is minor personality, not a statement.

### Where material / glass belongs (resolving the earlier debate)

From first principles, translucent material is neither required nor forbidden — it is one optional solution to invariant 6 (feel summoned) applied to the outer edge only, and it must degrade to solid. Covering rows or preview text in glass violates invariants 2, 3, and 4, and matches the legibility retreat Apple itself made after Liquid Glass shipped too transparent. So "glass vs warm" was a category error: glass is at most an outer-shell edge toggle, not the product's style. The `tauri-apps/window-vibrancy` plugin can provide it (macOS NSVisualEffectView, Windows Mica/Acrylic), but Windows Mica is unreliable (issue #141) — treat the solid fallback as the default, glass as progressive enhancement.

### What first principles rule out (reasoning cuts both ways)

- Neumorphism / soft low-contrast surfaces — kills recognition (invariant 2).
- Skinned or themed / skeuomorphic looks — steal attention from content (invariant 3).
- Brutalism / maximalism — noise competes with scanning (invariant 2).
- Full-surface glassmorphism — kills contrast, taxes GPU, and looks different on every wallpaper (invariants 2, 4, 6).
- Brand color used broadly — collides with the user's own copied colors and content (invariant 3).
