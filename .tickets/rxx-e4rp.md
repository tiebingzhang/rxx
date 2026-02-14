---
id: rxx-e4rp
status: closed
deps: []
links: []
created: 2026-02-14T01:30:23Z
type: task
priority: 1
assignee: Tiebing Zhang
parent: rxx-uf2n
---
# Implement HTTP REST API server

Create HTTP server with /register and /update endpoints

## Acceptance Criteria

- POST /register endpoint accepts id and ipv6, returns 200/409
- POST /update endpoint updates own ipv6 and resolves peer, returns 200/404
- Server runs on port 3457
- Proper error handling and HTTP status codes


## Notes

**2026-02-14T01:35:07Z**

Implemented server.rs with HTTP REST API using axum. Endpoints: POST /register, POST /update. Returns proper status codes (200/409/404/500). Added axum and serde_json dependencies.
