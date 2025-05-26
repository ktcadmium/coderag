#!/bin/bash

# Test MCP server with empty lines in the protocol
# This tests if our server correctly handles empty lines between messages

echo "Testing MCP server with empty lines..."

# Create a test script that sends messages with empty lines
cat > /tmp/mcp_empty_line_test.txt << 'EOF'
{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}

{"jsonrpc":"2.0","method":"initialized","params":{},"id":2}


{"jsonrpc":"2.0","method":"tools/list","params":{},"id":3}

EOF

echo "=== Test 1: Sending messages with empty lines ==="
cat /tmp/mcp_empty_line_test.txt | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp --debug 2>&1

echo -e "\n=== Test 2: Multiple empty lines between messages ==="
cat > /tmp/mcp_multiple_empty_lines.txt << 'EOF'
{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}



{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}
EOF

cat /tmp/mcp_multiple_empty_lines.txt | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp --debug 2>&1

echo -e "\n=== Test 3: Empty line at start ==="
cat > /tmp/mcp_empty_start.txt << 'EOF'

{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}
EOF

cat /tmp/mcp_empty_start.txt | /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp --debug 2>&1

echo -e "\nDone testing empty line handling"
