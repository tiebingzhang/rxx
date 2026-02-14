---
id: rxx-xj0y
status: closed
deps: []
links: []
created: 2026-02-14T05:59:02Z
type: feature
priority: 1
assignee: Tiebing Zhang
tags: [security, database]
---
# Add nonce-based authentication for IP address updates

When client registers with server, server returns a nonce. This nonce is required when client updates its IP address to prevent unauthorized updates. Also skip database write if IP address hasn't changed.

## Acceptance Criteria

1. Server generates and returns nonce during client registration\n2. Nonce is stored in database associated with client\n3. IP update requests require valid nonce\n4. Server validates nonce before allowing IP update\n5. Database write is skipped if new IP matches existing IP\n6. Invalid nonce returns error to client


## Notes

**2026-02-14T05:59:37Z**

Client implementation: Store received nonce in ~/.rxx.conf file for subsequent IP update requests

**2026-02-14T06:07:30Z**

Implementation complete:
- Added nonce column to database schema
- Server generates 32-char alphanumeric nonce on registration
- Server returns nonce in RegisterResponse JSON
- Client stores nonce in ~/.rxx.conf
- UpdateRequest now requires nonce field
- Server validates nonce before allowing IP updates (returns 401 UNAUTHORIZED if invalid)
- Database write skipped if IP address unchanged (logs 'IP unchanged, skipping DB write')
- Added rand crate dependency for nonce generation
- All tests pass, builds successfully for musl target
