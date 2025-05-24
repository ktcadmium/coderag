use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid params: {0}")]
    InvalidParams(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpErrorResponse>,
    pub id: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct McpErrorResponse {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpError {
    pub fn to_error_response(&self) -> McpErrorResponse {
        match self {
            McpError::Parse(msg) => McpErrorResponse {
                code: -32700,
                message: msg.clone(),
                data: None,
            },
            McpError::InvalidRequest(msg) => McpErrorResponse {
                code: -32600,
                message: msg.clone(),
                data: None,
            },
            McpError::MethodNotFound(msg) => McpErrorResponse {
                code: -32601,
                message: msg.clone(),
                data: None,
            },
            McpError::InvalidParams(msg) => McpErrorResponse {
                code: -32602,
                message: msg.clone(),
                data: None,
            },
            McpError::Internal(msg) => McpErrorResponse {
                code: -32603,
                message: msg.clone(),
                data: None,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListToolsResponse {
    pub tools: Vec<Tool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchDocsParams {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

fn default_limit() -> usize {
    5
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub content: String,
    pub url: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchDocsResponse {
    pub results: Vec<SearchResult>,
    pub query: String,
    pub total_results: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListDocsResponse {
    pub sources: Vec<DocSource>,
    pub total_documents: usize,
    pub last_updated: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DocSource {
    pub url: String,
    pub document_count: usize,
    pub last_crawled: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CrawlDocsParams {
    pub url: String,
    #[serde(default = "default_crawl_mode")]
    pub mode: String, // "single", "section", "full"
    #[serde(default = "default_focus")]
    pub focus: String, // "api", "examples", "changelog", "quickstart", "all"
    #[serde(default = "default_max_pages")]
    pub max_pages: usize,
}

fn default_crawl_mode() -> String {
    "single".to_string()
}

fn default_focus() -> String {
    "all".to_string()
}

fn default_max_pages() -> usize {
    100
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CrawlDocsResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReloadDocsResponse {
    pub status: String,
    pub documents_loaded: usize,
    pub message: String,
}
