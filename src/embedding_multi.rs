use anyhow::{Result, anyhow};
use candle_core::Device;
use tracing::{info, debug, warn, error};

/// Embedding service that tries multiple approaches to find the best one
pub struct MultiEmbeddingService {
    strategy: EmbeddingStrategy,
}

#[derive(Debug)]
enum EmbeddingStrategy {
    Glowrs(GlowrsService),
    FastEmbed(FastEmbedService),
    // Fallback(MockService),
}

impl MultiEmbeddingService {
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing multi-strategy embedding service...");
        
        // Try strategies in order of preference
        
        // Strategy 1: Glowrs (pure Candle)
        info!("🧪 Trying Glowrs (SentenceTransformers for Candle)...");
        match GlowrsService::new().await {
            Ok(service) => {
                info!("✅ Glowrs strategy successful!");
                return Ok(Self {
                    strategy: EmbeddingStrategy::Glowrs(service),
                });
            }
            Err(e) => {
                warn!("⚠️ Glowrs strategy failed: {}", e);
            }
        }
        
        // Strategy 2: FastEmbed (ONNX Runtime)
        info!("🧪 Trying FastEmbed (ONNX Runtime)...");
        match FastEmbedService::new().await {
            Ok(service) => {
                info!("✅ FastEmbed strategy successful!");
                return Ok(Self {
                    strategy: EmbeddingStrategy::FastEmbed(service),
                });
            }
            Err(e) => {
                warn!("⚠️ FastEmbed strategy failed: {}", e);
            }
        }
        
        Err(anyhow!("All embedding strategies failed"))
    }
    
    pub async fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        match &mut self.strategy {
            EmbeddingStrategy::Glowrs(service) => service.embed(text).await,
            EmbeddingStrategy::FastEmbed(service) => service.embed(text).await,
        }
    }
    
    pub fn strategy_name(&self) -> &str {
        match &self.strategy {
            EmbeddingStrategy::Glowrs(_) => "Glowrs (Candle)",
            EmbeddingStrategy::FastEmbed(_) => "FastEmbed (ONNX)",
        }
    }
}

// Glowrs implementation
struct GlowrsService {
    model: glowrs::SentenceEmbeddingsBuilder,
}

impl GlowrsService {
    async fn new() -> Result<Self> {
        info!("Loading SentenceTransformers model with Glowrs...");
        
        // Try common high-quality embedding models that should work with standard BERT
        let model_names = [
            "sentence-transformers/all-MiniLM-L6-v2",      // Fast, good quality
            "sentence-transformers/all-mpnet-base-v2",     // High quality
            "sentence-transformers/all-MiniLM-L12-v2",     // Medium size
        ];
        
        for model_name in &model_names {
            info!("Trying model: {}", model_name);
            match glowrs::SentenceEmbeddingsBuilder::remote(model_name.to_string()).build() {
                Ok(model) => {
                    info!("✅ Successfully loaded model: {}", model_name);
                    return Ok(Self { model });
                }
                Err(e) => {
                    warn!("Failed to load {}: {}", model_name, e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("Failed to load any Glowrs model"))
    }
    
    async fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating Glowrs embedding for: '{}'", text);
        
        let embeddings = self.model.encode(&[text])
            .map_err(|e| anyhow!("Glowrs encoding failed: {}", e))?;
        
        if embeddings.is_empty() {
            return Err(anyhow!("No embeddings returned"));
        }
        
        info!("✅ Generated Glowrs embedding: {} dimensions", embeddings[0].len());
        Ok(embeddings.into_iter().next().unwrap())
    }
}

// FastEmbed implementation  
struct FastEmbedService {
    model: fastembed::TextEmbedding,
}

impl FastEmbedService {
    async fn new() -> Result<Self> {
        info!("Loading embedding model with FastEmbed...");
        
        // Try different FastEmbed models
        let models = [
            fastembed::EmbeddingModel::AllMiniLML6V2,    // Fast and efficient
            fastembed::EmbeddingModel::BGEBaseEN,        // Good quality
            fastembed::EmbeddingModel::AllMiniLML12V2,   // Medium size
        ];
        
        for model_type in &models {
            info!("Trying FastEmbed model: {:?}", model_type);
            match fastembed::TextEmbedding::try_new(fastembed::InitOptions {
                model_name: *model_type,
                show_download_progress: true,
                ..Default::default()
            }) {
                Ok(model) => {
                    info!("✅ Successfully loaded FastEmbed model: {:?}", model_type);
                    return Ok(Self { model });
                }
                Err(e) => {
                    warn!("Failed to load {:?}: {}", model_type, e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("Failed to load any FastEmbed model"))
    }
    
    async fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating FastEmbed embedding for: '{}'", text);
        
        let embeddings = self.model.embed(vec![text], None)
            .map_err(|e| anyhow!("FastEmbed encoding failed: {}", e))?;
        
        if embeddings.is_empty() {
            return Err(anyhow!("No embeddings returned"));
        }
        
        let embedding = embeddings.into_iter().next().unwrap();
        info!("✅ Generated FastEmbed embedding: {} dimensions", embedding.len());
        Ok(embedding)
    }
}