use crate::crawler::types::DocumentChunk;
use regex::Regex;
use std::collections::HashSet;

/// Text chunker for splitting documents into manageable pieces
#[derive(Debug)]
pub struct TextChunker {
    chunk_size: usize,
    overlap: usize,
    min_chunk_size: usize,
    seen_content_hashes: HashSet<u64>,
}

impl TextChunker {
    pub fn new() -> Self {
        Self {
            chunk_size: 1500,    // Ideal chunk size for AI context
            overlap: 200,        // Overlap to maintain context
            min_chunk_size: 100, // Don't create tiny chunks
            seen_content_hashes: HashSet::new(),
        }
    }

    /// Create a new TextChunker with persistent deduplication
    pub fn with_persistent_deduplication(existing_hashes: HashSet<u64>) -> Self {
        Self {
            chunk_size: 1500,
            overlap: 200,
            min_chunk_size: 100,
            seen_content_hashes: existing_hashes,
        }
    }

    /// Get the current set of seen content hashes for persistence
    pub fn get_seen_hashes(&self) -> &HashSet<u64> {
        &self.seen_content_hashes
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl TextChunker {
    pub fn chunk_text(&mut self, text: &str) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();

        // First, identify code blocks and their positions
        let code_blocks = self.find_code_blocks(text);

        // Split text into sections by headers
        let sections = self.split_by_headers(text);

        // Process each section
        for section in sections {
            let section_chunks = self.chunk_section(section, &code_blocks);
            chunks.extend(section_chunks);
        }

        // Ensure we have proper overlap
        self.add_overlap(&mut chunks, text);

        // Deduplicate and filter chunks
        self.deduplicate_and_filter(chunks)
    }

    fn deduplicate_and_filter(&mut self, chunks: Vec<DocumentChunk>) -> Vec<DocumentChunk> {
        let mut filtered_chunks = Vec::new();

        for chunk in chunks {
            // Skip if content is too short or low quality
            if !self.is_quality_content(&chunk.content) {
                continue;
            }

            // Calculate content hash for deduplication
            let content_hash = self.calculate_content_hash(&chunk.content);

            // Skip if we've seen this content before (including across sessions)
            if self.seen_content_hashes.contains(&content_hash) {
                continue;
            }

            // Add to persistent hash set
            self.seen_content_hashes.insert(content_hash);
            filtered_chunks.push(chunk);
        }

        filtered_chunks
    }

    fn is_quality_content(&self, content: &str) -> bool {
        let trimmed = content.trim();

        // Must meet minimum length
        if trimmed.len() < self.min_chunk_size {
            return false;
        }

        // Must have substantial alphabetic content
        let alpha_count = trimmed.chars().filter(|c| c.is_alphabetic()).count();
        let total_chars = trimmed.len();

        if alpha_count < total_chars / 3 {
            return false;
        }

        // Check for navigation-like patterns
        let nav_indicators = [
            "skip to",
            "toggle",
            "menu",
            "navigation",
            "breadcrumb",
            "| next |",
            "| previous |",
            "| index |",
            "table of contents",
        ];

        let lower_content = trimmed.to_lowercase();
        for indicator in &nav_indicators {
            if lower_content.contains(indicator) {
                return false;
            }
        }

        // Check if it's mostly punctuation
        let punct_count = trimmed.chars().filter(|c| c.is_ascii_punctuation()).count();
        if punct_count > alpha_count {
            return false;
        }

        // Check for repeated patterns (like navigation)
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() > 1 {
            let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
            // If more than 50% of lines are duplicates, it's likely navigation
            if unique_lines.len() < lines.len() / 2 {
                return false;
            }
        }

        true
    }

    fn calculate_content_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Normalize content for hashing (remove extra whitespace, lowercase)
        let normalized = content
            .trim()
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        hasher.finish()
    }

