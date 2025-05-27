//! Vector database module for storing and searching document embeddings
//!
//! This module provides a file-based vector database implementation optimized
//! for documentation retrieval in the CodeRAG system.

#![allow(dead_code)]
#![allow(unused_imports)]

mod chunking;
mod hybrid_search;
mod indexing;
mod quantization;
mod search;
mod storage;
mod types;

pub use chunking::{Chunk, ChunkingStrategy, EnhancedChunker};
pub use hybrid_search::{
    hybrid_search, BM25Index, HybridSearchOptions, HybridSearchResult, KeywordSearchParams,
};
pub use indexing::{HnswIndex, HnswParams, HnswStats};
pub use quantization::{QuantizationMethod, VectorQuantizer};
pub use search::{cosine_similarity, SearchOptions, SearchResult};
pub use storage::VectorStorage;
pub use types::{ContentType, Document, DocumentMetadata};

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Main vector database interface combining storage and search capabilities
pub struct VectorDatabase {
    storage: VectorStorage,
    index: Option<HnswIndex>,
    quantizer: Option<VectorQuantizer>,
}

impl VectorDatabase {
    /// Create a new vector database instance
    pub fn new<P: AsRef<Path>>(data_path: P) -> Result<Self> {
        let storage = VectorStorage::new(data_path)?;

        Ok(Self {
            storage,
            index: None,
            quantizer: None,
        })
    }

    /// Create a new vector database instance with HNSW indexing
    pub fn with_hnsw<P: AsRef<Path>>(
        data_path: P,
        dimension: usize,
        params: HnswParams,
    ) -> Result<Self> {
        let storage = VectorStorage::new(data_path)?;
        let index = Some(HnswIndex::new(dimension, params));

        Ok(Self {
            storage,
            index,
            quantizer: None,
        })
    }

    /// Create a new vector database instance with vector quantization
    pub fn with_quantization<P: AsRef<Path>>(
        data_path: P,
        dimension: usize,
        method: QuantizationMethod,
    ) -> Result<Self> {
        let storage = VectorStorage::new(data_path)?;
        let quantizer = Some(VectorQuantizer::new(method, dimension));

        Ok(Self {
            storage,
            index: None,
            quantizer,
        })
    }

    /// Load the database from persistent storage
    pub fn load(&mut self) -> Result<()> {
        // Load storage first
        self.storage.load()?;

        // Initialize HNSW index if enabled
        if let Some(index) = &mut self.index {
            // If index is empty, build it from storage
            if index.is_empty() {
                let entries = self.storage.get_all_entries();
                for entry in entries {
                    index.add(entry.id.clone(), entry.vector.clone())?;
                }
            }
        }

        // Initialize quantizer if enabled
        if let Some(quantizer) = &mut self.quantizer {
            // Initialize with all vectors
            let entries = self.storage.get_all_entries();
            let vectors: Vec<_> = entries.iter().map(|e| e.vector.clone()).collect();

            if !vectors.is_empty() {
                quantizer.initialize(&vectors)?;
            }
        }

        Ok(())
    }

    /// Add a document with its embedding to the database
    pub fn add_document(&mut self, doc: Document, embedding: Vec<f32>) -> Result<String> {
        // Add to storage
        let id = self.storage.add_document(doc, embedding.clone())?;

        // Add to HNSW index if enabled
        if let Some(index) = &mut self.index {
            let vector = types::Vector::new(embedding.clone());
            index.add(id.clone(), vector)?;
        }

        Ok(id)
    }

    /// Search for similar documents using the appropriate search method
    pub fn search(
        &self,
        query_embedding: &[f32],
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        // If HNSW index is enabled, use it for search
        if let Some(index) = &self.index {
            // Use HNSW search
            let results = index.search(query_embedding, options.limit)?;

            // Convert to SearchResult format
            let mut search_results = Vec::with_capacity(results.len());

            for (id, score) in results {
                if let Some(document) = self.storage.get_document(&id) {
                    // Apply filters
                    if let Some(ref source_filter) = options.source_filter {
                        if !document.url.contains(source_filter) {
                            continue;
                        }
                    }

                    if let Some(content_type_filter) = options.content_type_filter {
                        if document.metadata.content_type != content_type_filter {
                            continue;
                        }
                    }

                    if let Some(min_score) = options.min_score {
                        if score < min_score {
                            continue;
                        }
                    }

                    search_results.push(SearchResult {
                        document: document.clone(),
                        score,
                    });
                }
            }

            Ok(search_results)
        } else {
            // Fall back to standard search
            search::search_documents(&self.storage, query_embedding, options)
        }
    }

