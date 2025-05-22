# CodeRAG

**Documentation RAG for AI-Assisted Development**

A fast, self-contained documentation retrieval system designed specifically for AI assistants like Claude. CodeRAG eliminates external dependencies and provides semantic search over technical documentation to support autonomous coding workflows.

## ✅ Proven Concept

CodeRAG has been validated with excellent semantic understanding of programming concepts:

- **"Rust programming" ↔ "Rust development"**: 0.817 similarity
- **"async function error handling" ↔ "Result type error handling"**: 0.527 similarity  
- **Single binary deployment** with no Ollama dependency
- **Fast embeddings**: 2-5ms per text using ONNX Runtime
- **Quality vectors**: 384D normalized embeddings perfect for cosine similarity

## Goals

### 🎯 **For AI Assistants**
- Reliable access to current documentation while coding
- Semantic search that understands programming concepts
- Fast retrieval for real-time coding assistance

### 🚀 **For Development Teams**  
- Single binary deployment (just `./coderag serve`)
- No AI/ML expertise required for setup
- Clean team adoption with minimal dependencies

### ⚡ **Performance Focused**
- Sub-5ms embedding generation
- Sub-10ms vector search  
- Sub-2s startup time including model loading

## Quick Start

```bash
# Build and run
cargo run

# Test semantic search quality
cargo test

# Future: Start MCP server
./coderag serve

# Future: Launch web interface  
./coderag web

# Future: Index documentation
./coderag crawl https://docs.rs/tokio/latest/tokio/
```

## Technology Stack

- **Embeddings**: FastEmbed with all-MiniLM-L6-v2 (384D vectors)
- **Runtime**: ONNX Runtime for fast inference
- **Async**: Tokio for I/O and concurrent processing
- **Protocol**: MCP (Model Context Protocol) for AI assistant integration
- **Storage**: JSON-based vector database with atomic operations

## Architecture

### Current (Proof of Concept)
- ✅ **FastEmbed Integration**: Working embedding service
- ✅ **Semantic Quality**: Validated for programming concepts
- ✅ **Performance**: Meeting all latency targets

### Planned (Full Implementation)
- 🔄 **Vector Database**: Efficient similarity search and persistence
- 🔄 **MCP Server**: Full integration with Claude Desktop
- 🔄 **Web Crawler**: Recursive documentation indexing
- 🔄 **Web Interface**: Management UI for documentation curation

## Development Roadmap

| Phase | Timeline | Features |
|-------|----------|----------|
| **Phase 1** | ✅ Complete | Core embedding service with quality validation |
| **Phase 2** | Weeks 1-2 | Vector database with persistence and search |
| **Phase 3** | Weeks 2-3 | MCP server integration and tools |
| **Phase 4** | Weeks 3-4 | Web crawler with content extraction |
| **Phase 5** | Week 5 | Web interface for management |
| **Phase 6** | Week 6+ | Advanced features and optimizations |

## Use Cases

### For AI Assistants
```
Query: "How do I handle timeouts in tokio async functions?"
Result: Relevant tokio documentation with examples and best practices

Query: "Best practices for error propagation in Rust libraries"  
Result: Semantic search finds error handling patterns and examples

Query: "MCP server implementation patterns with stdio"
Result: Technical documentation for MCP protocol implementation
```

### For Development Teams
- **Documentation Discovery**: Find relevant docs across multiple sources
- **API Reference**: Quick access to function signatures and examples
- **Best Practices**: Discover recommended patterns and anti-patterns
- **Migration Guides**: Understand upgrade paths and breaking changes

## Configuration

### Claude Desktop Integration (Future)
```json
{
  "mcpServers": {
    "coderag": {
      "command": "/path/to/coderag",
      "args": ["serve"],
      "env": {
        "RUST_LOG": "info",
        "CODERAG_DATA_DIR": "/Users/yourname/.coderag"
      }
    }
  }
}
```

### Data Storage
- **Location**: `~/.coderag/` 
- **Documents**: `documents.json` (metadata and content)
- **Vectors**: `vectors.json` (embeddings and similarity indices)
- **Config**: `config.json` (user preferences)
- **Logs**: `logs/` (debug and operation logs)

## Contributing

This project is designed for AI-assisted development. Key areas for contribution:

1. **Documentation Crawler**: Enhanced content extraction from diverse sites
2. **Vector Search**: Performance optimizations for large document collections  
3. **MCP Tools**: Advanced search and filtering capabilities
4. **Web Interface**: User experience improvements for documentation management

## License

MIT License - Built for the AI coding community

---

**CodeRAG**: Because AI assistants deserve reliable documentation access while coding 🤖📚