#!/usr/bin/env python3
"""
MCP Server Comparison Tester
Compare behavior of multiple MCP servers to find differences
"""

import subprocess
import json
import time
import sys
import threading
import queue
from dataclasses import dataclass
from typing import List, Dict, Any, Optional
import difflib

@dataclass
class ServerConfig:
    name: str
    command: List[str]
    env: Optional[Dict[str, str]] = None

@dataclass
class TestResult:
    server: str
    request: Dict[str, Any]
    response: Optional[Dict[str, Any]]
    response_time: float
    raw_bytes: bytes
    error: Optional[str] = None

class MCPServerTester:
    def __init__(self, servers: List[ServerConfig]):
        self.servers = servers
        self.results: Dict[str, List[TestResult]] = {s.name: [] for s in servers}

    def start_server(self, config: ServerConfig):
        """Start an MCP server and return the process"""
        env = None
        if config.env:
            import os
            env = os.environ.copy()
            env.update(config.env)

        proc = subprocess.Popen(
            config.command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            bufsize=0
        )

        # Give server time to start
        time.sleep(1)

        return proc

    def send_request(self, proc, request: Dict[str, Any]) -> TestResult:
        """Send a request and capture the response"""
        server_name = "unknown"
        start_time = time.time()

        try:
            # Send request
            req_str = json.dumps(request) + '\n'
            proc.stdin.write(req_str.encode())
            proc.stdin.flush()

            # Read response (blocking with reasonable timeout via thread)
            raw_response = b""
            def read_line():
                nonlocal raw_response
                raw_response = proc.stdout.readline()

            thread = threading.Thread(target=read_line)
            thread.daemon = True
            thread.start()
            thread.join(timeout=5.0)

            if thread.is_alive():
                raise TimeoutError("Response timeout")
            response_time = time.time() - start_time

            if raw_response:
                try:
                    response = json.loads(raw_response.decode())
                    return TestResult(
                        server=server_name,
                        request=request,
                        response=response,
                        response_time=response_time,
                        raw_bytes=raw_response
                    )
                except json.JSONDecodeError as e:
                    return TestResult(
                        server=server_name,
                        request=request,
                        response=None,
                        response_time=response_time,
                        raw_bytes=raw_response,
                        error=f"JSON decode error: {e}"
                    )
            else:
                return TestResult(
                    server=server_name,
                    request=request,
                    response=None,
                    response_time=response_time,
                    raw_bytes=b"",
                    error="No response"
                )

        except Exception as e:
            return TestResult(
                server=server_name,
                request=request,
                response=None,
                response_time=time.time() - start_time,
                raw_bytes=b"",
                error=str(e)
            )

    def run_test_sequence(self):
        """Run the same test sequence on all servers"""
        test_requests = [
            {
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {"capabilities": {}, "protocolVersion": "2024-11-05"},
                "id": 1
            },
            {
                "jsonrpc": "2.0",
                "method": "initialized",
                "params": {}
                # Note: No ID - this is a notification
            },
            {
                "jsonrpc": "2.0",
                "method": "tools/list",
                "params": {},
                "id": 2
            }
        ]

        for config in self.servers:
            print(f"\n=== Testing {config.name} ===")
            print(f"Command: {' '.join(config.command)}")

            try:
                proc = self.start_server(config)

                for request in test_requests:
                    print(f"\nSending: {request.get('method')}...")
                    result = self.send_request(proc, request)
                    result.server = config.name
                    self.results[config.name].append(result)

                    if result.error:
                        print(f"  Error: {result.error}")
                    else:
                        print(f"  Response time: {result.response_time:.3f}s")
                        if result.response:
                            print(f"  Response keys: {list(result.response.keys())}")

                # Clean shutdown
                proc.terminate()
                proc.wait(timeout=5)

            except Exception as e:
                print(f"  Server error: {e}")

    def compare_results(self):
        """Compare results across servers"""
        print("\n\n=== COMPARISON RESULTS ===")

        if len(self.servers) < 2:
            print("Need at least 2 servers to compare")
            return

        # Get reference server (first one that's not coderag)
        reference = None
        comparison = None
        for name in self.results:
            if "coderag" not in name.lower():
                reference = name
            else:
                comparison = name

        if not reference or not comparison:
            print("Could not determine reference and comparison servers")
            return

        print(f"\nComparing {comparison} against {reference}")

        # Compare each request
        for i, (ref_result, comp_result) in enumerate(zip(
            self.results[reference],
            self.results[comparison]
        )):
            print(f"\n--- Request {i+1}: {ref_result.request.get('method')} ---")

            # Compare response times
            print(f"Response time: {reference}={ref_result.response_time:.3f}s, "
                  f"{comparison}={comp_result.response_time:.3f}s")

            # Compare raw bytes
            if ref_result.raw_bytes != comp_result.raw_bytes:
                print("\n⚠️  RAW BYTES DIFFER!")
                print(f"{reference} ({len(ref_result.raw_bytes)} bytes): {ref_result.raw_bytes[:50]}...")
                print(f"{comparison} ({len(comp_result.raw_bytes)} bytes): {comp_result.raw_bytes[:50]}...")

                # Show hex diff of first difference
                for j, (b1, b2) in enumerate(zip(ref_result.raw_bytes, comp_result.raw_bytes)):
                    if b1 != b2:
                        print(f"First difference at byte {j}: {reference}=0x{b1:02x}, {comparison}=0x{b2:02x}")
                        break

            # Compare JSON structure
            if ref_result.response and comp_result.response:
                ref_json = json.dumps(ref_result.response, sort_keys=True, indent=2)
                comp_json = json.dumps(comp_result.response, sort_keys=True, indent=2)

                if ref_json != comp_json:
                    print("\n⚠️  JSON STRUCTURE DIFFERS!")
                    diff = difflib.unified_diff(
                        ref_json.splitlines(),
                        comp_json.splitlines(),
                        fromfile=reference,
                        tofile=comparison,
                        lineterm=''
                    )
                    for line in list(diff)[:20]:  # Show first 20 lines of diff
                        print(line)

def main():
    # Define servers to test
    servers = [
        # Our server
        ServerConfig(
            name="coderag",
            command=["/Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp"]
        ),

        # Example of other servers (you'll need to adjust these based on what's available)
        # ServerConfig(
        #     name="sqlite",
        #     command=["uvx", "mcp-server-sqlite", "--db-path", "/tmp/test.db"]
        # ),
    ]

    # Check if we have at least sqlite server available
    try:
        result = subprocess.run(["which", "uvx"], capture_output=True)
        if result.returncode == 0:
            servers.append(ServerConfig(
                name="sqlite",
                command=["uvx", "mcp-server-sqlite", "--db-path", "/tmp/test.db"]
            ))
    except:
        pass

    if len(servers) < 2:
        print("⚠️  WARNING: Only testing coderag server. Add more servers for comparison.")
        print("Consider installing: uvx install mcp-server-sqlite")

    tester = MCPServerTester(servers)
    tester.run_test_sequence()
    tester.compare_results()

if __name__ == "__main__":
    main()
