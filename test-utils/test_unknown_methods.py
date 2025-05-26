#!/usr/bin/env python3
"""Test how server handles unknown or additional MCP methods"""

import subprocess
import json
import time

def send_and_receive(proc, request):
    """Send request and get response"""
    req_str = json.dumps(request) + '\n'
    proc.stdin.write(req_str.encode())
    proc.stdin.flush()

    # Give server time to respond
    time.sleep(0.1)

    # Read response line
    line = proc.stdout.readline()
    if line:
        return json.loads(line.decode())
    return None

def test_methods():
    """Test various MCP methods"""
    proc = subprocess.Popen(
        ['/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=False
    )

    # Initialize first
    print("1. Testing initialize...")
    resp = send_and_receive(proc, {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {"capabilities": {}, "protocolVersion": "2024-11-05"},
        "id": 1
    })
    print(f"   Response: {resp is not None}")

    # Send initialized notification
    print("\n2. Testing initialized notification...")
    proc.stdin.write(b'{"jsonrpc":"2.0","method":"initialized","params":{}}\n')
    proc.stdin.flush()
    time.sleep(0.1)

    # Test methods that might be expected
    test_methods = [
        ("tools/list", {}, 2),
        ("resources/list", {}, 3),
        ("prompts/list", {}, 4),
        ("logging/levels", {}, 5),
        ("capabilities", {}, 6),
        ("ping", {}, 7),
        ("health", {}, 8),
    ]

    for method, params, req_id in test_methods:
        print(f"\n{req_id}. Testing {method}...")
        resp = send_and_receive(proc, {
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": req_id
        })

        if resp:
            if "result" in resp:
                print(f"   ✅ Success: {list(resp.get('result', {}).keys())}")
            elif "error" in resp:
                print(f"   ❌ Error: {resp['error'].get('message', 'Unknown error')}")
        else:
            print("   ⚠️  No response")

    proc.terminate()

if __name__ == "__main__":
    test_methods()
