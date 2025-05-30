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

| Platform | Architecture | Archive | Raw Binary |
|----------|-------------|---------|------------|
| macOS | Apple Silicon (M1/M2/M3) | `coderag-mcp-macos-arm64.tar.gz` | `coderag-mcp-macos-arm64` |
| macOS | Intel | `coderag-mcp-macos-amd64.tar.gz` | `coderag-mcp-macos-amd64` |
| macOS | Universal | `coderag-mcp-macos-universal.tar.gz` | `coderag-mcp-macos-universal` |
| Linux | x86_64 | `coderag-mcp-linux-amd64.tar.gz` | `coderag-mcp-linux-amd64` |
| Linux | ARM64 | `coderag-mcp-linux-arm64.tar.gz` | `coderag-mcp-linux-arm64` |
| Windows* | x86_64 | Coming soon | Coming soon |
| Windows* | ARM64 | Coming soon | Coming soon |

*Windows builds are temporarily unavailable due to a linking issue with the embedding library. Track progress at [#1](https://github.com/ktcadmium/coderag/issues/1).

**Archives** include README and LICENSE files. **Raw binaries** are just the executable - perfect for automated installs.

#### Using Archive (includes docs):
```bash
tar xzf coderag-mcp-*.tar.gz
chmod +x coderag-mcp-*
sudo mv coderag-mcp-* /usr/local/bin/coderag-mcp
```

#### Using Raw Binary (quick install):
```bash
# Download directly to /usr/local/bin (example for macOS ARM64)
sudo curl -L https://github.com/ktcadmium/coderag/releases/latest/download/coderag-mcp-macos-arm64 \
  -o /usr/local/bin/coderag-mcp
sudo chmod +x /usr/local/bin/coderag-mcp
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
      "args": []
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

## AI Assistant Compatibility

CodeRAG works with multiple AI coding assistants, but the experience varies:

### Cursor IDE ✅ **Full Support**
- **Autonomous Crawling**: AI assistant can directly use `crawl_docs` to index new documentation
- **Seamless Integration**: Just ask "Can you index the React documentation?" and it works
- **Smart Discovery**: AI automatically finds and indexes relevant docs for your questions
- **No Manual Steps**: Everything happens transparently through the MCP interface

### Claude Code ⚠️ **Search Only**
- **Search Works Perfectly**: AI assistant can search all indexed documentation
- **No Autonomous Crawling**: AI cannot directly crawl new documentation sources
- **Manual Indexing Required**: You must run the binary manually to add new docs:

```bash
# Example: Manually index React documentation for Claude Code
./coderag-mcp crawl https://react.dev/reference --mode single --focus all

# Then Claude Code can search the newly indexed docs
```

### Other MCP Clients
- **Compatibility**: Any MCP-compatible client should work
- **Feature Support**: Depends on the client's MCP implementation
- **Testing Needed**: Please report compatibility issues

### Why the Difference?

The difference comes from how each AI assistant handles MCP tool permissions:

- **Cursor IDE**: Allows AI assistants to call any available MCP tool autonomously
- **Claude Code**: Currently restricts certain MCP tools, requiring manual execution for crawling
- **Future**: Claude Code may add full MCP tool support in future updates

### Recommended Workflow

**For Cursor IDE users:**
```
1. Ask AI: "Can you search for React useEffect examples?"
2. AI automatically crawls React docs if not indexed
3. AI returns relevant examples from fresh documentation
```

**For Claude Code users:**
```
1. Check what's indexed: Ask AI to use list_docs
2. If needed docs missing: Run ./coderag-mcp crawl [url] manually
3. Ask AI: "Can you search for React useEffect examples?"
4. AI returns relevant examples from your pre-indexed documentation
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
2. Try running with debug logging: `coderag-mcp --debug`
3. Check for firewall or proxy issues blocking Hugging Face CDN

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
