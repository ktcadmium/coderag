#!/usr/bin/env python3
"""Test exact byte output formats"""

import subprocess
import sys
import time

def test_line_endings():
    """Test different line ending scenarios"""
    tests = [
        # Test 1: Standard single newline (what we currently do)
        (b'{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}\n', "Single LF"),

        # Test 2: CRLF line ending (Windows style)
        (b'{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}\r\n', "CRLF"),

        # Test 3: No trailing newline
        (b'{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}', "No newline"),

        # Test 4: Double newline
        (b'{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}\n\n', "Double LF"),
    ]

    for test_input, description in tests:
        print(f"\n=== Testing: {description} ===")
        print(f"Sending {len(test_input)} bytes: {repr(test_input[:50])}...")

        proc = subprocess.Popen(
            ['/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp'],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL
        )

        try:
            # Send test input
            proc.stdin.write(test_input)
            proc.stdin.flush()
            proc.stdin.close()

            # Read response with timeout
            proc.wait(timeout=2)
            output = proc.stdout.read()

            if output:
                print(f"Got response: {len(output)} bytes")
                lines = output.split(b'\n')
                for i, line in enumerate(lines):
                    if line:
                        print(f"  Line {i}: {repr(line[:80])}...")
            else:
                print("No response received")

        except subprocess.TimeoutExpired:
            print("Server hung (no response within 2 seconds)")
            proc.kill()
        except Exception as e:
            print(f"Error: {e}")
            proc.kill()

def test_write_timing():
    """Test if timing of writes matters"""
    print("\n=== Testing write timing ===")

    proc = subprocess.Popen(
        ['/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        bufsize=0  # Unbuffered
    )

    # Send initialize
    req1 = b'{"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"protocolVersion":"2024-11-05"},"id":1}\n'
    proc.stdin.write(req1)
    proc.stdin.flush()

    # Read response immediately
    time.sleep(0.1)

    # Check if data is available
    import select
    if select.select([proc.stdout], [], [], 0)[0]:
        # Read whatever is available
        import os
        os.set_blocking(proc.stdout.fileno(), False)
        try:
            data = proc.stdout.read()
            print(f"Immediate response available: {len(data)} bytes")
            print(f"Response: {repr(data[:100])}...")
        except:
            print("No immediate response")
    else:
        print("No data available immediately")

    proc.kill()

if __name__ == "__main__":
    test_line_endings()
    test_write_timing()
