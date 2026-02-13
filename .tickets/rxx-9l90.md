---
id: rxx-9l90
status: open
deps: [rxx-kgbq, rxx-h70o]
links: []
created: 2026-02-13T22:53:12Z
type: task
priority: 0
assignee: Tiebing Zhang
parent: rxx-wvdj
---
# Support custom certificate via CLI options

Load custom cert/key from file paths provided via CLI

## Acceptance Criteria

Load cert from --cert path; Load key from --key path; Validate file exists and is readable; Use custom cert instead of default when provided; Handle file read errors gracefully

