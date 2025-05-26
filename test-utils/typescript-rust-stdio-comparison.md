# TypeScript vs Rust MCP Server stdio Implementation Comparison

## Overview
This document compares the TypeScript MCP SDK's stdio implementation with our Rust implementation to identify key differences that might explain connection issues.

## Key Differences Found

### 1. Message Framing

**TypeScript Implementation:**
- Uses a `ReadBuffer` class that accumulates chunks
- Looks for `\n` as message delimiter
- Strips `\r` if present before the `\n`
- Handles partial messages by buffering until complete

**Rust Implementation:**
- Uses `BufReader` with `lines()` iterator
- Relies on Rust's built-in line reading
- May have different behavior with line endings

### 2. Line Ending Handling

**TypeScript:**
```typescript
// In ReadBuffer.readMessage()
const line = this._buffer.toString("utf8", 0, index).replace(/\r$/, '');
```
- Explicitly removes trailing `\r` before processing
- Handles both `\n` and `\r\n` endings

**Rust:**
```rust
// Using BufReader.lines()
for line in reader.lines() {
    let line = match line {
        Ok(l) => l,  // lines() automatically strips line endings
```
- `BufReader::lines()` strips both `\n` and `\r\n` automatically

### 3. Stream Buffering and Flushing

**TypeScript:**
```typescript
send(message: JSONRPCMessage): Promise<void> {
    return new Promise((resolve) => {
        const json = serializeMessage(message);
        if (this._stdout.write(json)) {
            resolve();
        } else {
            this._stdout.once("drain", resolve);
        }
    });
}
```
- Checks if write buffer is full
- Waits for "drain" event if needed
- Automatic flow control

**Rust:**
```rust
writeln!(&mut stdout, "{}", response_str)?;
stdout.flush()?;
```
- Always flushes after each write
- No buffer overflow handling
- Synchronous flushing

### 4. Error Handling

**TypeScript:**
```typescript
this._stdin.on("error", this._onerror);
```
- Uses Node.js event-based error handling
- Continues processing on most errors

**Rust:**
```rust
Err(e) => {
    if e.kind() == io::ErrorKind::UnexpectedEof || e.kind() == io::ErrorKind::BrokenPipe {
        info!("Client disconnected, shutting down gracefully");
        break;
    }
    return Err(e.into());
}
```
- Graceful handling of EOF and broken pipe
- Returns error for other cases

### 5. Initialization Sequence

**TypeScript:**
- Server waits for explicit `start()` call
- Event listeners are attached in `start()`
- Can handle async initialization

**Rust:**
- Starts reading immediately when `run_stdio()` is called
- No separate initialization phase
- Synchronous startup

### 6. JSON Serialization

**TypeScript:**
```typescript
export function serializeMessage(message: JSONRPCMessage): string {
    return JSON.stringify(message) + "\n";
}
```
- Simple JSON + newline

**Rust:**
```rust
let response_str = serde_json::to_string(&response)?;
writeln!(&mut stdout, "{}", response_str)?;
```
- Uses `writeln!` which adds platform-specific line ending
- May produce `\r\n` on Windows vs `\n` on Unix

### 7. Partial Message Handling

**TypeScript:**
- Accumulates partial messages in buffer
- Can handle messages split across multiple chunks

**Rust:**
- Relies on `BufReader` to handle this
- May fail if a JSON message is split across buffer boundaries

## Potential Issues in Our Rust Implementation

1. **Line Endings**: We might be sending `\r\n` on some platforms where TypeScript expects `\n`
2. **Buffering**: We don't handle partial messages as robustly
3. **Flush Timing**: We flush after every message, which might cause timing issues
4. **Empty Lines**: We skip empty lines, but TypeScript doesn't
5. **Notification Handling**: We correctly don't respond to notifications, matching TypeScript

## Recommendations for Rust Implementation

1. **Use consistent line endings:**
```rust
// Instead of writeln!
write!(&mut stdout, "{}\n", response_str)?;
```

2. **Consider implementing a proper read buffer:**
```rust
struct ReadBuffer {
    buffer: Vec<u8>,
}

impl ReadBuffer {
    fn read_message(&mut self) -> Option<String> {
        // Look for \n, handle partial messages
    }
}
```

3. **Handle stdout buffering more carefully:**
```rust
// Consider not flushing after every write
// Or use LineWriter for automatic line-based flushing
```

4. **Log raw bytes for debugging:**
```rust
debug!("Raw bytes sent: {:?}", response_str.as_bytes());
```

## Testing Recommendations

1. Test with both `\n` and `\r\n` line endings
2. Test with large messages that might span buffers
3. Test with rapid message sequences
4. Compare raw byte output between TypeScript and Rust servers
5. Use a protocol logger to capture exact bytes sent/received
