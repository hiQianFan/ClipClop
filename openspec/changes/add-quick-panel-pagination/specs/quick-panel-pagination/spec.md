## ADDED Requirements

### Requirement: Quick exposes all history pages

The Quick panel SHALL expose paged history without requiring the full history panel and SHALL retain ten row slots that fill the list's available height in its standard window.

#### Scenario: Last page is incomplete

- **WHEN** a page contains fewer records than the available row capacity
- **THEN** records remain top-aligned and the unused slots preserve the list, pager, and menu positions without becoming focusable

### Requirement: Quick pagination is continuously operable

The Quick panel SHALL support pointer pagination, PageUp/PageDown, and Up/Down traversal across page boundaries while retaining page-local number shortcuts.

#### Scenario: Selection crosses a page boundary

- **WHEN** the user moves down from the last item of a non-final page or up from the first item of a non-first page
- **THEN** Quick loads the adjacent page and selects its first or last item respectively

### Requirement: Quick paging preserves committed state

Quick SHALL atomically commit only the latest successful page request.

#### Scenario: Page request fails or returns out of order

- **WHEN** a page request fails or an older request resolves after a newer request
- **THEN** Quick retains the last committed page on failure and ignores stale responses

### Requirement: Quick starts from fresh history

Quick SHALL return to the newest page when shown or when history changes, and SHALL pass the current selection when opening full history.

#### Scenario: Quick is reopened after browsing older pages

- **WHEN** the Quick panel is shown again
- **THEN** it refreshes page one and selects the newest available record
