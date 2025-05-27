// Hybrid search implementation combining vector similarity and keyword search

use crate::vectordb::storage::VectorStorage;
use crate::vectordb::types::Document;
use crate::vectordb::SearchOptions;
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use tracing::debug;

/// Options for hybrid search
#[derive(Debug, Clone)]
pub struct HybridSearchOptions {
    /// Base search options
    pub base: SearchOptions,
    /// Enable hybrid search (if false, falls back to vector-only)
    pub enable_hybrid: bool,
    /// Weight for vector similarity (0.0 to 1.0)
    pub vector_weight: f32,
    /// Weight for keyword search (0.0 to 1.0)
    pub keyword_weight: f32,
    /// Parameters for keyword search
    pub keyword_params: KeywordSearchParams,
}

impl Default for HybridSearchOptions {
    fn default() -> Self {
        Self {
            base: SearchOptions::default(),
            enable_hybrid: true,
            vector_weight: 0.7,
            keyword_weight: 0.3,
            keyword_params: KeywordSearchParams::default(),
        }
    }
}

/// Parameters for keyword search
#[derive(Debug, Clone)]
pub struct KeywordSearchParams {
    /// K1 parameter for BM25 (controls term frequency saturation)
    pub k1: f32,
    /// B parameter for BM25 (controls length normalization)
    pub b: f32,
}

impl Default for KeywordSearchParams {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75 }
    }
}

/// Result from hybrid search including both scores
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    /// The document
    pub document: Document,
    /// Vector similarity score
    pub vector_score: f32,
    /// Keyword search score
    pub keyword_score: f32,
    /// Combined score
    pub combined_score: f32,
}

// Implement ordering for heap operations
impl PartialEq for HybridSearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.combined_score == other.combined_score
    }
}

impl Eq for HybridSearchResult {}

impl PartialOrd for HybridSearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HybridSearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.combined_score
            .partial_cmp(&other.combined_score)
            .unwrap_or(Ordering::Equal)
    }
}

/// BM25 index for keyword search
pub struct BM25Index {
    /// Document frequency for each term
    doc_freq: HashMap<String, usize>,
    /// Term frequency for each document
    term_freq: HashMap<String, HashMap<String, usize>>,
    /// Document lengths
    doc_lengths: HashMap<String, usize>,
    /// Average document length
    avg_doc_length: f32,
    /// Total number of documents
    doc_count: usize,
    /// BM25 parameters
    params: KeywordSearchParams,
}

impl BM25Index {
    /// Create a new BM25 index
    pub fn new(params: KeywordSearchParams) -> Self {
        Self {
            doc_freq: HashMap::new(),
            term_freq: HashMap::new(),
            doc_lengths: HashMap::new(),
            avg_doc_length: 0.0,
            doc_count: 0,
            params,
        }
    }

    /// Add a document to the index
    pub fn add_document(&mut self, doc_id: &str, content: &str) {
        // Tokenize content
        let tokens = self.tokenize(content);
        let doc_length = tokens.len();

        // Update document length
        self.doc_lengths.insert(doc_id.to_string(), doc_length);

        // Update term frequencies
        let mut doc_term_freq = HashMap::new();
        for token in &tokens {
            *doc_term_freq.entry(token.clone()).or_insert(0) += 1;
        }
        self.term_freq
            .insert(doc_id.to_string(), doc_term_freq.clone());

        // Update document frequencies
        for term in doc_term_freq.keys() {
            *self.doc_freq.entry(term.clone()).or_insert(0) += 1;
        }

        // Update document count and average length
        self.doc_count += 1;
        self.avg_doc_length =
            self.doc_lengths.values().sum::<usize>() as f32 / self.doc_count as f32;
    }

