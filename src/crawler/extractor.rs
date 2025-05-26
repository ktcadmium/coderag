use anyhow::Result;
use scraper::{Html, Selector};

/// Content extractor for cleaning and extracting main content from HTML
#[derive(Debug)]
pub struct ContentExtractor {
    // CSS selectors for content identification and filtering
    main_content_selector: Selector,
    article_selector: Selector,
    content_selector: Selector,
    nav_selector: Selector,
    footer_selector: Selector,
    header_selector: Selector,
    sidebar_selector: Selector,
    breadcrumb_selector: Selector,
    menu_selector: Selector,
}

impl ContentExtractor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            main_content_selector: Selector::parse("main").unwrap(),
            article_selector: Selector::parse("article, main, .documentation, .content, .docs, .markdown-body, .post-content, .entry-content")
                .unwrap(),
            content_selector: Selector::parse(".content, .docs, .documentation").unwrap(),
            // Enhanced selectors for content to exclude
            nav_selector: Selector::parse("nav, .navigation, .navbar, .nav, .menu, .toc, .table-of-contents").unwrap(),
            footer_selector: Selector::parse("footer, .footer").unwrap(),
            header_selector: Selector::parse("header, .header, .site-header").unwrap(),
            sidebar_selector: Selector::parse(".sidebar, .side-nav, .secondary, aside").unwrap(),
            breadcrumb_selector: Selector::parse(".breadcrumb, .breadcrumbs, .crumbs").unwrap(),
            menu_selector: Selector::parse(".menu, .dropdown, .submenu").unwrap(),
        })
    }

    pub fn extract_content(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        let document = Html::parse_document(html);

        // Extract title
        let title = self.extract_title(&document);

        // Remove unwanted elements more thoroughly
        let clean_html = self.remove_unwanted_elements_advanced(&document);
        let clean_doc = Html::parse_document(&clean_html);

        // Try to find main content area
        let content_html = self
            .find_main_content(&clean_doc)
            .unwrap_or_else(|| clean_html.clone());

        // Extract code blocks before converting to markdown
        let code_blocks = self.extract_code_blocks(&Html::parse_document(&content_html));

        // Convert to markdown with appropriate width (80 chars default)
        let markdown = html2text::from_read(content_html.as_bytes(), 80);

        // Post-process markdown to clean it up more thoroughly
        let cleaned_markdown = self.clean_markdown_advanced(&markdown);

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

    fn remove_unwanted_elements_advanced(&self, document: &Html) -> String {
        // Create a new document by cloning the original
        let mut html = document.html();
        let mut doc = Html::parse_document(&html);

        // Define comprehensive selectors for unwanted elements
        let unwanted_selectors = [
            // Scripts and styles
            "script",
            "style",
            "noscript",
            // Navigation elements
            "nav",
            ".navigation",
            ".navbar",
            ".nav",
            ".menu",
            ".toc",
            ".table-of-contents",
            // Headers and footers
            "header",
            ".header",
            ".site-header",
            "footer",
            ".footer",
            // Sidebars and asides
            ".sidebar",
            ".side-nav",
            ".secondary",
            "aside",
            // Breadcrumbs and menus
            ".breadcrumb",
            ".breadcrumbs",
            ".crumbs",
            ".menu",
            ".dropdown",
            ".submenu",
            // Common boilerplate
            ".skip-link",
            ".skip-to-content",
            ".screen-reader-text",
            // Social and sharing
            ".social",
            ".share",
            ".sharing",
            ".social-links",
            // Advertisements
            ".ad",
            ".ads",
            ".advertisement",
            ".banner",
            "[class*='ad-']",
            "[id*='ad-']",
            // Comments
            ".comments",
            ".comment-section",
            "#comments",
            // Related content that's often noisy
            ".related",
            ".recommended",
            ".suggestions",
            ".more-stories",
            // Cookie notices and popups
            ".cookie",
            ".popup",
            ".modal",
            ".overlay",
            // Search boxes
            ".search",
            ".search-box",
            ".search-form",
        ];

        // Remove elements by parsing fresh each time to avoid stale references
        for selector_str in &unwanted_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                // Collect elements to remove
                let elements_to_remove: Vec<_> =
                    doc.select(&selector).map(|el| el.html()).collect();

                // Remove each element by string replacement
                for element_html in elements_to_remove {
                    html = html.replace(&element_html, "");
                }

                // Re-parse to get clean DOM
                doc = Html::parse_document(&html);
            }
        }

        // Additional cleanup for elements with specific text content
        self.remove_elements_by_text_content(&mut html);

        html
    }

    fn remove_elements_by_text_content(&self, html: &mut String) {
        let doc = Html::parse_document(html);
        let all_elements = Selector::parse("*").unwrap();

        let boilerplate_texts = [
            "skip to main content",
            "skip to content",
            "skip navigation",
            "toggle navigation",
            "menu toggle",
            "search this site",
            "table of contents",
            "back to top",
            "scroll to top",
        ];

        for element in doc.select(&all_elements) {
            let text = element.text().collect::<String>().to_lowercase();
            let trimmed = text.trim();

            // Remove elements that contain only boilerplate text
            if boilerplate_texts
                .iter()
                .any(|&pattern| trimmed.contains(pattern))
                && trimmed.len() < 50
            {
                *html = html.replace(&element.html(), "");
            }

            // Remove copyright notices
            if trimmed.starts_with("©") || trimmed.starts_with("copyright") {
                *html = html.replace(&element.html(), "");
            }
        }
    }

    fn find_main_content(&self, document: &Html) -> Option<String> {
        // Enhanced content extraction for AI assistance - prioritize high-value content
        let content_selectors = [
            &self.article_selector,
            &self.main_content_selector,
            &self.content_selector,
        ];

        for selector in &content_selectors {
            if let Some(element) = document.select(selector).next() {
                let content = self.filter_ai_relevant_content(&element, document);
                if !content.trim().is_empty() && content.len() > 100 {
                    return Some(content);
                }
            }
        }

        // Try additional common content selectors
        let fallback_selectors = [
            "#content",
            "#main-content",
            ".main-content",
            ".content-wrapper",
            ".post-content",
            ".entry-content",
            ".markdown-body",
            ".documentation",
        ];

        for selector_str in &fallback_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let content = self.filter_ai_relevant_content(&element, document);
                    if !content.trim().is_empty() && content.len() > 100 {
                        return Some(content);
                    }
                }
            }
        }

        None
    }

    fn filter_ai_relevant_content(
        &self,
        element: &scraper::ElementRef,
        _document: &Html,
    ) -> String {
        // Create a filtered version that removes navigation and focuses on valuable content for AI
        let mut html = element.html();
        let doc = Html::parse_document(&html);

        // Remove navigation elements using our selectors
        let unwanted_selectors = [
            &self.nav_selector,
            &self.footer_selector,
            &self.header_selector,
            &self.sidebar_selector,
            &self.breadcrumb_selector,
            &self.menu_selector,
        ];

        for selector in &unwanted_selectors {
            let elements_to_remove: Vec<_> = doc.select(selector).map(|el| el.html()).collect();
            for element_html in elements_to_remove {
                html = html.replace(&element_html, "");
            }
        }

        // Additional cleanup for AI-focused content
        let doc = Html::parse_document(&html);
        let mut valuable_sections = Vec::new();

        // Prioritize sections with code examples and explanations
        let section_selector = Selector::parse("section, div, article").unwrap();
        for section in doc.select(&section_selector) {
            let section_text = section.text().collect::<String>();
            let section_html = section.html();

            // Include sections that contain code or substantial explanatory content
            if section_html.contains("<pre")
                || section_html.contains("<code")
                || section_html.contains("class=\"highlight\"")
                || section_text.to_lowercase().contains("example")
                || section_text.to_lowercase().contains("usage")
                || section_text.to_lowercase().contains("api")
                || (section_text.len() > 200
                    && section_text.chars().filter(|c| c.is_alphabetic()).count()
                        > section_text.len() / 3)
            {
                valuable_sections.push(section_html);
            }
        }

        // If we found valuable sections, use them; otherwise return filtered HTML
        if !valuable_sections.is_empty() {
            valuable_sections.join("\n")
        } else {
            html
        }
    }

    fn extract_code_blocks(&self, document: &Html) -> Vec<CodeBlock> {
        let mut code_blocks = Vec::new();

        // Enhanced code block extraction for AI assistance
        let code_selector =
            Selector::parse("pre code, pre, .highlight, .codehilite, .code-block").unwrap();
        for element in document.select(&code_selector) {
            let code = element.text().collect::<String>();

            // Skip very short code snippets that aren't useful for AI assistance
            if code.trim().len() < 10 {
                continue;
            }

            let language = self.detect_code_language(&element, &code);
            let context = self.extract_code_context(&element);
            let usage_example = self.is_usage_example(&element);
            let api_reference = self.is_api_reference(&element);

            code_blocks.push(CodeBlock {
                code: code.trim().to_string(),
                language,
                context: if context.is_empty() {
                    None
                } else {
                    Some(context)
                },
                usage_example,
                api_reference,
            });
        }

        code_blocks
    }

    fn detect_code_language(
        &self,
        element: &scraper::ElementRef,
        code_text: &str,
    ) -> Option<String> {
        // Try class attribute first (most reliable)
        if let Some(class) = element.value().attr("class") {
            if let Some(lang) = class.split("language-").nth(1) {
                return Some(lang.split_whitespace().next()?.to_string());
            }
            if let Some(lang) = class.split("lang-").nth(1) {
                return Some(lang.split_whitespace().next()?.to_string());
            }
            if let Some(lang) = class.split("highlight-").nth(1) {
                return Some(lang.split_whitespace().next()?.to_string());
            }
        }

        // Try data attributes
        if let Some(lang) = element.value().attr("data-lang") {
            return Some(lang.to_string());
        }
        if let Some(lang) = element.value().attr("data-language") {
            return Some(lang.to_string());
        }

        // Heuristic detection for common patterns (important for AI assistance)
        let code_lower = code_text.to_lowercase();
        if code_lower.contains("fn main()")
            || code_lower.contains("use std::")
            || code_lower.contains("cargo ")
        {
            Some("rust".to_string())
        } else if code_lower.contains("def ")
            || code_lower.contains("import ")
            || code_lower.contains("pip ")
        {
            Some("python".to_string())
        } else if code_lower.contains("function ")
            || code_lower.contains("const ")
            || code_lower.contains("npm ")
            || code_lower.contains("yarn ")
        {
            Some("javascript".to_string())
        } else if code_lower.contains("public class") || code_lower.contains("import java") {
            Some("java".to_string())
        } else if code_lower.contains("curl ")
            || code_lower.contains("wget ")
            || code_lower.contains("sudo ")
        {
            Some("bash".to_string())
        } else if code_lower.contains("select ")
            || code_lower.contains("insert ")
            || code_lower.contains("create table")
        {
            Some("sql".to_string())
        } else if code_lower.contains("<!doctype") || code_lower.contains("<html") {
            Some("html".to_string())
        } else if code_lower.contains("interface ")
            || code_lower.contains("type ") && code_lower.contains("=")
        {
            Some("typescript".to_string())
        } else {
            None
        }
    }

    fn extract_code_context(&self, element: &scraper::ElementRef) -> String {
        let mut context_parts = Vec::new();

        // Look for preceding explanatory text in parent elements
        let mut current = element.parent();
        let mut depth = 0;

        while let Some(parent) = current {
            if depth > 2 {
                break;
            } // Don't go too far up

            if let Some(parent_elem) = scraper::ElementRef::wrap(parent) {
                // Look for headings that provide context
                let heading_selector = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
                for heading in parent_elem.select(&heading_selector) {
                    let heading_text = heading.text().collect::<String>().trim().to_string();
                    if !heading_text.is_empty() && heading_text.len() < 100 {
                        context_parts.push(format!("Section: {}", heading_text));
                        break; // Only take the first relevant heading
                    }
                }

                // Look for preceding paragraphs with explanatory text
                let p_selector = Selector::parse("p").unwrap();
                for para in parent_elem.select(&p_selector) {
                    let para_text = para.text().collect::<String>().trim().to_string();
                    if para_text.len() > 30 && para_text.len() < 300 {
                        // Check if this paragraph explains the code
                        let para_lower = para_text.to_lowercase();
                        if para_lower.contains("example")
                            || para_lower.contains("usage")
                            || para_lower.contains("how to")
                            || para_lower.contains("following")
                            || para_lower.contains("this code")
                            || para_lower.contains("you can")
                        {
                            context_parts.push(para_text);
                            if context_parts.len() >= 2 {
                                break;
                            } // Limit context size
                        }
                    }
                }
            }

            current = parent.parent();
            depth += 1;
        }

        context_parts.join(" | ")
    }

    fn is_usage_example(&self, element: &scraper::ElementRef) -> bool {
        // Check surrounding text for usage example indicators
        let surrounding_text = self.get_surrounding_text(element).to_lowercase();

        surrounding_text.contains("example")
            || surrounding_text.contains("usage")
            || surrounding_text.contains("how to")
            || surrounding_text.contains("getting started")
            || surrounding_text.contains("quick start")
            || surrounding_text.contains("tutorial")
            || surrounding_text.contains("demo")
    }

    fn is_api_reference(&self, element: &scraper::ElementRef) -> bool {
        // Check if this appears to be API reference documentation
        let surrounding_text = self.get_surrounding_text(element).to_lowercase();

        surrounding_text.contains("api")
            || surrounding_text.contains("reference")
            || surrounding_text.contains("method")
            || surrounding_text.contains("function")
            || surrounding_text.contains("parameter")
            || surrounding_text.contains("return")
            || surrounding_text.contains("endpoint")
    }

    fn get_surrounding_text(&self, element: &scraper::ElementRef) -> String {
        // Get text from parent and sibling elements for context analysis
        let mut text_parts = Vec::new();

        if let Some(parent) = element.parent().and_then(scraper::ElementRef::wrap) {
            let parent_text = parent.text().collect::<String>();
            if parent_text.len() < 500 {
                // Avoid very large text blocks
                text_parts.push(parent_text);
            }
        }

        text_parts.join(" ")
    }

    fn clean_markdown_advanced(&self, markdown: &str) -> String {
        let mut cleaned = markdown.to_string();

        // Remove excessive blank lines using string operations instead of regex
        while cleaned.contains("\n\n\n") {
            cleaned = cleaned.replace("\n\n\n", "\n\n");
        }

        // Remove navigation-like patterns that made it through
        let nav_patterns = [
            "| |", // Empty table cells
            "* |", // Navigation bullets
            "Navigation",
            "Table of Contents",
            "Skip to",
            "Toggle",
            "Menu",
            "index | modules | next | previous |",
        ];

        // Filter lines to remove navigation patterns
        let lines: Vec<&str> = cleaned.lines().collect();
        let filtered_lines: Vec<&str> = lines
            .into_iter()
            .filter(|line| {
                let trimmed = line.trim();

                // Keep lines that have substantial content
                if trimmed.len() < 3 {
                    return false;
                }

                // Check for navigation patterns
                let lower_line = trimmed.to_lowercase();
                for pattern in &nav_patterns {
                    if lower_line.contains(&pattern.to_lowercase()) && trimmed.len() < 50 {
                        return false;
                    }
                }

                // Skip lines that are mostly punctuation
                let punct_count = trimmed.chars().filter(|c| c.is_ascii_punctuation()).count();
                let alpha_count = trimmed.chars().filter(|c| c.is_alphabetic()).count();

                // Keep if more alphabetic than punctuation, or if it's a code line
                alpha_count > punct_count
                    || trimmed.contains("def ")
                    || trimmed.contains("function")
                    || trimmed.contains("class ")
            })
            .collect();

        cleaned = filtered_lines.join("\n");

        // Clean up code blocks - simple string replacement
        cleaned = cleaned.replace("``` \n", "```\n");
        cleaned = cleaned.replace("```  \n", "```\n");

        // Final cleanup - remove excessive blank lines again
        while cleaned.contains("\n\n\n") {
            cleaned = cleaned.replace("\n\n\n", "\n\n");
        }

        // Trim whitespace
        cleaned.trim().to_string()
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
    pub context: Option<String>, // Surrounding explanatory text for AI understanding
    pub usage_example: bool,     // Whether this appears to be a usage example
    pub api_reference: bool,     // Whether this is API documentation code
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
