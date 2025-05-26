# Rust MCP SDK Design Document

## Overview

A proper Rust SDK for the Model Context Protocol (MCP) would provide a robust, type-safe, and idiomatic way to build MCP servers in Rust. Based on our experience with the CodeRAG server and analysis of the TypeScript/Python SDKs, here's a comprehensive design.

## Core Design Principles

1. **Type Safety**: Leverage Rust's type system for compile-time guarantees
2. **Async First**: Built on Tokio for modern async Rust
3. **Transport Agnostic**: Support stdio, HTTP, WebSocket transports
4. **Zero Copy**: Minimize allocations where possible
5. **Idiomatic Rust**: Follow Rust conventions and patterns

## Proposed Architecture

### 1. Core Types

```rust
// Core protocol types
pub mod protocol {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Request<T = Value> {
        pub jsonrpc: String,
        pub method: String,
        pub params: T,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id: Option<RequestId>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum RequestId {
        Number(i64),
        String(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response<T = Value> {
        pub jsonrpc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result: Option<T>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub error: Option<ErrorObject>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id: Option<RequestId>,
    }
}
```

### 2. Server Builder Pattern

```rust
use mcp_sdk::{Server, StdioTransport, Tool};

#[tokio::main]
async fn main() -> Result<()> {
    let server = Server::builder()
        .name("my-server")
        .version("1.0.0")
        .capabilities(Capabilities {
            tools: ToolsCapability::default(),
            resources: ResourcesCapability::default(),
            prompts: PromptsCapability::default(),
        })
        .tool(search_docs_tool())
        .tool(list_docs_tool())
        .resource_handler(handle_resources)
        .prompt_handler(handle_prompts)
        .build()?;

    // Connect to transport
    let transport = StdioTransport::new();
    server.serve(transport).await?;

    Ok(())
}
```

### 3. Tool Definition

```rust
fn search_docs_tool() -> Tool {
    Tool::builder()
        .name("search_docs")
        .description("Search documentation using semantic search")
        .parameters(json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "limit": {
                    "type": "number",
                    "default": 5
                }
            },
            "required": ["query"]
        }))
        .handler(|params| async move {
            // Tool implementation
            let query = params["query"].as_str().unwrap();
            let limit = params["limit"].as_u64().unwrap_or(5);

            // Perform search...
            Ok(json!({
                "results": []
            }))
        })
        .build()
}
```

### 4. Transport Layer

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn receive(&mut self) -> Result<Request>;
    async fn send(&mut self, response: Response) -> Result<()>;
}

pub struct StdioTransport {
    stdin: BufReader<Stdin>,
    stdout: Stdout,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            stdin: BufReader::new(io::stdin()),
            stdout: io::stdout(),
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn receive(&mut self) -> Result<Request> {
        let mut line = String::new();
        self.stdin.read_line(&mut line).await?;
        Ok(serde_json::from_str(&line)?)
    }

    async fn send(&mut self, response: Response) -> Result<()> {
        // Critical: Get fresh stdout handle for proper pipe flushing
        writeln!(io::stdout(), "{}", serde_json::to_string(&response)?)?;
        io::stdout().flush()?;
        Ok(())
    }
}
```

### 5. Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

impl McpError {
    pub fn to_error_object(&self) -> ErrorObject {
        match self {
            Self::MethodNotFound(_) => ErrorObject {
                code: -32601,
                message: self.to_string(),
                data: None,
            },
            Self::InvalidParams(_) => ErrorObject {
                code: -32602,
                message: self.to_string(),
                data: None,
            },
            Self::Parse(_) => ErrorObject {
                code: -32700,
                message: self.to_string(),
                data: None,
            },
            Self::Internal(_) => ErrorObject {
                code: -32603,
                message: self.to_string(),
                data: None,
            },
        }
    }
}
```

## Key Features to Include

### 1. Request Handlers with Type Safety

```rust
// Strongly typed request handlers
server.handle::<InitializeRequest, InitializeResponse>(
    |req| async move {
        Ok(InitializeResponse {
            protocol_version: PROTOCOL_VERSION,
            capabilities: server_capabilities(),
            server_info: ServerInfo {
                name: "my-server",
                version: "1.0.0",
            },
        })
    }
);
```

### 2. Middleware Support

```rust
// Logging middleware
server.middleware(|req, next| async move {
    let start = Instant::now();
    let method = req.method.clone();

    let result = next(req).await;

    let duration = start.elapsed();
    info!("Request {} took {:?}", method, duration);

    result
});
```

### 3. Resource Streaming

```rust
// Support for streaming large resources
server.resource_handler(|uri| {
    Box::pin(async_stream::stream! {
        let file = File::open(uri).await?;
        let reader = BufReader::new(file);

        while let Some(chunk) = reader.read_chunk().await? {
            yield ResourceChunk {
                data: chunk,
                done: false,
            };
        }

        yield ResourceChunk {
            data: vec![],
            done: true,
        };
    })
});
```

### 4. Testing Utilities

```rust
#[cfg(test)]
mod tests {
    use mcp_sdk::test::{TestClient, TestTransport};

    #[tokio::test]
    async fn test_tool_call() {
        let transport = TestTransport::new();
        let client = TestClient::new(transport);

        let response = client
            .call_tool("search_docs", json!({ "query": "rust" }))
            .await
            .unwrap();

        assert!(response["results"].is_array());
    }
}
```

## Lessons Learned from CodeRAG

1. **Stdout Buffering**: Always get fresh stdout handle when writing to pipes
2. **Notification Handling**: Clearly distinguish requests (with ID) from notifications (no ID)
3. **Error Responses**: Even parse errors need proper JSON-RPC error responses
4. **Async Services**: Many services need async initialization (e.g., loading models)
5. **Graceful Shutdown**: Handle EOF and broken pipes properly

## Implementation Roadmap

### Phase 1: Core Protocol
- [ ] Define all protocol types with serde
- [ ] Implement JSON-RPC request/response handling
- [ ] Create error type hierarchy
- [ ] Add comprehensive tests

### Phase 2: Server Framework
- [ ] Implement Server builder
- [ ] Add handler registration
- [ ] Create middleware system
- [ ] Implement lifecycle management

### Phase 3: Transports
- [ ] StdioTransport (with proper buffering fixes)
- [ ] HttpTransport
- [ ] WebSocketTransport
- [ ] TestTransport for testing

### Phase 4: Utilities
- [ ] Tool builder with validation
- [ ] Resource streaming support
- [ ] Prompt templates
- [ ] Configuration management

### Phase 5: Examples and Documentation
- [ ] Complete API documentation
- [ ] Example servers
- [ ] Migration guide from manual implementation
- [ ] Best practices guide

## Benefits of a Proper SDK

1. **Reduced Boilerplate**: No manual protocol implementation
2. **Type Safety**: Compile-time validation of handlers
3. **Transport Flexibility**: Easy to switch between stdio/HTTP/WebSocket
4. **Testing**: Built-in test utilities
5. **Best Practices**: Enforces proper patterns (like stdout flushing)
6. **Community**: Shared improvements benefit all users

## Open Questions

1. Should we support multiple async runtimes or just Tokio?
2. How much type inference vs explicit types for handlers?
3. Should tools be registered at compile time or runtime?
4. How to handle protocol version negotiation?
5. Built-in tracing/metrics support?

## Getting Started

To build this SDK:

1. Study the official TypeScript and Python SDKs for API patterns
2. Create a new crate: `mcp-sdk-rust`
3. Start with core protocol types
4. Add stdio transport with our buffering fix
5. Build up the server framework
6. Create examples based on CodeRAG patterns
