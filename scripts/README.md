# Test Scripts

This directory contains various test scripts for debugging and validating the CodeRAG MCP server.

## Scripts Overview

### `test-mcp-server.sh`

**Purpose**: Basic MCP protocol lifecycle test
**Usage**: `./test-mcp-server.sh [path-to-binary]`
**What it tests**:

- Initialize request/response
- Initialized notification
- Tools list
- Tool call (list_docs)

### `test_manual_mcp.py`

**Purpose**: Python-based manual MCP testing with debug output
**Usage**: `python3 test_manual_mcp.py`
**What it tests**:

- Full MCP communication with stderr logging
- Initialize and list_docs tool calls
- Process management and error handling

### `test_server_local.sh`

**Purpose**: Local server testing script
**Usage**: `./test_server_local.sh`
**What it tests**: Basic server startup and response

### `test_sdk_server.sh`

**Purpose**: SDK server testing
**Usage**: `./test_sdk_server.sh`
**What it tests**: SDK-specific functionality

## Running All Tests

```bash
# Make scripts executable
chmod +x scripts/*.sh

# Run basic protocol test
./scripts/test-mcp-server.sh

# Run detailed Python test
python3 scripts/test_manual_mcp.py

# Run other tests
./scripts/test_server_local.sh
./scripts/test_sdk_server.sh
```

## Test Results Pattern

**Important**: All these tests consistently PASS, showing the server works correctly. The issue is specifically with Claude Desktop integration, not with the MCP protocol implementation itself.

## Integration Testing

For comprehensive integration tests, use:

```bash
cargo test --test mcp_integration
```

This runs the full test suite in `tests/mcp_integration.rs`.
