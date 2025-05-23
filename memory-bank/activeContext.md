# CodeRAG Active Context

## Current Status
We are on the `phase-3-mcp-integration` branch, having just completed Phase 3 of the implementation roadmap.

## Recent Work Completed

### Phase 3: MCP Server Integration ✅
1. **Created MCP Protocol Implementation**
   - Custom JSON-RPC implementation (official SDK too early)
   - Full protocol types in `src/mcp/protocol.rs`
   - Stdio-based server for Claude Desktop compatibility

2. **Implemented All MCP Tools**
   - `search_docs`: Full semantic search with filtering
   - `list_docs`: Document inventory reporting
   - `crawl_docs`: Stub for Phase 4 implementation
   - `reload_docs`: Database reload functionality

3. **Built MCP Server Binary**
   - Standalone `coderag-mcp` executable
   - Command-line args for data directory and debug mode
   - Proper error handling and logging

4. **Added Integration Tests**
   - 7 comprehensive tests all passing
   - Tests cover all MCP methods and error cases
   - Note: ONNX warnings are harmless (known issue)

5. **Updated Documentation**
   - Added MCP configuration instructions
   - Documented all available tools
   - Added Claude Desktop setup guide

## Current Challenges

### Technical Debt
1. **Multiple Embedding Implementations**: Need to consolidate to just `embedding_basic.rs`
2. **Unused Imports**: Several warnings that should be cleaned up
3. **Hard-coded Values**: Some configuration should be externalized

### API Limitations
1. **List Docs Incomplete**: Currently returns placeholder data (no direct access to all docs)
2. **No Document Management**: Can't add/remove individual documents via MCP yet
3. **Search Only**: Need crawler implementation for actual document indexing

## Next Immediate Steps

### Before Merging
1. Clean up compilation warnings
2. Test MCP server with actual Claude Desktop
3. Add error handling for edge cases
4. Consider adding document management tools

### Phase 4 Planning (Web Crawler)
1. Choose HTML parsing library (likely `scraper`)
2. Implement recursive crawling logic
3. Add robots.txt compliance
4. Create content extraction pipeline
5. Handle rate limiting and retries

## Key Decisions Made

### MCP Implementation Approach
- Chose custom JSON-RPC over official SDK due to SDK immaturity
- Used stdio transport for maximum compatibility
- Implemented all tools even if some are stubs

### Error Handling Strategy
- Convert all errors to MCP error responses
- Log errors but don't crash server
- Provide helpful error messages to AI

### Testing Philosophy
- Integration tests over unit tests for MCP
- Test actual protocol communication
- Ensure graceful handling of malformed requests

## Important Patterns Discovered

### Async Service Initialization
```rust
// Services need async new() for model loading
pub async fn new() -> Result<Self>
```

### MCP Tool Response Format
```rust
// Tools must return this exact structure
{
  "content": [{
    "type": "text",
    "text": "json_stringified_response"
  }]
}
```

### File Path Handling
```rust
// Always use absolute paths in tools
// Expand ~ to home directory
// Create directories if missing
```

## Project Insights

1. **FastEmbed Quality**: Semantic similarity scores are excellent for programming concepts
2. **JSON Storage Sufficient**: No performance issues even with larger test sets
3. **MCP Protocol Simple**: Basic JSON-RPC implementation works perfectly
4. **Stdio Reliable**: No issues with pipe-based communication

## Configuration Notes

### Environment Variables
- `RUST_LOG`: Set to `debug` for verbose output
- `HOME`: Used for expanding `~` in paths

### Default Paths
- Data directory: `~/.coderag/`
- Database file: `coderag_vectordb.json`
- Models cached by FastEmbed: `~/.cache/fastembed/`

## Questions for Next Session

1. Should we add document management tools (add_doc, remove_doc)?
2. What documentation sources should the crawler prioritize?
3. Should we implement authentication for the future web UI?
4. Do we need to support multiple embedding models?
