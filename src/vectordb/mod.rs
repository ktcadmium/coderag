//! Vector database module for storing and searching document embeddings
//!
//! This module provides a file-based vector database implementation optimized
//! for documentation retrieval in the CodeRAG system.

#![allow(dead_code)]

mod search;
mod storage;
mod types;

pub use search::{SearchOptions, SearchResult};
pub use storage::VectorStorage;
pub use types::{ContentType, Document, DocumentMetadata};

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Main vector database interface combining storage and search capabilities
pub struct VectorDatabase {
    storage: VectorStorage,
}

impl VectorDatabase {
    /// Create a new vector database instance
    pub fn new<P: AsRef<Path>>(data_path: P) -> Result<Self> {
        let storage = VectorStorage::new(data_path)?;

        Ok(Self { storage })
    }

    /// Load the database from persistent storage
    pub fn load(&mut self) -> Result<()> {
        self.storage.load()
    }

    /// Add a document with its embedding to the database
    pub fn add_document(&mut self, doc: Document, embedding: Vec<f32>) -> Result<String> {
        self.storage.add_document(doc, embedding)
    }

    /// Search for similar documents
    pub fn search(
        &self,
        query_embedding: &[f32],
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        search::search_documents(&self.storage, query_embedding, options)
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
        self.storage.clear()
    }
}
