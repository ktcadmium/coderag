#!/bin/bash
# Test MCP server tools listing

(echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{"tools":{}}},"id":1}'
sleep 0.1
echo '{"jsonrpc":"2.0","method":"initialized","params":{}}'
sleep 0.1
echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}') | ./target/debug/coderag-mcp 2>/dev/null
