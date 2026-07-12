---
title: ClipClop
status: draft
created: 2026-07-10
updated: 2026-07-10
---

# PRD: ClipClop

> **2026-07-13 MVP 修订（优先于下文旧章节）**
>
> - 产品品牌统一为 `ClipClop`。
> - v1 默认捕获平台允许读取的全部受支持 flavor，不因 concealed/transient 标记或内容语义自动过滤。
> - 用户通过暂停记录、忽略指定来源应用、删除、清空和保留期限控制数据。
> - v1 采用 copy-only：Enter/双击将原始 flavor 写回系统剪贴板并关闭面板，不模拟直接粘贴，不申请 Accessibility 权限。
> - 默认保留格式复制；`⌘⇧C` / `Ctrl+Shift+C` 和操作菜单支持复制为纯文本。
> - Pin、类型筛选和 direct paste 延后到用户需求得到验证后；不属于 v1。
> - UI、颜色和布局以根目录 `DESIGN.md` 为准；技术边界以 `docs/architecture.md` 为准。

## 0. Document Purpose

This PRD defines the product scope for ClipClop so downstream UX, architecture, and implementation work can proceed from a stable product boundary. It builds on:

- `outputs/brainstorming/brainstorming-session-2026-06-15-1358.md`
- `outputs/research/clipboard-manager-market-research-2026-07-05.md`

## 1. Vision

ClipClop is a modern, lightweight, ready-out-of-the-box clipboard history tool for macOS and Windows.

The product helps users quickly recover and reuse recently copied content without turning clipboard management into a heavy workflow system. The core value is simple: copied text stays one shortcut away, searchable, private, and fast.

ClipClop intentionally avoids becoming an AI clipboard, knowledge base, note-taking system, automation platform, or team collaboration tool in v1.

The intended launch posture is a public, free desktop utility. The product should feel closer to Raycast Clipboard History in clarity and speed: a searchable history list, a fast preview/details area, source metadata, content metadata, and keyboard-first reuse.

## 2. Target User

### 2.1 Jobs To Be Done

- Recover something copied recently without switching back through apps.
- Search copied text quickly from the keyboard.
- Browse clipboard history by type and recency.
- Inspect where an item came from and what kind of content it is.
- Paste a selected historical item directly into the current app.
- Reuse frequent snippets without maintaining a full snippet manager.
- Keep clipboard history local and private by default.
- Use the tool immediately after installation with little or no setup.

### 2.2 Key User Journeys

- **UJ-1. Developer reuses a copied command without leaving the terminal.**
  A developer copies several commands and URLs while working across browser and terminal. In the terminal, they open ClipClop with the global shortcut, type a few characters, select the previous command, and press Enter. ClipClop places that item back on the system clipboard and pastes it into the terminal. The developer continues without returning to the original source.

- **UJ-2. Writer confirms a copied paragraph before pasting.**
  A writer opens the Quick Panel, selects a recent text item, reviews the preview, checks the source app and basic content metadata, then confirms. ClipClop pastes the selected content into the active editor and updates the system clipboard to that item.

- **UJ-3. User filters mixed clipboard history by content type.**
  A user has copied text, links, images, and files during the day. They open ClipClop, filter to the needed type, find the item by recency or search, preview it, then paste it into the current app.

## 3. Glossary

- **Clipboard History** — The local list of captured clipboard items.
- **Clipboard Item** — One captured clipboard entry.
- **Pinned Item** — A Clipboard Item fixed above normal recent history.
- **Quick Panel** — The keyboard-invoked ClipClop surface for browsing, searching, and reusing Clipboard Items.
- **Ignored App** — An application whose copied content ClipClop does not store.
- **Source App** — The application active when a Clipboard Item was captured.
- **Content Type** — The primary detected kind of a Clipboard Item, such as text, formatted text, link, image, file, or color.
- **Preview Pane** — The area that displays the selected Clipboard Item content and metadata before reuse.

## 4. Features

### 4.1 Clipboard Capture

**Description:** ClipClop captures clipboard changes while running in the background and stores them locally as Clipboard Items. MVP should support common clipboard types, not text only.

