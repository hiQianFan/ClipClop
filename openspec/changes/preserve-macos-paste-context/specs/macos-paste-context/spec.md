## ADDED Requirements

### Requirement: Non-activating clipboard panels

On macOS, ClipClop SHALL show main and quick clipboard panels without explicitly activating its application when an external application is active, while allowing the panel to receive keyboard input.

#### Scenario: Open a panel from Chrome

- **WHEN** the user summons a clipboard panel while Chrome is active
- **THEN** Chrome remains the active application and the panel accepts navigation, search, and input-method composition

#### Scenario: Switch between clipboard panels

- **WHEN** the user switches from quick to main
- **THEN** the original external paste target is preserved and stale blur callbacks do not hide the new panel

### Requirement: Keyboard handoff precedes paste injection

ClipClop SHALL complete native panel dismissal and verify that its panel no longer receives keyboard input before injecting paste; a matching frontmost PID alone SHALL NOT count as completed handoff.

#### Scenario: Target application remains active

- **WHEN** the panel closes for paste and the captured target remains active
- **THEN** ClipClop skips redundant application activation, performs bounded handoff checks, and injects paste at most once

#### Scenario: Handoff cannot complete

- **WHEN** panel dismissal or keyboard handoff fails or times out
- **THEN** ClipClop retains the copied content and returns an existing clipboard-only failure outcome without injecting paste

### Requirement: Changed targets invalidate pending paste

ClipClop SHALL cancel stale paste operations when another external application becomes active or a newer panel session invalidates the captured context.

#### Scenario: User changes application while paste is pending

- **WHEN** another external application becomes active before injection
- **THEN** ClipClop does not reactivate the old target or send paste to the new application

#### Scenario: Panel is reopened before injection

- **WHEN** a newer panel session starts before an old paste operation injects keys
- **THEN** the old operation leaves the clipboard available but does not inject keys

### Requirement: Native preview and frontend keyboard behavior remain operable

ClipClop SHALL preserve its existing frontend keyboard contracts and distinguish native preview interaction from external focus loss without requiring its application to be active throughout panel use.

#### Scenario: Quick Look temporarily receives keyboard input

- **WHEN** a managed preview opens and later closes
- **THEN** its parent panel is not incorrectly dismissed because the application is inactive and panel keyboard interaction can resume

#### Scenario: User clicks outside the panel

- **WHEN** focus leaves the panel for an unrelated external window
- **THEN** the existing dismissal policy hides the panel without stale focus restoration stealing input back

### Requirement: WPS compatibility requires native verification

The change SHALL NOT be reported as fixing WPS web search until the real macOS Chrome scenario preserves the search input context after both cancellation and automatic paste.

#### Scenario: Cancel without mouse interaction

- **WHEN** the user focuses WPS web search, opens ClipClop, dismisses its panel with Escape, and types without clicking
- **THEN** text enters the search input and does not edit the selected spreadsheet cell

#### Scenario: Paste into WPS web search

- **WHEN** the user focuses WPS web search and selects a text item in ClipClop for automatic paste
- **THEN** the text is inserted into the intended search input according to its selection and the spreadsheet cell is not modified

#### Scenario: Non-activating panel does not preserve web input

- **WHEN** native verification still redirects input to the spreadsheet cell
- **THEN** the result is recorded as a failed compatibility experiment and the implementation is not represented as a verified WPS fix
