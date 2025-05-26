#!/bin/bash

# Test continuous interaction without empty lines
echo "=== Test: Continuous interaction (no empty lines) ==="

(
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}'
sleep 0.1
echo '{"jsonrpc":"2.0","method":"initialized","params":{},"id":2}'
sleep 0.1
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":3}'
sleep 0.1
echo '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"list_docs","arguments":{}},"id":4}'
sleep 0.1
) | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp 2>&1 | grep -E "(Received:|Sending:|ERROR|Error|error)"

echo -e "\n=== Test: What happens with a single empty line? ==="

(
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}'
sleep 0.1
echo ''
sleep 0.1
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}'
sleep 0.1
) | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp 2>&1 | grep -E "(Received:|Sending:|shutting down|Client disconnected)"
