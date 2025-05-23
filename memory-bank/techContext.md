# CodeRAG Technical Context

## Technology Stack

### Core Dependencies
```toml
# Embedding and AI
fastembed = "4.8.0"           # ONNX-based embeddings, proven working

# Core Rust ML (for potential future custom models)
candle-core = "0.9.1"
candle-transformers = "0.9.1"
candle-nn = "0.9.1"

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# CLI and logging
clap = { version = "4.5", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"

# MCP Protocol (custom implementation)
jsonrpc-core = "18.0"
jsonrpc-derive = "18.0"
uuid = { version = "1.10", features = ["v4", "serde"] }
```

### Development Setup

#### Prerequisites
- Rust 1.70+ (for stable async traits)
- Cargo and rustup
- 4GB RAM minimum (for loading models)
- 500MB disk space (for ONNX models)

#### Build Commands
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Build MCP server only
cargo build --bin coderag-mcp

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Technical Architecture

### Project Structure
```
src/
├── lib.rs                    # Library exports
├── main.rs                   # Demo/test harness
├── embedding.rs              # Candle reference impl
├── embedding_basic.rs        # FastEmbed production impl
├── embedding_multi.rs        # Multi-strategy (disabled)
├── embedding_simple.rs       # Alternative (disabled)
├── vectordb/
│   ├── mod.rs               # Database interface
│   ├── storage.rs           # File persistence
│   ├── search.rs            # Similarity search
│   └── types.rs             # Data structures
├── mcp/
│   ├── mod.rs               # MCP module exports
│   ├── protocol.rs          # JSON-RPC types
│   ├── server.rs            # Stdio server
│   └── tools.rs             # Tool implementations
└── bin/
    └── mcp-server.rs        # MCP server entry point
```

### Key Technical Decisions

#### Why FastEmbed?
- **ONNX Runtime**: Production-ready, optimized inference
- **Model Management**: Auto-downloads and caches models
- **Performance**: 2-5ms per embedding on modern hardware
- **Compatibility**: Works across platforms without GPU

#### Why JSON Storage?
- **Debuggability**: Human-readable for troubleshooting
- **Simplicity**: No schema migrations or binary format issues
- **Sufficient**: Performance adequate for <1M documents
- **Atomic Writes**: Easy to implement with temp files

#### Why Custom MCP Implementation?
- **Official SDK Immature**: Still in early development
- **Control**: Full control over protocol implementation
- **Simplicity**: Direct JSON-RPC without abstraction layers
- **Compatibility**: Ensures Claude Desktop compatibility

### Performance Characteristics

#### Embedding Generation
- **Model Load**: ~2 seconds (first time)
- **Per Embedding**: 2-5ms (CPU)
- **Batch Size**: Currently 1, could batch for performance
- **Memory**: ~200MB for model

#### Vector Search
- **Algorithm**: O(n) linear scan with optimizations
- **10k docs**: <10ms search time
- **100k docs**: ~50ms search time
- **Memory**: ~400MB per 100k documents

#### MCP Communication
- **Protocol**: JSON-RPC over stdio
- **Latency**: <1ms for protocol overhead
- **Throughput**: Limited by embedding generation

### Development Patterns

#### Error Handling
```rust
// Application errors
use anyhow::{Result, Context};

// Library errors
use thiserror::Error;

// MCP errors mapped to codes
-32700: Parse error
-32600: Invalid request
-32601: Method not found
-32602: Invalid params
-32603: Internal error
```

#### Async Patterns
```rust
// All I/O operations are async
async fn operation() -> Result<T>

// Shared state with Arc<Mutex<T>>
let shared = Arc::new(Mutex::new(state));

// Concurrent operations with tokio::join!
let (a, b) = tokio::join!(op_a(), op_b());
```

#### Testing Strategy
- **Unit Tests**: Core algorithms (similarity, storage)
- **Integration Tests**: MCP protocol, end-to-end flows
- **Property Tests**: Future - embedding quality
- **Benchmarks**: Future - performance regression tests

### Deployment Considerations

#### Binary Size
- **Debug Build**: ~150MB (includes symbols)
- **Release Build**: ~40MB (stripped)
- **With Models**: +384MB (downloaded on first run)

#### Platform Support
- **macOS**: Full support (primary development)
- **Linux**: Full support (tested on Ubuntu)
- **Windows**: Should work, needs testing

#### Resource Requirements
- **CPU**: Any x86_64 or ARM64
- **RAM**: 512MB minimum, 2GB recommended
- **Disk**: 1GB for binary + models + data
- **Network**: Only for initial model download
