# CodeRAG Product Context

## Why CodeRAG Exists
AI coding assistants like Claude often struggle with accessing current, accurate documentation during autonomous coding sessions. While they have general knowledge, they lack real-time access to:
- Latest API documentation
- Framework-specific patterns
- Library updates and breaking changes
- Project-specific documentation

CodeRAG solves this by providing semantic search over curated documentation, enabling AI assistants to find relevant information quickly and accurately.

## Problems It Solves
1. **Outdated Knowledge**: AI models have training cutoffs; CodeRAG provides current documentation
2. **Context Limitations**: Instead of loading entire docs, CodeRAG finds relevant sections
3. **Search Quality**: Semantic search understands programming concepts better than keyword matching
4. **Deployment Complexity**: Single binary with no external dependencies (unlike Ollama-based solutions)

## How It Should Work

### For AI Assistants
1. AI queries CodeRAG through MCP tools during coding
2. Receives relevant documentation snippets with context
3. Uses information to write accurate, current code
4. Can verify API usage and best practices in real-time

### For Development Teams
1. Install single binary
2. Configure Claude Desktop with MCP settings
3. Optionally crawl documentation sources
4. AI assistant automatically uses CodeRAG during coding

## User Experience Goals
- **Zero Configuration**: Works out of the box with sensible defaults
- **Fast Response**: Sub-second query responses for fluid AI interactions
- **High Relevance**: Top results directly answer the query
- **Transparent Operation**: Clear feedback about what's indexed and searchable

## Target Users
1. **Primary**: AI coding assistants (Claude, GitHub Copilot, etc.)
2. **Secondary**: Development teams using AI assistants
3. **Future**: Direct developer access via CLI/Web UI

## Key Differentiators
- **Purpose-Built for AI**: Optimized for how AI assistants query documentation
- **Single Binary**: No Docker, no Ollama, no complex setup
- **Programming-Focused**: Embeddings and search tuned for code documentation
- **MCP Native**: First-class integration with Claude Desktop
