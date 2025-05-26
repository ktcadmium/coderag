# Testing MCP CodeRAG Access After Claude Restart

## Current Status (May 24, 2025)
- Fixed MCP server to not respond to notifications (requests without ID)
- Fixed stdout handling (using &mut stdout instead of &stdout)
- Server binary: `/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp`
- Test client: `/Users/ken/dev/MCP/mcp-coderag/test-utils/target/release/mcp-test-client`

## Quick Test After Restart

1. **Check if I have access to CodeRAG tools:**
   - Try using: `mcp__coderag__list_docs`
   - Should return: `{"sources": [], "total_documents": 0, "last_updated": null}`

2. **If no access, verify server works:**
   ```bash
   cd /Users/ken/dev/MCP/mcp-coderag/test-utils
   ./target/release/mcp-test-client --server ../target/release/coderag-mcp lifecycle
   ```

3. **Expected tools (should appear as mcp__coderag__*):**
   - mcp__coderag__search_docs
   - mcp__coderag__list_docs
   - mcp__coderag__crawl_docs
   - mcp__coderag__reload_docs

## If Still No Access

### Theory: Stdio Implementation Differences
Since I (Claude Code) can access mcp__sequentialthinking but not coderag, there might be subtle differences in how stdio is handled:

1. **Line endings**: Make sure each JSON message ends with a newline
2. **Buffering**: stdout.flush() after each write
3. **Error handling**: Graceful handling of EOF and broken pipes
4. **Initialization**: Proper response to initialize/initialized sequence

### Check MCP Configuration
```bash
cat ~/.mcp.json | jq '.mcpServers.coderag'
```

Should show:
```json
{
  "command": "/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp",
  "args": ["--debug"]
}
```

### Test Server Manually
```bash
# Simple test - should complete successfully
/Users/ken/dev/MCP/mcp-coderag/test-utils/target/release/mcp-test-client \
  --server /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp \
  lifecycle

# Debug test - shows all communication
/Users/ken/dev/MCP/mcp-coderag/test-utils/target/release/mcp-test-client \
  --server /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp \
  --debug lifecycle
```

### Manual Protocol Test
Try sending raw JSON-RPC messages:
```bash
# Test initialize sequence
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}},"id":1}' | \
  /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp --debug

# Should respond with server info
```

### Common Issues
1. **Wrong binary path**: Make sure using release build, not debug
2. **Old binary**: Rebuild with `cargo build --release`
3. **MCP not enabled**: I need to be restarted with MCP support
4. **Config not loaded**: ~/.mcp.json might not be read on startup
5. **Protocol mismatch**: Server might need exact protocol compliance

## What Success Looks Like
When working correctly, I should be able to:
```python
# This should work (as mcp__coderag__list_docs tool)
result = list_docs()
# Returns: {"sources": [], "total_documents": 0, "last_updated": null}

# And this should list 4 tools starting with mcp__coderag__
```

## Key Fixes Applied
1. Server was sending responses to ALL messages, including notifications. Fixed to only respond to requests with `id` field
2. Fixed stdout handling to use mutable reference (&mut stdout)

## Debugging Ideas
If still not working after restart:
1. Compare with working MCP servers (sequentialthinking runs via Docker)
2. Check if server needs to handle specific MCP lifecycle events
3. Verify JSON-RPC compliance - exact field names and structure
4. Test if server handles concurrent requests properly
