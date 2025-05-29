use anyhow::Result;
use clap::{Parser, Subcommand};
use coderag::crawler::{CrawlConfig, CrawlMode, Crawler, DocumentationFocus};
use coderag::embedding_basic::EmbeddingService;
use coderag::mcp::CodeRagServer;
use coderag::vectordb::VectorDatabase;
use rmcp::{transport::stdio, ServiceExt};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Parser, Debug)]
#[command(author, version, about = "CodeRAG MCP Server", long_about = None)]
struct Args {
    /// Data directory for storing embeddings and documents
    #[arg(short, long, default_value = "~/.coderag")]
    data_dir: String,

    /// Enable debug logging
    #[arg(long, action)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the MCP server (default)
    Serve,

    /// Crawl a documentation site directly
    Crawl {
        /// URL to crawl
        url: String,

        /// Crawl mode: single, section, or full
        #[arg(short, long, default_value = "single")]
        mode: String,

        /// Documentation focus: api, examples, changelog, quickstart, or all
        #[arg(short, long, default_value = "all")]
        focus: String,

        /// Maximum pages to crawl
        #[arg(long, default_value = "100")]
        max_pages: usize,

        /// Enable verbose debug output
        #[arg(short, long)]
        verbose: bool,
    },
}

// Custom exit function that avoids destructors
extern "C" fn force_exit() {
    unsafe {
        libc::_exit(0);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set HuggingFace user agent to prevent CDN blocking
    std::env::set_var("HF_HUB_USER_AGENT_ORIGIN", "CodeRAG/0.1.0");

    // Register our exit handler to run before ONNX cleanup
    unsafe {
        libc::atexit(force_exit);
    }

    let args = Args::parse();

    // Initialize logging based on command
    let (debug_level, verbose_crawl) = match &args.command {
        Some(Commands::Crawl { verbose, .. }) => (args.debug || *verbose, *verbose),
        _ => (args.debug, false),
    };

    tracing_subscriber::fmt()
        .with_max_level(if debug_level {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Expand home directory
    let data_dir = if args.data_dir.starts_with("~") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(args.data_dir.replacen("~", &home, 1))
    } else {
        PathBuf::from(args.data_dir)
    };

    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)?;

    match args.command {
        Some(Commands::Crawl {
            url,
            mode,
            focus,
            max_pages,
            ..
        }) => {
            // Run crawler directly
            run_crawler(data_dir, url, mode, focus, max_pages, verbose_crawl).await
        }
        Some(Commands::Serve) | None => {
            // Run MCP server (default behavior)
            tracing::info!(
                "Starting CodeRAG MCP server with data directory: {:?}",
                data_dir
            );
            tracing::info!("💡 FastEmbed model will be downloaded on first search request");

            // Create and start the MCP server using the official SDK
            let server = CodeRagServer::new(data_dir).await?;
            let service = server.serve(stdio()).await.inspect_err(|e| {
                tracing::error!("Failed to start MCP server: {:?}", e);
            })?;

            // Wait for the service to complete
            service.waiting().await?;

            // Exit cleanly without running destructors
            unsafe {
                libc::_exit(0);
            }
        }
    }
}

async fn run_crawler(
    data_dir: PathBuf,
    url: String,
    mode: String,
    focus: String,
    max_pages: usize,
    verbose: bool,
) -> Result<()> {
    tracing::info!("🕷️ Starting direct crawler");
    tracing::info!("URL: {}", url);
    tracing::info!("Mode: {}", mode);
    tracing::info!("Focus: {}", focus);
    tracing::info!("Max pages: {}", max_pages);

    // Parse crawl mode
    let crawl_mode = match mode.as_str() {
        "single" => CrawlMode::SinglePage,
        "section" => CrawlMode::Section,
        "full" => CrawlMode::FullDocs,
        _ => {
            anyhow::bail!(
                "Invalid crawl mode: {}. Use 'single', 'section', or 'full'",
                mode
            );
        }
    };

    // Parse documentation focus
    let doc_focus = match focus.as_str() {
        "api" => DocumentationFocus::ApiReference,
        "examples" => DocumentationFocus::Examples,
        "changelog" => DocumentationFocus::Changelog,
        "quickstart" => DocumentationFocus::QuickStart,
        "all" => DocumentationFocus::All,
        _ => {
            anyhow::bail!(
                "Invalid focus: {}. Use 'api', 'examples', 'changelog', 'quickstart', or 'all'",
                focus
            );
        }
    };

    // Parse URL to get domain
    let parsed_url = url::Url::parse(&url)?;
    let domain = parsed_url
        .domain()
        .ok_or_else(|| anyhow::anyhow!("Invalid URL: no domain found"))?
        .to_string();

    let mut allowed_domains = std::collections::HashSet::new();
    allowed_domains.insert(domain);

    // Create crawl config
    let config = CrawlConfig {
        start_url: url.clone(),
        mode: crawl_mode,
        focus: doc_focus,
        max_pages,
        max_depth: 3,
        concurrent_requests: 2,
        delay_ms: 500,
        user_agent: "CodeRAG/0.1.0 (AI Documentation Assistant)".to_string(),
        allowed_domains,
        url_patterns: coderag::crawler::UrlPatterns::default(),
    };

    // Initialize embedding service (lazy initialization - no model download yet)
    tracing::info!("📦 Creating embedding service...");
    let embedding_service = EmbeddingService::new().await?;
    tracing::info!("✅ Embedding service created (model will download on first use)");

    // Initialize vector database
    let db_path = data_dir.join("coderag_vectordb.json");
    tracing::info!("📂 Loading vector database from: {:?}", db_path);

    let mut vector_db = VectorDatabase::new(db_path.clone())?;
    if db_path.exists() {
        vector_db.load()?;
        tracing::info!(
            "📊 Loaded {} existing documents",
            vector_db.document_count()
        );
    }

    // Create crawler
    tracing::info!("🕷️ Creating crawler...");
    let mut crawler = Crawler::new(config.clone()).await?;
    tracing::info!("✅ Crawler initialized");

    // Run crawl with timeout and detailed progress
    tracing::info!("🌐 Starting crawl...");
    let crawl_timeout = Duration::from_secs(300); // 5 minutes

    match timeout(
        crawl_timeout,
        crawler.crawl(&embedding_service, &mut vector_db),
    )
    .await
    {
        Ok(Ok(crawled_urls)) => {
            tracing::info!(
                "✅ Crawl completed successfully! Crawled {} URLs",
                crawled_urls.len()
            );

            // Save database (documents were already stored during crawling)
            tracing::info!("💾 Saving vector database...");
            vector_db.save()?;

            tracing::info!("📊 Summary:");
            tracing::info!("  - URLs crawled: {}", crawled_urls.len());
            tracing::info!(
                "  - Total documents in database: {}",
                vector_db.document_count()
            );

            // List the crawled URLs if verbose
            if verbose {
                tracing::info!("📃 Crawled URLs:");
                for url in &crawled_urls {
                    tracing::info!("  - {}", url);
                }
            }

            Ok(())
        }
        Ok(Err(e)) => {
            tracing::error!("❌ Crawl failed: {}", e);
            Err(e)
        }
        Err(_) => {
            tracing::error!("❌ Crawl timeout ({}s)", crawl_timeout.as_secs());
            anyhow::bail!("Crawl timeout")
        }
    }
}
