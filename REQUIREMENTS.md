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
