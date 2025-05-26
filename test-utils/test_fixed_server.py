#!/usr/bin/env python3
"""Test the fixed server with minimal complexity"""

import subprocess
import json
import sys

# Start the server
proc = subprocess.Popen(
    ['/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp'],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    text=True,
    bufsize=1  # Line buffered
)

# Send initialize
request = {"jsonrpc": "2.0", "method": "initialize", "params": {"capabilities": {}, "protocolVersion": "2024-11-05"}, "id": 1}
print(f"Sending: {request['method']}")
proc.stdin.write(json.dumps(request) + '\n')
proc.stdin.flush()

# Read response
print("Reading response...")
response_line = proc.stdout.readline()

if response_line:
    print(f"✅ Got response: {len(response_line)} chars")
    try:
        parsed = json.loads(response_line)
        print(f"   ID: {parsed.get('id')}")
        print(f"   Has result: {'result' in parsed}")
        print(f"   Server info: {parsed.get('result', {}).get('serverInfo', {})}")
    except Exception as e:
        print(f"❌ Parse error: {e}")
else:
    print("❌ No response received")

# Test another request
request2 = {"jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 2}
print(f"\nSending: {request2['method']}")
proc.stdin.write(json.dumps(request2) + '\n')
proc.stdin.flush()

response2 = proc.stdout.readline()
if response2:
    print(f"✅ Got response: {len(response2)} chars")
else:
    print("❌ No response received")

proc.terminate()
