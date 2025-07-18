# Sketch - Multithreaded Web Scraper

A fast, multithreaded web scraper that crawls same-domain pages and outputs content as Markdown with embedded HTML.

## Features

- 🚀 **Multithreaded**: Concurrent processing with configurable batch sizes
- 🌐 **Cross-platform**: Works on Linux, macOS, and Windows
- ⚡ **Fast**: Async I/O with intelligent batching
- 🎯 **Same-domain crawling**: Automatically stays within the target domain
- 📝 **Markdown output**: Clean output format with embedded HTML content
- 🔧 **Configurable**: Adjustable batch sizes and page limits
- 🤝 **Respectful**: Built-in rate limiting to avoid overwhelming servers

## Installation

### Download Pre-built Binaries

Download the latest release for your platform from the [Releases page](../../releases):

- **Linux (glibc)**: `sketch-x86_64-unknown-linux-gnu.tar.gz`
- **Linux (musl)**: `sketch-x86_64-unknown-linux-musl.tar.gz` 
- **macOS (Intel)**: `sketch-x86_64-apple-darwin.tar.gz`
- **macOS (Apple Silicon)**: `sketch-aarch64-apple-darwin.tar.gz`
- **Windows**: `sketch-x86_64-pc-windows-msvc.zip`

Extract the archive and place the binary in your PATH.

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd sketch

# Build the release binary
cargo build --release

# The binary will be available at target/release/sketch
```

## Usage

### Basic Usage

```bash
# Scrape a website with default settings
sketch https://example.com
```

### Advanced Usage

```bash
# Custom batch size and page limit
sketch --batch-size 20 --max-pages 100 https://example.com

# Conservative crawling (smaller batches, fewer pages)
sketch --batch-size 5 --max-pages 25 https://example.com

# Save output to file
sketch https://example.com > output.md
```

### Command Line Options

```
Usage: sketch [OPTIONS] <URL>

Arguments:
  <URL>  The starting URL to crawl

Options:
  -b, --batch-size <BATCH_SIZE>  Number of URLs to process concurrently [default: 10]
  -m, --max-pages <MAX_PAGES>    Maximum number of pages to crawl [default: 50]
  -h, --help                     Print help
```

## How It Works

1. **Start**: Begins crawling from the provided URL
2. **Extract**: Extracts all same-domain links from each page
3. **Batch**: Groups URLs into configurable batches for concurrent processing
4. **Process**: Fetches pages concurrently within each batch
5. **Output**: Generates Markdown with embedded HTML content
6. **Repeat**: Continues until max pages reached or no more URLs to process

## Output Format

Each crawled page is output as a Markdown section:

```markdown
# Page Title

```html
<html content here>
```

# Another Page Title

```html
<more html content>
```
```

## Performance Tuning

- **Batch Size**: Increase `--batch-size` for faster crawling (but more resource usage)
- **Page Limit**: Adjust `--max-pages` based on your needs
- **Rate Limiting**: Built-in 100ms delay between requests to be respectful

## Development

### Prerequisites

- Rust 1.70+ 
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

### Dependencies

- `clap` - Command line argument parsing
- `reqwest` - HTTP client with async support
- `tokio` - Async runtime
- `kuchiki` - HTML parsing
- `url` - URL parsing and manipulation

## License

This project is open source. Please check the license file for details.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.