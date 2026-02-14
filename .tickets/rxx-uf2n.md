---
id: rxx-uf2n
status: closed
deps: []
links: []
created: 2026-02-14T01:30:15Z
type: epic
priority: 0
assignee: Tiebing Zhang
---
# Milestone 4: Central Server for ID-Based Peer Discovery

Enable users to discover peers by ID instead of IPv6 address via a central registration server

## Acceptance Criteria

- Server mode runs via rxx server subcommand
- Users can register IDs and resolve peer IPv6 addresses
- Send/receive commands support both ID and IPv6 addressing
- Config file management in ~/.rxx.conf
- Server unreachable errors handled gracefully


## Notes

**2026-02-14T01:43:07Z**

Milestone 4 completed. All features implemented: config management, SQLite database, HTTP REST API, server subcommand, register command, ID detection, and server resolution integration.
