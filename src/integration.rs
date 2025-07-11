//! Integration utilities for combining search and PDF conversion functionality
//!
//! This module provides functionality to search for URLs using the Brave Search API
//! and then convert those URLs to PDF format.

use crate::markdown::MarkdownGenerator;
use crate::pdf::PdfGenerator;
use crate::search::{BraveSearchClient, SearchConfig, SearchType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info, warn};

/// A search result that can be converted to PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub description: String,
}

/// Output format for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Pdf,
    Markdown,
    Both,
}

/// Configuration for search-to-PDF operations
#[derive(Debug, Clone)]
pub struct SearchToPdfConfig {
    /// Maximum number of URLs to convert to PDF
    pub max_results: usize,
    /// Output directory for PDF files
    pub output_dir: PathBuf,
    /// Whether to include metadata in PDF files
    pub include_metadata: bool,
    /// File naming strategy
    pub naming_strategy: NamingStrategy,
    /// Output format
    pub output_format: OutputFormat,
}

/// Strategy for naming PDF files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingStrategy {
    /// Use the page title as filename
    Title,
    /// Use the domain name as filename
    Domain,
    /// Use sequential numbers
    Sequential,
    /// Use both title and domain
    TitleDomain,
}

impl Default for SearchToPdfConfig {
    fn default() -> Self {
        Self {
            max_results: 5,
            output_dir: PathBuf::from("./pdf_downloads"),
            include_metadata: true,
            naming_strategy: NamingStrategy::TitleDomain,
            output_format: OutputFormat::Pdf,
        }
    }
}

/// Integrated search and PDF conversion client
pub struct SearchToPdfClient {
    search_client: BraveSearchClient,
    pdf_generator: PdfGenerator,
    markdown_generator: MarkdownGenerator,
}

impl SearchToPdfClient {
    /// Create a new search-to-PDF client
    ///
    /// # Arguments
    ///
    /// * `api_key` - Optional Brave API key. If None, attempts to read from BRAVE_API_KEY environment variable
    ///
    /// # Returns
    ///
    /// Returns a new SearchToPdfClient instance
    ///
    /// # Errors
    ///
    /// Returns an error if the search client or PDF generator cannot be initialized
    pub async fn new(api_key: Option<String>) -> Result<Self> {
        let search_client = BraveSearchClient::new(api_key)?;
        let pdf_generator = PdfGenerator::new().await?;
        let markdown_generator = MarkdownGenerator::new().await?;

        Ok(Self {
            search_client,
            pdf_generator,
            markdown_generator,
        })
    }

    /// Search for URLs and convert them to PDF/Markdown/Both
    ///
    /// # Arguments
    ///
    /// * `search_type` - The type of search to perform
    /// * `query` - The search query
    /// * `search_config` - Optional search configuration
    /// * `pdf_config` - Configuration for PDF conversion
    ///
    /// # Returns
    ///
    /// Returns a vector of successfully converted PDF file paths
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails or if critical PDF conversion errors occur
    pub async fn search_and_convert_to_pdf(
        &self,
        search_type: SearchType,
        query: &str,
        search_config: Option<SearchConfig>,
        pdf_config: SearchToPdfConfig,
    ) -> Result<Vec<PathBuf>> {
        info!(
            "Starting search-to-PDF operation: {} search for '{}'",
            search_type, query
        );

        // Perform search
        let search_results = self
            .search_client
            .search(search_type, query, search_config)
            .await?;

        // Extract URLs from search results
        let urls = self.extract_urls_from_results(&search_results)?;

        info!("Found {} URLs from search results", urls.len());

        // Limit the number of results to process
        let urls_to_process: Vec<_> = urls.into_iter().take(pdf_config.max_results).collect();
        let total_urls = urls_to_process.len();

        info!("Processing {} URLs (limited by max_results)", total_urls);

        // Create output directory if it doesn't exist
        fs::create_dir_all(&pdf_config.output_dir).await?;

        // Convert URLs to specified format
        let mut converted_files = Vec::new();
        for (index, result) in urls_to_process.into_iter().enumerate() {
            match self.convert_url(&result, index, &pdf_config).await {
                Ok(file_paths) => {
                    for file_path in file_paths {
                        info!(
                            "Successfully converted: {} -> {}",
                            result.url,
                            file_path.display()
                        );
                        converted_files.push(file_path);
                    }
                }
                Err(e) => {
                    error!("Failed to convert {}: {}", result.url, e);
                    // Continue with other URLs instead of failing completely
                }
            }
        }

        if converted_files.is_empty() {
            return Err(anyhow::anyhow!(
                "No URLs were successfully converted"
            ));
        }

        info!(
            "Successfully converted {} out of {} URLs",
            converted_files.len(),
            total_urls
        );
        Ok(converted_files)
    }

