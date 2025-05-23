use anyhow::Result;
use clap::Parser;
use coderag::mcp::McpServer;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "CodeRAG MCP Server", long_about = None)]
struct Args {
    /// Data directory for storing embeddings and documents
    #[arg(short, long, default_value = "~/.coderag")]
    data_dir: String,

    /// Enable debug logging
    #[arg(short, long, action)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(if args.debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
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

    // Start MCP server
    let server = McpServer::new(data_dir).await?;
    server.run_stdio().await?;

    Ok(())
}
