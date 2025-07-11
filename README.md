# Webpage Save - URL to PDF/Markdown Converter

A fast and reliable command-line tool for converting web pages to PDF or Markdown format using headless Chrome, with integrated Brave Search functionality.

## Features

- 🚀 **Fast PDF generation** using headless Chrome
- 📝 **Markdown conversion** for text-based output
- 🌐 **Support for HTTP, HTTPS, and file URLs**
- 📄 **A4 page format with proper margins**
- 🎯 **Simple command-line interface**
- 🔧 **Customizable output paths**
- 📊 **Verbose logging support**
- 🔍 **Brave Search API integration** for web, news, and local searches
- 🌐 **Multi-language search support** with country-specific results
- 📰 **Advanced search operators** including boolean queries
- 🔄 **Search-to-PDF/Markdown** functionality (search and convert results)
- 🎨 **Multiple output formats** (PDF, Markdown, or both)
- 📂 **Batch processing** for search results

## Installation

### From Source

```bash
git clone https://github.com/tacogips/webpage-save.git
cd webpage-save
cargo build --release
```

The binary will be available at `./target/release/webpage-save`.

### Prerequisites

- Rust (latest stable version)
- Chrome or Chromium browser installed on your system
- Brave Search API key (optional, for search functionality)

## Usage

### Basic Usage

Convert a URL to PDF:

```bash
webpage-save https://example.com
```

This will create a PDF file named `example.com.pdf` in the current directory.

Convert a URL to Markdown:

```bash
webpage-save https://example.com --format markdown
```

This will create a Markdown file named `example.com.md` in the current directory.

Convert a URL to both PDF and Markdown:

```bash
webpage-save https://example.com --format both
```

This will create both `example.com.pdf` and `example.com.md` files in the current directory.

### Brave Search Functionality

Perform searches using Brave Search API:

```bash
# Web search
webpage-save search web "rust programming language" --count 10

# News search
webpage-save search news "technology news" --count 5 --country US

# Local search
webpage-save search local "restaurants near me" --count 8

# Complex search with Japanese keywords and boolean operators
webpage-save search news '桜 ("開花" OR "満開" OR "花見" OR "春" OR "季節" OR "公園" OR "美しい" OR "自然")' --count 10 --country JP
```

### Search-to-PDF/Markdown Functionality

Search and automatically convert results to PDF or Markdown:

```bash
# Search and convert top 5 results to PDF
webpage-save search-to-pdf web "rust programming" --max-results 5

# Search and convert news to Markdown
webpage-save search-to-pdf news "artificial intelligence" --format markdown --max-results 3

# Search and convert to both PDF and Markdown
webpage-save search-to-pdf web "machine learning" --format both --output-dir ./downloads

# Local search with custom naming strategy
webpage-save search-to-pdf local "coffee shops Tokyo" --naming title --output-dir ./local_results
```

### Specify Output File

```bash
# PDF output
webpage-save https://example.com -o my_document.pdf

# Markdown output
webpage-save https://example.com --format markdown -o my_document.md

# Both PDF and Markdown output
webpage-save https://example.com --format both -o my_document
```

### Verbose Output

```bash
webpage-save https://example.com -v
```

### Complete Example

```bash
# Convert a Japanese website to PDF with verbose output
webpage-save "https://www.aspicjapan.org/asu/article/14780" -o aspicjapan_article.pdf -v

# Convert to Markdown with custom wait time
webpage-save "https://example.com" --format markdown --wait 5 -o example.md
```

## Command-Line Options

### URL Conversion

```
webpage-save [OPTIONS] <URL>

Arguments:
  <URL>  URL to convert to PDF/Markdown

Options:
  -o, --output <FILE>    Output file path (optional, defaults to hostname.pdf/.md)
  -f, --format <FORMAT>  Output format (pdf, markdown, both) [default: pdf]
  -v, --verbose          Verbose output
  -w, --wait <WAIT>      Wait time in seconds before generating content (for dynamic content) [default: 2]
  -h, --help             Print help
  -V, --version          Print version
```

### Brave Search

```
webpage-save search [OPTIONS] <SEARCH_TYPE> <QUERY>

Arguments:
  <SEARCH_TYPE>  Type of search to perform [possible values: web, news, local]
  <QUERY>        Search query

Options:
  -c, --count <COUNT>          Number of results to return
  -o, --offset <OFFSET>        Pagination offset
      --country <COUNTRY>      Country code for news/local searches
  -l, --language <LANGUAGE>    Language code for news searches
  -f, --freshness <FRESHNESS>  Freshness filter for news searches (h, d, w, m, y)
      --api-key <API_KEY>      Brave API key (optional, can also use BRAVE_API_KEY environment variable)
  -h, --help                   Print help
```

### Search-to-PDF/Markdown

