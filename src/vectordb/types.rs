//! Common types and structures for the vector database

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Unique identifier for vectors/documents
pub type VectorId = String;

/// A vector embedding (384 dimensions for all-MiniLM-L6-v2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub values: Vec<f32>,
}

impl Vector {
    /// Create a new vector from values
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    /// Get the dimension of the vector
    pub fn dimension(&self) -> usize {
        self.values.len()
    }

    /// Calculate cosine similarity with another vector
    pub fn cosine_similarity(&self, other: &Vector) -> f32 {
        if self.dimension() != other.dimension() {
            return 0.0;
        }

        let dot_product: f32 = self.values
            .iter()
            .zip(&other.values)
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.values.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    /// Normalize the vector to unit length
    pub fn normalize(&mut self) {
        let norm: f32 = self.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut self.values {
                *value /= norm;
            }
        }
    }
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub content_type: ContentType,
    pub language: Option<String>,
    pub last_updated: Option<SystemTime>,
    pub tags: Vec<String>,
}

/// Type of content in the document
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    Documentation,
    CodeExample,
    Tutorial,
    Reference,
    BlogPost,
    Other,
}

/// A document with its content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub url: String,
    pub title: Option<String>,
    pub section: Option<String>,
    pub metadata: DocumentMetadata,
}

impl Document {
    /// Get a preview of the content (first 200 chars)
    pub fn preview(&self) -> &str {
        let end = self.content.len().min(200);
        &self.content[..end]
    }
}

/// Stored vector entry combining document and embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: VectorId,
    pub document: Document,
    pub vector: Vector,
    pub indexed_at: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
        let v2 = Vector::new(vec![1.0, 0.0, 0.0]);
        let v3 = Vector::new(vec![0.0, 1.0, 0.0]);
        
        assert!((v1.cosine_similarity(&v2) - 1.0).abs() < 0.0001);
        assert!((v1.cosine_similarity(&v3) - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_vector_normalization() {
        let mut v = Vector::new(vec![3.0, 4.0, 0.0]);
        v.normalize();
        
        let magnitude: f32 = v.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.0001);
    }
}