#!/bin/bash

echo "=== Testing Response Format ==="

# Send initialize request and capture response
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}' | \
    /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp 2>/dev/null | \
    python3 -m json.tool

echo -e "\n=== Expected format based on TypeScript SDK ==="
echo "Should have:"
echo "- jsonrpc: '2.0'"
echo "- result with capabilities, protocolVersion, serverInfo"
echo "- id matching the request"

echo -e "\n=== Testing tools/list response ==="
(
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}'
echo '{"jsonrpc":"2.0","method":"initialized","params":{},"id":2}'
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":3}'
) | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp 2>/dev/null | tail -1 | python3 -m json.tool
