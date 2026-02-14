I would like to use Rust to develop a cli tool that allow me to send/receive files using UDP over IPv6.

Requirements:
1. tool is called "rxx"
2. tool can run either in send mode or receive mode.
3. tool uses IPv6 only and use UDP port 3457(client) and 3458 (server) for source and destination (since the receiver is also rxx). 
4. tool works even both sender and receiver are behind firewalls. it works like the following: 
    1. assumption is both server and client know each other want to communicate and know each other's IPv6 address
    2. server starts immedicately send a probe packet every second to the client ip and port. the client does the same. this should punch a hole in the firewall and allow them
    to talk to each other.
5. once udp comm is establish, tool should use QUIC to estalibsh a reliable connection over the UDP channel. Then send file over the QUIC protocol. 
6. Since QUIC needs certs, for now the tool should bundle a default self-signed test cert, but allow passing cert file as command line option
7. Tool is devloped in Rust
8. Users can register an alias/ID with a central server to enable peer discovery by ID instead of IPv6 address
9. The central server stores ID-to-IPv6 mappings in SQLite database
10. IDs are alphanumeric only, max 20 characters, case-insensitive, and cannot be overwritten (first-come-first-serve)
11. IPv6 addresses associated with IDs can be updated and have a TTL of 1 year
12. User configuration is stored in ~/.rxx.conf (TOML format) containing user_id and server_url
13. If ~/.rxx.conf doesn't exist or lacks user_id, the tool errors and prompts user to run `rxx register <id>`
14. Upon running send/receive commands, rxx reports its IPv6/ID to the server and resolves peer IPv6 by peer ID
15. The tool supports both ID-based and direct IPv6-based addressing
16. On every retry during hole punching, rxx re-resolves peer IPv6 from the server
17. If the server is unreachable, the tool errors and instructs user to use direct IPv6 mode
18. The server runs as part of the same rxx binary via `rxx server` subcommand
19. The server provides HTTP REST API on port 3457 at domain rxx.advistatech.com
20. Server has no authentication (open registration for non-production use)
