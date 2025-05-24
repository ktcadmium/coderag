use anyhow::Result;
use scraper::{Html, Selector};

pub struct ContentExtractor {
    #[allow(dead_code)]
    code_selector: Selector,
    pre_selector: Selector,
    article_selector: Selector,
    main_selector: Selector,
    #[allow(dead_code)]
    nav_selector: Selector,
    #[allow(dead_code)]
    footer_selector: Selector,
    #[allow(dead_code)]
    script_selector: Selector,
    #[allow(dead_code)]
    style_selector: Selector,
}

impl ContentExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            code_selector: Selector::parse("code, pre").unwrap(),
            pre_selector: Selector::parse("pre").unwrap(),
            article_selector: Selector::parse("article, main, .documentation, .content, .docs")
                .unwrap(),
            main_selector: Selector::parse("main").unwrap(),
            nav_selector: Selector::parse("nav, .navigation, .sidebar").unwrap(),
            footer_selector: Selector::parse("footer").unwrap(),
            script_selector: Selector::parse("script").unwrap(),
            style_selector: Selector::parse("style").unwrap(),
        })
    }

    pub fn extract_content(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        let document = Html::parse_document(html);

        // Extract title
        let title = self.extract_title(&document);

        // Remove unwanted elements
        let clean_html = self.remove_unwanted_elements(html);
        let clean_doc = Html::parse_document(&clean_html);

        // Try to find main content area
        let content_html = self
            .find_main_content(&clean_doc)
            .unwrap_or_else(|| clean_html.clone());

        // Extract code blocks before converting to markdown
        let code_blocks = self.extract_code_blocks(&Html::parse_document(&content_html));

        // Convert to markdown with appropriate width (80 chars default)
        let markdown = html2text::from_read(content_html.as_bytes(), 80);

        // Post-process markdown to clean it up
        let cleaned_markdown = self.clean_markdown(&markdown);

        // Extract metadata
        let metadata = self.extract_metadata(&document, url);

        Ok(ExtractedContent {
            title,
            markdown: cleaned_markdown,
            code_blocks,
            metadata,
        })
    }

    fn extract_title(&self, document: &Html) -> String {
        let title_selector = Selector::parse("title").unwrap();
        let h1_selector = Selector::parse("h1").unwrap();

        // Try <title> tag first
        if let Some(title) = document.select(&title_selector).next() {
            let text = title.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                return text;
            }
        }

        // Fall back to first <h1>
        if let Some(h1) = document.select(&h1_selector).next() {
            let text = h1.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                return text;
            }
        }

        "Untitled Document".to_string()
    }

    fn remove_unwanted_elements(&self, html: &str) -> String {
        let _document = Html::parse_document(html);
        let _result = String::new();

        // This is a simplified version - in production we'd use a proper HTML parser/modifier
        // For now, we'll use regex to remove script and style tags
        let mut clean = html.to_string();

        // Remove script tags and their content
        let script_re = regex::Regex::new(r"(?s)<script[^>]*>.*?</script>").unwrap();
        clean = script_re.replace_all(&clean, "").to_string();

        // Remove style tags and their content
        let style_re = regex::Regex::new(r"(?s)<style[^>]*>.*?</style>").unwrap();
        clean = style_re.replace_all(&clean, "").to_string();

        // Remove nav elements
        let nav_re = regex::Regex::new(r"(?s)<nav[^>]*>.*?</nav>").unwrap();
        clean = nav_re.replace_all(&clean, "").to_string();

        // Remove footer elements
        let footer_re = regex::Regex::new(r"(?s)<footer[^>]*>.*?</footer>").unwrap();
        clean = footer_re.replace_all(&clean, "").to_string();

        clean
    }

    fn find_main_content(&self, document: &Html) -> Option<String> {
        // Try to find the main content area
        if let Some(article) = document.select(&self.article_selector).next() {
            return Some(article.html());
        }

        if let Some(main) = document.select(&self.main_selector).next() {
            return Some(main.html());
        }

        None
    }

    fn extract_code_blocks(&self, document: &Html) -> Vec<CodeBlock> {
        let mut code_blocks = Vec::new();

        // Extract <pre> blocks (usually contain code)
        for element in document.select(&self.pre_selector) {
            let code = element.text().collect::<String>();
            let language = element
                .value()
                .classes()
                .find(|class| class.starts_with("language-"))
                .map(|class| class.trim_start_matches("language-").to_string());

            code_blocks.push(CodeBlock {
                code: code.trim().to_string(),
                language,
            });
        }

        code_blocks
    }

    fn clean_markdown(&self, markdown: &str) -> String {
        let mut cleaned = markdown.to_string();

        // Remove excessive blank lines
        let blank_line_re = regex::Regex::new(r"\n{3,}").unwrap();
        cleaned = blank_line_re.replace_all(&cleaned, "\n\n").to_string();

        // Clean up code blocks
        let code_block_re = regex::Regex::new(r"```\s*\n").unwrap();
        cleaned = code_block_re.replace_all(&cleaned, "```\n").to_string();

        // Trim whitespace
        cleaned = cleaned.trim().to_string();

        cleaned
    }

    fn extract_metadata(&self, document: &Html, url: &str) -> ContentMetadata {
        let meta_selector = Selector::parse("meta").unwrap();
        let mut metadata = ContentMetadata {
            url: url.to_string(),
            description: None,
            keywords: None,
            author: None,
            language: None,
            framework: None,
            version: None,
        };

        for element in document.select(&meta_selector) {
            if let Some(name) = element.value().attr("name") {
                let content = element.value().attr("content").map(|s| s.to_string());

                match name.to_lowercase().as_str() {
                    "description" => metadata.description = content,
                    "keywords" => metadata.keywords = content,
                    "author" => metadata.author = content,
                    _ => {}
                }
            }
        }

        // Try to detect framework from URL or content
        metadata.framework = self.detect_framework(url, &document.html());

        // Try to detect language
        metadata.language = self.detect_language(url, &document.html());

        metadata
    }

    fn detect_framework(&self, url: &str, content: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        let content_lower = content.to_lowercase();

        if url_lower.contains("react") || content_lower.contains("react.js") {
            Some("React".to_string())
        } else if url_lower.contains("vue") || content_lower.contains("vue.js") {
            Some("Vue".to_string())
        } else if url_lower.contains("angular") {
            Some("Angular".to_string())
        } else if url_lower.contains("django") {
            Some("Django".to_string())
        } else if url_lower.contains("flask") {
            Some("Flask".to_string())
        } else if url_lower.contains("rails") {
            Some("Rails".to_string())
        } else if url_lower.contains("tokio") {
            Some("Tokio".to_string())
        } else {
            None
        }
    }

    fn detect_language(&self, url: &str, _content: &str) -> Option<String> {
        let url_lower = url.to_lowercase();

        if url_lower.contains("python") || url_lower.contains("/py/") {
            Some("Python".to_string())
        } else if url_lower.contains("javascript") || url_lower.contains("/js/") {
            Some("JavaScript".to_string())
        } else if url_lower.contains("typescript") || url_lower.contains("/ts/") {
            Some("TypeScript".to_string())
        } else if url_lower.contains("rust") || url_lower.contains("/rs/") {
            Some("Rust".to_string())
        } else if url_lower.contains("java") && !url_lower.contains("javascript") {
            Some("Java".to_string())
        } else if url_lower.contains("csharp") || url_lower.contains("/cs/") {
            Some("C#".to_string())
        } else if url_lower.contains("golang") || url_lower.contains("/go/") {
            Some("Go".to_string())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub markdown: String,
    pub code_blocks: Vec<CodeBlock>,
    pub metadata: ContentMetadata,
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub code: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContentMetadata {
    pub url: String,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub version: Option<String>,
}
