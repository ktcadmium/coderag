#!/usr/bin/env python3
"""Examine exact bytes of response"""

import subprocess

# Send a simple request and examine the exact response bytes
proc = subprocess.Popen(
    ['/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp'],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL
)

request = b'{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}\n'
proc.stdin.write(request)
proc.stdin.close()

response = proc.stdout.read()

print(f"Response length: {len(response)} bytes")
print(f"Response ends with: {repr(response[-10:])}")
print(f"Hex of last 10 bytes: {' '.join(f'{b:02x}' for b in response[-10:])}")
print(f"\nChecking for line endings:")
print(f"Contains \\r\\n: {chr(13) + chr(10) in response.decode('utf-8')}")
print(f"Contains \\n\\n: {chr(10) + chr(10) in response.decode('utf-8')}")
newline_byte = b'\n'
print(f"Ends with newline: {response.endswith(newline_byte)}")
print(f"Number of newlines: {response.count(newline_byte)}")

# Also check how writeln! behaves
print("\n=== Rust writeln! behavior ===")
print("writeln! adds a newline at the end")
print("So if response_str doesn't end with newline, writeln! adds one")
print("If response_str already has a newline, writeln! adds another!")