    fn find_code_blocks(&self, text: &str) -> Vec<(usize, usize)> {
        let mut code_blocks = Vec::new();

        // Find markdown code blocks
        let code_block_re = Regex::new(r"(?s)```.*?```").unwrap();
        for mat in code_block_re.find_iter(text) {
            code_blocks.push((mat.start(), mat.end()));
        }

        // Find indented code blocks (4 spaces or tab)
        let indented_re = Regex::new(r"(?m)^(    |\t).*$").unwrap();
        let mut in_block = false;
        let mut block_start = 0;

        for (i, line) in text.lines().enumerate() {
            let line_start = text.lines().take(i).map(|l| l.len() + 1).sum::<usize>();

            if indented_re.is_match(line) {
                if !in_block {
                    in_block = true;
                    block_start = line_start;
                }
            } else if in_block {
                in_block = false;
                let block_end = line_start;
                code_blocks.push((block_start, block_end));
            }
        }

        // Sort by start position
        code_blocks.sort_by_key(|&(start, _)| start);
        code_blocks
    }

    fn split_by_headers<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let header_re = Regex::new(r"(?m)^#{1,3}\s+.+$").unwrap();
        let mut sections = Vec::new();
        let mut last_end = 0;

        for mat in header_re.find_iter(text) {
            if mat.start() > last_end {
                let section = &text[last_end..mat.start()];
                if !section.trim().is_empty() {
                    sections.push(section);
                }
            }
            last_end = mat.start();
        }

        if last_end < text.len() {
            let section = &text[last_end..];
            if !section.trim().is_empty() {
                sections.push(section);
            }
        }

        // If no headers found, treat entire text as one section
        if sections.is_empty() && !text.trim().is_empty() {
            sections.push(text);
        }

