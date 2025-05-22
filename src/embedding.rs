use anyhow::Result;
use candle_core::{Device, Tensor, DType};
use candle_core::utils::cuda_is_available;
use tracing::{info, debug, warn};

pub struct EmbeddingService {
    device: Device,
}

impl EmbeddingService {
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing Candle embedding service...");
        
        // Try to use GPU if available, fallback to CPU
        let device = if cuda_is_available() {
            info!("✅ CUDA available, using GPU");
            Device::new_cuda(0)?
        } else {
            info!("✅ Using CPU device");
            Device::Cpu
        };
        
        // Test basic tensor operations to validate Candle is working
        Self::test_tensor_operations(&device)?;
        
        info!("✅ Candle framework is working correctly");
        info!("✅ Device initialization successful");
        warn!("⚠️  nomic-embed-text requires custom NomicBert implementation");
        info!("💡 Alternative approaches: ONNX Runtime, different embedding models, or custom implementation");
        
        Ok(Self { device })
    }
    
    fn test_tensor_operations(device: &Device) -> Result<()> {
        info!("🧪 Testing basic tensor operations...");
        
        // Test tensor creation and basic operations
        let a = Tensor::arange(0f32, 6f32, device)?.reshape((2, 3))?;
        let b = Tensor::arange(0f32, 12f32, device)?.reshape((3, 4))?;
        let c = a.matmul(&b)?;
        
        info!("✅ Matrix multiplication successful: {:?}", c.shape());
        
        // Test embedding-like operation (random vector)
        let vocab_size = 30000;
        let embedding_dim = 768;
        let embedding_weights = Tensor::randn(0f32, 1f32, (vocab_size, embedding_dim), device)?;
        let token_ids = Tensor::new(&[1u32, 2u32, 3u32], device)?;
        
        // Simulate embedding lookup
        let embeddings = embedding_weights.index_select(&token_ids, 0)?;
        info!("✅ Embedding lookup successful: {:?}", embeddings.shape());
        
        Ok(())
    }
    
    pub async fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        debug!("🔄 Generating mock embedding for: '{}'", text);
        
        // For proof of concept, generate a deterministic "embedding" based on text
        // In real implementation, this would use the actual model
        let hash = self.simple_hash(text);
        let embedding = self.generate_mock_embedding(hash)?;
        
        info!("✅ Generated mock embedding: {} dimensions", embedding.len());
        
        Ok(embedding)
    }
    
    fn simple_hash(&self, text: &str) -> u64 {
        // Simple hash function for deterministic mock embeddings
        let mut hash = 0u64;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }
    
    fn generate_mock_embedding(&self, seed: u64) -> Result<Vec<f32>> {
        // Generate a deterministic 768-dimensional vector using Candle
        let embedding_dim = 768;
        
        // Create deterministic data based on seed
        let seed_f32 = (seed % 1000) as f32 / 1000.0;
        let data: Vec<f32> = (0..embedding_dim)
            .map(|i| ((i as f32 + seed_f32) * 0.1).sin())
            .collect();
        
        // Use Candle to create and process the tensor
        let tensor = Tensor::new(&data[..], &self.device)?;
        let embedding = tensor.to_vec1::<f32>()?;
        
        // Normalize to unit length (like real embeddings)
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = embedding.iter().map(|x| x / norm).collect();
        
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_embedding_service() -> Result<()> {
        let mut service = EmbeddingService::new().await?;
        
        // Test that we get consistent embeddings for same text
        let embedding1 = service.embed("test text").await?;
        let embedding2 = service.embed("test text").await?;
        assert_eq!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 768);
        
        // Test that different text gives different embeddings
        let embedding3 = service.embed("different text").await?;
        assert_ne!(embedding1, embedding3);
        
        // Test that embeddings are normalized
        let norm: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
        
        Ok(())
    }
}