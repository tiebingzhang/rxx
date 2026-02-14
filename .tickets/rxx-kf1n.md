---
id: rxx-kf1n
status: closed
deps: []
links: []
created: 2026-02-14T06:01:55Z
type: feature
priority: 2
assignee: Tiebing Zhang
tags: [ci, release, macos]
---
# Update GitHub workflow for Intel Mac builds and automatic releases

Enhance CI/CD workflow to build for Intel Mac (x86_64-apple-darwin) in addition to existing targets. Add automatic GitHub release creation when pushing semver tags (e.g., v1.0.0, v2.1.3).

## Acceptance Criteria

1. Workflow builds for x86_64-apple-darwin (Intel Mac) target\n2. Workflow detects semver tag pushes (pattern: v*.*.*)\n3. On semver tag, workflow creates GitHub release automatically\n4. Release includes binaries for all supported platforms\n5. Release notes are generated or extracted from tag message\n6. Existing builds continue to work


## Notes

**2026-02-14T06:09:44Z**

Updated GitHub workflow:
- Added Intel Mac build target (x86_64-apple-darwin) using macos-13 runner
- Added semver tag trigger (v*.*.*)
- Changed release step to auto-create releases on tag push
- Enabled automatic release notes generation
- All three binaries (Linux amd64, macOS ARM64, macOS Intel) will be attached to releases

**2026-02-14T06:14:40Z**

Updated to macos-15-intel runner (macos-13 deprecated as of Dec 2025)
