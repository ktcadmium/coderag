use crate::crawler::types::DocumentChunk;
use regex::Regex;

pub struct TextChunker {
    max_tokens: usize,
    #[allow(dead_code)]
    overlap_tokens: usize,
    min_chunk_size: usize,
}

impl TextChunker {
    pub fn new() -> Self {
        Self {
            max_tokens: 1500,    // Ideal chunk size for my context
            overlap_tokens: 200, // Overlap to maintain context
            min_chunk_size: 100, // Don't create tiny chunks
        }
    }
}

impl Default for TextChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl TextChunker {
    pub fn chunk_text(&self, text: &str) -> Vec<DocumentChunk> {
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

        chunks
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
                sections.push(&text[last_end..mat.start()]);
            }
            last_end = mat.start();
        }

        if last_end < text.len() {
            sections.push(&text[last_end..]);
        }

        sections
    }

    fn chunk_section(&self, section: &str, _code_blocks: &[(usize, usize)]) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let tokens = self.estimate_tokens(section);

        if tokens <= self.max_tokens {
            // Section fits in one chunk
            chunks.push(DocumentChunk {
                content: section.to_string(),
                start_char: 0,
                end_char: section.len(),
                has_code: self.contains_code(section),
                heading_context: self.extract_heading(section),
            });
        } else {
            // Need to split the section
            let paragraphs = self.split_into_paragraphs(section);
            let mut current_chunk = String::new();
            let mut current_start = 0;

            for para in paragraphs {
                let para_tokens = self.estimate_tokens(para);
                let current_tokens = self.estimate_tokens(&current_chunk);

                if current_tokens + para_tokens > self.max_tokens && !current_chunk.is_empty() {
                    // Save current chunk
                    chunks.push(DocumentChunk {
                        content: current_chunk.clone(),
                        start_char: current_start,
                        end_char: current_start + current_chunk.len(),
                        has_code: self.contains_code(&current_chunk),
                        heading_context: self.extract_heading(&current_chunk),
                    });

                    current_chunk.clear();
                    current_start += current_chunk.len();
                }

                current_chunk.push_str(para);
                current_chunk.push_str("\n\n");
            }

            // Don't forget the last chunk
            if !current_chunk.is_empty()
                && self.estimate_tokens(&current_chunk) >= self.min_chunk_size
            {
                chunks.push(DocumentChunk {
                    content: current_chunk.clone(),
                    start_char: current_start,
                    end_char: current_start + current_chunk.len(),
                    has_code: self.contains_code(&current_chunk),
                    heading_context: self.extract_heading(&current_chunk),
                });
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

    fn add_overlap(&self, _chunks: &mut [DocumentChunk], _original_text: &str) {
        // This is simplified - in production we'd implement proper overlap
        // For now, chunks are self-contained sections
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_chunking() {
        let chunker = TextChunker::new();
        let text = "# Header\n\nThis is a paragraph.\n\n## Subheader\n\nAnother paragraph.";
        let chunks = chunker.chunk_text(text);

        assert!(!chunks.is_empty());
        assert!(chunks[0].heading_context.is_some());
    }

    #[test]
    fn test_code_block_detection() {
        let chunker = TextChunker::new();
        let text = "Some text\n\n```rust\nfn main() {}\n```\n\nMore text";
        let chunks = chunker.chunk_text(text);

        assert!(chunks.iter().any(|c| c.has_code));
    }
}
