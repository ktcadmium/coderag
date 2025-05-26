# MCP Server Behavior Patterns & Best Practices

**Date**: May 26, 2025
**Context**: Lessons learned from implementing CodeRAG MCP server with FastEmbed integration

## Executive Summary

This document captures critical insights about MCP (Model Context Protocol) server behavior in Claude Desktop, particularly around sandbox restrictions and resource initialization patterns. These learnings are essential for future AI memory system development.

## Key Discovery: MCP Server Lifecycle Phases

### Phase 1: Startup (Restricted Sandbox)

- **Environment**: Limited file system access
- **Permissions**: Cannot write to user directories (including `~/.cache`)
- **Duration**: Server initialization and capability registration
- **Best Practice**: Minimal initialization only

### Phase 2: Runtime (Broader Permissions)

- **Environment**: Access to user directories for normal operations
- **Permissions**: Can read/write to `~/.cache`, `~/.config`, etc.
- **Duration**: Tool execution and normal operations
- **Best Practice**: Heavy resource loading during first tool call

## The Lazy Initialization Pattern

### Problem Statement

Large ML models (~90MB FastEmbed) cannot be downloaded during MCP server startup due to sandbox restrictions, causing "Read-only file system" errors.

### Solution: Lazy Loading

```rust
pub struct EmbeddingService {
    model: Arc<Mutex<Option<fastembed::TextEmbedding>>>,
    init_once: Once,
}

impl EmbeddingService {
    pub fn new() -> Self {
        // No model download during startup
        Self {
            model: Arc::new(Mutex::new(None)),
            init_once: Once::new(),
        }
    }

    fn ensure_initialized(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.init_once.call_once(|| {
            // Download model during first tool call (runtime phase)
            let model = fastembed::TextEmbedding::try_new(Default::default())?;
            *self.model.lock().unwrap() = Some(model);
        });
        Ok(())
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        self.ensure_initialized()?;
        // Use initialized model
    }
}
```

### Benefits

1. **Instant Startup**: Server starts in <1 second vs 1-2 minutes
2. **Sandbox Compatibility**: No file system writes during restricted phase
3. **Better UX**: No manual initialization steps required
4. **Cleaner Code**: Removes workaround complexity

## Comparison with Other MCP Servers

### Working Patterns

Most successful MCP servers follow these patterns:

1. **Lightweight startup**: No heavy resource loading
2. **Runtime initialization**: Load resources during first use
3. **Containerization**: Many use Docker/npx for consistent environments
4. **Stateless design**: Minimal persistent state during startup

### Anti-Patterns (What Doesn't Work)

1. **Heavy startup initialization**: Downloading large files during server start
2. **Eager loading**: Initializing all resources before first tool call
3. **Startup timeouts**: Long initialization periods that block Claude
4. **File system assumptions**: Assuming full permissions during startup

## Technical Implementation Details

### Cache Directory Strategy

```rust
fn get_cache_dir() -> PathBuf {
    // Use consistent cache directory across startup and runtime
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fastembed")
}
```

### Error Handling

```rust
// Graceful degradation for network issues
match self.ensure_initialized() {
    Ok(_) => { /* proceed with embedding */ },
    Err(e) => {
        eprintln!("Model initialization failed: {}", e);
        // Return appropriate error to MCP client
    }
}
```

### Performance Characteristics

- **First call**: 1-2 minutes (model download + embedding)
- **Subsequent calls**: 2-5ms (cached model)
- **Memory usage**: ~200MB base + model (~90MB)
- **Startup time**: <1 second (no model loading)

## Broader Implications for AI Systems

### Memory System Architecture

1. **Layered Initialization**: Different components initialize at different lifecycle phases
2. **Resource Management**: Heavy resources loaded on-demand, not eagerly
3. **Sandbox Awareness**: AI systems must be designed for restricted startup environments
4. **Graceful Degradation**: Handle initialization failures without breaking core functionality

### Development Patterns

1. **Test Early**: Validate startup behavior in restricted environments
2. **Measure Performance**: Track startup time as a key metric
3. **User Experience**: Prioritize fast feedback loops over eager optimization
4. **Documentation**: Capture environment-specific behaviors for future reference

## Testing Strategies

### Startup Validation

```bash
# Test fast startup (should complete in <5 seconds)
timeout 5s ./target/release/coderag-mcp --debug 2>&1 | head -10

# Expected: Server starts and registers capabilities quickly
```

### Runtime Validation

```bash
# Test first-use model download
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search_docs","arguments":{"query":"test"}}}' | ./target/release/coderag-mcp

# Expected: Model downloads, then search executes
```

### Performance Monitoring

```bash
# Monitor memory usage during initialization
ps aux | grep coderag-mcp

# Monitor file system access
fs_usage -w -f filesys ./target/release/coderag-mcp
```

## Future Considerations

### Model Bundling

For offline scenarios, consider:

1. **Lightweight bundled models**: Include small model for offline use
2. **Progressive enhancement**: Upgrade to larger model when network available
3. **Model selection**: Allow users to choose model size vs. quality tradeoffs

### Background Warming

Potential optimization:

1. **Background download**: Start model download after server initialization
2. **Progress indicators**: Show download progress to users
3. **Retry logic**: Handle network failures gracefully

### Multi-Model Support

Future architecture considerations:

1. **Model registry**: Support multiple embedding models
2. **Lazy loading per model**: Each model initializes independently
3. **Resource pooling**: Share models across multiple tools

## Lessons for AI Memory Systems

### Short-Term Memory

- Fast access patterns essential for conversation flow
- In-memory structures for recent context
- No heavy initialization required

### Medium-Term Memory

- Session-based patterns and preferences
- Lightweight persistence (JSON, SQLite)
- Background synchronization acceptable

### Long-Term Memory

- Large knowledge bases and learned patterns
- Heavy resources (vector databases, large models)
- **Must use lazy initialization pattern**
- Network-dependent resources loaded on-demand

## Conclusion

The lazy initialization pattern is a fundamental requirement for MCP servers that use large ML models. This pattern should be considered standard practice for any AI system component that:

1. Runs in sandboxed environments
2. Requires large resource downloads
3. Needs fast startup times
4. Serves interactive AI assistants

**Key Takeaway**: Design for the most restrictive environment first, then enhance capabilities during runtime when broader permissions are available.

---

**Document Status**: Living document - update as new MCP behavior patterns are discovered
**Next Review**: After successful Claude Desktop deployment testing
