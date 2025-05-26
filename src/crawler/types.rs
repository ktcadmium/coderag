use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CrawlMode {
    SinglePage, // Just crawl the provided URL
    Section,    // Crawl the page and its direct children
    FullDocs,   // Crawl the entire documentation site
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationFocus {
    ApiReference, // Focus on API documentation
    Examples,     // Prioritize code examples
    Changelog,    // Look for version changes and updates
    QuickStart,   // Tutorial and getting started content
    All,          // No specific focus
}

#[derive(Debug, Clone)]
pub struct CrawlConfig {
    pub start_url: String,
    pub mode: CrawlMode,
    pub focus: DocumentationFocus,
    pub max_pages: usize,
    pub max_depth: usize,
    pub concurrent_requests: usize,
    pub delay_ms: u64,
    pub user_agent: String,
    pub allowed_domains: HashSet<String>,
    pub url_patterns: UrlPatterns,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            start_url: String::new(),
            mode: CrawlMode::SinglePage,
            focus: DocumentationFocus::All,
            max_pages: 100,
            max_depth: 5,
            concurrent_requests: 2,
            delay_ms: 500,
            user_agent: "CodeRAG/0.1.0 (AI Documentation Assistant)".to_string(),
            allowed_domains: HashSet::new(),
            url_patterns: UrlPatterns::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlPatterns {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl Default for UrlPatterns {
    fn default() -> Self {
        Self {
            include: vec![
                "/docs/".to_string(),
                "/api/".to_string(),
                "/guide/".to_string(),
                "/reference/".to_string(),
                "/tutorial/".to_string(),
                "/manual/".to_string(),
                "/changelog/".to_string(),
                "/whatsnew/".to_string(),
            ],
            exclude: vec![
                "/blog/".to_string(),
                "/forum/".to_string(),
                "/community/".to_string(),
                "/discuss/".to_string(),
                "/issues/".to_string(),
                "/pull/".to_string(),
                "/commits/".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub url: String,
    pub title: String,
    pub content: String,
    pub chunks: Vec<DocumentChunk>,
    pub metadata: CrawlMetadata,
}

#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub content: String,
    pub start_char: usize,
    pub end_char: usize,
    pub has_code: bool,
    pub heading_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlMetadata {
    pub crawled_at: String,
    pub content_type: String,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CrawlProgress {
    pub pages_crawled: usize,
    pub pages_queued: usize,
    pub pages_failed: usize,
    pub current_url: Option<String>,
}
