---
id: SPEC-refactor-frontend-composition
companions:
  - design.md
  - specs/frontend-composition/spec.md
sources:
  - ../../../_bmad-output/frontend-architecture-review-2026-08-30.md
---

> **Canonical contract.** This SPEC and its companions define the complete frontend composition refactor. The source report remains traceability input; implementation follows the corrected boundaries in this change.

# Refactor frontend composition

## Why

ClipClop already has healthy feature and logic boundaries, but Settings and History view orchestrators have accumulated unrelated templates and CSS. A recent generated-CSS failure dropped the entire Settings stylesheet, showing that view composition and style ownership now create reliability and review risk.

## Capabilities

- id: CAP-1
  intent: Maintainers can determine one owner for frontend state, host access, DOM focus, presentation, and component CSS.
  success: The English and Chinese architecture documents state the same feature-first dependency and style-ownership rules, and implementation conforms to them.
- id: CAP-2
  intent: Maintainers can change independent Settings workflows without editing one monolithic view and style block.
  success: General platform integration, shortcuts, update status, and release notes have coherent component boundaries while Settings load/save/rollback and navigation remain single-owned.
- id: CAP-3
  intent: Maintainers can change History titlebar/menu and actionbar/confirmation behavior independently from session and focus orchestration.
  success: Those views own their markup and CSS while HistoryWorkspace retains session assembly, view lifecycle, DOM focus, and keyboard context.
- id: CAP-4
  intent: Structural frontend changes remain behavior-frozen and detectable by automated checks.
  success: Characterization tests, type checks, unit/component tests, and production builds pass without CSS generation warnings after every migration phase.
- id: CAP-5
  intent: Users encounter text-selection behavior only where editing or copying content is intentional.
  success: Static UI uses the default pointer and cannot be selected, while text-editing controls and the right-side text/link body preview retain native selection and copy.

## Constraints

- Preserve user-visible layout, wording, keyboard/focus, accessibility, updater, preview, IPC, and platform behavior except for the explicit desktop text-selection policy.
- Continue using Svelte 5, TypeScript, existing design tokens, native simple controls, and Bits UI for matching complex controls.
- Keep feature-first folders and existing session/store/logic ownership; presentation components do not call raw Tauri IPC.
- CSS moves with its DOM owner; `:global()` requires a real component or dynamic-content boundary.
- Shared primitives require three equivalent consumers and net maintenance reduction.

## Non-goals

- No full frontend DDD/Clean Architecture, Repository/UseCase/DI/event-bus framework, new state library, or speculative UI kit.
- No universal Row/Button/Keycap abstraction, broad token conversion, visual redesign, backend change, or line-count target.
- No suppression of selection inside text-editing controls or the right-side text/link body preview.
- No rewrite of existing HistorySession, PreviewSession, updater store, presentation, keyboard, pager, or API contracts.

## Success signal

Settings and History can be evolved through narrow, tested view boundaries without duplicated state or global style coupling, while macOS and Windows behavior remains indistinguishable from the pre-refactor build.

## Assumptions

- The current History feature boundary and DOM-free sessions are the baseline to preserve, not architecture debt to redesign.
- Root `DESIGN.md` remains the product-wide design authority; dated `_bmad-output` design files remain change-specific artifacts.
