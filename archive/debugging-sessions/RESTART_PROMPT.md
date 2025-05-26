# Post-Restart Prompt for Claude Code

Copy and paste this entire prompt after restarting Claude Code:

---

I just restarted you. We were working on the mcp-coderag project. We discovered that the MCP server wasn't working because of stdout buffering issues when connected to pipes (not terminals).

**The fix has been applied:**
- Modified `/Users/ken/dev/MCP/mcp-coderag/src/mcp/server.rs` to use fresh stdout handles
- Changed from `writeln!(&mut stdout, ...)` to `writeln!(io::stdout(), ...)`
- Rebuilt the binary at `/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp`

**Please immediately test if you now have access to the CodeRAG MCP tools:**
1. Try using `mcp__coderag__list_docs` - it should return `{"sources": [], "total_documents": 0, "last_updated": null}`
2. If that works, the fix succeeded!
3. If not, read `/Users/ken/dev/MCP/mcp-coderag/TROUBLESHOOTING_PLAN.md` for next steps

**Key files to reference:**
- `STDOUT_BUFFERING_FIX.md` - Explains the issue and solution
- `MCP_STDIO_TESTING_LOG.md` - Documents all tests performed
- `TROUBLESHOOTING_PLAN.md` - Step-by-step plan if still not working
- `RUST_SDK_DESIGN.md` - Design for a proper Rust MCP SDK
- `CLAUDE.md` - Updated with the fix and context

**What we learned:**
- The server was working correctly all along
- The issue was stdout buffering when piped
- Getting fresh stdout handles ensures proper flushing
- Test with: `echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}' | ./target/release/coderag-mcp`

**Current branch:** phase-4-web-crawler (Phase 4 complete, ready to merge)

Please test MCP access first, then let me know the results!
