#!/bin/bash
# Test MCP server with initialize request

echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}' | ./target/debug/coderag-mcp --debug
