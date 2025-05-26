#!/bin/bash
# Complete test of MCP server functionality

echo "Testing CodeRAG MCP Server Complete Flow..."
echo "=========================================="

# Test the full flow that Claude Desktop would use
(
  echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}'
  sleep 0.1
  echo '{"jsonrpc":"2.0","method":"initialized","params":{},"id":2}'
  sleep 0.1
  echo '{"jsonrpc":"2.0","method":"tools/list","id":3}'
  sleep 0.1
  # Send EOF to close stdin properly
) | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp 2>/dev/null

echo ""
echo "Exit code: $?"
