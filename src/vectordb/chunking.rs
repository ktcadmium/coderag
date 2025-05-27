// Improved chunking strategy for better document representation

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use tracing::debug;

/// Chunking strategy for document splitting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkingStrategy {
    /// Fixed size with overlap
    FixedSizeOverlap {
        /// Chunk size in characters
        chunk_size: usize,
        /// Overlap size in characters
        overlap: usize,
    },
    /// Split at semantic boundaries (paragraphs, sections)
    SemanticBoundaries {
        /// Maximum chunk size in characters
        max_size: usize,
        /// Minimum chunk size in characters
        min_size: usize,
    },
    /// Split at section headings
    HeadingBased {
        /// Maximum chunk size in characters
        max_size: usize,
        /// Minimum chunk size in characters
        min_size: usize,
    },
}

impl Default for ChunkingStrategy {
    fn default() -> Self {
        Self::FixedSizeOverlap {
            chunk_size: 1000,
            overlap: 200,
        }
    }
}

/// Document chunk with content and metadata
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Chunk content
    pub content: String,
    /// Chunk heading or title
    pub heading: Option<String>,
    /// Heading hierarchy context
    pub heading_context: Option<String>,
    /// Whether chunk contains code
    pub has_code: bool,
    /// Position in original document (for ordering)
    pub position: usize,
    /// Hash of chunk content (for deduplication)
    pub content_hash: u64,
}

/// Enhanced text chunker with multiple strategies
pub struct EnhancedChunker {
    /// Chunking strategy to use
    strategy: ChunkingStrategy,
    /// Content hash set for deduplication
    seen_content_hashes: HashSet<u64>,
}

impl EnhancedChunker {
    /// Create a new chunker with the specified strategy
    pub fn new(strategy: ChunkingStrategy) -> Self {
        Self {
            strategy,
            seen_content_hashes: HashSet::new(),
        }
    }

