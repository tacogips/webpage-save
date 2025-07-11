//! PDF generation utilities for converting URLs and HTML to PDF format
//!
//! This module provides functionality to convert web pages to PDF documents
//! using headless Chrome browser automation.

use anyhow::Result;
use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::{Browser, LaunchOptions};
use std::path::Path;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::fs;
use url::Url;

/// PDF generator that uses headless Chrome to convert URLs and HTML to PDF
pub struct PdfGenerator {
    browser: Browser,
}

impl PdfGenerator {
    /// Create a new PDF generator instance
    ///
    /// # Errors
    ///
    /// Returns an error if the browser cannot be launched
    pub async fn new() -> Result<Self> {
        let browser = Browser::new(
            LaunchOptions::default_builder()
                .headless(true)
                .sandbox(false)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build launch options: {}", e))?,
        )?;

        Ok(Self { browser })
    }

    /// Convert a URL to PDF
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to convert to PDF
    /// * `output_path` - Optional output file path. If None, returns PDF data without saving
    ///
    /// # Returns
    ///
    /// Returns the PDF data as bytes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL is invalid or cannot be accessed
    /// - The browser fails to load the page
    /// - PDF generation fails
    /// - File I/O operations fail
    pub async fn url_to_pdf(&self, url: &str, output_path: Option<&Path>) -> Result<Vec<u8>> {
        // Validate URL
        let parsed_url = Url::parse(url)?;
        if !matches!(parsed_url.scheme(), "http" | "https" | "file") {
            return Err(anyhow::anyhow!(
                "Only HTTP, HTTPS, and file URLs are supported"
            ));
        }

        // Create new tab
        let tab = self.browser.new_tab()?;

        // Navigate to URL
        tab.navigate_to(url)?;

        // Wait for page to load
        tab.wait_until_navigated()?;

        // Wait a bit more for dynamic content to load
        tokio::time::sleep(Duration::from_millis(2000)).await;

        // Configure PDF options
        let pdf_options = PrintToPdfOptions {
            landscape: Some(false),
            display_header_footer: Some(false),
            print_background: Some(true),
            scale: Some(1.0),
            paper_width: Some(8.27),  // A4 width in inches
            paper_height: Some(11.7), // A4 height in inches
            margin_top: Some(0.4),
            margin_bottom: Some(0.4),
            margin_left: Some(0.4),
            margin_right: Some(0.4),
            page_ranges: None,
            ignore_invalid_page_ranges: Some(false),
            header_template: None,
            footer_template: None,
            prefer_css_page_size: Some(false),
            transfer_mode: None,
            generate_document_outline: Some(false),
            generate_tagged_pdf: Some(false),
        };

        // Generate PDF
        let pdf_data = tab.print_to_pdf(Some(pdf_options))?;

        // Save to file if output path is provided
        if let Some(path) = output_path {
            fs::write(path, &pdf_data).await?;
        }

        Ok(pdf_data)
    }

    /// Convert HTML content to PDF
    ///
    /// # Arguments
    ///
    /// * `html_content` - The HTML content to convert to PDF
    /// * `output_path` - Optional output file path. If None, returns PDF data without saving
    ///
    /// # Returns
    ///
    /// Returns the PDF data as bytes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The browser fails to load the HTML content
    /// - PDF generation fails
    /// - File I/O operations fail
    pub async fn html_to_pdf(
        &self,
        html_content: &str,
        output_path: Option<&Path>,
    ) -> Result<Vec<u8>> {
        // Create a temporary HTML file
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path();
        fs::write(temp_path, html_content).await?;

        // Convert file URL to PDF
        let file_url = format!("file://{}", temp_path.display());
        self.url_to_pdf(&file_url, output_path).await
    }
}

impl Drop for PdfGenerator {
    fn drop(&mut self) {
        // Browser cleanup is handled automatically
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_html_to_pdf() -> Result<()> {
        let generator = PdfGenerator::new().await?;
        let html = r#"
            <html>
            <head>
                <title>Test PDF</title>
            </head>
            <body>
                <h1>Test PDF</h1>
                <p>This is a test PDF generated from HTML.</p>
            </body>
            </html>
        "#;

        let pdf_data = generator.html_to_pdf(html, None).await?;
        assert!(!pdf_data.is_empty());
        assert!(pdf_data.starts_with(b"%PDF"));
        Ok(())
    }

    #[tokio::test]
    async fn test_url_to_pdf_invalid_url() -> Result<()> {
        let generator = PdfGenerator::new().await?;
        let result = generator.url_to_pdf("invalid-url", None).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_url_to_pdf_with_file() -> Result<()> {
        let generator = PdfGenerator::new().await?;
        let temp_file = NamedTempFile::new()?;
        let html = r#"
            <html>
            <head>
                <title>Test PDF File</title>
            </head>
            <body>
                <h1>Test PDF File</h1>
                <p>This PDF should be saved to a file.</p>
            </body>
            </html>
        "#;

        let pdf_data = generator.html_to_pdf(html, Some(temp_file.path())).await?;
        assert!(!pdf_data.is_empty());

        // Check that file was created
        let file_content = std::fs::read(temp_file.path())?;
        assert_eq!(pdf_data, file_content);
        Ok(())
    }

    #[tokio::test]
    async fn test_unsupported_scheme() -> Result<()> {
        let generator = PdfGenerator::new().await?;
        let result = generator.url_to_pdf("ftp://example.com", None).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_file_url_support() -> Result<()> {
        let generator = PdfGenerator::new().await?;
        let html = r#"<html><body><h1>File URL Test</h1></body></html>"#;
        let temp_file = NamedTempFile::new()?;
        std::fs::write(temp_file.path(), html)?;

        let file_url = format!("file://{}", temp_file.path().display());
        let result = generator.url_to_pdf(&file_url, None).await;
        assert!(result.is_ok());
        Ok(())
    }
}
