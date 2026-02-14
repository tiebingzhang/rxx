# rxx - UDP/IPv6 File Transfer Tool

A Rust CLI tool for sending and receiving files over IPv6 using UDP hole punching and QUIC protocol.

## Features

- **ID-Based Transfer**: Send files using memorable user IDs instead of IPv6 addresses
- **UDP Hole Punching**: Establishes bidirectional UDP communication through firewalls
- **QUIC Protocol**: Reliable, encrypted file transfer over UDP
- **Progress Indicator**: Real-time progress bar showing bytes transferred and percentage
- **File Integrity**: SHA256 hash verification ensures file integrity
- **Self-Signed Certificates**: Automatic generation with custom certificate support
- **IPv6 Native**: Built for IPv6 networking
- **Central Server**: Optional registration server for ID-to-IP mapping
- **Hooks**: Execute custom commands when files are received (see [HOOKS.md](HOOKS.md))
- **Error Handling**: Comprehensive error handling for network and file I/O operations

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/rxx`.

## Usage

### Register Your ID (Optional)

Register a memorable user ID with the central server:

```bash
rxx register <user-id> [OPTIONS]

Arguments:
  <user-id>  User ID (alphanumeric, dots, hyphens, underscores; must start/end with alphanumeric; max 20 chars)

Options:
  --server <url>    Server URL (default: http://rxx.advistatech.com:3457)
```

Example:
```bash
rxx register alice
rxx register alice-bob
rxx register user.name_123 --server http://localhost:3457
```

### Send a File

```bash
rxx send <file> <destination> [OPTIONS]

Arguments:
  <file>         File to send
  <destination>  Destination (IPv6 address or user ID)

Options:
  --cert <path>    Path to custom certificate file
  --key <path>     Path to custom private key file
```

Example:
```bash
# Send using user ID
rxx send myfile.txt alice

# Send using IPv6 address
rxx send myfile.txt ::1
rxx send document.pdf 2001:db8::1 --cert cert.pem --key key.pem
```

### Receive a File

```bash
rxx receive <source> [OPTIONS]

Arguments:
  <source>  Source (IPv6 address or user ID)

Options:
  -o, --output <path>    Output directory for received file (default: current directory)
  --cert <path>          Path to custom certificate file
  --key <path>           Path to custom private key file
```

Example:
```bash
# Receive using user ID
rxx receive bob

# Receive using IPv6 address
rxx receive ::1
rxx receive 2001:db8::1 --output /tmp/downloads
```

### Run Registration Server

```bash
rxx server [OPTIONS]

Options:
  --db <path>      Database file path (default: rxx.db)
  --port <port>    Port to listen on (default: 3457)
```

Example:
```bash
rxx server
rxx server --port 8080 --db /var/lib/rxx/registry.db
```

### Show IPv6 Addresses

```bash
rxx ip
```

Lists all available IPv6 addresses on your system.

## How It Works

1. **UDP Hole Punching**: Both peers exchange probe packets to establish a bidirectional UDP channel through NAT/firewalls
2. **QUIC Connection**: After UDP channel is established, a QUIC connection is created (receiver acts as server, sender as client)
3. **File Transfer**: File metadata (name, size) is sent first, followed by file content in 64KB chunks
4. **Integrity Verification**: SHA256 hash is calculated during transfer and verified on the receiver side
5. **Progress Display**: Real-time progress bar shows transfer status

## Certificate Management

### Default Behavior
- Receiver automatically generates a self-signed certificate if none is provided
- Certificate validation is skipped on the client side for simplicity

### Custom Certificates
For production use, you can provide custom certificates:

Generate ECDSA certificate (required for QUIC):
```bash
openssl ecparam -genkey -name prime256v1 -out key.pem
openssl req -new -x509 -key key.pem -out cert.pem -days 365
```

Use custom certificate:
```bash
rxx receive ::1 --cert cert.pem --key key.pem
```

## Technical Details

- **Language**: Rust
- **QUIC Library**: quinn
- **Crypto**: rustls with ring provider
- **Async Runtime**: tokio
- **CLI**: clap
- **Hashing**: SHA256 via sha2 crate
- **Progress**: indicatif

## Architecture

```
┌─────────────┐                    ┌─────────────┐
│   Sender    │                    │  Receiver   │
│  (Client)   │                    │  (Server)   │
└──────┬──────┘                    └──────┬──────┘
       │                                  │
       │  1. UDP Probe Packets            │
       │─────────────────────────────────>│
       │<─────────────────────────────────│
       │  2. UDP Channel Established      │
       │                                  │
       │  3. QUIC Connection              │
       │─────────────────────────────────>│
       │                                  │
       │  4. File Metadata                │
       │─────────────────────────────────>│
       │                                  │
       │  5. File Content (chunks)        │
       │─────────────────────────────────>│
       │                                  │
       │  6. SHA256 Hash                  │
       │─────────────────────────────────>│
       │                                  │
       │  7. Verification & Completion    │
       └──────────────────────────────────┘
```

## Development

### Build
```bash
cargo build
```

### Test
```bash
cargo test
```

### Format
```bash
cargo fmt --all
```

### Lint
```bash
cargo clippy
```

## Project Status

All three milestones completed:

- ✅ **Milestone 1**: Basic UDP Hole Punching
- ✅ **Milestone 2**: QUIC Connection Establishment  
- ✅ **Milestone 3**: File Transfer with Progress
- ✅ **Milestone 4**: Central Server for ID-Based Peer Discovery

## License

MIT
