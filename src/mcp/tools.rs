use crate::crawler::{CrawlConfig, CrawlMode, Crawler, DocumentationFocus};
use crate::mcp::protocol::*;
use crate::vectordb::{SearchOptions, VectorDatabase};
use crate::EmbeddingService;
use anyhow::Result;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tracing::{error, info};
use url::Url;

pub struct McpTools {
    embedding_service: EmbeddingService,
    vector_db: VectorDatabase,
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
                        "mode": {
                            "type": "string",
                            "description": "Crawl mode: 'single' (just this page), 'section' (page and children), 'full' (entire site)",
                            "default": "single",
                            "enum": ["single", "section", "full"]
                        },
                        "focus": {
                            "type": "string",
                            "description": "Content focus: 'api', 'examples', 'changelog', 'quickstart', or 'all'",
                            "default": "all",
                            "enum": ["api", "examples", "changelog", "quickstart", "all"]
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
            content_type_filter: params
                .content_type
                .as_ref()
                .and_then(|ct| match ct.as_str() {
                    "documentation" => Some(crate::vectordb::ContentType::Documentation),
                    "code" => Some(crate::vectordb::ContentType::CodeExample),
                    "tutorial" => Some(crate::vectordb::ContentType::Tutorial),
                    "reference" => Some(crate::vectordb::ContentType::Reference),
                    _ => None,
                }),
        };
        let results = self.vector_db.search(&query_embedding, search_options)?;

        // Convert to response format
        let search_results: Vec<SearchResult> = results
            .into_iter()
            .map(|result| {
                let doc = &result.document;
                let mut metadata = HashMap::new();
                metadata.insert(
                    "content_type".to_string(),
                    format!("{:?}", doc.metadata.content_type),
                );
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
                    metadata: if metadata.is_empty() {
                        None
                    } else {
                        Some(metadata)
                    },
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

        // Get documents grouped by source
        let docs_by_source = self.vector_db.get_documents_by_source();

        let sources: Vec<DocSource> = docs_by_source
            .into_iter()
            .map(|(url, docs)| DocSource {
                url,
                document_count: docs.len(),
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
        info!("🕷️ Starting crawl of: {}", params.url);

        // Parse the crawl mode
        let mode = match params.mode.as_str() {
            "single" => CrawlMode::SinglePage,
            "section" => CrawlMode::Section,
            "full" => CrawlMode::FullDocs,
            _ => CrawlMode::SinglePage,
        };

        // Parse the documentation focus
        let focus = match params.focus.as_str() {
            "api" => DocumentationFocus::ApiReference,
            "examples" => DocumentationFocus::Examples,
            "changelog" => DocumentationFocus::Changelog,
            "quickstart" => DocumentationFocus::QuickStart,
            _ => DocumentationFocus::All,
        };

        // Parse URL to get allowed domain
        let parsed_url = match Url::parse(&params.url) {
            Ok(url) => url,
            Err(e) => {
                return Ok(CrawlDocsResponse {
                    status: "error".to_string(),
                    message: format!("Invalid URL: {}", e),
                });
            }
        };

        let mut allowed_domains = HashSet::new();
        if let Some(host) = parsed_url.host_str() {
            allowed_domains.insert(host.to_string());
        }

        // Create crawler configuration
        let config = CrawlConfig {
            start_url: params.url.clone(),
            mode,
            focus,
            max_pages: params.max_pages,
            allowed_domains,
            ..Default::default()
        };

        // Create and run crawler
        let crawler = match Crawler::new(config).await {
            Ok(c) => c,
            Err(e) => {
                return Ok(CrawlDocsResponse {
                    status: "error".to_string(),
                    message: format!("Failed to create crawler: {}", e),
                });
            }
        };

        // Run the crawl
        match crawler
            .crawl(&self.embedding_service, &mut self.vector_db)
            .await
        {
            Ok(crawled_urls) => {
                // Save the updated database
                if let Err(e) = self.vector_db.save() {
                    error!("Failed to save database after crawl: {}", e);
                }

                Ok(CrawlDocsResponse {
                    status: "success".to_string(),
                    message: format!(
                        "Successfully crawled {} pages from {}",
                        crawled_urls.len(),
                        params.url
                    ),
                })
            }
            Err(e) => {
                error!("Crawl failed: {}", e);
                Ok(CrawlDocsResponse {
                    status: "error".to_string(),
                    message: format!("Crawl failed: {}", e),
                })
            }
        }
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
