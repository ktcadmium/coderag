#!/bin/bash
# Wrapper script for MCP server to add logging

LOG_FILE="/tmp/coderag-mcp-$(date +%Y%m%d-%H%M%S).log"

echo "=== MCP Server Wrapper Started ===" >> "$LOG_FILE"
echo "Date: $(date)" >> "$LOG_FILE"
echo "Args: $@" >> "$LOG_FILE"
echo "PWD: $(pwd)" >> "$LOG_FILE"
echo "ENV:" >> "$LOG_FILE"
env | grep -E "(PATH|HOME|USER|SHELL)" >> "$LOG_FILE"
echo "===" >> "$LOG_FILE"

# Run the actual server with logging
exec /Users/ken/dev/MCP/mcp-coderag/target/release/coderag-mcp "$@" 2>&1 | tee -a "$LOG_FILE"
