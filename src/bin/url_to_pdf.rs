//! Command-line tool for converting URLs to PDF and performing Brave searches
//!
//! This binary provides a command-line interface for converting web pages to PDF format
//! using headless Chrome and for performing web, news, and local searches using Brave Search API.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info};
use webpage_save::integration::{NamingStrategy, OutputFormat as IntegrationOutputFormat, SearchToPdfClient, SearchToPdfConfig};
use webpage_save::markdown::MarkdownGenerator;
use webpage_save::pdf::PdfGenerator;
use webpage_save::search::{BraveSearchClient, SearchConfig, SearchType};

#[derive(Parser)]
#[command(name = "webpage-save")]
#[command(about = "Convert URLs to PDF/Markdown using headless Chrome or perform Brave searches")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// URL to convert to PDF (when no subcommand is used)
    #[arg(value_name = "URL")]
    url: Option<String>,

    /// Output file path (optional, defaults to hostname.pdf or hostname.md)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Output format (pdf or markdown)
    #[arg(short, long, value_enum, default_value = "pdf")]
    format: OutputFormat,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Wait time in seconds before generating PDF (for dynamic content)
    #[arg(short, long, default_value = "2")]
    wait: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Perform a Brave search
    Search {
        /// Type of search to perform
        #[arg(value_enum)]
        search_type: SearchTypeArg,

        /// Search query
        query: String,

        /// Number of results to return
        #[arg(short, long)]
        count: Option<usize>,

        /// Pagination offset
        #[arg(short = 'o', long)]
        offset: Option<usize>,

        /// Country code for news/local searches
        #[arg(long)]
        country: Option<String>,

        /// Language code for news searches
        #[arg(short, long)]
        language: Option<String>,

        /// Freshness filter for news searches (h, d, w, m, y)
        #[arg(short, long)]
        freshness: Option<String>,

        /// Brave API key (optional, can also use BRAVE_API_KEY environment variable)
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Search and convert results to PDF/Markdown
    SearchToPdf {
        /// Type of search to perform
        #[arg(value_enum)]
        search_type: SearchTypeArg,

        /// Search query
        query: String,

        /// Maximum number of results to convert to PDF
        #[arg(short, long, default_value = "5")]
        max_results: usize,

        /// Output directory for PDF files
        #[arg(short, long, default_value = "./pdf_downloads")]
        output_dir: PathBuf,

        /// Output format (pdf, markdown, or both)
        #[arg(long, value_enum, default_value = "pdf")]
        format: OutputFormat,

        /// File naming strategy
        #[arg(long, value_enum, default_value = "title-domain")]
        naming: NamingStrategyArg,

        /// Number of search results to return
        #[arg(short, long)]
        count: Option<usize>,

        /// Pagination offset
        #[arg(long)]
        offset: Option<usize>,

        /// Country code for news/local searches
        #[arg(long)]
        country: Option<String>,

        /// Language code for news searches
        #[arg(short, long)]
        language: Option<String>,

        /// Freshness filter for news searches (h, d, w, m, y)
        #[arg(short, long)]
        freshness: Option<String>,

        /// Brave API key (optional, can also use BRAVE_API_KEY environment variable)
        #[arg(long)]
        api_key: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum OutputFormat {
    Pdf,
    Markdown,
    Both,
}

#[derive(clap::ValueEnum, Clone)]
enum SearchTypeArg {
    Web,
    News,
    Local,
}

impl From<SearchTypeArg> for SearchType {
    fn from(arg: SearchTypeArg) -> Self {
        match arg {
            SearchTypeArg::Web => SearchType::Web,
            SearchTypeArg::News => SearchType::News,
            SearchTypeArg::Local => SearchType::Local,
        }
    }
}

#[derive(clap::ValueEnum, Clone)]
enum NamingStrategyArg {
    Title,
    Domain,
    Sequential,
    #[value(name = "title-domain")]
    TitleDomain,
}

impl From<NamingStrategyArg> for NamingStrategy {
    fn from(arg: NamingStrategyArg) -> Self {
        match arg {
            NamingStrategyArg::Title => NamingStrategy::Title,
            NamingStrategyArg::Domain => NamingStrategy::Domain,
            NamingStrategyArg::Sequential => NamingStrategy::Sequential,
            NamingStrategyArg::TitleDomain => NamingStrategy::TitleDomain,
        }
    }
}

impl From<OutputFormat> for IntegrationOutputFormat {
    fn from(arg: OutputFormat) -> Self {
        match arg {
            OutputFormat::Pdf => IntegrationOutputFormat::Pdf,
            OutputFormat::Markdown => IntegrationOutputFormat::Markdown,
            OutputFormat::Both => IntegrationOutputFormat::Both,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt().with_env_filter("debug").init();
    } else {
        tracing_subscriber::fmt().with_env_filter("info").init();
    }

    match cli.command {
        Some(Commands::Search {
            search_type,
            query,
            count,
            offset,
            country,
            language,
            freshness,
            api_key,
        }) => {
            // Handle search command
            info!(
                "Performing {} search for: {}",
                SearchType::from(search_type.clone()),
                query
            );

            // Create search client
            let client = match BraveSearchClient::new(api_key) {
                Ok(client) => client,
                Err(e) => {
                    error!("Failed to initialize Brave search client: {}", e);
                    eprintln!("✗ Failed to initialize Brave search client: {}", e);
                    eprintln!(
                        "  Make sure to set BRAVE_API_KEY environment variable or use --api-key"
                    );
                    std::process::exit(1);
                }
            };

            // Create search configuration
            let config = SearchConfig {
                count,
                offset,
                country,
                language,
                freshness,
            };

            // Perform search
            match client
                .search(search_type.into(), &query, Some(config))
                .await
            {
                Ok(results) => {
                    println!("Search Results:");
                    println!("==============");
                    println!("{}", results);
                }
                Err(e) => {
                    error!("Search failed: {}", e);
                    eprintln!("✗ Search failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::SearchToPdf {
            search_type,
            query,
            max_results,
            output_dir,
            format,
            naming,
            count,
            offset,
            country,
            language,
            freshness,
            api_key,
        }) => {
            // Handle search-to-PDF command
            info!(
                "Performing {} search-to-PDF for: {} (max results: {})",
                SearchType::from(search_type.clone()),
                query,
                max_results
            );

            // Create search-to-PDF client
            let client = match SearchToPdfClient::new(api_key).await {
                Ok(client) => client,
                Err(e) => {
                    error!("Failed to initialize search-to-PDF client: {}", e);
                    eprintln!("✗ Failed to initialize search-to-PDF client: {}", e);
                    eprintln!(
                        "  Make sure to set BRAVE_API_KEY environment variable or use --api-key"
                    );
                    std::process::exit(1);
                }
            };

            // Create search configuration
            let search_config = SearchConfig {
                count,
                offset,
                country,
                language,
                freshness,
            };

            // Create PDF configuration
            let pdf_config = SearchToPdfConfig {
                max_results,
                output_dir,
                include_metadata: true,
                naming_strategy: naming.into(),
                output_format: format.into(),
            };

            // Perform search and convert to PDF
            match client
                .search_and_convert_to_pdf(
                    search_type.into(),
                    &query,
                    Some(search_config),
                    pdf_config,
                )
                .await
            {
                Ok(output_files) => {
                    println!("✓ Successfully converted {} URLs:", output_files.len());
                    for (index, output_path) in output_files.iter().enumerate() {
                        println!("  {}. {}", index + 1, output_path.display());
                    }
                }
                Err(e) => {
                    error!("Search-to-format operation failed: {}", e);
                    eprintln!("✗ Search-to-format operation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            // Handle URL to PDF conversion (legacy behavior)
            let url = match cli.url {
                Some(url) => url,
                None => {
                    eprintln!("✗ No URL provided for PDF conversion");
                    eprintln!("  Use 'webpage-save <URL>' or 'webpage-save search <type> <query>'");
                    std::process::exit(1);
                }
            };

            // Check if output path was provided
            let output_provided = cli.output.is_some();
            
            // Generate output filename if not provided
            let output_path = match cli.output {
                Some(path) => path,
                None => {
                    let parsed_url = url::Url::parse(&url)?;
                    let host = parsed_url.host_str().unwrap_or("unknown");
                    let extension = match cli.format {
                        OutputFormat::Pdf => "pdf",
                        OutputFormat::Markdown => "md",
                        OutputFormat::Both => "pdf", // Default to PDF for primary filename
                    };
                    let filename = format!("{}.{}", host, extension);
                    PathBuf::from(filename)
                }
            };

            match cli.format {
                OutputFormat::Pdf => {
                    info!("Converting URL to PDF: {}", url);
                    info!("Output file: {}", output_path.display());
                    info!("Wait time: {} seconds", cli.wait);

                    // Create PDF generator
                    let generator = match PdfGenerator::new().await {
                        Ok(generator) => {
                            info!("PDF generator initialized successfully");
                            generator
                        }
                        Err(e) => {
                            error!("Failed to initialize PDF generator: {}", e);
                            eprintln!("✗ Failed to initialize PDF generator: {}", e);
                            std::process::exit(1);
                        }
                    };

                    // Convert URL to PDF
                    match generator.url_to_pdf(&url, Some(&output_path)).await {
                        Ok(pdf_data) => {
                            info!("PDF generated successfully ({} bytes)", pdf_data.len());
                            println!("✓ Successfully generated PDF ({} bytes)", pdf_data.len());
                            println!("✓ Saved to: {}", output_path.display());
                        }
                        Err(e) => {
                            error!("Failed to generate PDF: {}", e);
                            eprintln!("✗ Failed to generate PDF: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                OutputFormat::Markdown => {
                    info!("Converting URL to Markdown: {}", url);
                    info!("Output file: {}", output_path.display());

                    // Create Markdown generator
                    let generator = match MarkdownGenerator::new().await {
                        Ok(generator) => {
                            info!("Markdown generator initialized successfully");
                            generator
                        }
                        Err(e) => {
                            error!("Failed to initialize Markdown generator: {}", e);
                            eprintln!("✗ Failed to initialize Markdown generator: {}", e);
                            std::process::exit(1);
                        }
                    };

                    // Convert URL to Markdown
                    match generator.url_to_markdown(&url, Some(&output_path)).await {
                        Ok(markdown_data) => {
                            info!(
                                "Markdown generated successfully ({} chars)",
                                markdown_data.len()
                            );
                            println!(
                                "✓ Successfully generated Markdown ({} chars)",
                                markdown_data.len()
                            );
                            println!("✓ Saved to: {}", output_path.display());
                        }
                        Err(e) => {
                            error!("Failed to generate Markdown: {}", e);
                            eprintln!("✗ Failed to generate Markdown: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                OutputFormat::Both => {
                    info!("Converting URL to both PDF and Markdown: {}", url);
                    
                    // Generate PDF path
                    let pdf_path = if output_provided {
                        // If output is specified, use that for PDF and generate MD path
                        output_path.clone()
                    } else {
                        let parsed_url = url::Url::parse(&url)?;
                        let host = parsed_url.host_str().unwrap_or("unknown");
                        PathBuf::from(format!("{}.pdf", host))
                    };
                    
                    // Generate Markdown path
                    let md_path = if output_provided {
                        // If output is specified, change extension to .md
                        output_path.with_extension("md")
                    } else {
                        let parsed_url = url::Url::parse(&url)?;
                        let host = parsed_url.host_str().unwrap_or("unknown");
                        PathBuf::from(format!("{}.md", host))
                    };

                    // Create PDF generator
                    let pdf_generator = match PdfGenerator::new().await {
                        Ok(generator) => {
                            info!("PDF generator initialized successfully");
                            generator
                        }
                        Err(e) => {
                            error!("Failed to initialize PDF generator: {}", e);
                            eprintln!("✗ Failed to initialize PDF generator: {}", e);
                            std::process::exit(1);
                        }
                    };

                    // Create Markdown generator
                    let md_generator = match MarkdownGenerator::new().await {
                        Ok(generator) => {
                            info!("Markdown generator initialized successfully");
                            generator
                        }
                        Err(e) => {
                            error!("Failed to initialize Markdown generator: {}", e);
                            eprintln!("✗ Failed to initialize Markdown generator: {}", e);
                            std::process::exit(1);
                        }
                    };

                    // Convert URL to PDF
                    match pdf_generator.url_to_pdf(&url, Some(&pdf_path)).await {
                        Ok(pdf_data) => {
                            info!("PDF generated successfully ({} bytes)", pdf_data.len());
                            println!("✓ Successfully generated PDF ({} bytes)", pdf_data.len());
                            println!("✓ Saved to: {}", pdf_path.display());
                        }
                        Err(e) => {
                            error!("Failed to generate PDF: {}", e);
                            eprintln!("✗ Failed to generate PDF: {}", e);
                            std::process::exit(1);
                        }
                    }

                    // Convert URL to Markdown
                    match md_generator.url_to_markdown(&url, Some(&md_path)).await {
                        Ok(markdown_data) => {
                            info!(
                                "Markdown generated successfully ({} chars)",
                                markdown_data.len()
                            );
                            println!(
                                "✓ Successfully generated Markdown ({} chars)",
                                markdown_data.len()
                            );
                            println!("✓ Saved to: {}", md_path.display());
                        }
                        Err(e) => {
                            error!("Failed to generate Markdown: {}", e);
                            eprintln!("✗ Failed to generate Markdown: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
