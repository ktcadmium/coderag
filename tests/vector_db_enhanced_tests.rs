//! Integration tests for enhanced vector database

use crate::vectordb::{
    ChunkingStrategy, ContentType, Document, DocumentMetadata, EnhancedChunker, HnswParams,
    HybridSearchOptions, QuantizationMethod, VectorDatabase,
};
use anyhow::Result;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

/// Create a test document
fn create_test_document(id: &str, content: &str, url: &str) -> Document {
    Document {
        id: id.to_string(),
        content: content.to_string(),
        url: url.to_string(),
        title: Some(format!("Title {}", id)),
        section: None,
        metadata: DocumentMetadata {
            content_type: ContentType::Documentation,
            language: Some("en".to_string()),
            last_updated: Some(SystemTime::now()),
            tags: vec!["test".to_string()],
        },
    }
}

/// Test the enhanced vector database with HNSW indexing
#[tokio::test]
async fn test_vector_db_with_hnsw() -> Result<()> {
    // Create temp directory for test
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_hnsw_vectors.json");

    // Create database with HNSW indexing
    let mut db = VectorDatabase::with_hnsw(db_path.clone(), 3, HnswParams::default())?;

    // Add documents
    let docs = vec![
        (
            "1",
            "Rust is a systems programming language focused on safety and performance",
            "https://example.com/rust",
            vec![1.0, 0.1, 0.1],
        ),
        (
            "2",
            "Python is a high-level programming language known for its readability",
            "https://example.com/python",
            vec![0.1, 1.0, 0.1],
        ),
        (
            "3",
            "JavaScript is a web programming language used for frontend development",
            "https://example.com/js",
            vec![0.1, 0.1, 1.0],
        ),
    ];

    for (id, content, url, vector) in docs {
        let doc = create_test_document(id, content, url);
        db.add_document(doc, vector)?;
    }

    // Save and reload
    db.save()?;
    db.load()?;

    // Test search
    let query = vec![0.9, 0.1, 0.1]; // Similar to document 1 (Rust)
    let options = crate::vectordb::SearchOptions {
        limit: 2,
        min_score: None,
        source_filter: None,
        content_type_filter: None,
    };

    let results = db.search(&query, options)?;

    // Should find document 1 as the top result
    assert!(!results.is_empty());
    assert_eq!(results[0].document.id, "1");

    // Check index stats
    let stats = db.index_stats().unwrap();
    assert_eq!(stats.node_count, 3);

    Ok(())
}

/// Test the enhanced vector database with quantization
#[tokio::test]
async fn test_vector_db_with_quantization() -> Result<()> {
    // Create temp directory for test
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_quantized_vectors.json");

    // Create database with quantization
    let mut db =
        VectorDatabase::with_quantization(db_path.clone(), 3, QuantizationMethod::Scalar8Bit)?;

    // Add documents
    let docs = vec![
        (
            "1",
            "Rust is a systems programming language focused on safety and performance",
            "https://example.com/rust",
            vec![1.0, 0.1, 0.1],
        ),
        (
            "2",
            "Python is a high-level programming language known for its readability",
            "https://example.com/python",
            vec![0.1, 1.0, 0.1],
        ),
        (
            "3",
            "JavaScript is a web programming language used for frontend development",
            "https://example.com/js",
            vec![0.1, 0.1, 1.0],
        ),
    ];

    for (id, content, url, vector) in docs {
        let doc = create_test_document(id, content, url);
        db.add_document(doc, vector)?;
    }

    // Save and reload
    db.save()?;
    db.load()?;

    // Test search
    let query = vec![0.9, 0.1, 0.1]; // Similar to document 1 (Rust)
    let options = crate::vectordb::SearchOptions {
        limit: 2,
        min_score: None,
        source_filter: None,
        content_type_filter: None,
    };

    let results = db.search(&query, options)?;

    // Should find document 1 as the top result
    assert!(!results.is_empty());
    assert_eq!(results[0].document.id, "1");

    // Check quantizer params
    let params = db.quantizer_params().unwrap();
    assert_eq!(params["method"], "scalar_8bit");

    Ok(())
}

