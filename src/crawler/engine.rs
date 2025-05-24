use anyhow::Result;
use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::Client;
// use robotparser::RobotFileParser; // TODO: Find alternative crate
use chrono::Utc;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use url::Url;

use crate::crawler::{
    ContentExtractor, CrawlConfig, CrawlMetadata, CrawlMode, CrawlProgress, CrawlResult,
    TextChunker,
};
use crate::embedding_basic::EmbeddingService;
use crate::vectordb::VectorDatabase;

type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>;

pub struct Crawler {
    config: CrawlConfig,
    client: Client,
    rate_limiter: SharedRateLimiter,
    extractor: ContentExtractor,
    chunker: TextChunker,
    visited_urls: Arc<Mutex<HashSet<String>>>,
    url_queue: Arc<Mutex<VecDeque<(String, usize)>>>, // (url, depth)
    progress: Arc<Mutex<CrawlProgress>>,
    // robots_cache: Arc<Mutex<HashMap<String, RobotFileParser>>>, // TODO: Add back with alternative crate
}

impl Crawler {
    pub async fn new(config: CrawlConfig) -> Result<Self> {
        // Create HTTP client with appropriate headers
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(Duration::from_secs(30))
            .build()?;

        // Create rate limiter based on config
        let quota = Quota::per_second(
            NonZeroU32::new(config.concurrent_requests as u32)
                .unwrap_or(NonZeroU32::new(2).unwrap()),
        );
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Ok(Self {
            config,
            client,
            rate_limiter,
            extractor: ContentExtractor::new()?,
            chunker: TextChunker::new(),
            visited_urls: Arc::new(Mutex::new(HashSet::new())),
            url_queue: Arc::new(Mutex::new(VecDeque::new())),
            progress: Arc::new(Mutex::new(CrawlProgress {
                pages_crawled: 0,
                pages_queued: 0,
                pages_failed: 0,
                current_url: None,
            })),
            // robots_cache: Arc::new(Mutex::new(HashMap::new())), // TODO: Add back
        })
    }

