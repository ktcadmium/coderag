#!/bin/bash

# Test the new SDK-based MCP server

echo "Testing initialize request..."
echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}' | ./target/debug/coderag-mcp 2>&1 | grep -A5 -B5 "protocolVersion"

echo -e "\nTesting tools/list request..."
(echo '{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{}},"id":1}'; echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}') | ./target/debug/coderag-mcp 2>&1 | grep -A20 "search_docs"
