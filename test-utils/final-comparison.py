#!/usr/bin/env python3
"""Final comparison test using proper line buffering"""

import subprocess
import json
import time
import os

def test_server_properly(name, command):
    """Test server with proper line buffering"""
    print(f"\n{'='*60}")
    print(f"Testing: {name}")
    print(f"Command: {' '.join(command)}")
    print(f"{'='*60}")

    # Start server
    proc = subprocess.Popen(
        command,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        bufsize=1,  # Line buffered
        universal_newlines=True,  # Text mode with proper line handling
        env=os.environ.copy()
    )

    # Give it time to start
    time.sleep(1)

    # Test sequence
    test_sequence = [
        {
            "request": {"jsonrpc": "2.0", "method": "initialize", "params": {"capabilities": {}, "protocolVersion": "2024-11-05"}, "id": 1},
            "expect_response": True
        },
        {
            "request": {"jsonrpc": "2.0", "method": "initialized", "params": {}},  # No ID - notification
            "expect_response": False
        },
        {
            "request": {"jsonrpc": "2.0", "method": "tools/list", "params": {}, "id": 2},
            "expect_response": True
        }
    ]

    all_responses = []

    for i, test in enumerate(test_sequence):
        request = test["request"]
        expect_response = test["expect_response"]

        print(f"\n--- Request {i+1}: {request.get('method')} ---")

        # Send request
        req_str = json.dumps(request)
        print(f"Sending: {req_str}")

        try:
            proc.stdin.write(req_str + '\n')
            proc.stdin.flush()
        except BrokenPipeError:
            print("ERROR: Broken pipe - server died")
            break

        if expect_response:
            # Try to read response
            try:
                # Set a short timeout using alarm (Unix only)
                import signal

                def timeout_handler(signum, frame):
                    raise TimeoutError("Response timeout")

                signal.signal(signal.SIGALRM, timeout_handler)
                signal.alarm(3)  # 3 second timeout

                response_line = proc.stdout.readline()
                signal.alarm(0)  # Cancel alarm

                if response_line:
                    print(f"Response: {response_line.strip()}")
                    try:
                        parsed = json.loads(response_line)
                        all_responses.append(parsed)
                        print(f"Parsed successfully: {list(parsed.keys())}")
                    except json.JSONDecodeError as e:
                        print(f"JSON parse error: {e}")
                else:
                    print("No response received (empty line)")

            except TimeoutError:
                print("Response timeout after 3 seconds")
            except Exception as e:
                print(f"Error reading response: {e}")
        else:
            print("(No response expected for notification)")
            time.sleep(0.1)  # Give server time to process

    # Check if server is still alive
    if proc.poll() is None:
        print(f"\n✅ Server still running")
        proc.terminate()
        proc.wait(timeout=2)
    else:
        print(f"\n❌ Server exited with code: {proc.poll()}")

    return all_responses

# Test our server
coderag_responses = test_server_properly(
    "CodeRAG MCP Server",
    ["/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp"]
)

# For comparison, you could test another server here
# other_responses = test_server_properly("Other", ["other_command"])

print("\n" + "="*60)
print("SUMMARY")
print("="*60)
print(f"CodeRAG responses received: {len(coderag_responses)}")
for i, resp in enumerate(coderag_responses):
    print(f"  Response {i+1}: id={resp.get('id')}, has_result={('result' in resp)}, has_error={('error' in resp)}")
