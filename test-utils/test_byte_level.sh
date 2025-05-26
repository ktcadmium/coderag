#!/bin/bash

echo "=== Testing byte-level communication ==="

# Test with proper messages
(
echo -n '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}'
echo ""  # newline
sleep 0.1
echo -n '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}'
echo ""  # newline
) | python3 /Users/ken/dev/MCP/mcp-coderag/test-utils/stdio-logger.py /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp 2>&1 | grep -v "Schema error:"
