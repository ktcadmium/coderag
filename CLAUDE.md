# CLAUDE.md - Developer Guide for Claude Code

This file provides guidance to Claude Code (claude.ai/code) when working with the CodeRAG codebase.

## Memory Bank Context

The complete project context is maintained in the memory-bank directory. These files are automatically included:

@memory-bank/projectbrief.md
@memory-bank/productContext.md
@memory-bank/systemPatterns.md
@memory-bank/techContext.md
@memory-bank/activeContext.md
@memory-bank/progress.md

## Quick Development Reference

### Current Status
- **Branch**: `phase-3-mcp-integration` (Phase 3 complete, ready to merge)
- **Next Phase**: Web crawler implementation (Phase 4)

### Key Commands
```bash
# Build and test
cargo build
cargo test
cargo build --bin coderag-mcp

# Run with logging
RUST_LOG=debug cargo run

# Run MCP server
./target/debug/coderag-mcp --debug
```

### Project Structure
```
src/
├── lib.rs                    # Library exports
├── main.rs                   # Demo/test harness
├── embedding_basic.rs        # FastEmbed implementation (PRIMARY)
├── vectordb/                 # Vector database module
├── mcp/                      # MCP server implementation
└── bin/
    └── mcp-server.rs        # MCP server binary
```

### Development Patterns

#### Error Handling
```rust
use anyhow::Result;  // For application code
use thiserror::Error; // For library errors
```

#### Async Services
```rust
// Services need async initialization
pub async fn new() -> Result<Self>
```

#### MCP Response Format
```rust
// Tools must return this structure
{
  "content": [{
    "type": "text",
    "text": "json_stringified_response"
  }]
}
```

### Testing Approach
- Integration tests over unit tests for MCP functionality
- Test files in `tests/` directory
- Run with `cargo test --test <test_name>`

### Performance Targets
- Embedding: <5ms (achieved: 2-5ms)
- Search: <10ms for 10k docs
- Startup: <2s including model load

### Known Issues
- ONNX schema warnings in tests (harmless)
- Some unused imports need cleanup
- List docs returns placeholder data

## Working with Claude Code

When using Claude Code for development:

1. **Memory bank is auto-loaded** - Context from included files above
2. **Check activeContext.md** - Current work and decisions
3. **Update memory bank after significant changes** - Keep it current
4. **Follow established patterns** - Consistency matters

## Contributing

When making changes:
- Update relevant memory bank files
- Add tests for new functionality
- Keep performance targets in mind
- Document significant decisions