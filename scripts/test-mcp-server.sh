#!/bin/bash

# Test script for MCP server
# This simulates what Claude Desktop would send

SERVER="${1:-./target/release/coderag-mcp}"

echo "Testing MCP server: $SERVER"
echo "======================================"

# Create a temp file for communication
TEMP_FILE=$(mktemp)

# Function to send a request and read response
send_request() {
    local request="$1"
    local expect_response="${2:-true}"

    echo "➡️  Sending: $request"
    echo "$request" | $SERVER 2>/dev/null | head -n 1 > $TEMP_FILE

    if [ "$expect_response" = "true" ]; then
        if [ -s $TEMP_FILE ]; then
            echo "⬅️  Response: $(cat $TEMP_FILE)"
            echo
        else
            echo "❌ No response received!"
            echo
        fi
    else
        if [ -s $TEMP_FILE ]; then
            echo "❌ Unexpected response for notification: $(cat $TEMP_FILE)"
            echo
        else
            echo "✅ No response (correct for notification)"
            echo
        fi
    fi
}

# Test 1: Initialize
echo "1. Initialize"
send_request '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'

# Test 2: Initialized (notification - no ID)
echo "2. Initialized notification"
send_request '{"jsonrpc":"2.0","method":"initialized","params":{}}' false

# Test 3: List tools
echo "3. List tools"
send_request '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

# Test 4: Call a tool
echo "4. Call list_docs tool"
send_request '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_docs","arguments":{}}}'

# Clean up
rm -f $TEMP_FILE

echo "======================================"
echo "Test complete!"