/// Test the enhanced vector database with hybrid search
#[tokio::test]
async fn test_vector_db_with_hybrid_search() -> Result<()> {
    // Create temp directory for test
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test_hybrid_vectors.json");

    // Create database
    let mut db = VectorDatabase::new(db_path.clone())?;

    // Add documents
    let docs = vec![
        (
            "1",
            "Rust is a systems programming language focused on safety and performance",
            "https://example.com/rust",
            vec![1.0, 0.1, 0.1],
        ),
        (
            "2",
            "Python is a high-level programming language known for its readability",
            "https://example.com/python",
            vec![0.1, 1.0, 0.1],
        ),
        (
            "3",
            "JavaScript is a web programming language used for frontend development",
            "https://example.com/js",
            vec![0.1, 0.1, 1.0],
        ),
        (
            "4",
            "Rust and C++ are both systems programming languages with different approaches to memory safety",
            "https://example.com/compare",
            vec![0.8, 0.0, 0.2],
        ),
    ];

    for (id, content, url, vector) in docs {
        let doc = create_test_document(id, content, url);
        db.add_document(doc, vector)?;
    }

    // Save and reload
    db.save()?;
    db.load()?;

    // Test hybrid search
    let query_embedding = vec![0.9, 0.2, 0.1]; // Similar to document 1 (Rust)
    let query_text = "memory safety programming"; // Keywords match doc 4 (Rust and C++)

    let options = HybridSearchOptions {
        base: crate::vectordb::SearchOptions {
            limit: 2,
            min_score: None,
            source_filter: None,
            content_type_filter: None,
        },
        enable_hybrid: true,
        vector_weight: 0.6,
        keyword_weight: 0.4,
        keyword_params: crate::vectordb::hybrid_search::KeywordSearchParams::default(),
    };

    let results = db.hybrid_search(&query_embedding, query_text, options)?;

    // Should find both doc 1 and doc 4
    assert_eq!(results.len(), 2);

    // Doc 4 should rank higher due to keyword match on "memory safety"
    assert_eq!(results[0].document.id, "4");

    Ok(())
}

/// Test the enhanced chunking functionality
#[test]
fn test_enhanced_chunking() -> Result<()> {
    let text = "# Heading 1\n\nThis is paragraph 1.\n\n## Heading 2\n\nThis is paragraph 2.\n\nThis is paragraph 3.\n\n# Heading 3\n\nFinal paragraph.";

    // Test different chunking strategies
    let mut chunker = EnhancedChunker::new(ChunkingStrategy::HeadingBased {
        max_size: 200,
        min_size: 10,
    });

    let chunks = chunker.chunk_text(text);

    // Should create at least 2 chunks based on headings
    assert!(chunks.len() >= 2);

    // Check that heading is captured
    assert_eq!(chunks[0].heading, Some("Heading 1".to_string()));

    // Try semantic boundaries
    let mut chunker = EnhancedChunker::new(ChunkingStrategy::SemanticBoundaries {
        max_size: 50,
        min_size: 10,
    });

    let chunks = chunker.chunk_text(text);

    // Should create multiple chunks
    assert!(chunks.len() > 2);

    Ok(())
}

/// Performance test for HNSW indexing vs linear search
#[tokio::test]
#[ignore] // This test is slow, so only run when needed
async fn test_hnsw_performance() -> Result<()> {
    use rand::Rng;
    use std::time::Instant;

    // Create temp directory for test
    let temp_dir = TempDir::new()?;

    // Create databases
    let db_path_hnsw = temp_dir.path().join("test_hnsw_perf.json");
    let db_path_linear = temp_dir.path().join("test_linear_perf.json");

    let mut db_hnsw = VectorDatabase::with_hnsw(db_path_hnsw, 64, HnswParams::default())?;
    let mut db_linear = VectorDatabase::new(db_path_linear)?;

    // Create a lot of random documents
    let mut rng = rand::thread_rng();
    let num_docs = 1000;

    for i in 0..num_docs {
        // Create random vector
        let mut vector = Vec::with_capacity(64);
        for _ in 0..64 {
            vector.push(rng.gen::<f32>());
        }

        // Create document
        let doc = create_test_document(
            &i.to_string(),
            &format!("Document {}", i),
            &format!("https://example.com/doc{}", i),
        );

        // Add to both databases
        db_hnsw.add_document(doc.clone(), vector.clone())?;
        db_linear.add_document(doc, vector)?;
    }

    // Save and reload
    db_hnsw.save()?;
    db_linear.save()?;
    db_hnsw.load()?;
    db_linear.load()?;

    // Create random query vector
    let mut query = Vec::with_capacity(64);
    for _ in 0..64 {
        query.push(rng.gen::<f32>());
    }

    // Test HNSW search performance
    let options = crate::vectordb::SearchOptions {
        limit: 10,
        min_score: None,
        source_filter: None,
        content_type_filter: None,
    };

    let start = Instant::now();
    let results_hnsw = db_hnsw.search(&query, options.clone())?;
    let hnsw_time = start.elapsed();

    let start = Instant::now();
    let results_linear = db_linear.search(&query, options)?;
    let linear_time = start.elapsed();

    println!("HNSW search time: {:?}", hnsw_time);
    println!("Linear search time: {:?}", linear_time);
    println!(
        "Speedup: {:.2}x",
        linear_time.as_micros() as f64 / hnsw_time.as_micros() as f64
    );

    // Should be significantly faster
    assert!(hnsw_time < linear_time);

    // Results might be different but should have the same length
    assert_eq!(results_hnsw.len(), results_linear.len());

    Ok(())
}