    /// Search for documents matching the query
    pub fn search(&self, query: &str, limit: usize) -> Vec<(String, f32)> {
        // Tokenize query
        let query_tokens = self.tokenize(query);

        // Calculate BM25 scores for all documents
        let mut scores = HashMap::new();

        for (doc_id, doc_terms) in &self.term_freq {
            let mut score = 0.0;
            let doc_length = self.doc_lengths.get(doc_id).unwrap_or(&0);

            for query_term in &query_tokens {
                if let Some(term_freq) = doc_terms.get(query_term) {
                    // Get document frequency
                    let df = self.doc_freq.get(query_term).unwrap_or(&0);

                    // Calculate IDF
                    let idf =
                        ((self.doc_count as f32 - *df as f32 + 0.5) / (*df as f32 + 0.5)).ln();

                    // Calculate BM25 score component
                    let tf = *term_freq as f32;
                    let dl = *doc_length as f32;
                    let avgdl = self.avg_doc_length;

                    let numerator = tf * (self.params.k1 + 1.0);
                    let denominator =
                        tf + self.params.k1 * (1.0 - self.params.b + self.params.b * (dl / avgdl));

                    score += idf * (numerator / denominator);
                }
            }

            if score > 0.0 {
                scores.insert(doc_id.clone(), score);
            }
        }

        // Sort by score and return top k
        let mut results: Vec<(String, f32)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        results.truncate(limit);

        results
    }

    /// Simple tokenization (can be improved)
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Get index statistics
    pub fn stats(&self) -> BM25Stats {
        BM25Stats {
            doc_count: self.doc_count,
            term_count: self.doc_freq.len(),
            avg_doc_length: self.avg_doc_length,
        }
    }
}

/// BM25 index statistics
pub struct BM25Stats {
    pub doc_count: usize,
    pub term_count: usize,
    pub avg_doc_length: f32,
}

