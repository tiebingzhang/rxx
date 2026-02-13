---
id: rxx-bcty
status: closed
deps: []
links: []
created: 2026-02-13T22:52:49Z
type: task
priority: 0
assignee: Tiebing Zhang
parent: rxx-wvdj
---
# Generate and bundle default self-signed certificate

Create module to generate self-signed cert/key using rcgen and bundle as default

## Acceptance Criteria

rcgen dependency added; Self-signed cert generated at runtime or bundled; Cert and key available for QUIC server/client; Certificate validity period set appropriately


## Notes

**2026-02-13T22:55:51Z**

Completed: Added rcgen 0.13 dependency. Created cert module with generate_self_signed_cert() and load_cert_from_file() functions. Certificate generation tested and working. Returns PEM-encoded cert and key.

**2026-02-13T22:56:07Z**

Completed: Added rcgen 0.13 dependency. Created cert module with generate_self_signed_cert() and load_cert_from_file() functions. Certificate generation tested and working. Returns PEM-encoded cert and key.
