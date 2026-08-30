## ADDED Requirements

### Requirement: Feature-first frontend boundaries

Frontend code SHALL remain organized by product feature. Each feature SHALL own its transport adapters, state/logic, view components, and tests without introducing mirrored DDD/Clean Architecture directory layers.

#### Scenario: A feature behavior changes

- **WHEN** History, Settings, Updater, or Onboarding behavior changes
- **THEN** its implementation remains in the owning feature directory
- **AND** no generic Repository, UseCase class, DI container, or event bus is introduced solely to route the change

### Requirement: One-directional frontend composition

Routes SHALL compose feature orchestrators; orchestrators MAY own session assembly, DOM focus, view lifecycle, and cross-child commands; presentation components SHALL receive data and emit narrow callbacks; state/logic SHALL reach the host only through the feature `api.ts` adapter.

#### Scenario: A presentation component initiates an action

- **WHEN** a Settings or History child component receives user input
- **THEN** it invokes a supplied callback or an explicitly owned feature action
- **AND** it does not call raw Tauri `invoke()`

#### Scenario: Focus is coordinated

- **WHEN** a panel lifecycle or interaction transition requires DOM focus
- **THEN** the owning feature orchestrator performs that side effect
- **AND** session/store modules remain DOM-free

### Requirement: Responsibility-driven component extraction

A component SHALL be extracted only when it has an independent axis of change, isolates a locally complex interaction, or creates a meaningful test boundary. Source line count alone SHALL NOT require extraction, and a wrapper with no behavior or ownership SHALL NOT be introduced.

#### Scenario: History views are split

- **WHEN** titlebar/application-menu or actionbar/confirmation behavior is moved out of HistoryWorkspace
- **THEN** the new component owns its markup, local interaction, and CSS
- **AND** HistoryWorkspace retains HistorySession/PreviewSession assembly, DOM focus, keyboard context, and view lifecycle

#### Scenario: Settings views are split

- **WHEN** General, shortcut, update, or release-note behavior is moved out of SettingsView
- **THEN** each new component owns one coherent workflow
- **AND** SettingsView retains settings load/save/rollback, Tabs, close lifecycle, and simple sections without independent behavior

### Requirement: Single-owner reactive state

Long-lived state SHALL have one authoritative owner. Child components SHALL NOT mirror Settings, HistorySession, PreviewSession, or updater-store state merely to cross a component boundary.

#### Scenario: Update UI is extracted

- **WHEN** update status and actions move into a child component
- **THEN** the existing updater store remains the async lifecycle owner
- **AND** no duplicate download/install state machine is created

#### Scenario: Controlled Bits UI state is extracted

- **WHEN** a menu, Tabs, or AlertDialog moves into a child component
- **THEN** its controlled open/value state has one owner
- **AND** keyboard mode coordination does not create a second mirrored control state

### Requirement: Component-local style ownership

CSS SHALL live with the component that owns the styled DOM. Global CSS SHALL contain only tokens, reset/base rules, and styles proven to be truly cross-application. `:global()` SHALL be limited to Bits UI or other cross-Svelte DOM boundaries and dynamic rendered content; it SHALL NOT be used as an implicit cross-feature style dependency.

#### Scenario: Scoped component CSS moves

- **WHEN** markup is extracted from SettingsView or HistoryWorkspace
- **THEN** its applicable CSS moves in the same change
- **AND** the production build emits no CSS syntax or minification warning

#### Scenario: A Bits UI descendant needs styling

- **WHEN** Svelte scoping cannot reach a headless primitive's rendered descendant
- **THEN** the narrow affected selector MAY use `:global()`
- **AND** unrelated component rules remain scoped

### Requirement: Evidence-gated shared primitives

A shared UI primitive SHALL be introduced only after at least three consumers demonstrate equivalent semantics, DOM structure, interaction, and styling, and the extraction reduces net maintenance code. Visual resemblance alone SHALL NOT merge feature-specific controls.

#### Scenario: Similar rows are reviewed

- **WHEN** History, Quick, Onboarding, or Settings rows appear visually similar
- **THEN** they remain feature-local unless their ARIA semantics, interaction, structure, and sizing are equivalent
- **AND** no generic Row component is created speculatively

### Requirement: Behavior-frozen frontend migration

Except for the explicit desktop text-selection policy, this architecture change SHALL preserve user-visible layout, wording, keyboard/focus behavior, accessibility semantics, update behavior, preview behavior, IPC contracts, and platform branches.

#### Scenario: A migration phase completes

- **WHEN** a Settings or History responsibility is moved
- **THEN** characterization tests pass before and after the move
- **AND** `pnpm check`, `pnpm test`, and `pnpm build` pass
- **AND** production CSS generation has no syntax/minification warning

### Requirement: Desktop text-selection policy

Static application UI SHALL use the default desktop pointer and SHALL NOT permit text selection. Text-editing controls and the right-side text/link body preview SHALL preserve native text cursor, selection, and copy behavior.

#### Scenario: User drags across static UI

- **WHEN** the pointer moves or drags across titles, list rows, settings help, menus, status text, release notes, file paths, or preview metadata
- **THEN** the pointer is not a text-selection I-beam
- **AND** no text selection is created

#### Scenario: User edits a text control

- **WHEN** the user interacts with a text/search input, textarea, or explicitly editable element
- **THEN** the native text cursor and selection behavior remain available
- **AND** checkbox, switch, button, and select labels remain non-selectable

#### Scenario: User selects preview body text

- **WHEN** the selected clip renders text or link content in the right-side `.preview-body.text-preview`
- **THEN** the body text displays a text cursor and can be selected and copied
- **AND** adjacent preview metadata, file paths, color values, and loading placeholders remain non-selectable
