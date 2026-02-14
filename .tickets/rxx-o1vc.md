---
id: rxx-o1vc
status: closed
deps: [rxx-74kk, rxx-e4rp]
links: []
created: 2026-02-14T01:30:35Z
type: task
priority: 1
assignee: Tiebing Zhang
parent: rxx-uf2n
---
# Implement 'rxx register' command

Add registration command to CLI

## Acceptance Criteria

- rxx register <id> validates ID format (alphanumeric, max 20 chars)
- Detects local IPv6 address
- Calls server /register endpoint
- Creates ~/.rxx.conf on success
- Clear error messages on failure (duplicate ID, server unreachable)


## Notes

**2026-02-14T01:40:20Z**

Implemented Register subcommand. Validates ID (alphanumeric, max 20 chars), gets local IPv6, calls /register endpoint, saves config on success. Added reqwest and if-addrs dependencies.