        sections
    }

    fn chunk_section(&self, section: &str, _code_blocks: &[(usize, usize)]) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let tokens = self.estimate_tokens(section);

        if tokens <= self.chunk_size {
            // Section fits in one chunk
            let chunk = DocumentChunk {
                content: section.to_string(),
                start_char: 0,
                end_char: section.len(),
                has_code: self.contains_code(section),
                heading_context: self.extract_heading(section),
            };

            // Only add if it's quality content
            if self.is_quality_content(&chunk.content) {
                chunks.push(chunk);
            }
        } else {
            // Need to split the section
            let paragraphs = self.split_into_paragraphs(section);
            let mut current_chunk = String::new();
            let mut current_start = 0;

            for para in paragraphs {
                let para_tokens = self.estimate_tokens(para);
                let current_tokens = self.estimate_tokens(&current_chunk);

                if current_tokens + para_tokens > self.chunk_size && !current_chunk.is_empty() {
                    // Save current chunk if it's quality content
                    let chunk = DocumentChunk {
                        content: current_chunk.clone(),
                        start_char: current_start,
                        end_char: current_start + current_chunk.len(),
                        has_code: self.contains_code(&current_chunk),
                        heading_context: self.extract_heading(&current_chunk),
                    };

                    if self.is_quality_content(&chunk.content) {
                        chunks.push(chunk);
                    }

                    current_chunk.clear();
                    current_start += current_chunk.len();
                }

                current_chunk.push_str(para);
                current_chunk.push_str("\n\n");
            }

            // Don't forget the last chunk
            if !current_chunk.is_empty() {
                let chunk = DocumentChunk {
                    content: current_chunk.clone(),
                    start_char: current_start,
                    end_char: current_start + current_chunk.len(),
                    has_code: self.contains_code(&current_chunk),
                    heading_context: self.extract_heading(&current_chunk),
                };

                if self.is_quality_content(&chunk.content) {
                    chunks.push(chunk);
                }
            }
        }

        chunks
    }

    fn split_into_paragraphs<'a>(&self, text: &'a str) -> Vec<&'a str> {
        text.split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect()
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        // Simple estimation: ~4 characters per token on average
        // This is good enough for chunking purposes
        text.len() / 4
    }

    fn contains_code(&self, text: &str) -> bool {
        text.contains("```")
            || text.contains("    ")
            || text.contains("\t")
            || text.contains("function")
            || text.contains("class")
            || text.contains("def ")
            || text.contains("const ")
            || text.contains("let ")
            || text.contains("var ")
    }

    fn extract_heading(&self, text: &str) -> Option<String> {
        let header_re = Regex::new(r"(?m)^#{1,3}\s+(.+)$").unwrap();

        if let Some(captures) = header_re.captures(text) {
            if let Some(heading) = captures.get(1) {
                return Some(heading.as_str().to_string());
            }
        }

        None
    }

    fn add_overlap(&self, chunks: &mut Vec<DocumentChunk>, original_text: &str) {
        // Implement intelligent overlap for AI assistance
        if chunks.len() < 2 {
            return;
        }

        let mut overlapped_chunks = Vec::new();

        for (i, chunk) in chunks.iter().enumerate() {
            let mut enhanced_chunk = chunk.clone();

            // Add context from previous chunk if it contains important information
            if i > 0 {
                let prev_chunk = &chunks[i - 1];
                if let Some(overlap_text) =
                    self.extract_overlap_context(prev_chunk, chunk, original_text)
                {
                    enhanced_chunk.content =
                        format!("...{}\n\n{}", overlap_text, enhanced_chunk.content);
                }
            }

            // Add context from next chunk if current chunk ends mid-concept
            if i < chunks.len() - 1 {
                let next_chunk = &chunks[i + 1];
                if self.needs_forward_context(chunk) {
                    if let Some(forward_context) =
                        self.extract_forward_context(chunk, next_chunk, original_text)
                    {
                        enhanced_chunk.content =
                            format!("{}\n\n{}...", enhanced_chunk.content, forward_context);
                    }
                }
            }

            overlapped_chunks.push(enhanced_chunk);
        }

        *chunks = overlapped_chunks;
    }

    fn extract_overlap_context(
        &self,
        prev_chunk: &DocumentChunk,
        _current_chunk: &DocumentChunk,
        _original_text: &str,
    ) -> Option<String> {
        // Extract the last meaningful content from previous chunk for context
        let prev_lines: Vec<&str> = prev_chunk.content.lines().collect();
        let overlap_size = std::cmp::min(self.overlap / 10, prev_lines.len()); // Convert chars to approximate lines

        if overlap_size > 0 {
            let overlap_lines = &prev_lines[prev_lines.len() - overlap_size..];
            let overlap_text = overlap_lines.join("\n").trim().to_string();

            // Only include if it provides meaningful context
            if overlap_text.len() > 20
                && !overlap_text
                    .chars()
                    .all(|c| c.is_whitespace() || c.is_ascii_punctuation())
            {
                return Some(overlap_text);
            }
        }

        None
    }

    fn needs_forward_context(&self, chunk: &DocumentChunk) -> bool {
        let content = &chunk.content;

        // Check if chunk ends in a way that suggests continuation
        content.ends_with(':')
            || content.ends_with("following")
            || content.ends_with("example")
            || content.ends_with("see")
            || content.contains("continued")
            || content.contains("next section")
    }

    fn extract_forward_context(
        &self,
        _current_chunk: &DocumentChunk,
        next_chunk: &DocumentChunk,
        _original_text: &str,
    ) -> Option<String> {
        // Extract the beginning of next chunk for forward context
        let next_lines: Vec<&str> = next_chunk.content.lines().collect();
        let context_size = std::cmp::min(3, next_lines.len()); // Just a few lines for forward context

        if context_size > 0 {
            let context_lines = &next_lines[..context_size];
            let context_text = context_lines.join("\n").trim().to_string();

            if context_text.len() > 20 {
                return Some(context_text);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_chunking() {
        let mut chunker = TextChunker::new();
        // Make the test content longer to pass quality filtering
        let text = "# Header\n\nThis is a substantial paragraph with enough content to pass the quality filtering. It contains meaningful text that would be useful for AI assistance and documentation purposes.\n\n## Subheader\n\nAnother paragraph with sufficient content to demonstrate the chunking functionality. This paragraph also contains enough text to be considered quality content by the filtering system.";
        let chunks = chunker.chunk_text(text);

        assert!(!chunks.is_empty(), "Chunks should not be empty");
        if !chunks.is_empty() {
            assert!(
                chunks[0].heading_context.is_some(),
                "First chunk should have heading context"
            );
        }
    }

    #[test]
    fn test_code_block_detection() {
        let mut chunker = TextChunker::new();
        // Make the test content longer to pass quality filtering
        let text = "This is a comprehensive example showing how to use Rust code in documentation. The following code demonstrates a simple main function that is commonly used in Rust applications.\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n    println!(\"The answer is: {}\", x);\n}\n```\n\nThis code example shows the basic structure of a Rust program with variable declaration and printing functionality.";
        let chunks = chunker.chunk_text(text);

        assert!(!chunks.is_empty(), "Chunks should not be empty");
        assert!(
            chunks.iter().any(|c| c.has_code),
            "At least one chunk should contain code"
        );
    }
}
