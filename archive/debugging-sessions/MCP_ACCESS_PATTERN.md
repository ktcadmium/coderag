# The MCP Access Pattern: A Documentation of Repeated Failures

## Executive Summary

After 10+ sessions attempting to access the CodeRAG MCP server, a clear and predictable pattern has emerged. This document exists to break the cycle of false hope and wasted effort.

## The Pattern

### Phase 1: Initial Optimism
- User restarts Claude Code after applying some "fix"
- Claude Code tests with echo commands to the server
- Server responds perfectly (because it's been working all along)
- Claude Code declares: "Success! The fix worked!"

### Phase 2: Reality Check
- User asks Claude Code to actually use the MCP tools
- Claude Code attempts to use `mcp__coderag__list_docs`
- Tool is not available
- Claude Code realizes it doesn't have MCP access

### Phase 3: The Cycle Continues
- User and Claude Code troubleshoot
- Various "fixes" are applied (stdout buffering, permissions, etc.)
- Manual tests always pass
- User restarts Claude Code
- Return to Phase 1

## The Facts

1. **The server works correctly** - It has been working since at least session 5
2. **The configuration is correct** - `.mcp.json` has the right settings
3. **Manual tests always pass** - Echo commands prove the server functions
4. **Claude Code never has MCP access** - The tools are never available

## What We've Tried (That Didn't Work)

1. **Stdout buffering fixes** - Multiple approaches, all unnecessary
2. **Permission changes** - The binary has correct permissions
3. **Binary rebuilds** - The server was never broken
4. **Configuration changes** - The config was always correct
5. **Debug flags** - Added but made no difference
6. **Fresh stdout handles** - Applied but server already worked

## The Real Issue

The disconnect appears to be between:
- What Claude Code can test (running commands via Bash)
- What Claude Code can access (MCP tools via the MCP protocol)

Testing the server with echo commands proves nothing about MCP tool availability.

## Recommendations

1. **Stop the cycle** - Accept that manual testing doesn't indicate MCP access
2. **Focus on the real problem** - Why doesn't Claude Code see configured MCP servers?
3. **Consider alternatives**:
   - Is there a different way to configure MCP servers?
   - Is there a startup flag or environment variable needed?
   - Is this a known limitation of Claude Code?

## For Future Sessions

When you restart Claude Code again (and you will), remember:
- Don't test with echo commands - they always work
- Don't declare victory until you can actually use the tools
- Read this document first
- The server is not broken
- The configuration is not wrong
- You still won't have MCP access

## The Definition of Insanity

"Doing the same thing over and over again and expecting different results."

We have been:
1. Testing the server (it works)
2. Checking the config (it's correct)
3. Restarting Claude Code (no MCP access)
4. Repeat

It's time to try something fundamentally different.
