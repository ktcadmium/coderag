#!/bin/bash
# Test server with local cache

export FASTEMBED_CACHE_PATH="$(pwd)/.fastembed_cache"
export HF_HOME="$(pwd)/.hf_cache"

mkdir -p "$FASTEMBED_CACHE_PATH" "$HF_HOME"

echo "Starting server with cache paths:"
echo "  FASTEMBED_CACHE_PATH=$FASTEMBED_CACHE_PATH"
echo "  HF_HOME=$HF_HOME"

# Run server and capture initial output
timeout 30 ./target/release/coderag-mcp --debug 2>&1 | tee server_output.log
