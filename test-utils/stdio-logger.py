#!/usr/bin/env python3
"""
Stdio logger that captures exact byte-level communication between MCP client and server.
Usage: python stdio-logger.py <server_command>
"""

import sys
import subprocess
import threading
import time
import os
from datetime import datetime

def log_bytes(prefix, data):
    """Log bytes with hex representation"""
    timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{timestamp}] {prefix}: {len(data)} bytes", file=sys.stderr)

    # Show both hex and ASCII representation
    hex_str = ' '.join(f'{b:02x}' for b in data)
    ascii_str = ''.join(chr(b) if 32 <= b < 127 else '.' for b in data)

    print(f"  HEX: {hex_str}", file=sys.stderr)
    print(f"  ASCII: {ascii_str}", file=sys.stderr)
    print(f"  RAW: {repr(data)}", file=sys.stderr)
    print("", file=sys.stderr)

def relay_stdin_to_process(process):
    """Relay stdin to the subprocess, logging all data"""
    try:
        while True:
            # Read raw bytes from stdin
            data = sys.stdin.buffer.read(1024)
            if not data:
                log_bytes("STDIN EOF", b"")
                break

            log_bytes("STDIN", data)
            process.stdin.write(data)
            process.stdin.flush()

    except Exception as e:
        print(f"Error in stdin relay: {e}", file=sys.stderr)
    finally:
        try:
            process.stdin.close()
        except:
            pass

def relay_process_to_stdout(process):
    """Relay subprocess stdout to our stdout, logging all data"""
    try:
        while True:
            # Read raw bytes from process
            data = process.stdout.read(1024)
            if not data:
                log_bytes("PROCESS STDOUT EOF", b"")
                break

            log_bytes("PROCESS STDOUT", data)
            sys.stdout.buffer.write(data)
            sys.stdout.buffer.flush()

    except Exception as e:
        print(f"Error in stdout relay: {e}", file=sys.stderr)

def relay_process_stderr(process):
    """Relay subprocess stderr to our stderr"""
    try:
        for line in process.stderr:
            print(f"[PROCESS STDERR] {line.rstrip()}", file=sys.stderr)
    except Exception as e:
        print(f"Error in stderr relay: {e}", file=sys.stderr)

def main():
    if len(sys.argv) < 2:
        print("Usage: python stdio-logger.py <server_command> [args...]", file=sys.stderr)
        sys.exit(1)

    server_command = sys.argv[1:]
    print(f"Starting server: {' '.join(server_command)}", file=sys.stderr)
    print("=" * 60, file=sys.stderr)

    # Start the subprocess
    process = subprocess.Popen(
        server_command,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        bufsize=0  # Unbuffered
    )

    # Create threads for relaying data
    stdin_thread = threading.Thread(target=relay_stdin_to_process, args=(process,))
    stdout_thread = threading.Thread(target=relay_process_to_stdout, args=(process,))
    stderr_thread = threading.Thread(target=relay_process_stderr, args=(process,))

    # Start all threads
    stdin_thread.start()
    stdout_thread.start()
    stderr_thread.start()

    # Wait for process to complete
    exit_code = process.wait()
    print(f"\nProcess exited with code: {exit_code}", file=sys.stderr)

    # Wait for threads to complete
    stdin_thread.join(timeout=1)
    stdout_thread.join(timeout=1)
    stderr_thread.join(timeout=1)

if __name__ == "__main__":
    main()
