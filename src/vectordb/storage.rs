//! File-based persistence for vector database

use crate::vectordb::types::{Document, Vector, VectorEntry};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info};

/// Storage format version for compatibility
const STORAGE_VERSION: u32 = 1;

/// Storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageMetadata {
    version: u32,
    created_at: SystemTime,
    last_modified: SystemTime,
    document_count: usize,
}

/// Main storage structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageData {
    metadata: StorageMetadata,
    entries: Vec<VectorEntry>,
}

/// File-based vector storage implementation
pub struct VectorStorage {
    data_path: PathBuf,
    data: StorageData,
    modified: bool,
}

impl VectorStorage {
    /// Create a new storage instance
    pub fn new<P: AsRef<Path>>(data_path: P) -> Result<Self> {
        let data_path = data_path.as_ref().to_path_buf();

        // Create parent directory if needed
        if let Some(parent) = data_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = StorageData {
            metadata: StorageMetadata {
                version: STORAGE_VERSION,
                created_at: SystemTime::now(),
                last_modified: SystemTime::now(),
                document_count: 0,
            },
            entries: Vec::new(),
        };

        Ok(Self {
            data_path,
            data,
            modified: false,
        })
    }

    /// Load data from persistent storage
    pub fn load(&mut self) -> Result<()> {
        if self.data_path.exists() {
            info!("Loading vectors from {:?}", self.data_path);

            let contents =
                fs::read_to_string(&self.data_path).context("Failed to read storage file")?;

            self.data =
                serde_json::from_str(&contents).context("Failed to deserialize storage data")?;

            // Check version compatibility
            if self.data.metadata.version != STORAGE_VERSION {
                anyhow::bail!(
                    "Storage version mismatch: expected {}, found {}",
                    STORAGE_VERSION,
                    self.data.metadata.version
                );
            }

            debug!("Loaded {} documents", self.data.entries.len());
        }

        Ok(())
    }

    /// Save data to persistent storage
    pub fn save(&self) -> Result<()> {
        // Update metadata
        let mut data = self.data.clone();
        data.metadata.last_modified = SystemTime::now();
        data.metadata.document_count = data.entries.len();

        // Write to temporary file first
        let temp_path = self.data_path.with_extension("tmp");
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(&temp_path, json)?;

        // Atomic rename
        fs::rename(&temp_path, &self.data_path)?;

        info!(
            "Saved {} documents to {:?}",
            data.entries.len(),
            self.data_path
        );
        Ok(())
    }

    /// Add a new document with its embedding
    pub fn add_document(&mut self, document: Document, embedding: Vec<f32>) -> Result<String> {
        let id = document.id.clone();

        let entry = VectorEntry {
            id: id.clone(),
            document,
            vector: Vector::new(embedding),
            indexed_at: SystemTime::now(),
        };

        self.data.entries.push(entry);
        self.modified = true;

        Ok(id)
    }

    /// Get all vector entries
    pub fn get_all_entries(&self) -> &[VectorEntry] {
        &self.data.entries
    }

    /// Get a specific document by ID
    pub fn get_document(&self, id: &str) -> Option<&Document> {
        self.data
            .entries
            .iter()
            .find(|e| e.id == id)
            .map(|e| &e.document)
    }

    /// Remove a document by ID
    pub fn remove_document(&mut self, id: &str) -> Result<bool> {
        let original_len = self.data.entries.len();
        self.data.entries.retain(|e| e.id != id);

        if self.data.entries.len() < original_len {
            self.modified = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Remove all documents from a specific source URL
    pub fn remove_documents_by_source(&mut self, source_url: &str) -> Result<usize> {
        let original_len = self.data.entries.len();
        self.data.entries.retain(|e| e.document.url != source_url);

        let removed_count = original_len - self.data.entries.len();
        if removed_count > 0 {
            self.modified = true;
        }

        Ok(removed_count)
    }

    /// Remove documents older than specified age in days
    pub fn remove_documents_by_age(&mut self, max_age_days: u64) -> Result<usize> {
        use std::time::Duration;

        let cutoff_time = SystemTime::now()
            .checked_sub(Duration::from_secs(max_age_days * 24 * 60 * 60))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let original_len = self.data.entries.len();
        self.data
            .entries
            .retain(|e| e.document.metadata.last_updated.unwrap_or(e.indexed_at) > cutoff_time);

        let removed_count = original_len - self.data.entries.len();
        if removed_count > 0 {
            self.modified = true;
        }

        Ok(removed_count)
    }

    /// Get total number of documents
    pub fn document_count(&self) -> usize {
        self.data.entries.len()
    }

    /// Get all entries
    pub fn get_entries(&self) -> &[VectorEntry] {
        &self.data.entries
    }

    /// Clear all documents
    pub fn clear(&mut self) -> Result<()> {
        self.data.entries.clear();
        self.modified = true;
        Ok(())
    }

    /// Check if data has been modified since last save
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().join("test_vectors.json");

        let mut storage = VectorStorage::new(&storage_path)?;

        // Test adding documents
        let doc = Document {
            id: "test1".to_string(),
            content: "Test content".to_string(),
            url: "https://example.com".to_string(),
            title: Some("Test".to_string()),
            section: None,
            metadata: crate::vectordb::types::DocumentMetadata {
                content_type: crate::vectordb::types::ContentType::Documentation,
                language: Some("en".to_string()),
                last_updated: None,
                tags: vec!["test".to_string()],
            },
        };

        let embedding = vec![0.1, 0.2, 0.3];
        storage.add_document(doc, embedding)?;

        assert_eq!(storage.document_count(), 1);

        // Test save and load
        storage.save()?;

        let mut storage2 = VectorStorage::new(&storage_path)?;
        storage2.load()?;

        assert_eq!(storage2.document_count(), 1);

        Ok(())
    }
}
