# CodeRAG

**AI-Powered Documentation Search for Better Code**

CodeRAG gives AI coding assistants like Claude instant access to up-to-date documentation through semantic search. No more outdated information or hallucinated APIs - just accurate, relevant documentation when you need it.

## Features

- 🚀 **Lightning Fast**: Get relevant documentation in milliseconds
- 🎯 **Semantic Search**: Understands programming concepts, not just keywords
- 📦 **Single Binary**: No Docker, no dependencies, just download and run
- 🤖 **Claude Desktop Ready**: Works seamlessly with MCP (Model Context Protocol)
- 📚 **Smart Indexing**: Crawl and index any documentation site
- 🔄 **Lazy Loading**: AI model downloads automatically on first use
- 🛡️ **Robust**: Handles network restrictions and sandbox environments
- 📁 **Per-Project Databases**: Each project maintains its own isolated documentation

## Installation

### Build from Source

Requirements:

- Rust 1.70 or later
- 4GB RAM (for embedding models)
- Internet connection (for initial model download)

```bash
git clone https://github.com/yourusername/coderag.git
cd coderag
cargo build --release --bin coderag-mcp
```

The binary will be at `target/release/coderag-mcp`.

### Development Workflow

Use the included Taskfile for common operations:

```bash
# Install Task runner (if not already installed)
brew install go-task/tap/go-task  # macOS
# or: go install github.com/go-task/task/v3/cmd/task@latest

# Quick development check
task

# Build release binary
task release

# Test crawling functionality
task crawl-test

# See all available tasks
task --list
```

## Quick Start

### 1. Configure Claude Desktop

Add CodeRAG to your Claude Desktop configuration:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "coderag": {
      "command": "/path/to/coderag-mcp",
      "args": [],
      "env": {
        "HF_HUB_USER_AGENT_ORIGIN": "CodeRAG/0.1.0"
      }
    }
  }
}
```

### 2. Start Using It!

Once configured, restart Claude Desktop. CodeRAG will start automatically when Claude needs it.

**First Use**: The AI model (~90MB) downloads automatically on your first search. This takes 1-2 minutes but only happens once.

Example queries:

- "Search for async error handling in Rust"
- "Find tokio timeout examples"
- "Show me how to use MCP tools"

## Available MCP Tools

CodeRAG provides these tools to AI assistants:

### `search_docs`

Search indexed documentation with semantic understanding:

```json
{
  "query": "async timeout handling",
  "limit": 5,
  "source_filter": "docs.rs",
  "content_type": "documentation"
}
```

### `list_docs`

See what documentation is indexed:

```json
{}
```

### `crawl_docs`

Index new documentation sources:

```json
{
  "url": "https://docs.rs/tokio/latest/",
  "mode": "single",
  "focus": "all",
  "max_pages": 100
}
```

**Crawl Modes:**

- `single`: Just the specified page (recommended for MCP)
- `section`: Page and its direct children
- `full`: Entire documentation site

**Focus Options:**

- `api`: API reference documentation
- `examples`: Code examples and tutorials
- `changelog`: Version history and updates
- `quickstart`: Getting started guides
- `all`: No specific focus (recommended)

### `reload_docs`

Refresh the document database from disk:

```json
{}
```

## How It Works

1. **Lazy Initialization**: Model downloads only when first needed, avoiding startup delays
2. **Semantic Understanding**: Uses all-MiniLM-L6-v2 embeddings (384 dimensions) for concept matching
3. **Fast Search**: Vector similarity search with cosine distance
4. **MCP Integration**: Standard JSON-RPC communication with Claude Desktop
5. **Smart Storage**: Project-specific databases in `.coderag/` or global fallback

## Per-Project Databases

CodeRAG automatically detects your project context and maintains isolated documentation databases:

### How It Works

1. **Automatic Detection**: Looks for project markers (`.git`, `package.json`, `Cargo.toml`, etc.)
2. **Local Storage**: Creates `.coderag/vectordb.json` in your project root
3. **Automatic Gitignore**: Adds `.coderag/` to your `.gitignore` automatically
4. **Fallback**: Uses global database (`~/.coderag/`) when not in a project

### Benefits

- **Relevant Results**: Each project only searches its own documentation
- **Fast Context Switching**: No need to manage databases manually
- **Clean Repositories**: Vector databases excluded from version control
- **Efficient Storage**: Only index the docs you actually need per project

### Example

```bash
# In a Rust project
cd my-rust-project
# CodeRAG automatically uses my-rust-project/.coderag/vectordb.json

# In a Node.js project
cd my-node-project
# CodeRAG automatically uses my-node-project/.coderag/vectordb.json

# Outside any project
cd /tmp
# CodeRAG falls back to ~/.coderag/coderag_vectordb.json
```

## Performance

- **Search Speed**: <10ms for typical document collections
- **Embedding Generation**: 2-5ms per query (after model loading)
- **Model Loading**: ~4ms warm-up time (after initial download)
- **Startup Time**: Instant (model loads on first search)
- **Memory Usage**: ~200MB base + document storage

## Troubleshooting

### ONNX Schema Warnings

You may see ONNX schema warnings during model loading - these are harmless and don't affect functionality.

### Network Issues

If model download fails:

1. Check your internet connection
2. Try running the crawler directly: `./coderag-mcp crawl https://httpbin.org --verbose`
3. Ensure the `HF_HUB_USER_AGENT_ORIGIN` environment variable is set

### Debug Mode

Run with debug logging to see detailed operation:

```bash
./coderag-mcp --debug
```

### Data Location

CodeRAG stores data in `~/.coderag/` by default. You can change this with `--data-dir`.

## Architecture

### Embedding Strategy

- **Model**: all-MiniLM-L6-v2 (384-dimensional vectors)
- **Provider**: FastEmbed with ONNX Runtime
- **Initialization**: Lazy loading on first search request

### Storage

- **Format**: JSON-based vector database
- **Location**: `~/.coderag/coderag_vectordb.json`
- **Persistence**: Atomic writes with temp file + rename

### MCP Integration

- **Protocol**: JSON-RPC over stdio
- **Transport**: Standard MCP stdio transport
- **Error Handling**: Proper MCP error codes and messages

## Current Status

✅ **Stable Features:**

- Semantic search with fast embeddings
- MCP server with Claude Desktop integration
- Single-page documentation crawling
- Lazy model initialization
- Robust error handling and network compatibility

🚧 **In Development:**

- Multi-page crawling in MCP context
- Web UI for management
- Additional embedding models

## Contributing

CodeRAG is open source! Check out our [developer documentation](CLAUDE.md) and memory bank in `memory-bank/` for technical details.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

Built with ❤️ for the AI coding community
