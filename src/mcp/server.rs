use crate::mcp::protocol::*;
use crate::mcp::tools::McpTools;
use anyhow::Result;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use tokio::sync::Mutex;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub struct McpServer {
    tools: Arc<Mutex<McpTools>>,
}

impl McpServer {
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let tools = McpTools::new(data_dir).await?;
        Ok(Self {
            tools: Arc::new(Mutex::new(tools)),
        })
    }
    
    pub async fn run_stdio(&self) -> Result<()> {
        info!("🚀 Starting MCP server on stdio");
        
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = BufReader::new(stdin);
        
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            
            debug!("Received: {}", line);
            
            match serde_json::from_str::<McpRequest>(&line) {
                Ok(request) => {
                    let response = self.handle_request(request).await;
                    let response_str = serde_json::to_string(&response)?;
                    
                    debug!("Sending: {}", response_str);
                    
                    writeln!(&stdout, "{}", response_str)?;
                    stdout.flush()?;
                }
                Err(e) => {
                    error!("Failed to parse request: {}", e);
                    let error_response = McpResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(McpError::Parse(e.to_string()).to_error_response()),
                        id: None,
                    };
                    
                    let response_str = serde_json::to_string(&error_response)?;
                    writeln!(&stdout, "{}", response_str)?;
                    stdout.flush()?;
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "initialized" => self.handle_initialized(request).await,
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_tool_call(request).await,
            _ => {
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(McpError::MethodNotFound(request.method).to_error_response()),
                    id: request.id,
                }
            }
        }
    }
    
    async fn handle_initialize(&self, request: McpRequest) -> McpResponse {
        info!("🤝 Handling initialize request");
        
        let result = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "coderag",
                "version": "0.1.0"
            }
        });
        
        McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        }
    }
    
    async fn handle_initialized(&self, request: McpRequest) -> McpResponse {
        info!("✅ Server initialized");
        
        McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(Value::Null),
            error: None,
            id: request.id,
        }
    }
    
    async fn handle_list_tools(&self, request: McpRequest) -> McpResponse {
        info!("🔧 Listing available tools");
        
        let tools = McpTools::list_available_tools();
        let result = json!({
            "tools": tools
        });
        
        McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        }
    }
    
    async fn handle_tool_call(&self, request: McpRequest) -> McpResponse {
        let params = match request.params {
            Some(Value::Object(map)) => map,
            _ => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(McpError::InvalidParams("Expected object params".to_string()).to_error_response()),
                    id: request.id,
                };
            }
        };
        
        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(McpError::InvalidParams("Missing tool name".to_string()).to_error_response()),
                    id: request.id,
                };
            }
        };
        
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
        
        info!("🔨 Calling tool: {}", tool_name);
        
        let mut tools = self.tools.lock().await;
        
        let result = match tool_name {
            "search_docs" => {
                match serde_json::from_value::<SearchDocsParams>(arguments) {
                    Ok(params) => {
                        match tools.search_docs(params).await {
                            Ok(response) => match serde_json::to_value(response) {
                                Ok(value) => Ok(value),
                                Err(e) => Err(McpError::Internal(e.to_string())),
                            },
                            Err(e) => Err(McpError::Internal(e.to_string())),
                        }
                    }
                    Err(e) => Err(McpError::InvalidParams(e.to_string())),
                }
            }
            "list_docs" => {
                match tools.list_docs().await {
                    Ok(response) => match serde_json::to_value(response) {
                        Ok(value) => Ok(value),
                        Err(e) => Err(McpError::Internal(e.to_string())),
                    },
                    Err(e) => Err(McpError::Internal(e.to_string())),
                }
            }
            "crawl_docs" => {
                match serde_json::from_value::<CrawlDocsParams>(arguments) {
                    Ok(params) => {
                        match tools.crawl_docs(params).await {
                            Ok(response) => match serde_json::to_value(response) {
                                Ok(value) => Ok(value),
                                Err(e) => Err(McpError::Internal(e.to_string())),
                            },
                            Err(e) => Err(McpError::Internal(e.to_string())),
                        }
                    }
                    Err(e) => Err(McpError::InvalidParams(e.to_string())),
                }
            }
            "reload_docs" => {
                match tools.reload_docs().await {
                    Ok(response) => match serde_json::to_value(response) {
                        Ok(value) => Ok(value),
                        Err(e) => Err(McpError::Internal(e.to_string())),
                    },
                    Err(e) => Err(McpError::Internal(e.to_string())),
                }
            }
            _ => Err(McpError::MethodNotFound(format!("Unknown tool: {}", tool_name))),
        };
        
        // Save database after modifications
        if matches!(tool_name, "search_docs" | "crawl_docs") {
            if let Err(e) = tools.save_database().await {
                warn!("Failed to save database: {}", e);
            }
        }
        
        match result {
            Ok(value) => McpResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(json!({
                    "content": [{
                        "type": "text",
                        "text": serde_json::to_string_pretty(&value).unwrap_or_else(|_| "Error serializing response".to_string())
                    }]
                })),
                error: None,
                id: request.id,
            },
            Err(e) => McpResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(e.to_error_response()),
                id: request.id,
            },
        }
    }
}