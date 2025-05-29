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
- 🏗️ **Multi-Architecture**: Pre-built binaries for Linux, macOS, and Windows

## Installation

### Download Pre-Built Binaries (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/ktcadmium/coderag/releases/latest):

| Platform | Architecture | Download |
|----------|-------------|----------|
| macOS | Apple Silicon (M1/M2/M3) | `coderag-mcp-macos-arm64.tar.gz` |
| macOS | Intel | `coderag-mcp-macos-amd64.tar.gz` |
| macOS | Universal (both) | `coderag-mcp-macos-universal.tar.gz` |
| Linux | x86_64 | `coderag-mcp-linux-amd64.tar.gz` |
| Linux | ARM64 | `coderag-mcp-linux-arm64.tar.gz` |
| Windows | x86_64 | `coderag-mcp-windows-amd64.zip` |
| Windows | ARM64 | `coderag-mcp-windows-arm64.zip` |

Extract and make executable (macOS/Linux):
```bash
tar xzf coderag-mcp-*.tar.gz
chmod +x coderag-mcp-*
# Move to a directory in your PATH, e.g.:
sudo mv coderag-mcp-* /usr/local/bin/coderag-mcp
```

### Build from Source

Requirements:
- Rust 1.70 or later
- Internet connection (for initial model download)

```bash
git clone --recursive https://github.com/ktcadmium/coderag.git
cd coderag
cargo build --release --bin coderag-mcp
```

The binary will be at `target/release/coderag-mcp`.

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
      "command": "/usr/local/bin/coderag-mcp",
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

## Per-Project Documentation

CodeRAG automatically maintains separate documentation databases for each project:

- **Automatic Detection**: Recognizes projects by `.git`, `package.json`, `Cargo.toml`, etc.
- **Local Storage**: Creates `.coderag/vectordb.json` in your project root
- **Git Integration**: Automatically adds `.coderag/` to `.gitignore`
- **Global Fallback**: Uses `~/.coderag/` when not in a project

This means:
- Each project searches only its relevant documentation
- No manual database switching needed
- Documentation stays with the project (but not in git)
- Fast, focused search results

## Available MCP Tools

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
See what documentation is currently indexed:
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

### `manage_docs`
Manage your documentation database:
```json
{
  "operation": "delete|expire|refresh",
  "target": "url or source pattern",
  "max_age_days": 30,
  "dry_run": true
}
```

**Operations:**
- `delete`: Remove specific documentation
- `expire`: Remove documents older than specified days
- `refresh`: Re-crawl and update existing documentation

### `reload_docs`
Refresh the document database from disk:
```json
{}
```

## Performance

- **Search Speed**: <10ms for typical document collections
- **Embedding Generation**: 2-5ms per query (after model loading)
- **Model Loading**: ~4ms warm-up time (after initial download)
- **Startup Time**: Instant (model loads on first search)
- **Memory Usage**: ~200MB base + document storage

## Development

Use the included Taskfile for common operations:

```bash
# Install Task runner (if not already installed)
brew install go-task/tap/go-task  # macOS
# or: go install github.com/go-task/task/v3/cmd/task@latest

# Quick development check
task

# Build release binary
task release

# Build for all platforms
task release-all

# See all available tasks
task --list
```

## Troubleshooting

### Model Download Issues
If the model download fails:
1. Check your internet connection
2. Ensure `HF_HUB_USER_AGENT_ORIGIN` is set in your config
3. Try running directly: `HF_HUB_USER_AGENT_ORIGIN=CodeRAG/0.1.0 coderag-mcp --debug`

### Debug Mode
Run with debug logging to see detailed operation:
```bash
coderag-mcp --debug
```

### ONNX Schema Warnings
You may see ONNX schema warnings during model loading - these are harmless and don't affect functionality.

## Architecture

### Embedding Strategy
- **Model**: all-MiniLM-L6-v2 (384-dimensional vectors)
- **Provider**: FastEmbed with ONNX Runtime
- **Initialization**: Lazy loading on first search request

### Storage
- **Format**: JSON-based vector database
- **Per-Project**: `.coderag/vectordb.json` in project directories
- **Global Fallback**: `~/.coderag/coderag_vectordb.json`
- **Persistence**: Atomic writes with temp file + rename

### MCP Integration
- **Protocol**: JSON-RPC over stdio
- **Transport**: Standard MCP stdio transport
- **Error Handling**: Proper MCP error codes and messages

## Contributing

CodeRAG is open source! Check out our [developer documentation](CLAUDE.md) and memory bank in `memory-bank/` for technical details.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

Built with ❤️ for the AI coding community
