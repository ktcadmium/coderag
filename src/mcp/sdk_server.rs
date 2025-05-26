use crate::crawler::{CrawlConfig, CrawlMode, DocumentationFocus};
use crate::vectordb::{SearchOptions, VectorDatabase};
use crate::EmbeddingService;
use rmcp::{model::*, tool, Error as McpError, ServerHandler};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use url::Url;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchDocsParams {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub source_filter: Option<String>,
    pub content_type: Option<String>,
}

fn default_limit() -> usize {
    5
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CrawlDocsParams {
    pub url: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_focus")]
    pub focus: String,
    #[serde(default = "default_max_pages")]
    pub max_pages: usize,
}

fn default_mode() -> String {
    "single".to_string()
}

fn default_focus() -> String {
    "all".to_string()
}

fn default_max_pages() -> usize {
    100
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

#[derive(Clone)]
pub struct CodeRagServer {
    embedding_service: Arc<Mutex<EmbeddingService>>,
    vector_db: Arc<Mutex<VectorDatabase>>,
}

#[tool(tool_box)]
impl CodeRagServer {
    pub async fn new(data_dir: PathBuf) -> anyhow::Result<Self> {
        info!("🚀 Initializing CodeRAG server...");
        info!("📂 Data directory: {:?}", data_dir);

        info!("⏳ Creating embedding service (model downloads on first search)...");
        let embedding_service = EmbeddingService::new().await?;

        info!("📊 Initializing vector database...");
        let db_path = data_dir.join("coderag_vectordb.json");
        let mut vector_db = VectorDatabase::new(&db_path)?;

        // Try to load existing data
        match vector_db.load() {
            Ok(_) => info!("✅ Loaded existing vector database"),
            Err(_) => info!("📝 Starting with empty vector database"),
        }

        info!("✅ CodeRAG server initialization complete!");

        Ok(Self {
            embedding_service: Arc::new(Mutex::new(embedding_service)),
            vector_db: Arc::new(Mutex::new(vector_db)),
        })
    }

    #[tool(
        description = "Search your indexed documentation using semantic search. Use this tool when you need current, accurate information about programming frameworks, libraries, APIs, or coding concepts to help with development tasks. This is YOUR resource - use it proactively when you encounter unfamiliar technologies or need to verify current best practices."
    )]
    async fn search_docs(
        &self,
        #[tool(aggr)] params: SearchDocsParams,
    ) -> Result<CallToolResult, McpError> {
        let embedding_service = self.embedding_service.lock().await;
        let vector_db = self.vector_db.lock().await;

        let SearchDocsParams {
            query,
            limit,
            source_filter,
            content_type,
        } = params;

        // Generate embedding for query
        let query_embedding = embedding_service
            .embed(&query)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Prepare search options
        let options = SearchOptions {
            limit,
            min_score: None,
            source_filter,
            content_type_filter: content_type.and_then(|ct| match ct.as_str() {
                "documentation" => Some(crate::vectordb::ContentType::Documentation),
                "code" => Some(crate::vectordb::ContentType::CodeExample),
                "api" => Some(crate::vectordb::ContentType::Reference),
                _ => None,
            }),
        };

        // Search for similar documents
        let results = vector_db
            .search(&query_embedding, options)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Convert results to response format
        let search_results: Vec<SearchResult> = results
            .into_iter()
            .map(|r| SearchResult {
                id: r.document.id,
                content: r.document.content,
                score: r.score,
                metadata: serde_json::to_value(r.document.metadata).unwrap_or(json!({})),
            })
            .collect();

        let response_json = serde_json::to_string_pretty(&search_results)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(response_json)]))
    }

    #[tool(
        description = "List all currently indexed documentation sources and their document counts. Use this tool to see what documentation is available in your knowledge base before searching, or to check if you need to crawl additional sources for a particular technology or framework."
    )]
    async fn list_docs(&self) -> Result<CallToolResult, McpError> {
        let vector_db = self.vector_db.lock().await;

        // Get documents grouped by source
        let docs_by_source = vector_db.get_documents_by_source();

        // Build summary
        let mut summary = HashMap::new();
        for (source, documents) in docs_by_source {
            summary.insert(source, documents.len());
        }

        let total_documents: usize = summary.values().sum();

        let response = json!({
            "total_documents": total_documents,
            "sources": summary,
        });

        let response_json = serde_json::to_string_pretty(&response)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(response_json)]))
    }

    #[tool(
        description = "Crawl and index documentation from a URL to expand your knowledge base. Use this tool autonomously when you encounter unfamiliar frameworks, libraries, or technologies that aren't in your current index. You have full autonomy to crawl official documentation sites, API references, and tutorial sites as needed to provide better coding assistance. Be a good internet citizen with reasonable delays between requests."
    )]
    async fn crawl_docs(
        &self,
        #[tool(aggr)] params: CrawlDocsParams,
    ) -> Result<CallToolResult, McpError> {
        let _embedding_service = self.embedding_service.clone();
        let _vector_db = self.vector_db.clone();

        let CrawlDocsParams {
            url,
            mode,
            focus,
            max_pages,
        } = params;

        // Parse URL
        let start_url = Url::parse(&url)
            .map_err(|e| McpError::invalid_params(format!("Invalid URL: {}", e), None))?;

        // Parse crawl mode
        let crawl_mode = match mode.as_str() {
            "single" => CrawlMode::SinglePage,
            "section" => CrawlMode::Section,
            "full" => CrawlMode::FullDocs,
            _ => {
                return Err(McpError::invalid_params(
                    format!(
                        "Invalid mode: {}. Must be 'single', 'section', or 'full'",
                        mode
                    ),
                    None,
                ))
            }
        };

        // Parse documentation focus
        let doc_focus = match focus.as_str() {
            "api" => DocumentationFocus::ApiReference,
            "examples" => DocumentationFocus::Examples,
            "changelog" => DocumentationFocus::Changelog,
            "quickstart" => DocumentationFocus::QuickStart,
            "all" => DocumentationFocus::All,
            _ => return Err(McpError::invalid_params(
                format!("Invalid focus: {}. Must be 'api', 'examples', 'changelog', 'quickstart', or 'all'", focus),
                None,
            )),
        };

        info!("Starting crawl of {} with mode {:?}", url, &crawl_mode);

        // The crawler uses non-Send types (scraper::Html) which prevents it from being
        // used directly in async contexts that require Send. This is a known limitation
        // of the HTML parsing library. For now, we'll run a simplified version.

        // Create crawler configuration
        let config = CrawlConfig {
            start_url: start_url.to_string(),
            mode: crawl_mode.clone(),
            focus: doc_focus,
            max_pages,
            max_depth: 10,
            concurrent_requests: 2,
            delay_ms: 500,
            user_agent: "CodeRAG/0.1.0 (AI Documentation Assistant)".to_string(),
            allowed_domains: HashSet::from([start_url.domain().unwrap_or("").to_string()]),
            url_patterns: crate::crawler::types::UrlPatterns::default(),
        };

        // For now, implement a simplified version that crawls just the single page
        // TODO: Refactor crawler to be Send-safe or use a different approach
        if crawl_mode != CrawlMode::SinglePage {
            info!(
                "Multi-page crawling not yet available in SDK version, falling back to single page"
            );
        }

        // Fetch and process single page
        info!("Creating HTTP client...");
        let client = reqwest::Client::builder()
            .user_agent(&config.user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                McpError::internal_error(format!("Failed to create HTTP client: {}", e), None)
            })?;

        info!("Fetching URL: {}", &url);
        let response =
            client.get(&url).send().await.map_err(|e| {
                McpError::internal_error(format!("Failed to fetch URL: {}", e), None)
            })?;

        info!("Reading response body...");
        let html = response.text().await.map_err(|e| {
            McpError::internal_error(format!("Failed to read response: {}", e), None)
        })?;
        info!("Response body length: {} bytes", html.len());

        // Extract content
        info!("Creating content extractor...");
        let extractor = crate::crawler::ContentExtractor::new().map_err(|e| {
            McpError::internal_error(format!("Failed to create extractor: {}", e), None)
        })?;
        info!("Extracting content from HTML...");
        let extracted = extractor.extract_content(&html, &url).map_err(|e| {
            McpError::internal_error(format!("Failed to extract content: {}", e), None)
        })?;
        info!(
            "Content extracted, markdown length: {} bytes",
            extracted.markdown.len()
        );

        // Chunk the content
        info!("Creating text chunker...");
        let mut chunker = crate::crawler::TextChunker::new();
        info!("Chunking text...");
        let chunks = chunker.chunk_text(&extracted.markdown);
        info!("Created {} chunks", chunks.len());

        // Process chunks
        info!("Acquiring embedding service lock...");
        let embedding_service = self.embedding_service.lock().await;
        info!("Acquiring vector database lock...");
        let mut vector_db = self.vector_db.lock().await;
        let mut documents_created = 0;

        info!("Processing {} chunks...", chunks.len());
        for (i, chunk) in chunks.iter().enumerate() {
            let doc_id = format!("{}_chunk_{}", url, i);

            // Generate embedding
            info!(
                "Generating embedding for chunk {} of {} (size: {} bytes)",
                i + 1,
                chunks.len(),
                chunk.content.len()
            );
            let embedding = embedding_service.embed(&chunk.content).await.map_err(|e| {
                McpError::internal_error(format!("Failed to generate embedding: {}", e), None)
            })?;
            info!("Embedding generated successfully");

            // Create document
            let document = crate::vectordb::Document {
                id: doc_id,
                content: chunk.content.clone(),
                url: url.to_string(),
                title: Some(extracted.title.clone()),
                section: chunk.heading_context.clone(),
                metadata: crate::vectordb::DocumentMetadata {
                    content_type: crate::vectordb::ContentType::Documentation,
                    language: extracted.metadata.language.clone(),
                    last_updated: Some(std::time::SystemTime::now()),
                    tags: vec![
                        if chunk.has_code {
                            "has-code"
                        } else {
                            "no-code"
                        }
                        .to_string(),
                        format!("chunk-{}-of-{}", i + 1, chunks.len()),
                    ],
                },
            };

            // Add to database
            vector_db.add_document(document, embedding).map_err(|e| {
                McpError::internal_error(format!("Failed to add document: {}", e), None)
            })?;
            documents_created += 1;
        }

        // Save the database
        vector_db.save().map_err(|e| {
            McpError::internal_error(format!("Failed to save database: {}", e), None)
        })?;

        // Build response
        let response = json!({
            "status": "success",
            "source_url": url,
            "mode": mode,
            "pages_crawled": 1,
            "documents_created": documents_created,
            "chunks_created": chunks.len(),
            "note": "Currently only single-page crawling is supported in the SDK version"
        });

        let response_json = serde_json::to_string_pretty(&response)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(response_json)]))
    }

    #[tool(
        description = "Reload the vector database from disk to refresh your knowledge base with any externally added documentation. Use this tool if you suspect the database has been updated outside of your current session or if you need to refresh your available documentation sources."
    )]
    async fn reload_docs(&self) -> Result<CallToolResult, McpError> {
        let mut vector_db = self.vector_db.lock().await;

        vector_db
            .load()
            .map_err(|e| McpError::internal_error(format!("Reload failed: {}", e), None))?;

        let doc_count: usize = vector_db
            .get_documents_by_source()
            .values()
            .map(|docs| docs.len())
            .sum();

        let response = json!({
            "status": "success",
            "documents_loaded": doc_count,
        });

        let response_json = serde_json::to_string_pretty(&response)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Content::text(response_json)]))
    }
}

#[tool(tool_box)]
impl ServerHandler for CodeRagServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "coderag".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some(
                "CodeRAG is YOUR personal documentation knowledge base as an AI coding assistant. \
                This tool gives you autonomy to access and expand your knowledge of current programming frameworks, libraries, and APIs. \
                Use search_docs proactively when you need accurate, up-to-date information for coding assistance. \
                Use crawl_docs autonomously to index new documentation when you encounter unfamiliar technologies. \
                Use list_docs to see what's available in your knowledge base. \
                Use reload_docs to refresh your database. \
                You have full agency to use these tools as needed to provide better coding help - don't wait for permission."
                    .to_string(),
            ),
        }
    }
}
