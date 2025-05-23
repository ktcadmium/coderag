use anyhow::{anyhow, Result};
use tracing::{debug, info};

/// Basic working embedding service using FastEmbed
pub struct EmbeddingService {
    model: fastembed::TextEmbedding,
}

impl EmbeddingService {
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing FastEmbed embedding service...");

        // Use the default model first to test basic functionality
        info!("🧪 Loading default embedding model...");

        let model = fastembed::TextEmbedding::try_new(
            fastembed::InitOptions::new(fastembed::EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true),
        )?;

        info!("✅ Successfully loaded all-MiniLM-L6-v2 model");
        Ok(Self { model })
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("🔄 Generating embedding for: '{}'", text);

        let embeddings = self.model.embed(vec![text], None)?;

        if embeddings.is_empty() {
            return Err(anyhow!("No embeddings returned"));
        }

        let embedding = embeddings.into_iter().next().unwrap();
        info!("✅ Generated embedding: {} dimensions", embedding.len());
        Ok(embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_service() -> Result<()> {
        let service = EmbeddingService::new().await?;
        let embedding = service.embed("test text").await?;
        assert!(!embedding.is_empty());
        Ok(())
    }
}