    pub async fn crawl(
        &self,
        embedding_service: &EmbeddingService,
        vector_db: &mut VectorDatabase,
    ) -> Result<Vec<String>> {
        // Initialize the queue with the start URL
        {
            let mut queue = self.url_queue.lock().await;
            queue.push_back((self.config.start_url.clone(), 0));
        }

        let mut crawled_urls = Vec::new();

        // Main crawl loop
        while let Some((url, depth)) = self.get_next_url().await {
            // Check if we've reached our limits
            if crawled_urls.len() >= self.config.max_pages {
                tracing::info!("Reached max pages limit: {}", self.config.max_pages);
                break;
            }

            if depth > self.config.max_depth {
                tracing::debug!("Skipping {} - exceeds max depth", url);
                continue;
            }

            // Update progress
            {
                let mut progress = self.progress.lock().await;
                progress.current_url = Some(url.clone());
            }

            // TODO: Check robots.txt when we have a working crate
            // if !self.is_allowed_by_robots(&url).await {
            //     tracing::warn!("Blocked by robots.txt: {}", url);
            //     continue;
            // }

            // Rate limiting
            self.rate_limiter.until_ready().await;

            // Crawl the page
            match self
                .crawl_page(&url, depth, embedding_service, vector_db)
                .await
            {
                Ok(result) => {
                    crawled_urls.push(url.clone());

                    // Update progress
                    {
                        let mut progress = self.progress.lock().await;
                        progress.pages_crawled += 1;
                    }

                    // Extract and queue new URLs based on crawl mode
                    if self.should_follow_links(depth) {
                        self.extract_and_queue_urls(&result, depth + 1).await?;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to crawl {}: {}", url, e);
                    let mut progress = self.progress.lock().await;
                    progress.pages_failed += 1;
                }
            }

            // Add delay between requests
            sleep(Duration::from_millis(self.config.delay_ms)).await;
        }

        Ok(crawled_urls)
    }

    async fn get_next_url(&self) -> Option<(String, usize)> {
        let mut queue = self.url_queue.lock().await;
        queue.pop_front()
    }

    async fn crawl_page(
        &self,
        url: &str,
        _depth: usize,
        embedding_service: &EmbeddingService,
        vector_db: &mut VectorDatabase,
    ) -> Result<CrawlResult> {
        // Mark as visited
        {
            let mut visited = self.visited_urls.lock().await;
            visited.insert(url.to_string());
        }

        // Fetch the page
        let response = self.client.get(url).send().await?;

        // Handle rate limiting (429) with exponential backoff
        if response.status() == 429 {
            tracing::warn!("Rate limited at {}, backing off", url);
            sleep(Duration::from_secs(10)).await;
            return Err(anyhow::anyhow!("Rate limited"));
        }

        let html = response.text().await?;

        // Extract content
        let extracted = self.extractor.extract_content(&html, url)?;

        // Chunk the content
        let chunks = self.chunker.chunk_text(&extracted.markdown);

        // Create documents and add to vector database
        for (i, chunk) in chunks.iter().enumerate() {
            let doc_id = format!("{}_chunk_{}", url, i);

            // Generate embedding
            let embedding = embedding_service.embed(&chunk.content).await?;

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
            vector_db.add_document(document, embedding)?;
        }

        // Create crawl result
        let result = CrawlResult {
            url: url.to_string(),
            title: extracted.title,
            content: extracted.markdown,
            chunks,
            metadata: CrawlMetadata {
                crawled_at: Utc::now().to_rfc3339(),
                content_type: "documentation".to_string(),
                language: extracted.metadata.language,
                framework: extracted.metadata.framework,
                version: extracted.metadata.version,
            },
        };

        Ok(result)
    }

    fn should_follow_links(&self, current_depth: usize) -> bool {
        match self.config.mode {
            CrawlMode::SinglePage => false,
            CrawlMode::Section => current_depth == 0,
            CrawlMode::FullDocs => current_depth < self.config.max_depth,
        }
    }

    async fn extract_and_queue_urls(&self, result: &CrawlResult, next_depth: usize) -> Result<()> {
        let base_url = Url::parse(&result.url)?;
        let html = Html::parse_document(&result.content);
        let link_selector = Selector::parse("a[href]").unwrap();

        let mut new_urls = Vec::new();

        for element in html.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(absolute_url) = base_url.join(href) {
                    let url_str = absolute_url.to_string();

                    // Check if we should crawl this URL
                    if self.should_crawl_url(&url_str).await {
                        new_urls.push((url_str, next_depth));
                    }
                }
            }
        }

        // Add URLs to queue
        let mut queue = self.url_queue.lock().await;
        let visited = self.visited_urls.lock().await;

        for (url, depth) in new_urls {
            if !visited.contains(&url) && !queue.iter().any(|(u, _)| u == &url) {
                queue.push_back((url, depth));
            }
        }

        // Update progress
        let mut progress = self.progress.lock().await;
        progress.pages_queued = queue.len();

        Ok(())
    }

    async fn should_crawl_url(&self, url: &str) -> bool {
        // Check if URL matches our patterns
        let matches_include = self
            .config
            .url_patterns
            .include
            .iter()
            .any(|pattern| url.contains(pattern));

        let matches_exclude = self
            .config
            .url_patterns
            .exclude
            .iter()
            .any(|pattern| url.contains(pattern));

        if matches_exclude {
            return false;
        }

        // If we have include patterns, URL must match at least one
        if !self.config.url_patterns.include.is_empty() && !matches_include {
            return false;
        }

        // Check domain restrictions
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                // If allowed_domains is empty, allow all domains
                if self.config.allowed_domains.is_empty() {
                    return true;
                }

                // Otherwise, check if the domain is allowed
                return self.config.allowed_domains.contains(host);
            }
        }

        false
    }

    // TODO: Implement robots.txt checking when we have a working crate
    // async fn is_allowed_by_robots(&self, url: &str) -> bool {
    //     true // Allow all for now
    // }

    pub async fn get_progress(&self) -> CrawlProgress {
        self.progress.lock().await.clone()
    }
}