    /// Search for similar documents using hybrid search (vector + keyword)
    pub fn hybrid_search(
        &self,
        query_embedding: &[f32],
        query_text: &str,
        options: HybridSearchOptions,
    ) -> Result<Vec<HybridSearchResult>> {
        hybrid_search::hybrid_search(&self.storage, query_embedding, query_text, options)
    }

    /// Get total number of documents
    pub fn document_count(&self) -> usize {
        self.storage.document_count()
    }

    /// Get all documents grouped by source URL
    pub fn get_documents_by_source(&self) -> HashMap<String, Vec<&Document>> {
        use std::collections::HashMap;

        let mut source_map: HashMap<String, Vec<&Document>> = HashMap::new();

        for entry in self.storage.get_entries() {
            let source = if entry.document.url.is_empty() {
                "local".to_string()
            } else {
                entry.document.url.clone()
            };
            source_map.entry(source).or_default().push(&entry.document);
        }

        source_map
    }

    /// Save the database to disk
    pub fn save(&self) -> Result<()> {
        self.storage.save()
    }

    /// Clear all documents from the database
    pub fn clear(&mut self) -> Result<()> {
        self.storage.clear()?;

        // Clear HNSW index if enabled
        if let Some(index) = &mut self.index {
            *index = HnswIndex::new(index.stats().dimension, HnswParams::default());
        }

        // Clear quantizer cache if enabled
        if let Some(quantizer) = &mut self.quantizer {
            quantizer.clear_cache();
        }

        Ok(())
    }

    /// Remove documents from a specific source URL
    pub fn remove_documents_by_source(&mut self, source_url: &str) -> Result<usize> {
        // Get IDs to remove
        let _ids_to_remove: Vec<String> = self
            .storage
            .get_entries()
            .iter()
            .filter(|e| e.document.url == source_url)
            .map(|e| e.id.clone())
            .collect();

        // Remove from storage
        let removed_count = self.storage.remove_documents_by_source(source_url)?;

        // Remove from HNSW index if enabled
        if let Some(index) = &mut self.index {
            // Rebuild index (simple approach - could be optimized)
            *index = HnswIndex::new(index.stats().dimension, HnswParams::default());

            // Rebuild from remaining entries
            let entries = self.storage.get_all_entries();
            for entry in entries {
                index.add(entry.id.clone(), entry.vector.clone())?;
            }
        }

        Ok(removed_count)
    }

    /// Remove documents older than specified age in days
    pub fn remove_documents_by_age(&mut self, max_age_days: u64) -> Result<usize> {
        // Get IDs to remove (need to do this before removal)
        let now = std::time::SystemTime::now();
        let cutoff = now
            .checked_sub(std::time::Duration::from_secs(max_age_days * 24 * 60 * 60))
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        let _ids_to_remove: Vec<String> = self
            .storage
            .get_entries()
            .iter()
            .filter(|e| e.document.metadata.last_updated.unwrap_or(e.indexed_at) <= cutoff)
            .map(|e| e.id.clone())
            .collect();

        // Remove from storage
        let removed_count = self.storage.remove_documents_by_age(max_age_days)?;

        // Remove from HNSW index if enabled
        if let Some(index) = &mut self.index {
            // Rebuild index (simple approach - could be optimized)
            *index = HnswIndex::new(index.stats().dimension, HnswParams::default());

            // Rebuild from remaining entries
            let entries = self.storage.get_all_entries();
            for entry in entries {
                index.add(entry.id.clone(), entry.vector.clone())?;
            }
        }

        Ok(removed_count)
    }

    /// Get HNSW index statistics if available
    pub fn index_stats(&self) -> Option<HnswStats> {
        self.index.as_ref().map(|idx| idx.stats())
    }

    /// Get quantizer parameters if available
    pub fn quantizer_params(&self) -> Option<serde_json::Value> {
        self.quantizer.as_ref().map(|q| q.parameters_json())
    }
}