**Functional Requirements:**

#### FR-1: Capture clipboard items

ClipClop can capture new Clipboard Items when the system clipboard changes. Realizes UJ-1, UJ-2, UJ-3.

**Consequences (testable):**
- Copying supported content creates a new Clipboard Item.
- Re-copying identical content does not create noisy duplicates within a short deduplication window.
- Captured items persist after app restart.

#### FR-2: Capture multiple content types

ClipClop can capture common Content Types including plain text, formatted text, links, images, files, and colors where the platform exposes them reliably. Realizes UJ-3.

**Consequences (testable):**
- Each item has a Content Type.
- Users can filter history by Content Type.
- Unsupported clipboard payloads fail silently without breaking capture of later items.

#### FR-3: Capture source metadata

ClipClop records the Source App and capture time for each Clipboard Item when available. Realizes UJ-2, UJ-3.

**Consequences (testable):**
- The Preview Pane shows Source App for captured items when available.
- Items captured without Source App metadata still remain usable.

### 4.2 Quick Panel

**Description:** The Quick Panel is the main product surface. It opens from a global shortcut, supports immediate typing to search, shows grouped recent history, and includes a Preview Pane with content and metadata.

**Functional Requirements:**

#### FR-4: Open Quick Panel with global shortcut

Users can open ClipClop from any app with a configurable global shortcut. Realizes UJ-1, UJ-2, UJ-3.

**Consequences (testable):**
- The Quick Panel opens over the current workspace.
- Search input is focused immediately.
- The shortcut can be changed in Settings.

#### FR-5: Browse continuous history

Users can browse Clipboard History as a continuous recent-first list. Realizes UJ-3.

**Consequences (testable):**
- Recent items are ordered newest-first.
- Copied time is shown as row metadata, not as primary section grouping.
- Selection updates the Preview Pane.

#### FR-6: Preview selected item

Users can view selected Clipboard Item content and metadata before reuse. Realizes UJ-2.

**Consequences (testable):**
- Preview Pane shows the original Clipboard Item payload, Source App, Content Type, character count for text-like items, and capture time.
- ClipClop does not summarize, explain, rewrite, classify, title, or enrich copied content with generated meaning.
- Large content is previewed without freezing the panel.

#### FR-7: Filter by content type

Users can filter Clipboard History by Content Type. Realizes UJ-3.

**Consequences (testable):**
- All Types shows every supported item.
- Selecting a type hides unrelated items.

### 4.3 Search

**Description:** Search narrows Clipboard History locally and quickly. Search should prioritize recent exact and partial matches before older matches.

**Functional Requirements:**

#### FR-8: Search clipboard history

Users can search Clipboard History from the Quick Panel. Realizes UJ-1, UJ-2.

**Consequences (testable):**
- Typing filters visible items without manual submit.
- Search matches text-like content and metadata where useful.
- Empty query restores recent history.

### 4.4 Reuse and Paste

**Description:** Selecting an item and confirming should put that item back onto the system clipboard and paste it into the active app.

**Functional Requirements:**

#### FR-9: Confirm selected item to paste

Users can press Enter or double-click to reuse the selected Clipboard Item. Realizes UJ-1, UJ-2, UJ-3.

**Consequences (testable):**
- The selected item becomes the current system clipboard value.
- ClipClop pastes it into the previously active app at the active cursor location.
- If direct paste is blocked by OS permissions, the item is still copied and the user receives clear permission guidance.

#### FR-10: Support copy-only fallback

Users can copy a selected item without automatic paste. Realizes UJ-2.

**Consequences (testable):**
- A copy-only action updates the system clipboard but does not synthesize paste input.
- This remains available even when direct paste is enabled.

### 4.5 Organization and Cleanup

**Description:** ClipClop supports the minimum organization needed for daily use: pin, delete, clear, and ignore sources.

**Functional Requirements:**

#### FR-11: Pin items

Users can pin and unpin Clipboard Items. Realizes UJ-1.

**Consequences (testable):**
- Pinned Items remain available above normal recent history.
- Unpinning returns the item to normal history ordering.

