//! # Webpage Save - URL to PDF/Markdown Conversion Tool
//!
//! A Rust library and CLI tool for converting URLs to PDF documents using headless Chrome or Markdown format.
//!
//! ## Features
//!
//! - Convert web pages to PDF format
//! - Convert web pages to Markdown format
//! - Perform Brave searches (web, news, local)
//! - Integrated search-to-PDF functionality
//! - Command-line interface for easy usage
//! - Asynchronous processing for better performance
//! - Proper error handling and logging
//!
//! ## Usage
//!
//! ```bash
//! # Convert a URL to PDF
//! webpage-save https://example.com -o output.pdf
//!
//! # Convert a URL to Markdown
//! webpage-save https://example.com -f markdown -o output.md
//!
//! # Perform a Brave search
//! webpage-save search web "rust programming"
//! webpage-save search news "latest tech news"
//! webpage-save search local "coffee shops near me"
//!
//! # Search and convert results to PDF
//! webpage-save search-to-pdf web "rust programming" --max-results 3
//! webpage-save search-to-pdf news "latest tech news" --output-dir ./news_pdfs
//! webpage-save search-to-pdf local "coffee shops near me" --naming title
//! ```

/// PDF generation utilities for converting URLs and HTML to PDF format
pub mod pdf;

/// Markdown generation utilities for converting URLs and HTML to Markdown format
pub mod markdown;

/// Brave search utilities for web, news, and local searches
pub mod search;

/// Integration utilities for combining search and PDF conversion functionality
pub mod integration;
