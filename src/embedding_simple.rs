use anyhow::{Result, anyhow};
use tracing::{info, debug, warn};

/// Simple, working embedding service using FastEmbed
pub struct EmbeddingService {
    model: fastembed::TextEmbedding,
    model_name: String,
}

impl EmbeddingService {
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing FastEmbed embedding service...");
        
        // Try different models in order of preference for documentation use
        let models = [
            (fastembed::EmbeddingModel::AllMiniLML6V2, "all-MiniLM-L6-v2 (Fast, Good Quality)"),
            (fastembed::EmbeddingModel::BGEBaseENV15, "BGE-Base-EN-v1.5 (High Quality)"),
            (fastembed::EmbeddingModel::AllMiniLML12V2, "all-MiniLM-L12-v2 (Balanced)"),
            (fastembed::EmbeddingModel::BGESmallENV15, "BGE-Small-EN-v1.5 (Fast)"),
        ];
        
        for (model_type, description) in &models {
            info!("🧪 Trying: {}", description);
            match fastembed::TextEmbedding::try_new(Default::default())
                .and_then(|builder| builder.with_model(*model_type).build()) {
                Ok(model) => {
                    info!("✅ Successfully loaded: {}", description);
                    return Ok(Self { 
                        model,
                        model_name: description.to_string(),
                    });
                }
                Err(e) => {
                    warn!("❌ Failed to load {}: {}", description, e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("Failed to load any FastEmbed model"))
    }
    
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        debug!("🔄 Generating embedding for: '{}'", text);
        
        let embeddings = self.model.embed(vec![text], None)
            .map_err(|e| anyhow!("FastEmbed encoding failed: {}", e))?;
        
        if embeddings.is_empty() {
            return Err(anyhow!("No embeddings returned"));
        }
        
        let embedding = embeddings.into_iter().next().unwrap();
        info!("✅ Generated embedding: {} dimensions", embedding.len());
        Ok(embedding)
    }
    
    pub fn model_name(&self) -> &str {
        &self.model_name
    }
    
    /// Test embedding quality by checking semantic similarity
    pub async fn test_quality(&self) -> Result<()> {
        info!("🧪 Testing embedding quality with semantic similarity...");
        
        let test_pairs = [
            ("documentation", "docs"),
            ("function", "method"),
            ("Rust programming", "Rust development"),
            ("machine learning", "artificial intelligence"),
        ];
        
        for (text1, text2) in &test_pairs {
            let emb1 = self.embed(text1).await?;
            let emb2 = self.embed(text2).await?;
            
            let similarity = cosine_similarity(&emb1, &emb2);
            info!("📊 Similarity between '{}' and '{}': {:.3}", text1, text2, similarity);
        }
        
        Ok(())
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_embedding_service() -> Result<()> {
        let service = EmbeddingService::new().await?;
        
        // Test that we get consistent embeddings for same text
        let embedding1 = service.embed("test text").await?;
        let embedding2 = service.embed("test text").await?;
        assert_eq!(embedding1, embedding2);
        assert!(!embedding1.is_empty());
        
        // Test that different text gives different embeddings
        let embedding3 = service.embed("different text").await?;
        assert_ne!(embedding1, embedding3);
        
        // Test semantic similarity
        let similarity = cosine_similarity(&embedding1, &embedding2);
        assert!((similarity - 1.0).abs() < 1e-6); // Should be exactly 1.0
        
        Ok(())
    }
}