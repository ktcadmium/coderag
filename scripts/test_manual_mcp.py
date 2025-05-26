#!/usr/bin/env python3
"""Manual test for CodeRAG MCP server to debug issues"""

import json
import subprocess
import sys

def send_message(proc, msg):
    """Send a JSON-RPC message to the server"""
    msg_str = json.dumps(msg)
    print(f"Sending: {msg_str}", file=sys.stderr)
    proc.stdin.write(msg_str + '\n')
    proc.stdin.flush()

    # Read response
    response = proc.stdout.readline()
    if response:
        print(f"Received: {response}", file=sys.stderr)
        return json.loads(response)
    return None

def main():
    # Start the server
    proc = subprocess.Popen(
        ['/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp', '--debug'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=0
    )

    try:
        # Send initialize request
        init_msg = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }

        response = send_message(proc, init_msg)
        if response:
            print(f"Initialize response: {json.dumps(response, indent=2)}", file=sys.stderr)

        # Try list_docs
        list_msg = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "list_docs",
                "arguments": {}
            }
        }

        response = send_message(proc, list_msg)
        if response:
            print(f"List docs response: {json.dumps(response, indent=2)}", file=sys.stderr)

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
    finally:
        # Check stderr for any errors
        stderr = proc.stderr.read()
        if stderr:
            print(f"Server stderr:\n{stderr}", file=sys.stderr)

        proc.terminate()
        proc.wait()

if __name__ == "__main__":
    main()
