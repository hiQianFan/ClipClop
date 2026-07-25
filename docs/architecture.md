# ClipClop architecture

[简体中文](architecture.zh-CN.md) | English

This document describes the boundaries that must remain stable as ClipClop evolves. It focuses on runtime ownership, privacy-sensitive flows, and the panel lifecycle rather than listing every source file.

## Runtime map

ClipClop has two application layers:

- `src/` is the Svelte UI. Feature code calls Tauri commands through local `api.ts` modules and owns DOM focus, keyboard interaction, rendering, and accessibility.
- `src-tauri/src/` is the Rust host. It owns the system clipboard, persistence, global shortcuts, native windows, preview integration, paste injection, logging, and updates.

Tauri commands are adapters. Business rules belong in their feature modules, and native window behavior belongs behind the `window` module.

## Panel lifecycle

The main window behaves like a transient command panel rather than a regular application window. Native focus and DOM focus are separate:

1. Rust begins a new panel generation.
2. Rust sizes and shows the native window.
3. The platform adapter requests foreground activation.
4. Rust emits `panel_shown`.
5. Svelte resets the browsing session and assigns DOM focus.

Do not emit `panel_shown` before native show and activation have been attempted. Otherwise the webview can focus an element while its native window is still in the background.

### State model

`PanelLifecycleState` is the authoritative native lifecycle:

```text
Hidden → Showing → Focused → BlurPending → Hidden
                    ↑             |
                    └─────────────┘
```

Every show starts a new generation. A pending blur token contains its generation and revision; a new show, refocus, or hide invalidates it. Startup blur is ignored until the panel has acquired focus at least once.

`PreviewState` is separate and represents only native preview activity. A native preview may temporarily take focus without causing the panel to hide.

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
