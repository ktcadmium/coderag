use anyhow::Result;
use tracing::{info, error};
use std::path::PathBuf;

mod embedding;
mod embedding_basic;
mod vectordb;

use crate::vectordb::{VectorDatabase, Document, DocumentMetadata, ContentType, SearchOptions};

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
                
                // Calculate cosine similarity
                let dot_product: f32 = emb1.iter().zip(emb2.iter()).map(|(a, b)| a * b).sum();
                let norm1: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm2: f32 = emb2.iter().map(|x| x * x).sum::<f32>().sqrt();
                let similarity = if norm1 > 0.0 && norm2 > 0.0 {
                    dot_product / (norm1 * norm2)
                } else {
                    0.0
                };
                info!("📊 '{}' ↔ '{}': {:.3}", text1, text2, similarity);
            }
            
            info!(""); // Separator
            
            // Test the vector database
            info!(""); // Separator
            info!("🗄️  Testing vector database functionality...");
            
            // Initialize vector database
            let db_path = PathBuf::from("./test_vectordb.json");
            let mut db = VectorDatabase::new(db_path.clone())?;
            
            // Create sample documents
            let sample_docs = vec![
                Document {
                    id: "doc1".to_string(),
                    content: "Tokio is an asynchronous runtime for Rust that provides async I/O, timers, and other async primitives.".to_string(),
                    url: "https://docs.rs/tokio".to_string(),
                    title: Some("Tokio Documentation".to_string()),
                    section: Some("Introduction".to_string()),
                    metadata: DocumentMetadata {
                        content_type: ContentType::Documentation,
                        language: Some("en".to_string()),
                        last_updated: None,
                        tags: vec!["async".to_string(), "runtime".to_string(), "tokio".to_string()],
                    },
                },
                Document {
                    id: "doc2".to_string(),
                    content: "Error handling in Rust uses the Result type which can be Ok(T) for success or Err(E) for errors.".to_string(),
                    url: "https://doc.rust-lang.org/book/error-handling".to_string(),
                    title: Some("The Rust Book - Error Handling".to_string()),
                    section: Some("Result Type".to_string()),
                    metadata: DocumentMetadata {
                        content_type: ContentType::Tutorial,
                        language: Some("en".to_string()),
                        last_updated: None,
                        tags: vec!["error-handling".to_string(), "result".to_string()],
                    },
                },
                Document {
                    id: "doc3".to_string(),
                    content: "FastEmbed-rs provides high-performance embedding generation using ONNX Runtime for Rust applications.".to_string(),
                    url: "https://github.com/anth-vk/fastembed-rs".to_string(),
                    title: Some("FastEmbed Rust Documentation".to_string()),
                    section: Some("Overview".to_string()),
                    metadata: DocumentMetadata {
                        content_type: ContentType::Documentation,
                        language: Some("en".to_string()),
                        last_updated: None,
                        tags: vec!["embeddings".to_string(), "ml".to_string(), "onnx".to_string()],
                    },
                },
            ];
            
            // Add documents to the database
            for doc in sample_docs {
                info!("📄 Adding document: {}", doc.title.as_ref().unwrap_or(&doc.id));
                let embedding = service.embed(&doc.content).await?;
                db.add_document(doc, embedding)?;
            }
            
            // Save to disk
            db.save()?;
            info!("💾 Saved {} documents to database", db.document_count());
            
            // Test search functionality
            info!(""); // Separator
            info!("🔍 Testing semantic search...");
            
            let queries = vec![
                "How do I handle errors in async Rust code?",
                "What is Tokio used for?",
                "How to generate embeddings in Rust?",
            ];
            
            for query in queries {
                info!(""); // Separator for each query
                info!("Query: '{}'", query);
                
                let query_embedding = service.embed(query).await?;
                let results = db.search(
                    &query_embedding,
                    SearchOptions {
                        limit: 2,
                        min_score: Some(0.3),
                        source_filter: None,
                        content_type_filter: None,
                    }
                )?;
                
                for (i, result) in results.iter().enumerate() {
                    info!("  {}. [Score: {:.3}] {}", 
                          i + 1, 
                          result.score, 
                          result.document.title.as_ref().unwrap_or(&result.document.id));
                    info!("     URL: {}", result.document.url);
                    info!("     Preview: {}...", 
                          result.document.content.chars().take(80).collect::<String>());
                }
            }
            
            // Clean up test file
            std::fs::remove_file(db_path).ok();
        }
        Err(e) => {
            error!("❌ Failed to initialize embedding service: {}", e);
            info!("💡 This might be due to network issues or missing ONNX runtime");
            return Err(e);
        }
    }
    
    Ok(())
}
