# Hooks Feature

The rxx tool supports optional hooks that can be triggered on specific events.

## Configuration

Hooks are configured in the user config file (`~/.rxx.conf`). Add a `[hooks]` section:

```toml
user_id = "alice"
server_url = "http://rxx.advistatech.com:3457"
nonce = "your-nonce-here"

[hooks]
file_received = "/path/to/your/hook-script.sh"
```

## Available Hooks

### file-received Hook

This hook is executed after a file is successfully received and verified (SHA256 hash check passed).

**Arguments passed to the hook command:**
1. `sender_id` - The user ID of the sender
2. `filename` - Name of the received file
3. `file_size` - Size of the file in bytes

**Example hook script:**

```bash
#!/bin/bash
# File received hook
SENDER_ID="$1"
FILENAME="$2"
FILE_SIZE="$3"

echo "File received from: $SENDER_ID"
echo "Filename: $FILENAME"
echo "Size: $FILE_SIZE bytes"

# Log to file
echo "$(date): Received $FILENAME ($FILE_SIZE bytes) from $SENDER_ID" >> ~/rxx-received.log

# Send notification (example)
# notify-send "File Received" "Got $FILENAME from $SENDER_ID"
```

## Hook Execution Details

- **Timeout**: Hooks have a 10-second execution timeout
- **Async execution**: Hooks run in a separate task and don't block the receiver
- **Error handling**: If a hook fails or times out, a warning is logged but the file transfer is still considered successful
- **Shell execution**: Hooks are executed via `sh -c`, so you can use shell commands directly

## Example Usage

1. Create your hook script:
```bash
cat > ~/my-hook.sh << 'EOF'
#!/bin/bash
echo "Received: $2 from $1 ($3 bytes)" >> ~/rxx.log
EOF
chmod +x ~/my-hook.sh
```

2. Edit your config file (`~/.rxx.conf`):
```toml
user_id = "alice"
server_url = "http://rxx.advistatech.com:3457"

[hooks]
file_received = "~/my-hook.sh"
```

3. Receive files as normal:
```bash
rxx receive bob
```

The hook will be automatically executed after each successful file transfer.
