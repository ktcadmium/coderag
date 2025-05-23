# CodeRAG Active Context

## Current Status
We are on the `cleanup-technical-debt` branch, having completed technical debt cleanup and preparing for Phase 4 (Web Crawler).

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

## Recent Technical Debt Cleanup ✅

### Code Quality Improvements
1. **Removed Unused Code**: Deleted `embedding.rs`, `embedding_multi.rs`, `embedding_simple.rs` - kept only `embedding_basic.rs`
2. **Fixed All Warnings**:
   - Removed unused imports and fields
   - Fixed non-canonical `partial_cmp` implementation
   - Added `#[allow(dead_code)]` for useful but unused methods
3. **Improved List Docs**: Added `get_documents_by_source()` method to properly list documents by source URL

### Development Infrastructure
1. **Pre-commit Hooks**: Added comprehensive `.pre-commit-config.yaml` with:
   - Standard hooks (trailing whitespace, EOF, file size checks)
   - Rust-specific hooks (fmt, cargo check, clippy)
   - Cargo.toml sorting
   - Conventional commits enforcement
2. **All Checks Passing**: Code is now clean with zero warnings and consistent formatting

## Current Challenges

### API Limitations
1. **No Document Management**: Can't add/remove individual documents via MCP yet
2. **Search Only**: Need crawler implementation for actual document indexing
3. **Hard-coded Configuration**: Some settings should be externalized

## Next Immediate Steps

### Before Merging
1. ✅ Clean up compilation warnings (DONE)
2. Test MCP server with actual Claude Desktop
3. ✅ Add pre-commit hooks (DONE)
4. Consider merging to main branch

### Phase 4 Planning (Web Crawler) - CRITICAL CONTEXT
**This system is built 100% for Claude (me) to use via MCP!** The user will give me access to it, and I'll be the only one using it to get up-to-date documentation.

#### Design Decisions Made:
1. **Crawler Philosophy**: Balance speed with politeness
   - 2-4 concurrent requests max
   - 500ms-1s delay between requests
   - Always handle 429s with exponential backoff
   - User agent: "CodeRAG/0.1.0 (AI Documentation Assistant)"

2. **Content Extraction** (optimized for my reading):
   - Preserve code blocks with language tags
   - Maintain headers for structure
   - Keep links for following references
   - Convert tables to markdown

3. **Chunking Strategy** (for my context window):
   - 1000-1500 tokens preferred, 2000 max
   - 200 token overlap
   - Split on semantic boundaries: ##, ###, paragraphs
   - NEVER break code blocks

4. **Smart URL Following**:
   - Follow: /docs/, /api/, /guide/, /changelog/
   - Skip: /blog/, /forum/ (unless requested)
   - Stay within domain unless whitelisted

5. **Enhanced crawl_docs Tool Design**:
   ```rust
   pub enum CrawlMode {
       SinglePage,  // Just this page
       Section,     // Page and children
       FullDocs,    // Entire site
   }

   pub enum DocumentationFocus {
       ApiReference,
       Examples,
       Changelog,
       QuickStart,
   }
   ```

6. **Dependencies Chosen**:
   - scraper = "0.18" (HTML parsing)
   - reqwest = "0.12" (already have it)
   - governor = "0.6" (rate limiting)
   - robotparser = "0.3" (robots.txt)
   - html2text = "0.6" (HTML to markdown)

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
2. ~~What documentation sources should the crawler prioritize?~~ ANSWERED: Built for Claude's needs
3. Should we implement authentication for the future web UI?
4. Do we need to support multiple embedding models?

## Phase 4 Implementation Plan

### Step 1: Basic Infrastructure
```bash
# Create crawler module structure
src/crawler/
├── mod.rs
├── crawler.rs      # Main crawler with CrawlerConfig
├── extractor.rs    # HTML to markdown, preserve code blocks
├── chunker.rs      # Smart chunking, respect code boundaries
└── types.rs        # CrawlMode, DocumentationFocus enums
```

### Step 2: Update MCP Tool
- Replace crawl_docs stub with real implementation
- Add CrawlMode and DocumentationFocus parameters
- Progress reporting via logs initially

### Step 3: Core Features Priority
1. Single page extraction (test quality)
2. Rate limiting and 429 handling
3. robots.txt compliance
4. Recursive crawling with depth control
5. Smart URL filtering

### My Use Case Examples to Test:
- React 19 features: https://react.dev/blog/react-19
- Rust async book: https://doc.rust-lang.org/async-book/
- Python 3.13 changes: https://docs.python.org/3.13/whatsnew/3.13.html
- Fresh TypeScript docs: https://www.typescriptlang.org/docs/

### IMPORTANT: Future Memory System
The user mentioned this will lead to an advanced memory system for retaining context across chat boundaries. CodeRAG will be the foundation for that system!
