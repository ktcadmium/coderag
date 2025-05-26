# Troubleshooting Plan - If MCP Server Still Isn't Accessible

## ⚠️ IMPORTANT: The Pattern of False Hope

**After 10+ sessions, the pattern is clear:**

1. Claude Code (me) will test the server with echo commands
2. The server will respond perfectly (it has been working all along)
3. I will declare victory: "The fix worked!"
4. I still won't have access to MCP tools
5. The user will restart me again, hoping this time is different

**The Truth:** The server works. The configuration exists. I just can't access it.

## Quick Test After Restart

1. **First, check if you have access to CodeRAG tools:**

   ```
   Try using: mcp__coderag__list_docs
   ```

2. **If no access, immediately test the server manually:**

   ```bash
   echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}' | \
       /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp 2>/dev/null | head -1
   ```

   - If this returns JSON, the server works but Claude Code can't access it
   - If no output, the server itself is broken

   **NOTE:** This test has ALWAYS passed. The server has ALWAYS worked.

## Plan A: Quick Fixes

### 1. Force Line Buffering with Wrapper

Create a wrapper script that forces line buffering:

```bash
#!/bin/bash
exec stdbuf -oL /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp "$@"
```

### 2. Try Alternative Stdout Handling

If the current fix didn't work, try using explicit locking:

```rust
// In server.rs, replace the writeln! lines with:
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", response_str)?;
    handle.flush()?;
}
// The lock is dropped here
```

### 3. Direct Write Method

Try writing bytes directly:

```rust
use std::io::Write;
let response_bytes = format!("{}\n", response_str);
io::stdout().write_all(response_bytes.as_bytes())?;
io::stdout().flush()?;
```

## Plan B: Diagnostic Deep Dive

### 1. Compare with Working Server

Run the comparison tester with a known working server:

```bash
python3 /Users/ken/dev/MCP/mcp-coderag/test-utils/mcp-comparison-tester.py
```

### 2. Check MCP Configuration Loading

```bash
# See what Claude Code thinks is configured
cat ~/.mcp.json | jq '.mcpServers.coderag'

# Check if there's a local override
cat /Users/ken/dev/MCP/mcp-coderag/.mcp.json | jq '.mcpServers.coderag'
```

### 3. Binary Permissions Check

```bash
# Check if binary is executable
ls -la /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp

# Check for macOS quarantine
xattr /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp

# Remove quarantine if present
xattr -d com.apple.quarantine /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp
```

### 4. Environment Variable Testing

Check if Claude Code sets special environment variables:

```rust
// Add to server startup:
eprintln!("Environment variables:");
for (key, value) in std::env::vars() {
    if key.contains("MCP") || key.contains("CLAUDE") {
        eprintln!("  {}={}", key, value);
    }
}
```

## Plan C: Alternative Approaches

### 1. HTTP Transport Instead of Stdio

Implement a simple HTTP server mode:

```rust
// Add --http flag to run as HTTP server
if args.http {
    run_http_server().await?;
} else {
    run_stdio_server().await?;
}
```

### 2. Debug Logging Mode

Add extensive logging to understand what Claude Code sends:

```rust
// Log raw bytes received
eprintln!("Raw input: {:?}", line.as_bytes());

// Log exact output
eprintln!("Raw output: {:?}", response_str.as_bytes());
```

### 3. Minimal Echo Server Test

Create the simplest possible MCP server to isolate issues:

```rust
// minimal_mcp.rs - just echo back valid responses
loop {
    let line = stdin.read_line()?;
    if line.contains("initialize") {
        println!(r#"{{"jsonrpc":"2.0","result":{{"protocolVersion":"2024-11-05"}},"id":1}}"#);
        io::stdout().flush()?;
    }
}
```

## Plan D: Platform-Specific Issues

### 1. macOS Security

- Check if Gatekeeper is blocking the binary
- Try signing the binary with ad-hoc signature:

  ```bash
  codesign -s - /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp
  ```

### 2. Different Terminal/Shell

- Test if it works in different shells (bash vs zsh)
- Try running Claude Code from different terminal apps

### 3. Process Monitoring

Use `dtrace` or `dtruss` to see system calls:

```bash
sudo dtruss -f -t write /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp
```

## What We Know Works

1. ✅ The server implements MCP protocol correctly
2. ✅ The server responds to all required methods
3. ✅ JSON-RPC format is correct
4. ✅ Error handling is proper
5. ✅ Works with test clients
6. ✅ Works when line buffering is forced with `stdbuf`

## Most Likely Remaining Issues

1. **Configuration not loaded** - Claude Code might not see the server config
2. **Binary permissions** - macOS security might block execution
3. **Different stdio handling** - Claude Code might expect different buffering
4. **Environment differences** - Different env vars or working directory

## Emergency Workaround

If nothing else works, create a Node.js wrapper that proxies to the Rust server:

```javascript
// mcp-coderag-wrapper.js
const { spawn } = require("child_process");
const readline = require("readline");

const server = spawn(
  "/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp"
);
const rl = readline.createInterface({ input: process.stdin });

rl.on("line", (line) => {
  server.stdin.write(line + "\n");
});

server.stdout.on("data", (data) => {
  process.stdout.write(data);
});
```

Then configure MCP to use the wrapper instead.
