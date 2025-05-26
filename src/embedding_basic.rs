use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::{Arc, Mutex, Once};
use tracing::{debug, error, info, warn};

/// Embedding service using FastEmbed with lazy initialization
pub struct EmbeddingService {
    model: Arc<Mutex<Option<TextEmbedding>>>,
    init_once: Once,
}

impl Drop for EmbeddingService {
    fn drop(&mut self) {
        // Attempt to clean up the model gracefully
        if let Ok(mut guard) = self.model.try_lock() {
            if guard.is_some() {
                debug!("🧹 Cleaning up embedding model...");
                *guard = None;
                debug!("✅ Embedding model cleanup completed");
            }
        } else {
            // If we can't acquire the lock, just warn and continue
            // This prevents the mutex lock failure crash during shutdown
            warn!("⚠️ Could not acquire lock for embedding model cleanup - continuing shutdown");
        }
    }
}

impl EmbeddingService {
    /// Create a new embedding service with lazy initialization
    pub async fn new() -> Result<Self> {
        info!("🚀 Creating FastEmbed embedding service (lazy initialization)");
        info!("📦 Model: all-MiniLM-L6-v2 (384 dimensions)");
        info!("💡 Model will be downloaded on first use (~90MB, 1-2 minutes)");

        Ok(Self {
            model: Arc::new(Mutex::new(None)),
            init_once: Once::new(),
        })
    }

    /// Ensure the model is initialized (download and load if needed)
    fn ensure_initialized(&self) -> Result<()> {
        let mut init_result = Ok(());

        self.init_once.call_once(|| {
            info!("🔄 First embedding request - initializing FastEmbed model...");
            info!("📥 Downloading all-MiniLM-L6-v2 model (~90MB)...");
            info!("⏳ This may take 1-2 minutes on first run...");

            // Set cache directory
            let cache_dir = std::env::var("FASTEMBED_CACHE_PATH")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    std::path::PathBuf::from(format!("{}/.cache/fastembed", home))
                });

            info!("📂 Using cache directory: {:?}", cache_dir);

            // Try to initialize the model with better error handling
            match Self::try_initialize_model(&cache_dir) {
                Ok(model) => {
                    info!("✅ Successfully loaded all-MiniLM-L6-v2 model");
                    info!("🔄 Warming up model...");

                    // Warm up the model with a test embedding
                    let start = std::time::Instant::now();
                    match model.embed(vec!["test"], None) {
                        Ok(_) => {
                            let duration = start.elapsed();
                            info!(
                                "✅ Model fully initialized and ready (warm-up took {:?})",
                                duration
                            );

                            // Store the initialized model
                            if let Ok(mut guard) = self.model.lock() {
                                *guard = Some(model);
                            } else {
                                init_result = Err(anyhow::anyhow!("Failed to acquire model lock"));
                            }
                        }
                        Err(e) => {
                            init_result = Err(anyhow::anyhow!("Model warm-up failed: {}", e));
                        }
                    }
                }
                Err(e) => {
                    init_result = Err(e);
                }
            }
        });

        init_result
    }

    /// Try to initialize the FastEmbed model with comprehensive error handling
    fn try_initialize_model(cache_dir: &std::path::Path) -> Result<TextEmbedding> {
        let init_options =
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_cache_dir(cache_dir.to_path_buf());

        match TextEmbedding::try_new(init_options) {
            Ok(model) => Ok(model),
            Err(e) => {
                error!("❌ Failed to initialize FastEmbed model: {}", e);

                // Provide helpful error messages based on the error type
                let error_msg = format!("{}", e);

                if error_msg.contains("Failed to retrieve") || error_msg.contains("download") {
                    error!("🌐 Network Error: Unable to download the embedding model");
                    error!("💡 This usually happens when:");
                    error!("   1. Network restrictions prevent downloading large files");
                    error!("   2. Corporate firewall blocks the download");
                    error!("   3. Temporary network connectivity issues");
                    error!("");
                    error!("🔧 Possible solutions:");
                    error!("   1. Try running outside of Claude Desktop first:");
                    error!("      cargo run --release --bin coderag-mcp crawl https://example.com");
                    error!("   2. Check your network connection");
                    error!("   3. Try again later (might be a temporary CDN issue)");
                    error!(
                        "   4. Contact your network administrator if behind a corporate firewall"
                    );

                    Err(anyhow::anyhow!(
                        "Failed to download FastEmbed model. This appears to be a network connectivity issue. \
                        The model download works in local environments but may fail in restricted environments like Claude Desktop. \
                        Try running the crawler directly first: `cargo run --release --bin coderag-mcp crawl https://example.com`"
                    ))
                } else if error_msg.contains("permission") || error_msg.contains("access") {
                    error!("🔒 Permission Error: Unable to write to cache directory");
                    error!("📂 Cache directory: {:?}", cache_dir);
                    error!("💡 Try setting FASTEMBED_CACHE_PATH to a writable directory");

                    Err(anyhow::anyhow!(
                        "Permission denied writing to cache directory: {:?}. \
                        Set FASTEMBED_CACHE_PATH environment variable to a writable directory.",
                        cache_dir
                    ))
                } else {
                    error!("❓ Unexpected error during model initialization");
                    error!("📝 Error details: {}", e);

                    Err(anyhow::anyhow!("FastEmbed initialization failed: {}", e))
                }
            }
        }
    }

    /// Generate embedding for a single text
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(vec![text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding generated"))
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Ensure model is initialized
        self.ensure_initialized()
            .context("Failed to initialize embedding model")?;

        // Get the model from the mutex
        let model_guard = self
            .model
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire model lock"))?;

        let model = model_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Model not initialized"))?;

        // Generate embeddings for each text
        let mut all_embeddings = Vec::new();

        for text in &texts {
            debug!(
                "🔄 Generating embedding for: '{}'",
                if text.len() > 50 {
                    format!("{}...", &text[..50])
                } else {
                    text.clone()
                }
            );

            let embeddings = model.embed(vec![text.as_str()], None).with_context(|| {
                format!(
                    "Failed to generate embedding for text: {}",
                    if text.len() > 100 {
                        format!("{}...", &text[..100])
                    } else {
                        text.clone()
                    }
                )
            })?;

            if let Some(embedding) = embeddings.first() {
                debug!("✅ Generated embedding: {} dimensions", embedding.len());
                all_embeddings.push(embedding.clone());
            } else {
                return Err(anyhow::anyhow!("No embedding generated for text"));
            }
        }

        Ok(all_embeddings)
    }

    /// Get the embedding dimension - useful for validation and debugging
    #[allow(dead_code)]
    pub fn dimension(&self) -> usize {
        384 // all-MiniLM-L6-v2 produces 384-dimensional embeddings
    }

    /// Validate that an embedding has the correct dimensions
    #[allow(dead_code)]
    pub fn validate_embedding(&self, embedding: &[f32]) -> anyhow::Result<()> {
        if embedding.len() != self.dimension() {
            anyhow::bail!(
                "Invalid embedding dimension: expected {}, got {}",
                self.dimension(),
                embedding.len()
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_service() -> Result<()> {
        let service = EmbeddingService::new().await?;
        let embedding = service.embed("test text").await?;
        assert_eq!(embedding.len(), 384);
        Ok(())
    }
}
