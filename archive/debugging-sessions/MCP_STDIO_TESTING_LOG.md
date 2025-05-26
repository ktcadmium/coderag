# MCP Stdio Testing Log

## Problem Statement

The CodeRAG MCP server passes all test client tests but fails to be accessible from Claude Code after restart. This has happened 5+ times, indicating our testing methodology is inadequate.

## Test Results Log

### Test 1: Basic Lifecycle Test (PASSES - False Positive)

```bash
./test-utils/target/release/mcp-test-client --server ./target/release/coderag-mcp lifecycle
```

Result: ✅ All steps pass, but this doesn't guarantee Claude Code access

### Test 2: Debug Lifecycle Test (PASSES - Shows Details)

```bash
./test-utils/target/release/mcp-test-client --server ./target/release/coderag-mcp --debug lifecycle
```

Result: ✅ Shows proper JSON-RPC communication, but still doesn't guarantee access

## Known Working MCP Servers

1. `mcp__sequentialthinking` - Runs via Docker
2. Other servers in ~/.mcp.json that work

## Reference Implementations to Study

- <https://github.com/modelcontextprotocol/servers>
- Official TypeScript SDK examples
- Official Python SDK examples
- GitHub's new Go implementation

## Hypotheses to Test

### H1: Line Ending Issues

- Our server might not be sending proper line endings (\n)
- Test: Compare exact byte output with working servers

### H2: Buffering Issues

- stdout might not be flushing properly
- Test: Add explicit flush after every write

### H3: Protocol Compliance

- Might be missing required fields or sending extra data
- Test: Record and compare exact protocol exchanges

### H4: Initialization Sequence

- Might not handle the full initialization handshake correctly
- Test: Log every message during Claude Code startup

### H5: Error Handling

- Might crash silently on certain inputs
- Test: Add comprehensive error logging

### H6: Concurrent Request Handling

- Might fail when multiple requests arrive quickly
- Test: Send rapid-fire requests

### H7: EOF/Pipe Handling

- Might not handle EOF or broken pipes correctly
- Test: Simulate various disconnection scenarios

## Testing Methodology

### Phase 1: Forensic Analysis

1. Study working MCP server implementations
2. Compare line-by-line with our implementation
3. Document all differences

### Phase 2: Protocol Recording

1. Create a protocol logger that records ALL communication
2. Compare working vs non-working servers byte-by-byte
3. Identify exact divergence point

### Phase 3: Stress Testing

1. Test with malformed JSON
2. Test with rapid requests
3. Test with unexpected disconnections
4. Test with very large payloads

### Phase 4: Environment Testing

1. Test with different terminal emulators
2. Test with different shell environments
3. Test with different stdio configurations

## Next Steps

1. Download and analyze reference MCP servers
2. Create more comprehensive test scenarios
3. Build a protocol comparison tool
4. Document every test result here

## Test Results

### Test A: Empty Line Handling

Date: May 24, 2025
Finding: Server DOES handle empty lines correctly - continues processing after empty lines

### Test B: Byte-Level Analysis

Date: May 24, 2025
Finding:

- Both requests arrived in single buffer (166 bytes)
- Server responded with both responses correctly
- Clean exit with code 0
- Line endings are proper `\n` (0x0a)

### Test C: Continuous Interaction

Date: May 24, 2025
Finding: Server successfully processes 4 consecutive requests without issues

### Test D: EOF Handling

Date: May 24, 2025
Finding: Server correctly detects EOF and exits cleanly

## Key Findings So Far

1. **Empty line skip is NOT the issue** - Server continues processing after empty lines
2. **Server handles multiple requests correctly** when sent from test client
3. **Clean shutdown** - Exit code 0, no crashes
4. **Line endings are correct** - Using proper `\n` characters

## Remaining Hypotheses

### H8: Response Format Issue

The server might be sending responses in a format that Claude Code doesn't recognize as valid MCP responses

### H9: Initialization Sequence Timing

The server might be responding too quickly or too slowly for Claude Code's expectations

### H10: Missing Protocol Features

Claude Code might expect additional protocol features that our server doesn't implement

### Test E: Claude Simulation
Date: May 24, 2025
Finding: Complete MCP client simulation shows server working perfectly
- Initialization sequence: ✅
- Edge cases (large IDs, notifications): ✅
- Server handles everything correctly

### Test F: Line Ending Variations
Date: May 24, 2025
Finding: Server correctly handles all line ending formats:
- LF (\n): ✅ Works
- CRLF (\r\n): ✅ Works
- No newline: ✅ Works
- Double newline: ✅ Works

### Test G: Response Byte Analysis
Date: May 24, 2025
Finding: Response format is perfect:
- Ends with exactly one \n (0x0a)
- No CRLF, no double newlines
- 160 bytes for initialize response

### Test H: Unknown Methods
Date: May 24, 2025
Finding: Server correctly handles all MCP methods:
- tools/list: ✅ Implemented
- resources/list: ❌ Returns proper error
- prompts/list: ❌ Returns proper error
- Other unknown methods: ❌ Returns proper errors

## Comprehensive Test Summary

After extensive testing, the CodeRAG MCP server:
1. ✅ Implements the MCP protocol correctly
2. ✅ Handles all required methods (initialize, initialized, tools/list, tools/call)
3. ✅ Properly distinguishes requests from notifications
4. ✅ Sends responses in correct JSON-RPC format
5. ✅ Uses proper line endings (single \n)
6. ✅ Handles edge cases (large IDs, rapid requests, unknown methods)
7. ✅ Exits cleanly on EOF

## The Mystery Remains

Despite passing ALL tests, Claude Code still cannot access the server after restart. This suggests:
1. The issue is NOT with our stdio implementation
2. The issue is NOT with our protocol compliance
3. The issue is NOT with our response format

## Possible Remaining Issues

1. **Configuration Issue**: The server might need to be in a specific location or have specific permissions
2. **Environment Issue**: Claude Code might set specific environment variables that affect stdio
3. **Binary Format Issue**: Something about the Rust binary that Claude Code doesn't like
4. **Timing Issue**: Some very subtle timing requirement we haven't discovered

## Recommendation

Since we've exhausted stdio testing and found no issues, the problem likely lies elsewhere:
- Check if the binary needs specific permissions
- Verify the MCP configuration is being loaded correctly by Claude Code
- Consider if there's a platform-specific issue (macOS security, binary signing, etc.)
- Test with a minimal "echo" server to isolate the issue
