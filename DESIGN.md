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

## Open Questions / Future Enhancements
- Resume capability for interrupted transfers
- Multiple file support
- Directory transfer
- Compression
- Better certificate management (CA validation)
- IPv4 fallback support