/// Perform hybrid search combining vector similarity and keyword search
pub fn hybrid_search(
    storage: &VectorStorage,
    query_embedding: &[f32],
    query_text: &str,
    options: HybridSearchOptions,
) -> Result<Vec<HybridSearchResult>> {
    debug!("Performing hybrid search with query: {}", query_text);

    // If hybrid search is disabled, fall back to vector-only search
    if !options.enable_hybrid {
        let vector_results =
            crate::vectordb::search::search_documents(storage, query_embedding, options.base)?;

        // Convert to hybrid results with zero keyword score
        let results = vector_results
            .into_iter()
            .map(|r| HybridSearchResult {
                document: r.document,
                vector_score: r.score,
                keyword_score: 0.0,
                combined_score: r.score,
            })
            .collect();

        return Ok(results);
    }

    // Build BM25 index
    let mut bm25_index = BM25Index::new(options.keyword_params);

    // Index all documents
    let entries = storage.get_all_entries();
    for entry in entries {
        bm25_index.add_document(&entry.id, &entry.document.content);
    }

    // Get vector search results (get more than needed for re-ranking)
    let vector_limit = options.base.limit * 3;
    let vector_options = SearchOptions {
        limit: vector_limit,
        ..options.base.clone()
    };
    let vector_results =
        crate::vectordb::search::search_documents(storage, query_embedding, vector_options)?;

    // Get keyword search results
    let keyword_results = bm25_index.search(query_text, vector_limit);

    // Create a map of keyword scores
    let keyword_scores: HashMap<String, f32> = keyword_results.into_iter().collect();

    // Combine scores
    let mut combined_results = Vec::new();

    for vector_result in vector_results {
        let doc_id = &vector_result.document.id;
        let vector_score = vector_result.score;
        let keyword_score = keyword_scores.get(doc_id).copied().unwrap_or(0.0);

        // Normalize keyword score to 0-1 range (BM25 scores can be unbounded)
        let normalized_keyword_score = (keyword_score / (1.0 + keyword_score)).min(1.0);

        // Calculate combined score
        let combined_score = options.vector_weight * vector_score
            + options.keyword_weight * normalized_keyword_score;

        combined_results.push(HybridSearchResult {
            document: vector_result.document,
            vector_score,
            keyword_score: normalized_keyword_score,
            combined_score,
        });
    }

    // Also check keyword-only results that might not be in vector results
    for (doc_id, keyword_score) in keyword_scores {
        // Skip if already in results
        if combined_results.iter().any(|r| r.document.id == doc_id) {
            continue;
        }

        // Get document from storage
        if let Some(document) = storage.get_document(&doc_id) {
            // Calculate vector score
            let entry = storage
                .get_entries()
                .iter()
                .find(|e| e.id == doc_id)
                .unwrap();

            let vector_score =
                crate::vectordb::cosine_similarity(query_embedding, &entry.vector.values);

            // Apply filters
            if let Some(ref source_filter) = options.base.source_filter {
                if !document.url.contains(source_filter) {
                    continue;
                }
            }

            if let Some(content_type_filter) = options.base.content_type_filter {
                if document.metadata.content_type != content_type_filter {
                    continue;
                }
            }

            if let Some(min_score) = options.base.min_score {
                if vector_score < min_score {
                    continue;
                }
            }

            // Normalize keyword score
            let normalized_keyword_score = (keyword_score / (1.0 + keyword_score)).min(1.0);

            // Calculate combined score
            let combined_score = options.vector_weight * vector_score
                + options.keyword_weight * normalized_keyword_score;

            combined_results.push(HybridSearchResult {
                document: document.clone(),
                vector_score,
                keyword_score: normalized_keyword_score,
                combined_score,
            });
        }
    }

    // Sort by combined score and take top k
    let mut heap = BinaryHeap::new();

    for result in combined_results {
        heap.push(result);

        // Keep only top k results for efficiency
        if heap.len() > options.base.limit * 2 {
            heap.pop();
        }
    }

    // Extract final results
    let mut results: Vec<_> = heap.into_iter().collect();
    results.sort_by(|a, b| {
        b.combined_score
            .partial_cmp(&a.combined_score)
            .unwrap_or(Ordering::Equal)
    });
    results.truncate(options.base.limit);

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectordb::types::{ContentType, Document, DocumentMetadata, Vector};

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
                last_updated: None,
                tags: vec!["test".to_string()],
            },
        }
    }

    fn create_test_storage() -> VectorStorage {
        use std::path::PathBuf;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("test_vectors.json");

        let mut storage = VectorStorage::new(storage_path).unwrap();

        // Add some test documents
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
            storage.add_document(doc, Vector::new(vector)).unwrap();
        }

        storage
    }

    #[test]
    fn test_bm25_index() {
        // Create BM25 index
        let params = KeywordSearchParams::default();
        let mut index = BM25Index::new(params);

        // Add documents
        index.add_document("1", "rust systems programming safety performance");
        index.add_document("2", "python high level programming readability");
        index.add_document("3", "javascript web programming frontend");

        // Search
        let results = index.search("rust programming", 10);

        // Should find doc 1 first
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "1");

        // Stats
        let stats = index.stats();
        assert_eq!(stats.doc_count, 3);
    }

    #[test]
    fn test_hybrid_search() -> Result<()> {
        let storage = create_test_storage();

        // Search with hybrid approach
        let query_embedding = vec![0.9, 0.2, 0.1]; // Similar to document 1 (Rust)
        let query_text = "memory safety programming"; // Keywords match doc 4 (Rust and C++)

        let options = HybridSearchOptions {
            base: SearchOptions {
                limit: 2,
                min_score: None,
                source_filter: None,
                content_type_filter: None,
            },
            enable_hybrid: true,
            vector_weight: 0.6,
            keyword_weight: 0.4,
            keyword_params: KeywordSearchParams::default(),
        };

        let results = hybrid_search(&storage, &query_embedding, query_text, options)?;

        // Should find both doc 1 and doc 4
        assert_eq!(results.len(), 2);

        // Doc 4 should rank higher due to keyword match on "memory safety"
        assert_eq!(results[0].document.id, "4");

        // Check that scores are populated
        assert!(results[0].vector_score > 0.0);
        assert!(results[0].keyword_score > 0.0);
        assert!(results[0].combined_score > 0.0);

        Ok(())
    }

    #[test]
    fn test_vector_only_search() -> Result<()> {
        let storage = create_test_storage();

        // Search with vector only
        let query_embedding = vec![0.9, 0.2, 0.1]; // Similar to document 1 (Rust)
        let query_text = "memory safety programming"; // Keywords match doc 4 (Rust and C++)

        let options = HybridSearchOptions {
            base: SearchOptions {
                limit: 2,
                min_score: None,
                source_filter: None,
                content_type_filter: None,
            },
            enable_hybrid: false, // Disable hybrid search
            vector_weight: 1.0,
            keyword_weight: 0.0,
            keyword_params: KeywordSearchParams::default(),
        };

        let results = hybrid_search(&storage, &query_embedding, query_text, options)?;

        // Should find docs based on vector similarity only
        assert_eq!(results.len(), 2);

        // Doc 1 should rank higher due to vector similarity
        assert_eq!(results[0].document.id, "1");

        Ok(())
    }
}
