# CodeRAG Test Utilities

This directory contains test utilities for debugging and testing the CodeRAG MCP server.

## Structure

```
test-utils/
├── src/
│   ├── mcp_test_client.rs    # Programmatic MCP test client
│   └── mcp_protocol_logger.rs # Protocol logger/proxy
├── scripts/                   # Legacy test scripts
└── target/                    # Build output
```

## Building

```bash
cd test-utils
cargo build --release
```

## Tools

### MCP Test Client

A programmatic client for testing MCP servers without needing TTY interaction.

```bash
# Test initialization
./target/release/mcp-test-client --server ../target/release/coderag-mcp init

# Test full lifecycle
./target/release/mcp-test-client --server ../target/release/coderag-mcp lifecycle

# Test with debug output
./target/release/mcp-test-client --server ../target/release/coderag-mcp --debug lifecycle

# List available tools
./target/release/mcp-test-client --server ../target/release/coderag-mcp list-tools
```

### MCP Protocol Logger

Logs all MCP protocol communication between a client and server.

```bash
# Use as a proxy to log protocol exchanges
./target/release/mcp-protocol-logger --server ../target/release/coderag-mcp --log-file mcp.log

# With verbose output to stderr
./target/release/mcp-protocol-logger --server ../target/release/coderag-mcp --verbose
```

## Usage with Claude Code

These utilities allow Claude Code to test MCP servers programmatically without needing interactive TTY access. The test client can verify server functionality and the protocol logger helps debug communication issues.
