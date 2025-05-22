use anyhow::Result;
use tracing::{info, error};

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

mod embedding;
mod embedding_basic;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting CodeRAG - Documentation RAG for AI-Assisted Development");
    
    // Test text for programming RAG scenarios
    let test_texts = vec![
        "How do I create a vector database in Rust?".to_string(),
        "async function error handling with Result type".to_string(),
        "Candle tensor operations and GPU acceleration".to_string(),
        "MCP server implementation with stdio protocol".to_string(),
        "HTTP client configuration with reqwest".to_string(),
    ];
    
    match embedding_basic::EmbeddingService::new().await {
        Ok(service) => {
            info!("✅ Successfully initialized embedding service");
            
            // Test semantic similarity for programming concepts
            info!("🧪 Testing semantic similarity for programming concepts...");
            
            let similarity_tests = vec![
                ("async function error handling", "Result type error handling"),
                ("vector database", "embedding storage"),
                ("HTTP client", "reqwest configuration"),
                ("Rust programming", "Rust development"),
                ("MCP server", "stdio protocol server"),
            ];
            
            for (text1, text2) in similarity_tests {
                let emb1 = service.embed(text1).await?;
                let emb2 = service.embed(text2).await?;
                
                let similarity = cosine_similarity(&emb1, &emb2);
                info!("📊 '{}' ↔ '{}': {:.3}", text1, text2, similarity);
            }
            
            info!(""); // Separator
            
            // Test the provided texts
            for text in test_texts {
                match service.embed(&text).await {
                    Ok(embedding) => {
                        info!("Generated embedding for '{}': {} dimensions", 
                              text, embedding.len());
                        // Print first few dimensions as a sanity check
                        let preview: Vec<f32> = embedding.iter().take(5).cloned().collect();
                        info!("First 5 dimensions: {:?}", preview);
                        
                        // Test normalization
                        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                        info!("Embedding norm: {:.6} (close to 1.0 = normalized)", norm);
                    }
                    Err(e) => {
                        error!("Failed to generate embedding for '{}': {}", text, e);
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to initialize embedding service: {}", e);
            info!("💡 This might be due to network issues or missing ONNX runtime");
            return Err(e);
        }
    }
    
    Ok(())
}