    /// Set the chunking strategy
    pub fn with_strategy(mut self, strategy: ChunkingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Process text into chunks based on the selected strategy
    pub fn chunk_text(&mut self, text: &str) -> Vec<Chunk> {
        match self.strategy {
            ChunkingStrategy::FixedSizeOverlap {
                chunk_size,
                overlap,
            } => self.chunk_fixed_size(text, chunk_size, overlap),
            ChunkingStrategy::SemanticBoundaries { max_size, min_size } => {
                self.chunk_semantic_boundaries(text, max_size, min_size)
            }
            ChunkingStrategy::HeadingBased { max_size, min_size } => {
                self.chunk_heading_based(text, max_size, min_size)
            }
        }
    }

    /// Create fixed-size chunks with overlap
    fn chunk_fixed_size(&mut self, text: &str, chunk_size: usize, overlap: usize) -> Vec<Chunk> {
        // Check if text is shorter than chunk_size
        if text.len() <= chunk_size {
            return self.create_single_chunk(text, 0);
        }

        let mut chunks = Vec::new();
        let mut position = 0;

        // Find good split points (end of sentences or paragraphs)
        let mut start = 0;
        while start < text.len() {
            let end = if start + chunk_size >= text.len() {
                text.len()
            } else {
                // Find a good split point
                let potential_end = start + chunk_size;
                let mut end = potential_end;

                // Try to find sentence boundary
                let sentence_boundary = text
                    [start.min(text.len() - 1)..potential_end.min(text.len())]
                    .rfind(['.', '!', '?', '\n']);

                if let Some(boundary) = sentence_boundary {
                    end = start + boundary + 1;
                }

                end
            };

            let chunk_text = text[start..end].to_string();

            // Create chunk if not duplicate
            if let Some(chunk) = self.create_chunk_if_unique(&chunk_text, position) {
                chunks.push(chunk);
                position += 1;
            }

            // Move start position for next chunk
            start = if end == text.len() {
                // Reached the end
                end
            } else {
                // Move back by overlap amount, but ensure we make progress
                (end - overlap).max(start + 1)
            };
        }

        chunks
    }

    /// Create chunks based on semantic boundaries (paragraphs, sections)
    fn chunk_semantic_boundaries(
        &mut self,
        text: &str,
        max_size: usize,
        min_size: usize,
    ) -> Vec<Chunk> {
        // Split text into paragraphs
        let paragraphs: Vec<&str> = text.split("\n\n").collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut position = 0;

        for paragraph in paragraphs {
            // Skip empty paragraphs
            if paragraph.trim().is_empty() {
                continue;
            }

            // If adding this paragraph exceeds max_size and we have content,
            // create a chunk and start a new one
            if !current_chunk.is_empty()
                && current_chunk.len() + paragraph.len() + 2 > max_size
                && current_chunk.len() >= min_size
            {
                if let Some(chunk) = self.create_chunk_if_unique(&current_chunk, position) {
                    chunks.push(chunk);
                    position += 1;
                }
                current_chunk = String::new();
            }

            // Add paragraph to current chunk
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(paragraph);

            // If paragraph itself is large enough, create a chunk
            if paragraph.len() >= max_size {
                if let Some(chunk) = self.create_chunk_if_unique(&current_chunk, position) {
                    chunks.push(chunk);
                    position += 1;
                }
                current_chunk = String::new();
            }
        }

        // Add final chunk if not empty
        if !current_chunk.is_empty() && current_chunk.len() >= min_size {
            if let Some(chunk) = self.create_chunk_if_unique(&current_chunk, position) {
                chunks.push(chunk);
            }
        }

        chunks
    }

    /// Create chunks based on headings
    fn chunk_heading_based(&mut self, text: &str, max_size: usize, min_size: usize) -> Vec<Chunk> {
        // Split text into lines
        let lines: Vec<&str> = text.lines().collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_heading = None;
        let mut heading_context = None;
        let mut position = 0;

        // Keep track of heading hierarchy
        let mut heading_stack: Vec<String> = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            // Check if line is a heading
            let heading_level = self.get_heading_level(trimmed);

            if heading_level > 0 {
                // This is a heading
                let heading_text = self.extract_heading_text(trimmed, heading_level);

                // If we have content in the current chunk and it's enough,
                // save it before starting a new section
                if !current_chunk.is_empty() && current_chunk.len() >= min_size {
                    if let Some(chunk) = self.create_chunk_if_unique_with_heading(
                        &current_chunk,
                        current_heading.as_deref(),
                        heading_context.as_deref(),
                        position,
                    ) {
                        chunks.push(chunk);
                        position += 1;
                    }
                    current_chunk.clear();
                }

                // Update heading context based on heading level
                while heading_stack.len() >= heading_level {
                    heading_stack.pop();
                }
                heading_stack.push(heading_text.clone());

                // Build heading context string
                heading_context = if heading_stack.is_empty() {
                    None
                } else {
                    Some(heading_stack.join(" > "))
                };

                // Set current heading
                current_heading = Some(heading_text);

                // Add heading to chunk
                current_chunk = trimmed.to_string();
                current_chunk.push('\n');
            } else {
                // Regular content
                if !current_chunk.is_empty() {
                    // Check if adding this line would exceed max_size
                    if current_chunk.len() + line.len() + 1 > max_size {
                        // Save current chunk if it's large enough
                        if current_chunk.len() >= min_size {
                            if let Some(chunk) = self.create_chunk_if_unique_with_heading(
                                &current_chunk,
                                current_heading.as_deref(),
                                heading_context.as_deref(),
                                position,
                            ) {
                                chunks.push(chunk);
                                position += 1;
                            }
                            current_chunk = String::new();

                            // Keep heading context but reset current heading
                            // for continuation chunks
                            current_heading = None;
                        }
                    }
                }

                // Add line to current chunk
                if !current_chunk.is_empty() {
                    current_chunk.push('\n');
                }
                current_chunk.push_str(line);
            }
        }

        // Add final chunk if not empty
        if !current_chunk.is_empty() && current_chunk.len() >= min_size {
            if let Some(chunk) = self.create_chunk_if_unique_with_heading(
                &current_chunk,
                current_heading.as_deref(),
                heading_context.as_deref(),
                position,
            ) {
                chunks.push(chunk);
            }
        }

        chunks
    }

    /// Create a single chunk from the entire text
    fn create_single_chunk(&mut self, text: &str, position: usize) -> Vec<Chunk> {
        if let Some(chunk) = self.create_chunk_if_unique(text, position) {
            vec![chunk]
        } else {
            Vec::new()
        }
    }

    /// Create a chunk if content is unique (not seen before)
    fn create_chunk_if_unique(&mut self, content: &str, position: usize) -> Option<Chunk> {
        // Trim content
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Calculate content hash
        let content_hash = self.hash_content(trimmed);

        // Check if we've seen this content before
        if self.seen_content_hashes.contains(&content_hash) {
            debug!("Skipping duplicate chunk with hash {}", content_hash);
            return None;
        }

        // Add to seen hashes
        self.seen_content_hashes.insert(content_hash);

        // Detect if chunk has code
        let has_code =
            trimmed.contains("```") || trimmed.contains("    ") || trimmed.contains("\t");

        // Create chunk
        Some(Chunk {
            content: trimmed.to_string(),
            heading: None,
            heading_context: None,
            has_code,
            position,
            content_hash,
        })
    }

