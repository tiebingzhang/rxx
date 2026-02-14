---
id: rxx-szmc
status: closed
deps: [rxx-74kk, rxx-k3t8, rxx-o1vc]
links: []
created: 2026-02-14T01:30:43Z
type: task
priority: 1
assignee: Tiebing Zhang
parent: rxx-uf2n
---
# Integrate server resolution in send/receive

Update send/receive commands to use server for peer resolution

## Acceptance Criteria

- Check ~/.rxx.conf exists and has user_id, error if missing
- Call /update endpoint on startup to update own IPv6 and resolve peer
- Handle ID-based and IPv6-based peer addressing
- Clear error if server unreachable (instruct to use direct IPv6)
- Re-resolve peer IPv6 on every hole-punching retry


## Notes

**2026-02-14T01:43:02Z**

Integrated server resolution in send/receive. Updated Commands to accept String for peer addresses. Added net::resolve_peer() for ID resolution. Modified udp::punch_hole() to accept resolver callback for re-resolution on retries. Config check added to both commands.
