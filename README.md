# CodeRAG

**AI-Powered Documentation Search for Better Code**

CodeRAG gives AI coding assistants like Claude instant access to up-to-date documentation through semantic search. No more outdated information or hallucinated APIs - just accurate, relevant documentation when you need it.

## Features

- 🚀 **Lightning Fast**: Get relevant documentation in milliseconds
- 🎯 **Semantic Search**: Understands programming concepts, not just keywords
- 📦 **Single Binary**: No Docker, no dependencies, just download and run
- 🤖 **Claude Desktop Ready**: Works seamlessly with MCP (Model Context Protocol)
- 📚 **Smart Indexing**: Crawl and index any documentation site (coming soon)

## Installation

### Download Pre-built Binary (Recommended)

Coming soon! For now, build from source.

### Build from Source

Requirements:
- Rust 1.70 or later
- 4GB RAM (for embedding models)

```bash
git clone https://github.com/yourusername/coderag.git
cd coderag
cargo build --release --bin coderag-mcp
```

The binary will be at `target/release/coderag-mcp`.

## Quick Start

### 1. Run CodeRAG Server

```bash
# Start with default settings
./coderag-mcp

# Or with custom data directory
./coderag-mcp --data-dir /path/to/your/data
```

### 2. Configure Claude Desktop

Add CodeRAG to your Claude Desktop configuration:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
**Linux**: `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "coderag": {
      "command": "/path/to/coderag-mcp",
      "args": []
    }
  }
}
```

### 3. Start Using It!

Once configured, Claude can search documentation for you:

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
  "content_type": "code"
}
```

### `list_docs`
See what documentation is indexed:
```json
{}
```

### `crawl_docs` (Coming Soon)
Index new documentation sources:
```json
{
  "url": "https://docs.rs/tokio/latest/",
  "recursive": true,
  "max_pages": 100
}
```

### `reload_docs`
Refresh the document database:
```json
{}
```

## How It Works

1. **Semantic Understanding**: CodeRAG uses advanced embeddings to understand what you're looking for
2. **Fast Search**: Vector similarity search finds the most relevant documentation
3. **MCP Integration**: Claude Desktop communicates with CodeRAG through the Model Context Protocol
4. **Local Storage**: All data stored locally in `~/.coderag/`

## Performance

- **Search Speed**: <10ms for 10,000 documents
- **Embedding Generation**: 2-5ms per query
- **Startup Time**: ~2 seconds (including model loading)
- **Memory Usage**: ~200MB base + documents

## Troubleshooting

### ONNX Warnings
You may see ONNX schema warnings - these are harmless and don't affect functionality.

### Debug Mode
Run with debug logging to see what's happening:
```bash
./coderag-mcp --debug
```

### Data Location
CodeRAG stores data in `~/.coderag/` by default. You can change this with `--data-dir`.

## Roadmap

- [x] Semantic search engine
- [x] MCP server implementation
- [x] Claude Desktop integration
- [ ] Web crawler for documentation
- [ ] Web UI for management
- [ ] Multiple embedding models
- [ ] API access

## Contributing

CodeRAG is open source! Check out our [developer documentation](CLAUDE.md) and [contribution guidelines](CONTRIBUTING.md).

## License

MIT License - see [LICENSE](LICENSE) for details.

---

Built with ❤️ for the AI coding community
