---
id: rxx-lssg
status: closed
deps: [rxx-zcl5]
links: []
created: 2026-02-13T22:40:19Z
type: task
priority: 0
assignee: Tiebing Zhang
parent: rxx-9ofx
---
# Implement CLI argument parsing

Use clap to parse send/receive modes with IPv6 addresses and options

## Acceptance Criteria

Parse 'send <file> <ipv6>' command; Parse 'receive <ipv6>' command; Support --cert, --key, --output options; Display help and version; Validate IPv6 address format


## Notes

**2026-02-13T22:45:42Z**

Completed: Implemented CLI with clap derive API. Supports send/receive subcommands with IPv6 validation, file paths, and optional cert/key/output parameters. All acceptance criteria met.
