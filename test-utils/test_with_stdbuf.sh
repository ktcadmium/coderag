#!/bin/bash
# Test if using stdbuf to force line buffering helps

echo "Testing CodeRAG server with forced line buffering..."

# Use stdbuf to force line-buffered output
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}' | \
    stdbuf -oL /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp 2>/dev/null | \
    head -1 | \
    python3 -m json.tool

echo -e "\nIf the above shows formatted JSON, then stdout buffering is the issue."
