---
id: rxx-h70o
status: closed
deps: [rxx-26gg]
links: []
created: 2026-02-13T22:53:07Z
type: task
priority: 0
assignee: Tiebing Zhang
parent: rxx-wvdj
---
# Implement QUIC client (sender side)

Implement QUIC client that connects after UDP hole punching completes

## Acceptance Criteria

QUIC client connects to peer using established UDP socket; Client skips certificate validation; Client establishes QUIC connection successfully; Log connection status; Handle connection errors gracefully


## Notes

**2026-02-13T23:09:00Z**

Completed: Integrated QUIC client in send mode. After UDP hole punching, creates client config with crypto provider, connects to peer QUIC server. Updated UDP to use separate ports (3457 for client, 3458 for server). Added ring crypto provider. Note: UDP bidirectional detection needs refinement for proper synchronization.

**2026-02-13T23:11:40Z**

Fixed UDP bidirectional detection synchronization issue. Both sides now check for bidirectional communication after receiving PROBE_PACKET (not just after PROBE_ACK). This ensures both peers complete hole punching at the same time. The fix adds the bidirectional check in both branches: when receiving PROBE_PACKET and when receiving PROBE_ACK.

**2026-02-13T23:12:31Z**

Tested locally with server and client on ::1. Both sides complete UDP hole punching simultaneously and QUIC connection establishes successfully. Fix verified working.
