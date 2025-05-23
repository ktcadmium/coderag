# CodeRAG Project Brief

## Project Overview
CodeRAG is a documentation RAG (Retrieval-Augmented Generation) system designed specifically to give AI assistants (especially Claude) reliable, current documentation access while coding. This is a complete Rust rewrite of the original Go-based Omnidoc MCP server, optimized for autonomous AI development workflows.

## Core Objectives
1. **Eliminate External Dependencies**: Single binary deployment with no Ollama requirement
2. **Clean Team Adoption**: Simple setup for development teams with minimal AI/ML experience
3. **Autonomous AI Coding Support**: Reliable access to current documentation for AI assistants during programming
4. **Performance**: Fast embedding generation and vector search for real-time coding assistance
5. **Quality Documentation Retrieval**: Semantic search that understands programming concepts and technical relationships

## Key Requirements
- **MCP Server**: Stdio-based server compatible with Claude Desktop and other MCP clients
- **Embedded Vector Database**: Pure Rust implementation with efficient similarity search
- **FastEmbed Integration**: ONNX Runtime-based embeddings (all-MiniLM-L6-v2, 384 dimensions)
- **Web Interface**: Future - Management UI for documentation curation and monitoring
- **Web Crawler**: Future - Recursive crawling with content extraction and rate limiting
- **Semantic Search**: Context-aware search optimized for programming documentation

## Success Criteria
- Sub-5ms embedding generation (✅ Achieved: 2-5ms)
- Sub-10ms vector search
- Single binary deployment
- No AI/ML expertise required for operation
- Seamless Claude Desktop integration via MCP

## Project Constraints
- Must be written in Rust for performance and single-binary deployment
- Must use MCP (Model Context Protocol) for AI assistant integration
- Should minimize external dependencies
- Must handle documentation at scale (100k+ documents)