#### FR-12: Delete and clear history

Users can delete individual Clipboard Items and clear Clipboard History. Realizes UJ-2.

**Consequences (testable):**
- Deleting an item removes it from search and browse results.
- Clearing history removes non-pinned items after confirmation.

#### FR-13: Ignore apps

Users can exclude selected apps from Clipboard History capture. Realizes UJ-2.

**Consequences (testable):**
- Content copied while an Ignored App is active is not stored.
- Users can add and remove Ignored Apps in Settings.

#### FR-15: Ignore sensitive/concealed clipboard content

ClipClop does not store clipboard payloads that the source marks as concealed or transient (e.g. macOS `org.nspasteboard.ConcealedType` / `TransientType`; Windows clipboard-history exclusion flags). This makes "private by default" true for password managers and secure fields without user configuration. Realizes the privacy principle behind UJ-2.

**Consequences (testable):**
- Content copied from a password manager marked concealed is not captured.
- This filtering is on by default and requires no setup.
- Non-sensitive content copied later still captures normally.

### 4.6 Settings

**Description:** Settings should stay minimal and focused on making the default product safe and usable.

**Functional Requirements:**

#### FR-14: Configure core behavior

Users can configure global shortcut, launch at login, history retention, direct-paste behavior, and Ignored Apps.

**Consequences (testable):**
- Changes persist after restart.
- Defaults allow useful operation without opening Settings.

## 5. Non-Goals

- AI rewriting, summarization, translation, or classification.
- Cloud sync or multi-device sync.
- User accounts.
- Team sharing or collaboration.
- Plugin or scripting system.
- Knowledge base, note-taking, or second-brain workflows.
- Complex folder or tag taxonomy in MVP.
- Analytics or telemetry that uploads clipboard-derived content.

## 6. MVP Scope

### 6.1 In Scope

- macOS and Windows desktop app.
- Clipboard history for common content types.
- Global shortcut to open the Quick Panel.
- Local search.
- Preview Pane with content and metadata.
- Source App tracking when available.
- Copy or paste selected item, with Enter or double-click mapped to direct paste by default.
- Content type filtering.
- Pin and unpin items.
- Delete individual items.
- Clear all history.
- Local storage.
- Ignored App list.
- System light/dark theme support.

### 6.2 Out of Scope for MVP

- Cloud sync.
- Accounts.
- AI features.
- Plugin system.
- Complex organization with folders or tags.
- Team sharing.
- Full automation/workflow builder.
- Cross-device clipboard sharing.

## 7. Success Metrics

**Primary**

- **SM-1:** First successful reuse time — a new user can install, copy content, open ClipClop, and paste a historical item within 60 seconds. Validates FR-4, FR-8, FR-9.
- **SM-2:** Quick Panel responsiveness — opening the Quick Panel and filtering recent history feels instant on normal hardware. Validates FR-4, FR-8.
- **SM-3:** Default usefulness — a user can get value without changing Settings. Validates FR-1, FR-4, FR-9, FR-14.

**Secondary**

- **SM-4:** Source clarity — users can identify where a selected item came from when Source App metadata is available. Validates FR-3, FR-6.
- **SM-5:** Mixed-content usefulness — users can find non-text items through type filtering. Validates FR-2, FR-7.

**Counter-metrics**

- **SM-C1:** Feature count is not success. Adding AI, sync, plugins, or complex taxonomy should not be counted as progress for MVP.
- **SM-C2:** Capturing more data is not success if it weakens privacy expectations or makes the app feel heavy.

## 8. Open Questions

1. Which clipboard types are mandatory for public MVP beyond text, formatted text, links, images, files, and colors?
2. What is the target implementation stack for v1 UI: React, Svelte, Vue, or plain Web UI inside Tauri?
3. Should clipboard data be encrypted at rest for MVP?

## 9. Assumptions Index

- Public launch MVP scope.
- ClipClop is a free tool for v1.
- Common non-text clipboard types should be included in MVP.
- Enter or double-click confirms a selected item, updates the system clipboard, and pastes into the active cursor location.