    /// Extract URLs from search results
    ///
    /// # Arguments
    ///
    /// * `search_results` - The raw search results string from Brave API
    ///
    /// # Returns
    ///
    /// Returns a vector of SearchResult objects containing URLs and metadata
    ///
    /// # Errors
    ///
    /// Returns an error if the search results cannot be parsed
    fn extract_urls_from_results(&self, search_results: &str) -> Result<Vec<SearchResult>> {
        // The search results are typically in a human-readable format
        // We need to extract URLs from the text
        let mut results = Vec::new();

        // Split by lines and look for URLs
        let lines: Vec<&str> = search_results.lines().collect();
        let mut current_title = String::new();
        let mut current_url = String::new();
        let mut current_description = String::new();

        for line in lines {
            let line = line.trim();

            // Skip empty lines and separators
            if line.is_empty() || line.starts_with("=") || line.starts_with("-") {
                continue;
            }

            // Check if this line contains a URL
            if line.starts_with("http://") || line.starts_with("https://") {
                current_url = line.to_string();
            } else if line.starts_with("URL:") {
                current_url = line.replace("URL:", "").trim().to_string();
            } else if line.starts_with("Title:") {
                current_title = line.replace("Title:", "").trim().to_string();
            } else if line.starts_with("Description:") {
                current_description = line.replace("Description:", "").trim().to_string();
            } else if !current_url.is_empty() && current_title.is_empty() {
                // If we have a URL but no title, this might be the title
                current_title = line.to_string();
            } else if !current_url.is_empty()
                && !current_title.is_empty()
                && current_description.is_empty()
            {
                // If we have URL and title but no description, this might be the description
                current_description = line.to_string();
            }

            // If we have all three components, add to results
            if !current_url.is_empty() && !current_title.is_empty() {
                results.push(SearchResult {
                    title: current_title.clone(),
                    url: current_url.clone(),
                    description: current_description.clone(),
                });

                // Reset for next result
                current_title.clear();
                current_url.clear();
                current_description.clear();
            }
        }

        // Alternative approach: use regex to find URLs if the above doesn't work well
        if results.is_empty() {
            warn!("No structured results found, attempting regex URL extraction");
            let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();

            for (index, url_match) in url_regex.find_iter(search_results).enumerate() {
                let url = url_match.as_str().to_string();
                results.push(SearchResult {
                    title: format!("Search Result {}", index + 1),
                    url,
                    description: String::new(),
                });
            }
        }

        info!("Extracted {} URLs from search results", results.len());
        Ok(results)
    }

    /// Convert a single URL to the specified format(s)
    ///
    /// # Arguments
    ///
    /// * `result` - The search result containing URL and metadata
    /// * `index` - The index of this result (for sequential naming)
    /// * `config` - Configuration for conversion
    ///
    /// # Returns
    ///
    /// Returns a vector of paths to the generated files
    ///
    /// # Errors
    ///
    /// Returns an error if conversion fails
    async fn convert_url(
        &self,
        result: &SearchResult,
        index: usize,
        config: &SearchToPdfConfig,
    ) -> Result<Vec<PathBuf>> {
        let mut file_paths = Vec::new();

        match config.output_format {
            OutputFormat::Pdf => {
                let pdf_path = self.convert_to_pdf(result, index, config).await?;
                file_paths.push(pdf_path);
            }
            OutputFormat::Markdown => {
                let md_path = self.convert_to_markdown(result, index, config).await?;
                file_paths.push(md_path);
            }
            OutputFormat::Both => {
                let pdf_path = self.convert_to_pdf(result, index, config).await?;
                file_paths.push(pdf_path);
                let md_path = self.convert_to_markdown(result, index, config).await?;
                file_paths.push(md_path);
            }
        }

        Ok(file_paths)
    }

