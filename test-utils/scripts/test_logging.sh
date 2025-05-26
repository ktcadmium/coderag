#!/bin/bash

echo "Testing MCP server logging..."
echo

# Create a simple MCP request
REQUEST='{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}'

# Run the server and send the request
echo "$REQUEST" | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp --debug 2>stderr.log

echo
echo "=== STDERR OUTPUT ==="
cat stderr.log

echo
echo "=== Checking for any stdout logging ==="
echo "$REQUEST" | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp --debug 2>/dev/null | grep -v "jsonrpc" || echo "No unexpected stdout output found"

# Clean up
rm -f stderr.log
