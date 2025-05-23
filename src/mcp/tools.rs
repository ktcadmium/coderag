use crate::EmbeddingService;
use crate::vectordb::{VectorDatabase, SearchOptions, ContentType};
use crate::mcp::protocol::*;
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn, error};

pub struct McpTools {
    embedding_service: EmbeddingService,
    vector_db: VectorDatabase,
    data_dir: PathBuf,
}

impl McpTools {
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let embedding_service = EmbeddingService::new().await?;
        let mut vector_db = VectorDatabase::new(&data_dir)?;
        
        // Try to load existing data
        let _ = vector_db.load();
        
        Ok(Self {
            embedding_service,
            vector_db,
            data_dir,
        })
    }
    
    pub fn list_available_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "search_docs".to_string(),
                description: "Search documentation using semantic search".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results to return",
                            "default": 5
                        },
                        "source_filter": {
                            "type": "string",
                            "description": "Optional filter by documentation source URL"
                        },
                        "content_type": {
                            "type": "string",
                            "description": "Optional filter by content type"
                        }
                    },
                    "required": ["query"]
                }),
            },
            Tool {
                name: "list_docs".to_string(),
                description: "List all indexed documentation sources".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "crawl_docs".to_string(),
                description: "Crawl and index documentation from a URL".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to crawl"
                        },
                        "recursive": {
                            "type": "boolean",
                            "description": "Whether to crawl recursively",
                            "default": true
                        },
                        "max_pages": {
                            "type": "number",
                            "description": "Maximum number of pages to crawl",
                            "default": 100
                        }
                    },
                    "required": ["url"]
                }),
            },
            Tool {
                name: "reload_docs".to_string(),
                description: "Reload the document database from disk".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }
    
    pub async fn search_docs(&mut self, params: SearchDocsParams) -> Result<SearchDocsResponse> {
        info!("🔍 Searching for: {}", params.query);
        
        // Generate embedding for the query
        let query_embedding = self.embedding_service.embed(&params.query).await?;
        
        // Search the vector database
        let search_options = SearchOptions {
            limit: params.limit,
            min_score: None,
            source_filter: params.source_filter,
            content_type_filter: params.content_type.as_ref().and_then(|ct| {
                match ct.as_str() {
                    "documentation" => Some(crate::vectordb::ContentType::Documentation),
                    "code" => Some(crate::vectordb::ContentType::CodeExample),
                    "tutorial" => Some(crate::vectordb::ContentType::Tutorial),
                    "reference" => Some(crate::vectordb::ContentType::Reference),
                    _ => None,
                }
            }),
        };
        let results = self.vector_db.search(&query_embedding, search_options)?;
        
        // Convert to response format
        let search_results: Vec<SearchResult> = results
            .into_iter()
            .map(|result| {
                let doc = &result.document;
                let mut metadata = HashMap::new();
                metadata.insert("content_type".to_string(), format!("{:?}", doc.metadata.content_type));
                if let Some(lang) = &doc.metadata.language {
                    metadata.insert("language".to_string(), lang.clone());
                }
                for tag in &doc.metadata.tags {
                    metadata.insert(format!("tag_{}", tag), "true".to_string());
                }
                
                SearchResult {
                    title: doc.title.clone().unwrap_or_else(|| "Untitled".to_string()),
                    content: doc.content.clone(),
                    url: doc.url.clone(),
                    score: result.score,
                    metadata: if metadata.is_empty() { None } else { Some(metadata) },
                }
            })
            .collect();
        
        Ok(SearchDocsResponse {
            results: search_results,
            query: params.query,
            total_results: self.vector_db.document_count(),
        })
    }
    
    pub async fn list_docs(&self) -> Result<ListDocsResponse> {
        info!("📚 Listing all documentation sources");
        
        // Group documents by source URL
        let mut source_map: HashMap<String, usize> = HashMap::new();
        
        // Access the storage through VectorDatabase
        // For now, we'll just count total documents since we don't have direct access to entries
        let total_docs = self.vector_db.document_count();
        
        // TODO: Add a method to VectorDatabase to expose document listing
        // For now, return a placeholder response
        if total_docs > 0 {
            source_map.insert("local".to_string(), total_docs);
        }
        
        let sources: Vec<DocSource> = source_map
            .into_iter()
            .map(|(url, count)| DocSource {
                url,
                document_count: count,
                last_crawled: None, // TODO: Track crawl timestamps
            })
            .collect();
        
        Ok(ListDocsResponse {
            sources,
            total_documents: self.vector_db.document_count(),
            last_updated: None, // TODO: Track last update time
        })
    }
    
    pub async fn crawl_docs(&mut self, params: CrawlDocsParams) -> Result<CrawlDocsResponse> {
        warn!("🕷️ Crawl functionality not yet implemented");
        
        // TODO: Implement web crawler in Phase 4
        Ok(CrawlDocsResponse {
            status: "pending".to_string(),
            message: format!("Crawling functionality will be implemented in Phase 4. URL: {}", params.url),
        })
    }
    
    pub async fn reload_docs(&mut self) -> Result<ReloadDocsResponse> {
        info!("🔄 Reloading document database");
        
        match self.vector_db.load() {
            Ok(_) => {
                let count = self.vector_db.document_count();
                Ok(ReloadDocsResponse {
                    status: "success".to_string(),
                    documents_loaded: count,
                    message: format!("Successfully loaded {} documents", count),
                })
            }
            Err(e) => {
                error!("Failed to reload database: {}", e);
                Ok(ReloadDocsResponse {
                    status: "error".to_string(),
                    documents_loaded: 0,
                    message: format!("Failed to reload: {}", e),
                })
            }
        }
    }
    
    pub async fn save_database(&self) -> Result<()> {
        self.vector_db.save()?;
        Ok(())
    }
}