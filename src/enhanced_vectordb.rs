//! Module for enhanced vector database functionality.
//!
//! This module provides improvements to the base vector database implementation
//! including HNSW indexing, vector quantization, hybrid search, and enhanced
//! document chunking.

use crate::embedding_basic::EmbeddingService;
use crate::vectordb::{
    ChunkingStrategy, Document, DocumentMetadata, EnhancedChunker, HnswParams, HybridSearchOptions,
    QuantizationMethod, VectorDatabase,
};
use anyhow::Result;
use std::path::Path;
use tracing::{debug, info};

/// Enhanced vector database service with optimized search
pub struct EnhancedVectorDbService {
    /// Vector database with enhanced capabilities
    db: VectorDatabase,
    /// Chunking strategy for document processing
    chunker: EnhancedChunker,
    /// Database file path
    _db_path: std::path::PathBuf,
}

impl EnhancedVectorDbService {
    /// Create a new enhanced vector database service
    pub async fn new<P: AsRef<Path>>(data_dir: P, embedding_dimension: usize) -> Result<Self> {
        let db_path = data_dir.as_ref().join("enhanced_vectordb.json");

        info!("Creating enhanced vector database with HNSW indexing...");

        // Create database with HNSW indexing
        let db =
            VectorDatabase::with_hnsw(db_path.clone(), embedding_dimension, HnswParams::default())?;

        // Create enhanced chunker with heading-based strategy
        let chunker = EnhancedChunker::new(ChunkingStrategy::HeadingBased {
            max_size: 1500,
            min_size: 200,
        });

        Ok(Self {
            db,
            chunker,
            _db_path: db_path,
        })
    }

    /// Create a new enhanced vector database service with quantization
    pub async fn with_quantization<P: AsRef<Path>>(
        data_dir: P,
        embedding_dimension: usize,
    ) -> Result<Self> {
        let db_path = data_dir.as_ref().join("enhanced_quantized_vectordb.json");

        info!("Creating enhanced vector database with quantization...");

        // Create database with vector quantization
        let db = VectorDatabase::with_quantization(
            db_path.clone(),
            embedding_dimension,
            QuantizationMethod::Scalar8Bit,
        )?;

        // Create enhanced chunker with heading-based strategy
        let chunker = EnhancedChunker::new(ChunkingStrategy::HeadingBased {
            max_size: 1500,
            min_size: 200,
        });

        Ok(Self {
            db,
            chunker,
            _db_path: db_path,
        })
    }

    /// Load the database from disk
    pub async fn load(&mut self) -> Result<()> {
        info!("Loading enhanced vector database...");
        self.db.load()?;
        info!("Loaded {} documents", self.db.document_count());
        Ok(())
    }

    /// Save the database to disk
    pub async fn save(&self) -> Result<()> {
        info!("Saving enhanced vector database...");
        self.db.save()?;
        info!("Saved {} documents", self.db.document_count());
        Ok(())
    }

    /// Process a document and add it to the database
    pub async fn add_document(
        &mut self,
        embedding_service: &EmbeddingService,
        content: &str,
        url: &str,
        title: Option<&str>,
        content_type: crate::vectordb::ContentType,
    ) -> Result<Vec<String>> {
        // Chunk the content using enhanced chunker
        debug!("Chunking document: {}", url);
        let chunks = self.chunker.chunk_text(content);
        let total_chunks = chunks.len();
        debug!("Created {} chunks", total_chunks);

        let mut document_ids = Vec::with_capacity(total_chunks);

        // Process each chunk
        for (i, chunk) in chunks.into_iter().enumerate() {
            // Generate embedding
            debug!(
                "Generating embedding for chunk {} (size: {} bytes)",
                i + 1,
                chunk.content.len()
            );
            let embedding = embedding_service.embed(&chunk.content).await?;

            // Create document
            let doc_id = format!("{}_{}", url, i);
            let document = Document {
                id: doc_id.clone(),
                content: chunk.content,
                url: url.to_string(),
                title: title.map(|t| t.to_string()),
                section: chunk.heading_context,
                metadata: DocumentMetadata {
                    content_type,
                    language: None, // Could be detected
                    last_updated: Some(std::time::SystemTime::now()),
                    tags: vec![
                        if chunk.has_code {
                            "has-code"
                        } else {
                            "no-code"
                        }
                        .to_string(),
                        format!("chunk-{}-of-{}", i + 1, total_chunks),
                    ],
                },
            };

            // Add to database
            self.db.add_document(document, embedding)?;
            document_ids.push(doc_id);
        }

        Ok(document_ids)
    }

    /// Search for similar documents using hybrid search
    pub async fn search(
        &self,
        embedding_service: &EmbeddingService,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Document>> {
        // Generate embedding for query
        debug!("Generating embedding for query: {}", query);
        let query_embedding = embedding_service.embed(query).await?;

        // Prepare hybrid search options
        let options = HybridSearchOptions {
            base: crate::vectordb::SearchOptions {
                limit,
                min_score: Some(0.1), // Minimum similarity threshold
                source_filter: None,
                content_type_filter: None,
            },
            enable_hybrid: true,
            vector_weight: 0.7,
            keyword_weight: 0.3,
            keyword_params: crate::vectordb::KeywordSearchParams::default(),
        };

        // Perform hybrid search
        debug!("Performing hybrid search...");
        let results = self.db.hybrid_search(&query_embedding, query, options)?;

        // Convert to documents
        let documents = results.into_iter().map(|r| r.document).collect::<Vec<_>>();

        debug!("Found {} matching documents", documents.len());
        Ok(documents)
    }

    /// Get number of documents in the database
    pub fn document_count(&self) -> usize {
        self.db.document_count()
    }

    /// Get documents grouped by source
    pub fn get_documents_by_source(&self) -> HashMap<String, Vec<&Document>> {
        self.db.get_documents_by_source()
    }

    /// Clear all documents
    pub async fn clear(&mut self) -> Result<()> {
        info!("Clearing enhanced vector database...");
        self.db.clear()?;
        info!("Database cleared");
        Ok(())
    }

    /// Remove documents from a specific source
    pub async fn remove_documents_by_source(&mut self, source: &str) -> Result<usize> {
        info!("Removing documents from source: {}", source);
        let count = self.db.remove_documents_by_source(source)?;
        info!("Removed {} documents", count);
        Ok(count)
    }

    /// Remove documents older than specified age in days
    pub async fn remove_documents_by_age(&mut self, max_age_days: u64) -> Result<usize> {
        info!("Removing documents older than {} days", max_age_days);
        let count = self.db.remove_documents_by_age(max_age_days)?;
        info!("Removed {} documents", count);
        Ok(count)
    }

    /// Get HNSW index statistics
    pub fn index_stats(&self) -> Option<String> {
        self.db.index_stats().map(|stats| {
            format!(
                "HNSW Index Stats:\n- Documents: {}\n- Max Level: {}\n- Dimension: {}",
                stats.node_count, stats.max_level, stats.dimension
            )
        })
    }

    /// Change chunking strategy
    pub fn set_chunking_strategy(&mut self, strategy: ChunkingStrategy) {
        self.chunker.set_strategy(strategy);
    }
}

use std::collections::HashMap;
