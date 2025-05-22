//! Vector database module for storing and searching document embeddings
//! 
//! This module provides a file-based vector database implementation optimized
//! for documentation retrieval in the CodeRAG system.

mod search;
mod storage;
mod types;

pub use search::{SearchOptions, SearchResult};
pub use storage::VectorStorage;
pub use types::{Document, DocumentMetadata, ContentType, VectorEntry};

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Main vector database interface combining storage and search capabilities
pub struct VectorDatabase {
    storage: VectorStorage,
    data_path: PathBuf,
}

impl VectorDatabase {
    /// Create a new vector database instance
    pub fn new<P: AsRef<Path>>(data_path: P) -> Result<Self> {
        let data_path = data_path.as_ref().to_path_buf();
        let storage = VectorStorage::new(data_path.clone())?;
        
        Ok(Self {
            storage,
            data_path,
        })
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
    pub fn search(&self, query_embedding: &[f32], options: SearchOptions) -> Result<Vec<SearchResult>> {
        search::search_documents(&self.storage, query_embedding, options)
    }

    /// Get total number of documents
    pub fn document_count(&self) -> usize {
        self.storage.document_count()
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