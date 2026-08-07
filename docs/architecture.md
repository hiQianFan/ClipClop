# ClipClop architecture

[简体中文](architecture.zh-CN.md) | English

This document describes the boundaries that must remain stable as ClipClop evolves. It focuses on runtime ownership, privacy-sensitive flows, and the panel lifecycle rather than listing every source file.

## Runtime map

ClipClop has two application layers:

- `src/` is the Svelte UI. Feature code calls Tauri commands through local `api.ts` modules and owns DOM focus, keyboard interaction, rendering, and accessibility.
- `src-tauri/src/` is the Rust host. It owns the system clipboard, persistence, global shortcuts, native windows, preview integration, paste injection, logging, and updates.

Tauri commands are adapters. Business rules belong in their feature modules, and native window behavior belongs behind the `window` module.

History UI state is split by ownership: `HistorySession` owns the list, pagination, and selection; `PreviewSession` owns resource URLs, thumbnails, caches, debounce, and request invalidation versions. `HistoryWorkspace` orchestrates their call order and continues to own DOM focus, keyboard routing, and the multi-file `fileIndex` cursor. A session must not reach back into component state, and runtime state must not have duplicate owners.

The Rust host keeps concrete services; a single SQLite database or single platform implementation does not justify Repository, Factory, or DI interfaces. `AssetService` owns resources and thumbnails rendered in the webview. `ExternalPreviewService` owns Quick Look-style external preview and its temporary-file lifecycle. They share only a cheap cloned `HistoryService` handle. Platform paste code lives in platform submodules under `paste`; settings models, hotkey rules, and persistence service live in separate `settings` submodules. Storage migration and settings queries have separate implementation files while remaining methods on the one concrete `Database` type.

## Panel lifecycle

The main window behaves like a transient command panel rather than a regular application window. Native focus and DOM focus are separate:

1. Rust begins a new panel generation.
2. Rust sizes and shows the native window.
3. The platform adapter requests foreground activation.
4. Rust emits `panel_shown`.
5. Svelte handles `panel_shown` and assigns DOM focus.

Do not emit `panel_shown` before native show and activation have been attempted. Otherwise the webview can focus an element while its native window is still in the background.

On `panel_shown` the frontend does not blindly reset. Settings and onboarding are deliberate modes and are preserved across a summon (the history session underneath stays live via `history_changed`, so it is current on exit). Within history, the default is a fresh browsing session — jump to page 1, select the newest item, clear search — because the core loop is summon-and-paste. The `restore_browse_position` setting (off by default) instead resumes the last page, selection and search.

### State model

`PanelLifecycleState` is the authoritative native lifecycle:

```text
Hidden → Showing → Focused → BlurPending → Hidden
                    ↑             |
                    └─────────────┘
```

Every show starts a new generation. A pending blur token contains its generation and revision; a new show, refocus, or hide invalidates it. Startup blur is ignored until the panel has acquired focus at least once.

`PreviewState` is separate and represents only native preview activity. A native preview may temporarily take focus without causing the panel to hide.

### Keyboard command priority

DOM focus selects an input context; it must not disable window commands. The Svelte Workspace
owns one narrow window router with this priority: an already-handled event stops; panel
dismissal (`Command/Ctrl+W`) is focus-independent; Escape pops one active layer; focused
controls then own native keys; Browse owns list arrows and actions only in its context. Do not
move list navigation into the window router or duplicate panel commands in individual controls.

### Required invariants

- Never call `WebviewWindow::hide()` outside `window::hide_panel`.
- Every hide has a `HideReason`: blur, Escape, paste, or shortcut.
- Never combine preview state and panel lifecycle state.
- Never use multiple independent atomics as a logical lifecycle snapshot.
- Native focus events update `PanelLifecycleState`; frontend DOM focus does not replace native focus verification.
- Delayed work must carry a generation/revision token and re-check state on the main thread before changing the window.

## Platform adapters

Platform-specific behavior stays behind small adapters:

- `window/windows.rs` owns Win32 foreground activation.
- `window/macos.rs` owns NSPanel activation and Quick Look integration.
- `window.rs` owns platform-neutral orchestration and sizing.
- `window/lifecycle.rs` is a pure state machine and should remain unit-testable without a window.

### Windows focus policy

The Windows adapter first uses normal foreground APIs and verifies ownership with `GetForegroundWindow`. It must not use Tauri/tao's synthetic-Alt fallback or `AttachThreadInput`; both have caused event-loop re-entrancy or deadlock failures.

A foreground-lock timeout adjustment exists only as a compatibility fallback. It is restored by an RAII guard. Keep the fallback isolated, log its outcome without clipboard data, and remove it only after real-device evidence shows it is unnecessary.

### macOS preview policy

Quick Look is allowed to take focus without hiding the underlying panel. Explicit panel hide also closes Quick Look and clears preview activity.

## Logging

Diagnostic logs contain operational events and error text, never clipboard payloads or preview contents.

Windows GUI processes write to the per-app log file only. Do not add a Windows stderr target: a `tauri dev` child can outlive its terminal, and fern panics when it writes to the resulting broken pipe. Other platforms may also write to stderr for development.

## Persistence and history lifecycle

Schema changes increment `SCHEMA_VERSION` and migrate every supported released schema explicitly. Schema v5 preserves immutable creation time separately from last-used time; retention by age follows last-used time when recently used items are configured to move to the top. Downgrading a migrated database to `0.1.x` is unsupported.

History limits are enforced at capture and settings-update boundaries. The time and item-count limits are independent, and enabling both applies both. First-run quick start content is a fixed set of built-in examples and local resources; it never reads real clipboard history.

History deletion cleans external preview caches before committing the database deletion. A cache cleanup failure must preserve the database row so the operation can be retried and cannot leave a sensitive orphan file without a persistent identity. This ordering is a required behavior, not an interchangeable implementation detail.

## Verification gates

All changes must pass the automated checks in `CONTRIBUTING.md`. Changes involving focus, keyboard behavior, paste, preview, or panel lifecycle also require real-device verification:

1. Cold launch and type immediately.
2. Navigate with arrow keys immediately after launch.
3. Repeatedly show and hide with the global shortcut.
4. Open a second instance and confirm the existing process activates.
5. Switch away and back.
6. Paste, then summon the panel again.
7. Let the `tauri dev` parent terminal end and continue using the app.
8. Confirm the foreground window belongs to ClipClop and the log has no panic.
9. On macOS, open and close Quick Look and confirm selection is preserved.
10. When available, repeat on multiple monitors and scale factors.

Use synthetic clipboard data and do not attach private paths or clipboard contents to test evidence.

## Deliberate follow-ups

These are evolutionary improvements, not reasons to bypass the current boundaries:

- Measure how often Windows needs the foreground-lock fallback before deciding whether to remove it.
- Replace the short blocking debounce task only if focus churn shows measurable runtime cost.
- Move sizing into its own module if additional window types or sizing policies appear.
- Introduce an explicit `panel_activated` frontend event only if real devices show that `panel_shown` after activation is insufficient.
- Design separate lifecycle instances before adding another native window; do not reuse the singleton main-panel state.
