use coderag::mcp::{McpServer, McpRequest, McpResponse};
use coderag::vectordb::{Document, DocumentMetadata, ContentType};
use coderag::EmbeddingService;
use serde_json::json;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_mcp_server_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Test initialize request
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({})),
        id: Some(json!(1)),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[tokio::test]
async fn test_list_tools() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Test tools/list request
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/list".to_string(),
        params: None,
        id: Some(json!(2)),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
    
    let result = response.result.unwrap();
    let tools = result.get("tools").unwrap().as_array().unwrap();
    assert_eq!(tools.len(), 4); // search_docs, list_docs, crawl_docs, reload_docs
}

#[tokio::test]
async fn test_search_docs_empty() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Test search on empty database
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "search_docs",
            "arguments": {
                "query": "test query",
                "limit": 5
            }
        })),
        id: Some(json!(3)),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[tokio::test]
async fn test_list_docs_empty() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Test list documents on empty database
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_docs",
            "arguments": {}
        })),
        id: Some(json!(4)),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[tokio::test]
async fn test_reload_docs() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Test reload documents
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "reload_docs",
            "arguments": {}
        })),
        id: Some(json!(5)),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[tokio::test]
async fn test_crawl_docs_stub() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Test crawl documents (stub for now)
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "crawl_docs",
            "arguments": {
                "url": "https://example.com",
                "recursive": true,
                "max_pages": 10
            }
        })),
        id: Some(json!(6)),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.error.is_none());
    assert!(response.result.is_some());
}

#[tokio::test]
async fn test_invalid_method() {
    let temp_dir = TempDir::new().unwrap();
    let server = McpServer::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Test invalid method
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        method: "invalid/method".to_string(),
        params: None,
        id: Some(json!(7)),
    };
    
    let response = server.handle_request(request).await;
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32601); // Method not found
}