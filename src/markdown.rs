//! Markdown generation utilities for converting URLs and HTML to Markdown format
//!
//! This module provides functionality to convert web pages to Markdown documents
//! using HTML parsing and content extraction.

use anyhow::Result;
use reqwest::Client;
use select::document::Document;
use select::predicate::{Attr, Name};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use url::Url;

/// Markdown generator that fetches URLs and converts HTML to Markdown
pub struct MarkdownGenerator {
    client: Client,
}

impl MarkdownGenerator {
    /// Create a new Markdown generator instance
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created
    pub async fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("webpage-save-markdown-generator/1.0")
            .build()?;

        Ok(Self { client })
    }

    /// Convert a URL to Markdown
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to convert to Markdown
    /// * `output_path` - Optional output file path. If None, returns Markdown data without saving
    ///
    /// # Returns
    ///
    /// Returns the Markdown content as a String
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid or cannot be accessed
    /// - The HTTP request fails
    /// - HTML parsing fails
    /// - File I/O operations fail
    pub async fn url_to_markdown(&self, url: &str, output_path: Option<&Path>) -> Result<String> {
        // Validate URL
        let parsed_url = Url::parse(url)?;
        if !matches!(parsed_url.scheme(), "http" | "https") {
            return Err(anyhow::anyhow!("Only HTTP and HTTPS URLs are supported"));
        }

        // Fetch HTML content
        let response = self.client.get(url).send().await?;
        let html_content = response.text().await?;

        // Convert HTML to Markdown
        let markdown_content = self.html_to_markdown(&html_content, Some(url)).await?;

        // Save to file if output path is provided
        if let Some(path) = output_path {
            fs::write(path, &markdown_content).await?;
        }

        Ok(markdown_content)
    }

    /// Convert HTML content to Markdown
    ///
    /// # Arguments
    ///
    /// * `html_content` - The HTML content to convert to Markdown
    /// * `base_url` - Optional base URL for resolving relative links
    ///
    /// # Returns
    ///
    /// Returns the Markdown content as a String
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - HTML parsing fails
    /// - Markdown conversion fails
    pub async fn html_to_markdown(
        &self,
        html_content: &str,
        base_url: Option<&str>,
    ) -> Result<String> {
        // Extract main content from HTML
        let main_content = self.extract_main_content(html_content)?;

        // Convert HTML to Markdown using mdka
        let markdown_content = mdka::from_html(&main_content);

        // Add metadata header if base_url is provided
        let final_content = if let Some(url) = base_url {
            format!(
                "# {}\n\n*Source: [{}]({})*\n\n---\n\n{}",
                self.extract_title(html_content)
                    .unwrap_or_else(|| "Untitled".to_string()),
                url,
                url,
                markdown_content
            )
        } else {
            markdown_content
        };

        Ok(final_content)
    }

    /// Extract main content from HTML using various strategies
    ///
    /// # Arguments
    ///
    /// * `html_content` - The HTML content to extract from
    ///
    /// # Returns
    ///
    /// Returns the extracted HTML content as a String
    ///
    /// # Errors
    ///
    /// Returns an error if HTML parsing fails
    fn extract_main_content(&self, html_content: &str) -> Result<String> {
        let document = Document::from(html_content);

        // Try common content selectors in order of preference
        let tag_selectors = ["main", "article", "body"];
        let class_selectors = [
            "main-content",
            "content",
            "post-content",
            "entry-content",
            "article-content",
        ];
        let id_selectors = ["content"];

        // Try tag selectors first
        for &selector in &tag_selectors {
            if let Some(element) = document.find(Name(selector)).next() {
                return Ok(element.html());
            }
        }

        // Try class selectors
        for &class_name in &class_selectors {
            if let Some(element) = document.find(Attr("class", class_name)).next() {
                return Ok(element.html());
            }
        }

        // Try ID selectors
        for &id_name in &id_selectors {
            if let Some(element) = document.find(Attr("id", id_name)).next() {
                return Ok(element.html());
            }
        }

        // Fallback to body content
        if let Some(body) = document.find(Name("body")).next() {
            Ok(body.html())
        } else {
            // Last resort: return the entire document
            Ok(html_content.to_string())
        }
    }

    /// Extract title from HTML
    ///
    /// # Arguments
    ///
    /// * `html_content` - The HTML content to extract title from
    ///
    /// # Returns
    ///
    /// Returns the extracted title as an Option<String>
    fn extract_title(&self, html_content: &str) -> Option<String> {
        let document = Document::from(html_content);

        // Try various title selectors
        let tag_selectors = ["h1", "title"];
        let class_selectors = ["title", "post-title", "entry-title", "article-title"];

        // Try tag selectors first
        for &selector in &tag_selectors {
            if let Some(element) = document.find(Name(selector)).next() {
                let text = element.text().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        // Try Open Graph meta tag
        if let Some(element) = document.find(Attr("property", "og:title")).next() {
            if let Some(content) = element.attr("content") {
                return Some(content.to_string());
            }
        }

        // Try Twitter meta tag
        if let Some(element) = document.find(Attr("name", "twitter:title")).next() {
            if let Some(content) = element.attr("content") {
                return Some(content.to_string());
            }
        }

        // Try class selectors
        for &class_name in &class_selectors {
            if let Some(element) = document.find(Attr("class", class_name)).next() {
                let text = element.text().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_html_to_markdown() -> Result<()> {
        let generator = MarkdownGenerator::new().await?;
        let html = r#"
            <html>
            <head>
                <title>Test Markdown</title>
            </head>
            <body>
                <h1>Test Markdown</h1>
                <p>This is a test <strong>markdown</strong> generated from HTML.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
            </body>
            </html>
        "#;

        let markdown_content = generator.html_to_markdown(html, None).await?;
        assert!(!markdown_content.is_empty());
        assert!(markdown_content.contains("# Test Markdown"));
        assert!(markdown_content.contains("**markdown**"));
        Ok(())
    }

    #[tokio::test]
    async fn test_extract_main_content() -> Result<()> {
        let generator = MarkdownGenerator::new().await?;
        let html = r#"
            <html>
            <body>
                <header>Header content</header>
                <main>
                    <h1>Main Content</h1>
                    <p>This is the main content.</p>
                </main>
                <footer>Footer content</footer>
            </body>
            </html>
        "#;

        let main_content = generator.extract_main_content(html)?;
        assert!(main_content.contains("Main Content"));
        assert!(main_content.contains("main content"));
        assert!(!main_content.contains("Header content"));
        assert!(!main_content.contains("Footer content"));
        Ok(())
    }

    #[tokio::test]
    async fn test_extract_title() -> Result<()> {
        let generator = MarkdownGenerator::new().await?;
        let html = r#"
            <html>
            <head>
                <title>Page Title</title>
            </head>
            <body>
                <h1>Main Heading</h1>
                <p>Content</p>
            </body>
            </html>
        "#;

        let title = generator.extract_title(html);
        assert_eq!(title, Some("Main Heading".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_url_validation() -> Result<()> {
        let generator = MarkdownGenerator::new().await?;
        let result = generator.url_to_markdown("invalid-url", None).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_unsupported_scheme() -> Result<()> {
        let generator = MarkdownGenerator::new().await?;
        let result = generator.url_to_markdown("ftp://example.com", None).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_html_to_markdown_with_base_url() -> Result<()> {
        let generator = MarkdownGenerator::new().await?;
        let html = r#"
            <html>
            <head>
                <title>Test Page</title>
            </head>
            <body>
                <h1>Test Page</h1>
                <p>Test content</p>
            </body>
            </html>
        "#;

        let markdown_content = generator
            .html_to_markdown(html, Some("https://example.com"))
            .await?;
        assert!(markdown_content.contains("Source: [https://example.com](https://example.com)"));
        assert!(markdown_content.contains("# Test Page"));
        Ok(())
    }
}