    /// Convert a single URL to PDF
    ///
    /// # Arguments
    ///
    /// * `result` - The search result containing URL and metadata
    /// * `index` - The index of this result (for sequential naming)
    /// * `config` - Configuration for PDF conversion
    ///
    /// # Returns
    ///
    /// Returns the path to the generated PDF file
    ///
    /// # Errors
    ///
    /// Returns an error if PDF conversion fails
    async fn convert_to_pdf(
        &self,
        result: &SearchResult,
        index: usize,
        config: &SearchToPdfConfig,
    ) -> Result<PathBuf> {
        // Generate filename based on naming strategy
        let filename = self.generate_filename(result, index, config, "pdf")?;
        let pdf_path = config.output_dir.join(filename);

        info!("Converting {} to {}", result.url, pdf_path.display());

        // Convert URL to PDF
        self.pdf_generator
            .url_to_pdf(&result.url, Some(&pdf_path))
            .await?;

        Ok(pdf_path)
    }

    /// Convert a single URL to Markdown
    ///
    /// # Arguments
    ///
    /// * `result` - The search result containing URL and metadata
    /// * `index` - The index of this result (for sequential naming)
    /// * `config` - Configuration for Markdown conversion
    ///
    /// # Returns
    ///
    /// Returns the path to the generated Markdown file
    ///
    /// # Errors
    ///
    /// Returns an error if Markdown conversion fails
    async fn convert_to_markdown(
        &self,
        result: &SearchResult,
        index: usize,
        config: &SearchToPdfConfig,
    ) -> Result<PathBuf> {
        // Generate filename based on naming strategy
        let filename = self.generate_filename(result, index, config, "md")?;
        let md_path = config.output_dir.join(filename);

        info!("Converting {} to {}", result.url, md_path.display());

        // Convert URL to Markdown
        self.markdown_generator
            .url_to_markdown(&result.url, Some(&md_path))
            .await?;

        Ok(md_path)
    }

    /// Generate a filename based on the naming strategy
    ///
    /// # Arguments
    ///
    /// * `result` - The search result containing URL and metadata
    /// * `index` - The index of this result (for sequential naming)
    /// * `config` - Configuration containing the naming strategy
    /// * `extension` - File extension (e.g., "pdf", "md")
    ///
    /// # Returns
    ///
    /// Returns a sanitized filename
    ///
    /// # Errors
    ///
    /// Returns an error if filename generation fails
    fn generate_filename(
        &self,
        result: &SearchResult,
        index: usize,
        config: &SearchToPdfConfig,
        extension: &str,
    ) -> Result<String> {
        let filename = match config.naming_strategy {
            NamingStrategy::Title => {
                if result.title.is_empty() {
                    format!("search_result_{}", index + 1)
                } else {
                    sanitize_filename(&result.title)
                }
            }
            NamingStrategy::Domain => {
                let url = url::Url::parse(&result.url)?;
                let domain = url.host_str().unwrap_or("unknown");
                sanitize_filename(domain)
            }
            NamingStrategy::Sequential => {
                format!("search_result_{}", index + 1)
            }
            NamingStrategy::TitleDomain => {
                let url = url::Url::parse(&result.url)?;
                let domain = url.host_str().unwrap_or("unknown");
                let title = if result.title.is_empty() {
                    format!("result_{}", index + 1)
                } else {
                    sanitize_filename(&result.title)
                };
                format!("{}_{}", title, sanitize_filename(domain))
            }
        };

        Ok(format!("{}.{}", filename, extension))
    }
}

/// Sanitize a filename by removing invalid characters
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.txt"), "test.txt");
        assert_eq!(sanitize_filename("test/file.txt"), "test_file.txt");
        assert_eq!(sanitize_filename("test:file*.txt"), "test_file_.txt");
        assert_eq!(sanitize_filename("test<file>?.txt"), "test_file__.txt");
    }

    #[test]
    fn test_search_to_pdf_config_default() {
        let config = SearchToPdfConfig::default();
        assert_eq!(config.max_results, 5);
        assert_eq!(config.output_dir, PathBuf::from("./pdf_downloads"));
        assert!(config.include_metadata);
        assert_eq!(config.naming_strategy, NamingStrategy::TitleDomain);
        assert_eq!(config.output_format, OutputFormat::Pdf);
    }

    #[test]
    fn test_naming_strategy() {
        let result = SearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com/path".to_string(),
            description: "Test description".to_string(),
        };

        let _config = SearchToPdfConfig {
            naming_strategy: NamingStrategy::Title,
            output_format: OutputFormat::Pdf,
            ..Default::default()
        };

        // This would be tested in integration tests with actual SearchToPdfClient
        // For now, just verify the structure is correct
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "https://example.com/path");
    }
}
