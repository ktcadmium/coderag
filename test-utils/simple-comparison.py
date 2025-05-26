#!/usr/bin/env python3
"""Simple comparison between coderag and working server"""

import subprocess
import json
import time
import select

def test_server(name, command):
    """Test a single server"""
    print(f"\n=== Testing {name} ===")
    print(f"Command: {' '.join(command)}")

    # Start server
    proc = subprocess.Popen(
        command,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        bufsize=0,
        text=False
    )

    # Give it time to start
    time.sleep(2)

    # Send initialize
    request = {"jsonrpc": "2.0", "method": "initialize", "params": {"capabilities": {}, "protocolVersion": "2024-11-05"}, "id": 1}
    req_bytes = (json.dumps(request) + '\n').encode()

    print(f"\nSending {len(req_bytes)} bytes: {req_bytes[:50]}...")
    proc.stdin.write(req_bytes)
    proc.stdin.flush()

    # Check for response using select
    print("Waiting for response...")
    start = time.time()

    # Wait up to 5 seconds for data
    readable, _, _ = select.select([proc.stdout], [], [], 5.0)

    if readable:
        # Read available data
        response = proc.stdout.read(4096)  # Read up to 4KB
        elapsed = time.time() - start
        print(f"Got response in {elapsed:.3f}s: {len(response)} bytes")
        print(f"Raw: {response[:100]}...")

        # Try to parse as JSON lines
        for line in response.split(b'\n'):
            if line.strip():
                try:
                    parsed = json.loads(line)
                    print(f"Parsed: {json.dumps(parsed, indent=2)[:200]}...")
                except:
                    print(f"Non-JSON line: {line[:100]}...")
    else:
        print("No response after 5 seconds")

        # Check if process is still alive
        if proc.poll() is not None:
            print(f"Process exited with code: {proc.poll()}")
            stderr = proc.stderr.read()
            if stderr:
                print(f"Stderr: {stderr[:500]}")

    # Cleanup
    proc.terminate()
    try:
        proc.wait(timeout=2)
    except:
        proc.kill()

# Test our server
test_server("coderag", ["/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp"])

# Test npx ethereum server (should be quick to start)
print("\n" + "="*60)
test_server("ethereum", ["npx", "-y", "@modelcontextprotocol/server-ethereum"])
