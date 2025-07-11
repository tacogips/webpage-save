//! Brave search utilities for performing web, news, and local searches
//!
//! This module provides functionality to perform searches using the Brave Search API
//! through the bravesearch-mcp crate.

use anyhow::Result;
use bravesearch_mcp::tools::BraveSearchRouter;
use serde::{Deserialize, Serialize};
use std::env;

/// Search types supported by the Brave Search API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    /// Web search for general queries
    Web,
    /// News search for current events
    News,
    /// Local search for businesses and places
    Local,
}

impl std::fmt::Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::Web => write!(f, "web"),
            SearchType::News => write!(f, "news"),
            SearchType::Local => write!(f, "local"),
        }
    }
}

impl std::str::FromStr for SearchType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "web" => Ok(SearchType::Web),
            "news" => Ok(SearchType::News),
            "local" => Ok(SearchType::Local),
            _ => Err(anyhow::anyhow!("Invalid search type: {}", s)),
        }
    }
}

/// Configuration for search operations
#[derive(Debug, Clone, Default)]
pub struct SearchConfig {
    /// Number of results to return
    pub count: Option<usize>,
    /// Pagination offset
    pub offset: Option<usize>,
    /// Country code for news/local searches
    pub country: Option<String>,
    /// Language code for news searches
    pub language: Option<String>,
    /// Freshness filter for news searches (h, d, w, m, y)
    pub freshness: Option<String>,
}

/// Brave search client for performing various types of searches
pub struct BraveSearchClient {
    router: BraveSearchRouter,
}

impl BraveSearchClient {
    /// Create a new Brave search client
    ///
    /// # Arguments
    ///
    /// * `api_key` - Optional API key. If None, attempts to read from BRAVE_API_KEY environment variable
    ///
    /// # Returns
    ///
    /// Returns a new BraveSearchClient instance
    ///
    /// # Errors
    ///
    /// Returns an error if no API key is provided and BRAVE_API_KEY environment variable is not set
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let key = match api_key {
            Some(key) => key,
            None => env::var("BRAVE_API_KEY")
                .map_err(|_| anyhow::anyhow!("BRAVE_API_KEY environment variable not set"))?,
        };

        let router = BraveSearchRouter::new(key);
        Ok(Self { router })
    }

    /// Perform a web search
    ///
    /// # Arguments
    ///
    /// * `query` - The search query
    /// * `config` - Optional search configuration
    ///
    /// # Returns
    ///
    /// Returns the search results as a formatted string
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails
    pub async fn web_search(&self, query: &str, config: Option<SearchConfig>) -> Result<String> {
        let config = config.unwrap_or_default();
        let result = self
            .router
            .brave_web_search(query.to_string(), config.count, config.offset)
            .await;

        if result.starts_with("Error:") {
            return Err(anyhow::anyhow!("Search failed: {}", result));
        }

        Ok(result)
    }

    /// Perform a news search
    ///
    /// # Arguments
    ///
    /// * `query` - The search query
    /// * `config` - Optional search configuration
    ///
    /// # Returns
    ///
    /// Returns the search results as a formatted string
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails
    pub async fn news_search(&self, query: &str, config: Option<SearchConfig>) -> Result<String> {
        let config = config.unwrap_or_default();
        let result = self
            .router
            .brave_news_search(
                query.to_string(),
                config.count,
                config.offset,
                config.country,
                config.language,
                config.freshness,
            )
            .await;

        if result.starts_with("Error:") {
            return Err(anyhow::anyhow!("Search failed: {}", result));
        }

        Ok(result)
    }

    /// Perform a local search
    ///
    /// # Arguments
    ///
    /// * `query` - The search query
    /// * `config` - Optional search configuration
    ///
    /// # Returns
    ///
    /// Returns the search results as a formatted string
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails
    pub async fn local_search(&self, query: &str, config: Option<SearchConfig>) -> Result<String> {
        let config = config.unwrap_or_default();
        let result = self
            .router
            .brave_local_search(query.to_string(), config.count)
            .await;

        if result.starts_with("Error:") {
            return Err(anyhow::anyhow!("Search failed: {}", result));
        }

        Ok(result)
    }

    /// Perform a search based on the specified type
    ///
    /// # Arguments
    ///
    /// * `search_type` - The type of search to perform
    /// * `query` - The search query
    /// * `config` - Optional search configuration
    ///
    /// # Returns
    ///
    /// Returns the search results as a formatted string
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails
    pub async fn search(
        &self,
        search_type: SearchType,
        query: &str,
        config: Option<SearchConfig>,
    ) -> Result<String> {
        match search_type {
            SearchType::Web => self.web_search(query, config).await,
            SearchType::News => self.news_search(query, config).await,
            SearchType::Local => self.local_search(query, config).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_type_from_str() {
        assert_eq!("web".parse::<SearchType>().unwrap(), SearchType::Web);
        assert_eq!("news".parse::<SearchType>().unwrap(), SearchType::News);
        assert_eq!("local".parse::<SearchType>().unwrap(), SearchType::Local);
        assert_eq!("WEB".parse::<SearchType>().unwrap(), SearchType::Web);
        assert!("invalid".parse::<SearchType>().is_err());
    }

    #[test]
    fn test_search_type_display() {
        assert_eq!(SearchType::Web.to_string(), "web");
        assert_eq!(SearchType::News.to_string(), "news");
        assert_eq!(SearchType::Local.to_string(), "local");
    }

    #[test]
    fn test_search_config_default() {
        let config = SearchConfig::default();
        assert!(config.count.is_none());
        assert!(config.offset.is_none());
        assert!(config.country.is_none());
        assert!(config.language.is_none());
        assert!(config.freshness.is_none());
    }

    #[tokio::test]
    async fn test_brave_search_client_creation() {
        // Test with explicit API key
        let client = BraveSearchClient::new(Some("test_key".to_string()));
        assert!(client.is_ok());

        // Test without API key and no environment variable
        unsafe {
            std::env::remove_var("BRAVE_API_KEY");
        }
        let client = BraveSearchClient::new(None);
        assert!(client.is_err());
    }

    #[tokio::test]
    async fn test_search_with_mock_api() {
        // This test would require a mock API key or actual API access
        // For now, we just test the client creation
        let client = BraveSearchClient::new(Some("test_key".to_string()));
        assert!(client.is_ok());
    }
}
