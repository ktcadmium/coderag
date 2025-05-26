#!/usr/bin/env python3
"""
Simulate Claude Code's MCP client behavior to debug connection issues.
"""

import subprocess
import json
import sys
import time
import threading
import queue

class MCPClientSimulator:
    def __init__(self, server_cmd):
        self.server_cmd = server_cmd
        self.process = None
        self.response_queue = queue.Queue()
        self.stderr_lines = []

    def start_server(self):
        """Start the MCP server process"""
        print(f"Starting server: {' '.join(self.server_cmd)}", file=sys.stderr)
        self.process = subprocess.Popen(
            self.server_cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            bufsize=0,
            text=True
        )

        # Start threads to read stdout and stderr
        threading.Thread(target=self._read_stdout, daemon=True).start()
        threading.Thread(target=self._read_stderr, daemon=True).start()

    def _read_stdout(self):
        """Read stdout line by line"""
        try:
            for line in self.process.stdout:
                line = line.rstrip('\n')
                if line:
                    print(f"[STDOUT] {line}", file=sys.stderr)
                    try:
                        response = json.loads(line)
                        self.response_queue.put(response)
                    except json.JSONDecodeError as e:
                        print(f"[ERROR] Failed to parse JSON: {e}", file=sys.stderr)
                        print(f"[ERROR] Line was: {repr(line)}", file=sys.stderr)
        except Exception as e:
            print(f"[ERROR] stdout reader crashed: {e}", file=sys.stderr)

    def _read_stderr(self):
        """Read stderr"""
        try:
            for line in self.process.stderr:
                line = line.rstrip('\n')
                if line and "Schema error:" not in line:
                    self.stderr_lines.append(line)
                    print(f"[STDERR] {line}", file=sys.stderr)
        except Exception as e:
            print(f"[ERROR] stderr reader crashed: {e}", file=sys.stderr)

    def send_request(self, method, params=None, request_id=None):
        """Send a JSON-RPC request"""
        request = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params or {}
        }
        if request_id is not None:
            request["id"] = request_id

        request_str = json.dumps(request)
        print(f"\n[SEND] {request_str}", file=sys.stderr)

        # Send with newline
        self.process.stdin.write(request_str + '\n')
        self.process.stdin.flush()

    def wait_for_response(self, timeout=5):
        """Wait for a response"""
        try:
            response = self.response_queue.get(timeout=timeout)
            print(f"[RECV] {json.dumps(response, indent=2)}", file=sys.stderr)
            return response
        except queue.Empty:
            print(f"[ERROR] No response received within {timeout} seconds", file=sys.stderr)
            return None

    def test_initialization(self):
        """Test the initialization sequence"""
        print("\n=== Testing Initialization Sequence ===", file=sys.stderr)

        # Step 1: Send initialize
        self.send_request("initialize", {
            "capabilities": {},
            "protocolVersion": "2024-11-05"
        }, request_id=1)

        response = self.wait_for_response()
        if not response:
            return False

        # Validate response
        if response.get("id") != 1:
            print(f"[ERROR] Expected id=1, got {response.get('id')}", file=sys.stderr)
            return False

        if "result" not in response:
            print("[ERROR] No result in initialize response", file=sys.stderr)
            return False

        # Step 2: Send initialized notification (no ID)
        self.send_request("initialized", {})

        # Give it a moment to process
        time.sleep(0.1)

        # Step 3: List tools
        self.send_request("tools/list", {}, request_id=2)

        response = self.wait_for_response()
        if not response:
            return False

        if response.get("id") != 2:
            print(f"[ERROR] Expected id=2, got {response.get('id')}", file=sys.stderr)
            return False

        return True

    def test_edge_cases(self):
        """Test various edge cases"""
        print("\n=== Testing Edge Cases ===", file=sys.stderr)

        # Test 1: Send request with very large ID
        self.send_request("tools/list", {}, request_id=999999)
        response = self.wait_for_response(timeout=2)
        if response and response.get("id") != 999999:
            print("[ERROR] Large ID not preserved", file=sys.stderr)

        # Test 2: Send notification (no response expected)
        self.send_request("some/notification", {})
        time.sleep(0.5)

        # Test 3: Send another request to ensure server still alive
        self.send_request("tools/list", {}, request_id=3)
        response = self.wait_for_response(timeout=2)
        if not response:
            print("[ERROR] Server stopped responding after notification", file=sys.stderr)
            return False

        return True

    def shutdown(self):
        """Shutdown the server"""
        if self.process:
            self.process.terminate()
            self.process.wait(timeout=5)
            print(f"\n[INFO] Server exited with code: {self.process.returncode}", file=sys.stderr)

def main():
    if len(sys.argv) < 2:
        print("Usage: python test_claude_simulation.py <server_command>", file=sys.stderr)
        sys.exit(1)

    server_cmd = sys.argv[1:]

    simulator = MCPClientSimulator(server_cmd)
    simulator.start_server()

    # Give server time to start
    time.sleep(1)

    # Run tests
    if simulator.test_initialization():
        print("\n✅ Initialization test PASSED", file=sys.stderr)
    else:
        print("\n❌ Initialization test FAILED", file=sys.stderr)

    if simulator.test_edge_cases():
        print("\n✅ Edge case tests PASSED", file=sys.stderr)
    else:
        print("\n❌ Edge case tests FAILED", file=sys.stderr)

    # Shutdown
    simulator.shutdown()

    # Report any concerning stderr messages
    concerning = [line for line in simulator.stderr_lines if "ERROR" in line or "error" in line]
    if concerning:
        print("\n⚠️  Concerning stderr messages:", file=sys.stderr)
        for line in concerning:
            print(f"  - {line}", file=sys.stderr)

if __name__ == "__main__":
    main()