```
webpage-save search-to-pdf [OPTIONS] <SEARCH_TYPE> <QUERY>

Arguments:
  <SEARCH_TYPE>  Type of search to perform [possible values: web, news, local]
  <QUERY>        Search query

Options:
  -m, --max-results <MAX_RESULTS>  Maximum number of results to convert [default: 5]
  -o, --output-dir <OUTPUT_DIR>    Output directory for files [default: ./pdf_downloads]
      --format <FORMAT>            Output format (pdf, markdown, both) [default: pdf]
      --naming <NAMING>            File naming strategy (title, domain, sequential, title-domain) [default: domain]
      --country <COUNTRY>          Country code for news/local searches
  -l, --language <LANGUAGE>        Language code for news searches
  -f, --freshness <FRESHNESS>      Freshness filter for news searches (h, d, w, m, y)
      --api-key <API_KEY>          Brave API key (optional, can also use BRAVE_API_KEY environment variable)
  -w, --wait <WAIT>                Wait time in seconds before generating content [default: 2]
  -h, --help                       Print help
```

## Examples

### URL Conversion

#### Convert a news article to PDF

```bash
webpage-save "https://news.example.com/article/12345" -o news_article.pdf
```

#### Convert to Markdown

```bash
webpage-save "https://blog.example.com/article" --format markdown -o article.md
```

#### Convert with custom wait time for dynamic content

```bash
webpage-save "https://spa-website.com" -w 5 -o spa_content.pdf
```

### Brave Search Examples

#### Search for programming topics

```bash
webpage-save search web "rust async programming" --count 10
```

#### Search recent news with country filter

```bash
webpage-save search news "artificial intelligence" --count 5 --country US --freshness d
```

#### Complex search with Japanese keywords about nature

```bash
webpage-save search news '桜 ("開花" OR "満開" OR "花見" OR "春" OR "季節" OR "公園" OR "美しい" OR "自然")' --count 10 --country JP
```

#### Find local businesses

```bash
webpage-save search local "coffee shops Tokyo" --count 8
```

### Search-to-PDF/Markdown Examples

#### Search and convert web results to PDF

```bash
webpage-save search-to-pdf web "rust async programming" --max-results 3 --output-dir ./rust_docs
```

#### Search news and convert to Markdown

```bash
webpage-save search-to-pdf news "artificial intelligence" --format markdown --max-results 5 --output-dir ./ai_news
```

#### Search and convert to both formats

```bash
webpage-save search-to-pdf web "machine learning tutorials" --format both --max-results 3 --naming title
```

#### Local search with custom naming

```bash
webpage-save search-to-pdf local "restaurants Tokyo" --naming title --output-dir ./tokyo_restaurants --max-results 10
```

### Convert multiple URLs (using shell scripting)

```bash
#!/bin/bash
urls=(
    "https://example1.com"
    "https://example2.com"
    "https://example3.com"
)

for url in "${urls[@]}"; do
    webpage-save "$url" -v
done
```

## Library Usage

Webpage Save can also be used as a Rust library:

```rust
use webpage_save::pdf::PdfGenerator;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let generator = PdfGenerator::new().await?;
    
    // Convert URL to PDF
    let pdf_data = generator.url_to_pdf(
        "https://example.com",
        Some(Path::new("output.pdf"))
    ).await?;
    
    println!("PDF generated: {} bytes", pdf_data.len());
    
    // Convert HTML to PDF
    let html = r#"<html><body><h1>Hello World</h1></body></html>"#;
    let pdf_data = generator.html_to_pdf(
        html,
        Some(Path::new("html_output.pdf"))
    ).await?;
    
    Ok(())
}
```

## Configuration

### PDF Options

The tool generates PDFs with the following default settings:

- **Paper size**: A4 (8.27 x 11.7 inches)
- **Margins**: 0.4 inches on all sides
- **Background graphics**: Enabled
- **Orientation**: Portrait

### Browser Options

- **Headless mode**: Enabled
- **Sandbox**: Disabled (for compatibility)
- **Wait time**: 2 seconds (configurable)

### Brave Search API Setup

To use the search functionality, you need a Brave Search API key:

```bash
export BRAVE_API_KEY="your-api-key-here"
```

Or pass it directly with the `--api-key` option.

For detailed setup instructions, see the [bravesearch-mcp documentation](https://github.com/tacogips/bravesearch-mcp).

## Troubleshooting

### Chrome/Chromium Not Found

If you get an error about Chrome not being found:

1. Make sure Chrome or Chromium is installed
2. Check that it's in your system PATH
3. On Linux, you might need to install `chromium-browser` or `google-chrome-stable`

### Permission Errors

If you get permission errors:

```bash
# Make sure the binary is executable
chmod +x ./target/release/webpage-save
```

### Network Issues

For websites requiring authentication or special headers, the tool might not work as expected. Consider using a more specialized tool or implementing custom headers support.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Changelog

### v0.1.0
- Initial release
- Basic URL to PDF conversion
- Command-line interface
- Support for HTTP, HTTPS, and file URLs
- Configurable output paths and wait times