    /// Create a chunk with heading information if content is unique
    fn create_chunk_if_unique_with_heading(
        &mut self,
        content: &str,
        heading: Option<&str>,
        heading_context: Option<&str>,
        position: usize,
    ) -> Option<Chunk> {
        // Trim content
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Calculate content hash
        let content_hash = self.hash_content(trimmed);

        // Check if we've seen this content before
        if self.seen_content_hashes.contains(&content_hash) {
            debug!("Skipping duplicate chunk with hash {}", content_hash);
            return None;
        }

        // Add to seen hashes
        self.seen_content_hashes.insert(content_hash);

        // Detect if chunk has code
        let has_code =
            trimmed.contains("```") || trimmed.contains("    ") || trimmed.contains("\t");

        // Create chunk
        Some(Chunk {
            content: trimmed.to_string(),
            heading: heading.map(|s| s.to_string()),
            heading_context: heading_context.map(|s| s.to_string()),
            has_code,
            position,
            content_hash,
        })
    }

    /// Calculate a hash for content (for deduplication)
    fn hash_content(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Get heading level from a line (0 if not a heading)
    fn get_heading_level(&self, line: &str) -> usize {
        // Check for Markdown headings (# Heading)
        if line.starts_with('#') {
            let mut level = 0;
            for c in line.chars() {
                if c == '#' {
                    level += 1;
                } else {
                    break;
                }
            }
            if level > 0 && level <= 6 && line.len() > level && line.chars().nth(level) == Some(' ')
            {
                return level;
            }
        }

        // Check for underlined headings (Heading\n====== or Heading\n------)
        if line.chars().all(|c| c == '=' || c == '-') && line.len() >= 3 {
            return if line.contains('=') { 1 } else { 2 };
        }

        0 // Not a heading
    }

    /// Extract heading text from a heading line
    fn extract_heading_text(&self, line: &str, level: usize) -> String {
        if level > 0 && line.starts_with('#') {
            // Markdown heading (# Heading)
            line[level + 1..].trim().to_string()
        } else if line.chars().all(|c| c == '=' || c == '-') {
            // Underlined heading (previous line is the heading)
            String::new() // Can't extract text here, would need previous line
        } else {
            line.to_string()
        }
    }

    /// Clear the deduplication cache
    pub fn clear_deduplication_cache(&mut self) {
        self.seen_content_hashes.clear();
    }

    /// Get the current chunking strategy
    pub fn strategy(&self) -> ChunkingStrategy {
        self.strategy
    }

    /// Set a new chunking strategy
    pub fn set_strategy(&mut self, strategy: ChunkingStrategy) {
        self.strategy = strategy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_size_chunking() {
        let text = "This is a test paragraph. It contains multiple sentences.\n\nThis is another paragraph. It also has multiple sentences. This paragraph is longer than the first one.";

        let mut chunker = EnhancedChunker::new(ChunkingStrategy::FixedSizeOverlap {
            chunk_size: 50,
            overlap: 10,
        });

        let chunks = chunker.chunk_text(text);

        // Should create multiple chunks
        assert!(chunks.len() > 1);

        // Check content
        assert!(chunks[0].content.starts_with("This is a test paragraph"));
    }

    #[test]
    fn test_semantic_boundaries_chunking() {
        let text = "# Heading 1\n\nThis is paragraph 1.\n\n## Heading 2\n\nThis is paragraph 2.\n\nThis is paragraph 3.";

        let mut chunker = EnhancedChunker::new(ChunkingStrategy::SemanticBoundaries {
            max_size: 50,
            min_size: 10,
        });

        let chunks = chunker.chunk_text(text);

        // Should create at least 2 chunks
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_heading_based_chunking() {
        let text = "# Heading 1\n\nThis is paragraph 1.\n\n## Heading 2\n\nThis is paragraph 2.\n\nThis is paragraph 3.\n\n# Heading 3\n\nFinal paragraph.";

        let mut chunker = EnhancedChunker::new(ChunkingStrategy::HeadingBased {
            max_size: 200,
            min_size: 10,
        });

        let chunks = chunker.chunk_text(text);

        // Should create at least 2 chunks based on headings
        assert!(chunks.len() >= 2);

        // Check that heading is captured
        assert_eq!(chunks[0].heading, Some("Heading 1".to_string()));
    }

    #[test]
    fn test_deduplication() {
        let text = "This is a test paragraph.\n\nThis is a test paragraph.";

        let mut chunker = EnhancedChunker::new(ChunkingStrategy::default());

        let chunks = chunker.chunk_text(text);

        // Should only create one chunk despite duplicate paragraph
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_code_detection() {
        let text = "This is a paragraph.\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";

        let mut chunker = EnhancedChunker::new(ChunkingStrategy::default());

        let chunks = chunker.chunk_text(text);

        // Should detect code block
        assert!(chunks[0].has_code);
    }
}
