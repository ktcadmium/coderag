# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**CodeRAG** is a documentation RAG (Retrieval-Augmented Generation) system designed specifically to give AI assistants (especially Claude) reliable, current documentation access while coding. This is a complete rewrite of the original Go-based Omnidoc MCP server, optimized for autonomous AI development workflows.

### Primary Goals

1. **Eliminate External Dependencies**: Single binary deployment with no Ollama requirement
2. **Clean Team Adoption**: Simple setup for development teams with minimal AI/ML experience  
3. **Autonomous AI Coding Support**: Reliable access to current documentation for AI assistants during programming
4. **Performance**: Fast embedding generation and vector search for real-time coding assistance
5. **Quality Documentation Retrieval**: Semantic search that understands programming concepts and technical relationships

### Core Architecture

- **MCP Server**: Stdio-based server compatible with Claude Desktop and other MCP clients
- **Embedded Vector Database**: Pure Rust implementation with efficient similarity search
- **FastEmbed Integration**: ONNX Runtime-based embeddings (all-MiniLM-L6-v2, 384 dimensions)
- **Web Interface**: Management UI for documentation curation and monitoring
- **Advanced Web Crawler**: Recursive crawling with content extraction and rate limiting
- **Semantic Search**: Context-aware search optimized for programming documentation

## Proven Technology Stack

### ✅ **Validated in Proof of Concept**

**Embedding Strategy**: FastEmbed with all-MiniLM-L6-v2
- **Quality**: Excellent semantic understanding of programming concepts
- **Performance**: Fast inference (~2-5ms per embedding)
- **Deployment**: ONNX Runtime, auto-downloads models on first run
- **Dimensions**: 384D vectors, perfectly normalized for cosine similarity
- **Semantic Results**: 
  - "Rust programming" ↔ "Rust development": 0.817 similarity
  - "async function error handling" ↔ "Result type error handling": 0.527 similarity
  - Clear discrimination between unrelated concepts

**Core Dependencies**:
```toml
# Embedding and AI
fastembed = "4.8.0"           # ONNX-based embeddings, proven working

# Core Rust ML (for potential future custom models)
candle-core = "0.9.1"
candle-transformers = "0.9.1"
candle-nn = "0.9.1"

# Standard async/web stack
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Build and Run Commands

### Development

```bash
# Build the project
cargo build

# Run with semantic quality testing
cargo run

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Using Task Runner (Recommended)

```bash
# Build the binary
task build

# Run embedding service only
task embed

# Run full MCP server (future)
task server

# Run with web interface (future)
task web

# Run all tests
task test
```

## Code Architecture

### Current Structure (Proof of Concept)

```
src/
├── main.rs                    # Entry point with embedding quality testing
├── embedding_basic.rs         # FastEmbed integration (working)
├── embedding.rs              # Candle mock implementation (reference)
└── lib.rs                    # Future: library exports
```

### Target Architecture (Full Implementation)

```
src/
├── main.rs                    # CLI entry point
├── lib.rs                     # Library exports for MCP tools
├── embedding/
│   ├── mod.rs                 # Embedding service abstraction  
│   ├── fastembed.rs          # FastEmbed implementation (primary)
│   └── candle.rs             # Future: pure Candle implementation
├── vectordb/
│   ├── mod.rs                 # Vector database interface
│   ├── storage.rs            # File-based persistence
│   └── search.rs             # Similarity search algorithms
├── crawler/
│   ├── mod.rs                 # Web crawler interface
│   ├── html_parser.rs        # Content extraction
│   ├── rate_limiter.rs       # Adaptive rate limiting
│   └── robots.rs             # robots.txt compliance
├── mcp/
│   ├── mod.rs                 # MCP server implementation
│   ├── tools.rs              # MCP tool definitions
│   └── protocol.rs           # MCP protocol handling
└── web/
    ├── mod.rs                 # Web interface server
    ├── api.rs                # REST API endpoints
    └── static/               # Static web assets
```

## Implementation Roadmap

### Phase 1: Core Embedding Service ✅ COMPLETE
- [x] FastEmbed integration with all-MiniLM-L6-v2
- [x] Semantic quality validation for programming concepts
- [x] Error handling and model fallback strategies
- [x] Performance benchmarking (sub-5ms embedding generation)

### Phase 2: Vector Database (Weeks 1-2)
- [ ] Port vector storage from Go implementation
- [ ] Implement cosine similarity search
- [ ] Add persistence layer with atomic file operations
- [ ] Support for document metadata and sectioning
- [ ] Concurrent access with proper locking

### Phase 3: MCP Server Integration (Weeks 2-3)
- [ ] Official Rust MCP SDK integration
- [ ] Implement core MCP tools:
  - `search_docs`: Semantic search with quality scoring
  - `list_docs`: Document inventory with metadata
  - `crawl_docs`: Dynamic documentation indexing
  - `reload_docs`: Database refresh functionality
- [ ] Stdio protocol handling for Claude Desktop

### Phase 4: Web Crawler (Weeks 3-4)
- [ ] Port crawler logic from Go using `reqwest` + `scraper`
- [ ] Advanced HTML content extraction
- [ ] Adaptive rate limiting with 429 detection
- [ ] Robots.txt compliance
- [ ] Section-based content chunking

### Phase 5: Web Interface (Week 5)
- [ ] Management UI using `axum` or `warp`
- [ ] Real-time crawling status
- [ ] Document management (add/remove/update)
- [ ] Search testing interface
- [ ] Analytics and metrics dashboard

### Phase 6: Advanced Features (Week 6+)
- [ ] Multiple embedding model support
- [ ] Custom domain-specific fine-tuning
- [ ] Document versioning and change detection
- [ ] Distributed deployment options
- [ ] Performance optimizations and caching

## Key Design Decisions

