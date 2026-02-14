---
id: rxx-9dha
status: closed
deps: []
links: []
created: 2026-02-14T01:30:20Z
type: task
priority: 1
assignee: Tiebing Zhang
parent: rxx-uf2n
---
# Implement SQLite database schema and operations

Create SQLite database with registrations table and CRUD operations

## Acceptance Criteria

- Database schema created with id, ipv6, updated_at fields
- Insert/update/query operations implemented
- 1-year TTL enforcement on queries
- Case-insensitive ID handling


## Notes

**2026-02-14T01:33:47Z**

Implemented db.rs module with Database struct. Methods: open(), register(), update(), get_ipv6(). Enforces 1-year TTL and case-insensitive IDs. Added rusqlite dependency.
