//! Similarity search implementation for vector database

use crate::vectordb::storage::VectorStorage;
use crate::vectordb::types::{ContentType, Document};
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Search options for filtering and limiting results
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub limit: usize,
    /// Minimum similarity score (0.0 to 1.0)
    pub min_score: Option<f32>,
    /// Filter by source URL pattern
    pub source_filter: Option<String>,
    /// Filter by content type
    pub content_type_filter: Option<ContentType>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            min_score: None,
            source_filter: None,
            content_type_filter: None,
        }
    }
}

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
}

// For max heap ordering by score
impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for max heap, handle NaN cases
        other
            .score
            .partial_cmp(&self.score)
            .unwrap_or(Ordering::Equal)
    }
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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

/// Search documents in the storage by similarity to query embedding
pub fn search_documents(
    storage: &VectorStorage,
    query_embedding: &[f32],
    options: SearchOptions,
) -> Result<Vec<SearchResult>> {
    let mut heap = BinaryHeap::new();

    // Search through all entries
    for entry in storage.get_all_entries() {
        // Apply filters
        if let Some(ref source_filter) = options.source_filter {
            if !entry.document.url.contains(source_filter) {
                continue;
            }
        }

        if let Some(content_type_filter) = options.content_type_filter {
            if entry.document.metadata.content_type != content_type_filter {
                continue;
            }
        }

        // Calculate similarity
        let score = cosine_similarity(query_embedding, &entry.vector.values);

        // Apply minimum score filter
        if let Some(min_score) = options.min_score {
            if score < min_score {
                continue;
            }
        }

        // Add to results
        heap.push(SearchResult {
            document: entry.document.clone(),
            score,
        });

        // Keep only top K results for efficiency
        if heap.len() > options.limit * 2 {
            // Create temporary vector and sort
            let mut temp: Vec<_> = heap.into_iter().collect();
            temp.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
            temp.truncate(options.limit);
            heap = temp.into_iter().collect();
        }
    }

    // Extract final results
    let mut results: Vec<_> = heap.into_iter().collect();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    results.truncate(options.limit);

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // Identical vectors
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 0.0001);

        // Orthogonal vectors
        let v3 = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&v1, &v3) - 0.0).abs() < 0.0001);

        // Opposite vectors
        let v4 = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v1, &v4) - -1.0).abs() < 0.0001);
    }
}
