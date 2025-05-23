# CodeRAG System Patterns

## Architecture Overview
```
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐
│  Claude Desktop │────▶│  MCP Server  │────▶│ Embedding Svc   │
└─────────────────┘     └──────────────┘     └─────────────────┘
                               │                      │
                               ▼                      ▼
                        ┌──────────────┐     ┌─────────────────┐
                        │  Vector DB   │────▶│   FastEmbed     │
                        └──────────────┘     └─────────────────┘
```

## Key Design Patterns

### 1. Service Architecture
- **Modular Services**: Separate concerns for embedding, storage, and MCP
- **Async Everything**: Tokio-based async runtime for I/O operations
- **Dependency Injection**: Services composed at startup, not hardcoded

### 2. Embedding Service Pattern
```rust
// Multiple implementations, single interface
pub trait EmbeddingService {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

// Primary: FastEmbed (proven, fast)
// Fallback: Candle (pure Rust, future)
```

### 3. Vector Database Design
- **Layered Architecture**:
  - Storage Layer: File persistence with atomic operations
  - Search Layer: Similarity algorithms and filtering
  - API Layer: Clean public interface
- **JSON Storage**: Chosen for debuggability over binary formats
- **In-Memory Search**: All vectors loaded for fast similarity computation

### 4. MCP Protocol Implementation
- **JSON-RPC 2.0**: Standard protocol for tool communication
- **Stdio Transport**: Reliable pipe-based communication
- **Stateless Tools**: Each tool call is independent
- **Error Bubbling**: Errors converted to MCP error responses

## Critical Implementation Decisions

### Embedding Strategy
- **Model**: all-MiniLM-L6-v2 (384D vectors)
- **Provider**: FastEmbed with ONNX Runtime
- **Why**: Best balance of quality, speed, and deployment simplicity

### Storage Architecture
```
~/.coderag/
├── coderag_vectordb.json    # Single file database
└── logs/                    # Future: operational logs
```

### Search Algorithm
1. Cosine similarity for all vectors (parallelized)
2. BinaryHeap for efficient top-K selection
3. Optional filtering by source/type
4. Score normalization (0.0 to 1.0)

### Concurrency Model
- **Tokio Runtime**: Multi-threaded async executor
- **Arc<Mutex<T>>**: Shared state for MCP tools
- **Atomic Operations**: File writes use temp + rename

## Component Relationships

### Data Flow
1. **Indexing** (Future):
   ```
   URL → Crawler → HTML Parser → Chunker → Embedder → VectorDB
   ```

2. **Searching**:
   ```
   Query → Embedder → Vector → Similarity Search → Results
   ```

3. **MCP Communication**:
   ```
   Claude → JSON-RPC → MCP Server → Tool Handler → Response
   ```

### Error Handling Strategy
- **anyhow**: Application-level errors with context
- **thiserror**: Library errors with types
- **MCP Errors**: Mapped to JSON-RPC error codes
- **Graceful Degradation**: Operations continue despite partial failures

## Performance Optimizations
1. **Model Caching**: FastEmbed models loaded once, reused
2. **Vector Normalization**: Pre-normalized for faster cosine similarity
3. **Lazy Loading**: Documents loaded on-demand, not eagerly
4. **Batch Operations**: Future - batch embedding generation

## Security Considerations
- **Local Only**: No network access in current implementation
- **File Permissions**: Relies on OS-level security
- **Input Validation**: All MCP inputs validated before processing
- **No Secrets**: No API keys or credentials stored