### Embedding Strategy
**Choice**: FastEmbed with ONNX Runtime over pure Candle
**Rationale**: 
- Proven stability and performance
- Excellent semantic understanding of programming concepts
- No complex model implementation required
- Wide model compatibility (can easily switch embedding models)
- Mature ONNX ecosystem

### Data Storage
**Location**: `~/.mcp-docs/` (consistent with Go version)
**Format**: JSON for simplicity and debugging, with atomic writes
**Files**:
- `documents.json`: Document metadata and content
- `vectors.json`: Embedding vectors with similarity indices  
- `config.json`: User preferences and settings
- `logs/`: Debug and operation logs

### Concurrency Model
**Approach**: Tokio async runtime with structured concurrency
**Rationale**: 
- Natural fit for I/O heavy operations (crawling, embedding)
- Excellent performance for MCP stdio protocol
- Clean error propagation with `anyhow`
- Compatible with web server frameworks

## Development Guidelines

### Code Style
- **Error Handling**: Use `anyhow` for application errors, `thiserror` for library errors
- **Async**: Prefer `async/await` over blocking operations
- **Logging**: Use `tracing` with structured logging
- **Configuration**: Environment variables + config files, serde-based
- **Testing**: Unit tests for core logic, integration tests for MCP tools

### Documentation Standards
- **README.md**: User-facing documentation with setup and usage
- **CLAUDE.md**: AI assistant guidance (this file)
- **API Documentation**: Comprehensive rustdoc for all public APIs
- **Examples**: Working examples for common use cases

### Performance Targets
- **Embedding Generation**: < 5ms per text (achieved: ~2ms)
- **Vector Search**: < 10ms for 10k documents
- **Memory Usage**: < 100MB for typical documentation collections
- **Startup Time**: < 2 seconds including model loading

## MCP Integration

### Target MCP Tools

Based on the Go implementation, provide these enhanced tools:

#### `search_docs`
```json
{
  "query": "async error handling in Rust",
  "limit": 5,
  "source_filter": "docs.rs",  // Optional: filter by documentation source
  "content_type": "code"        // Optional: prefer code examples
}
```

Returns semantic search results with:
- Relevance scores and confidence metrics
- Document sections with preserved context
- Source URLs and metadata
- Related links and cross-references

#### `list_docs`
```json
{}
```

Returns comprehensive documentation inventory:
- All indexed sources with page counts
- Last update timestamps
- Content statistics (sections, code blocks, etc.)
- Quality metrics and coverage analysis

#### `crawl_docs`
```json
{
  "url": "https://docs.rs/tokio/latest/tokio/",
  "recursive": true,
  "max_pages": 100
}
```

Dynamically crawls and indexes documentation:
- Asynchronous processing with progress updates
- Respects robots.txt and rate limiting
- Automatic content extraction and sectioning
- Real-time embedding generation and storage

#### `reload_docs`
```json
{}
```

Refreshes the document database:
- Reloads from persistent storage
- Updates embedding indices
- Returns updated statistics

### Claude Desktop Configuration

```json
{
  "mcpServers": {
    "omnidoc-rust": {
      "command": "/path/to/omnidoc-rust",
      "args": ["--mode", "server"],
      "env": {
        "RUST_LOG": "info",
        "OMNIDOC_DATA_DIR": "/Users/yourname/.mcp-docs"
      }
    }
  }
}
```

## Testing Strategy

### Unit Tests
- Embedding service with mock inputs
- Vector similarity calculations
- Content extraction and parsing
- Configuration loading and validation

### Integration Tests
- End-to-end MCP tool functionality
- Web crawler with test documentation sites
- Database persistence and concurrent access
- Error handling and recovery scenarios

### Performance Tests
- Embedding generation benchmarks
- Vector search performance scaling
- Memory usage under load
- Concurrent request handling

### Quality Assurance
- Semantic search accuracy with programming queries
- Content extraction quality across different documentation sites
- Embedding consistency and reproducibility
- Cross-platform compatibility (macOS, Linux, Windows)

## Autonomous AI Coding Use Cases

This system is specifically designed to support AI assistants during autonomous coding sessions:

### Primary Use Cases
1. **API Reference Lookup**: Find specific function signatures and usage examples
2. **Error Resolution**: Semantic search for error patterns and solutions
3. **Best Practices**: Discover recommended patterns and anti-patterns
4. **Framework Integration**: Understand how to integrate different libraries
5. **Migration Guides**: Find upgrade paths and breaking changes

### Example Queries (AI Assistant Perspective)
- "How do I handle timeouts in tokio async functions?"
- "Best practices for error propagation in Rust libraries"
- "MCP server implementation patterns with stdio"
- "FastEmbed model configuration and performance tuning"
- "Semantic chunking strategies for documentation"

### Quality Metrics for AI Use
- **Relevance**: Does the top result directly answer the programming question?
- **Context**: Are code examples included with sufficient surrounding explanation?
- **Freshness**: Is the documentation current with latest library versions?
- **Completeness**: Are related concepts and alternatives also surfaced?

## Success Metrics

### Technical Metrics
- **Search Accuracy**: >90% relevant results for programming queries
- **Performance**: <5ms embedding, <10ms search, <2s startup
- **Reliability**: 99.9% uptime, graceful degradation on errors
- **Coverage**: Support for major Rust documentation sites (docs.rs, etc.)

### User Experience Metrics
- **Setup Time**: <5 minutes from download to first search
- **Learning Curve**: No AI/ML expertise required for operation
- **Documentation Quality**: High-quality extraction from diverse site formats
- **Integration**: Seamless Claude Desktop MCP experience

This Rust rewrite represents a fundamental improvement in deployment simplicity while maintaining the sophisticated RAG capabilities that make autonomous AI coding effective.