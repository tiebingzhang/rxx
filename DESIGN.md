# rxx - UDP/IPv6 File Transfer Tool Design

## Project Overview
A Rust CLI tool for sending/receiving files over IPv6 using UDP hole punching and QUIC protocol.

## Architecture

### Core Components
1. **CLI Parser** - Parse command line arguments (send/receive mode, addresses, options)
2. **UDP Hole Puncher** - Establish bidirectional UDP channel through firewalls
3. **QUIC Layer** - Reliable connection over UDP using quinn library
4. **File Transfer** - Stream file data over QUIC with progress indication
5. **Certificate Manager** - Handle self-signed certs (bundled + custom)

### Technology Stack
- **Language**: Rust
- **QUIC Library**: quinn (Rust QUIC implementation)
- **CLI Parsing**: clap
- **Async Runtime**: tokio
- **Certificate**: rcgen for self-signed certs

## Milestones

### Milestone 1: Basic UDP Hole Punching
**Goal**: Establish bidirectional UDP communication between two peers behind firewalls

**Success Criteria**:
- CLI can parse send/receive modes with IPv6 addresses
- Both peers send probe packets every second to each other
- Detect when bidirectional UDP channel is established
- Stop probing after successful bidirectional communication
- Timeout after 10 seconds with 3 retry attempts
- Log connection status clearly

**Deliverables**:
- Basic CLI structure with clap
- UDP socket creation and binding to port 3457
- Probe packet exchange logic
- Bidirectional channel detection
- Timeout and retry mechanism

### Milestone 2: QUIC Connection Establishment
**Goal**: Establish secure QUIC connection over the UDP channel

**Success Criteria**:
- Generate and bundle default self-signed certificate
- Support custom certificate via CLI option
- Receiver acts as QUIC server, sender as QUIC client
- Skip certificate validation
- Successfully establish QUIC connection after hole punching
- Handle QUIC connection errors gracefully

**Deliverables**:
- Certificate generation/loading module
- QUIC server implementation (receiver side)
- QUIC client implementation (sender side)
- Integration with UDP hole punching
- Error handling for connection failures

### Milestone 3: File Transfer with Progress
**Goal**: Transfer files reliably with user feedback

**Success Criteria**:
- Sender can specify file path to send
- Receiver saves file to current directory by default
- Support custom output path via CLI option
- Display progress indicator during transfer
- Verify file integrity after transfer
- Retry on network errors during transfer
- Clean connection closure and program exit after completion
- Handle file I/O errors appropriately

**Deliverables**:
- File streaming over QUIC
- Progress bar implementation
- File integrity verification (size/hash)
- Output path handling
- Network error retry logic
- Graceful shutdown

## Command Line Interface

### Send Mode
```bash
rxx send <file> <destination-ipv6> [OPTIONS]

Options:
  --cert <path>          Path to custom certificate file
  --key <path>           Path to custom private key file
  -h, --help             Print help
  -V, --version          Print version
```

### Receive Mode
```bash
rxx receive <source-ipv6> [OPTIONS]

Options:
  -o, --output <path>    Output directory for received file (default: current dir)
  --cert <path>          Path to custom certificate file
  --key <path>           Path to custom private key file
  -h, --help             Print help
  -V, --version          Print version
```

## Testing Strategy

### Unit Tests
- CLI argument parsing
- Probe packet format and parsing
- Certificate loading logic
- File path validation

### Integration Tests
- UDP hole punching between two local processes
- QUIC connection establishment
- Small file transfer end-to-end
- Error scenarios (timeout, invalid paths, network errors)

### Manual Testing
- Test between two machines on same network
- Test with firewall rules enabled
- Test with large files
- Test connection failures and retries

### Milestone 4: Central Server for ID-Based Peer Discovery
**Goal**: Enable users to discover peers by ID instead of IPv6 address via a central registration server

**Success Criteria**:
- Server mode runs via `rxx server` subcommand with HTTP REST API on port 3457
- Server stores ID-to-IPv6 mappings in SQLite database
- Server enforces ID constraints (alphanumeric, max 20 chars, case-insensitive, no overwrites)
- Server sets 1-year TTL on IPv6 addresses
- Registration command `rxx register <id>` creates ~/.rxx.conf with user_id and server_url
- Send/receive commands read ~/.rxx.conf and error if user_id is missing
- Send/receive commands accept both IDs and IPv6 addresses (auto-detect format)
- On every send/receive invocation, tool updates its own IPv6 and resolves peer IPv6 from server
- On every hole-punching retry, tool re-resolves peer IPv6 from server
- If server is unreachable, tool errors with clear message to use direct IPv6 mode
- Server returns appropriate HTTP status codes (200 for success, 409 for duplicate ID, etc.)

**Deliverables**:
- HTTP REST API server implementation with endpoints:
  - POST /register - Register new ID
  - POST /update - Update own IPv6 and resolve peer IPv6 (combined operation)
- SQLite database schema and operations
- Config file (~/.rxx.conf) management in TOML format
- `rxx register <id>` command implementation
- ID vs IPv6 address detection logic
- Server resolution integration in send/receive flows
- Server resolution on hole-punching retries
- Error handling for unreachable server

**API Endpoints**:
```
POST /register
Body: {"id": "alice", "ipv6": "2001:db8::1"}
Response: 200 OK | 409 Conflict

POST /update
Body: {"id": "alice", "ipv6": "2001:db8::1", "peer_id": "bob"}
Response: 200 OK {"peer_ipv6": "2001:db8::2"} | 404 Not Found
```

**Database Schema**:
```sql
CREATE TABLE registrations (
    id TEXT PRIMARY KEY,
    ipv6 TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
```

**Config File Format (~/.rxx.conf)**:
```toml
user_id = "alice"
server_url = "http://rxx.advistatech.com:3457"
```

## Open Questions / Future Enhancements
- Resume capability for interrupted transfers
- Multiple file support
- Directory transfer
- Compression
- Better certificate management (CA validation)
- IPv4 fallback support
- Server authentication/authorization
- Server clustering and high availability
