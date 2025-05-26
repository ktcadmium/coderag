#!/bin/bash
# Test MCP server with full conversation flow

echo "Testing CodeRAG MCP Server..."
echo "=============================="

# Create a test script that sends multiple requests
cat << 'EOF' | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp --debug 2>&1
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}
{"jsonrpc":"2.0","method":"tools/list","id":2}
EOF

echo ""
echo "Exit code: $?"
