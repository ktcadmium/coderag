use coderag::crawler::{CrawlConfig, CrawlMode, Crawler, DocumentationFocus};
use coderag::embedding_basic::EmbeddingService;
use coderag::vectordb::VectorDatabase;
use std::collections::HashSet;
use tempfile::tempdir;

#[tokio::test]
async fn test_single_page_crawl() {
    // Create temporary directory for test data
    let temp_dir = tempdir().unwrap();
    let data_path = temp_dir.path();

    // Initialize services
    let embedding_service = EmbeddingService::new().await.unwrap();
    let mut vector_db = VectorDatabase::new(data_path).unwrap();

    // Create crawler config for single page
    let mut allowed_domains = HashSet::new();
    allowed_domains.insert("example.com".to_string());

    let config = CrawlConfig {
        start_url: "https://example.com".to_string(),
        mode: CrawlMode::SinglePage,
        focus: DocumentationFocus::All,
        max_pages: 1,
        allowed_domains,
        ..Default::default()
    };

    // Create crawler
    let mut crawler = Crawler::new(config).await.unwrap();

    // Run crawl
    let result = crawler.crawl(&embedding_service, &mut vector_db).await;

    // Basic test - just ensure it doesn't panic
    // In a real test we'd mock the HTTP requests
    match result {
        Ok(urls) => {
            println!("Crawled {} URLs", urls.len());
        }
        Err(e) => {
            println!("Crawl error (expected in test): {}", e);
        }
    }
}

#[tokio::test]
async fn test_crawler_config_defaults() {
    let config = CrawlConfig::default();

    assert_eq!(config.max_pages, 100);
    assert_eq!(config.max_depth, 5);
    assert_eq!(config.concurrent_requests, 2);
    assert_eq!(config.delay_ms, 500);
    assert_eq!(
        config.user_agent,
        "CodeRAG/0.1.0 (AI Documentation Assistant)"
    );
}

#[tokio::test]
async fn test_url_pattern_defaults() {
    let config = CrawlConfig::default();
    let patterns = &config.url_patterns;

    // Check include patterns
    assert!(patterns.include.contains(&"/docs/".to_string()));
    assert!(patterns.include.contains(&"/api/".to_string()));
    assert!(patterns.include.contains(&"/guide/".to_string()));

    // Check exclude patterns
    assert!(patterns.exclude.contains(&"/blog/".to_string()));
    assert!(patterns.exclude.contains(&"/forum/".to_string()));
}
