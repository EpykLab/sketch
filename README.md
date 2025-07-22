# Sketch

<p align="center">
  <strong>A powerful web scraper that generates AI-ready prompts for the Scythe testing framework</strong>
</p>

<p align="center">
  <a href="https://github.com/EpykLab/sketch/releases"><img src="https://img.shields.io/github/v/release/EpykLab/sketch" alt="Latest Release"></a>
  <a href="https://github.com/EpykLab/sketch/actions"><img src="https://img.shields.io/github/actions/workflow/status/EpykLab/sketch/release.yml" alt="Build Status"></a>
  <a href="https://github.com/EpykLab/sketch/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

Sketch is a multithreaded web scraper that crawls websites and automatically generates comprehensive prompts for the [Scythe framework](https://github.com/EpykLab/scythe) - an advanced Python testing framework for web application security, load testing, and workflow validation.

## 🚀 What It Does

Sketch transforms web crawling into actionable testing prompts by:

1. **Crawling** websites with intelligent same-domain link extraction
2. **Analyzing** page structures and authentication patterns
3. **Generating** complete Scythe framework prompts with embedded HTML structures
4. **Organizing** pages by URL paths for easy test targeting

The output is a ready-to-use prompt that can be given to AI assistants to generate comprehensive Python testing code using the Scythe framework.

## ✨ Features

- 🚀 **Multithreaded crawling** with configurable batch processing
- 🎯 **Smart same-domain filtering** to stay within target scope
- 🔍 **Automatic authentication detection** (forms, tokens, etc.)
- 📊 **Organized HTML structure analysis** by page type
- 🤖 **AI-ready prompt generation** for immediate use
- ⚡ **High performance** with async I/O and intelligent batching
- 🛡️ **Respectful crawling** with built-in rate limiting
- 🌐 **Cross-platform** support (Linux, macOS)

## 📥 Installation

### Install with eget

```bash
eget EpykLab/sketch --to=$HOME/.local/bin/
```

### Download Pre-built Binaries

Download the latest release for your platform:

**Linux:**
```bash
wget https://github.com/EpykLab/sketch/releases/latest/download/sketch-x86_64-unknown-linux-gnu.tar.gz
tar -xzf sketch-x86_64-unknown-linux-gnu.tar.gz
sudo mv sketch /usr/local/bin/
```

**macOS (Intel):**
```bash
wget https://github.com/EpykLab/sketch/releases/latest/download/sketch-x86_64-apple-darwin.tar.gz
tar -xzf sketch-x86_64-apple-darwin.tar.gz
sudo mv sketch /usr/local/bin/
```

**macOS (Apple Silicon):**
```bash
wget https://github.com/EpykLab/sketch/releases/latest/download/sketch-aarch64-apple-darwin.tar.gz
tar -xzf sketch-aarch64-apple-darwin.tar.gz
sudo mv sketch /usr/local/bin/
```

### Build from Source

```bash
git clone https://github.com/EpykLab/sketch.git
cd sketch
cargo build --release
sudo cp target/release/sketch /usr/local/bin/
```

## 🎯 Usage

### Basic Web Crawling and Prompt Generation

```bash
# Generate a Scythe testing prompt for a website
sketch https://example.com

# Save the generated prompt to a file
sketch https://example.com > scythe_prompt.md
```

### Advanced Configuration

```bash
# Faster crawling with larger batches
sketch --batch-size 20 --max-pages 100 https://example.com

# Conservative crawling for sensitive sites
sketch --batch-size 5 --max-pages 25 https://example.com

# Quick reconnaissance scan
sketch --batch-size 15 --max-pages 50 https://target-app.com > security_test_prompt.md
```

### Command Line Options

```
Usage: sketch [OPTIONS] <URL>

Arguments:
  <URL>  The starting URL to crawl

Options:
  -b, --batch-size <BATCH_SIZE>  URLs to process concurrently [default: 10]
  -m, --max-pages <MAX_PAGES>    Maximum pages to crawl [default: 50]
  -h, --help                     Print help information
```

## 📋 Generated Prompt Structure

Sketch outputs a comprehensive prompt containing:

### 1. **Framework Introduction**
Complete Scythe framework documentation and best practices

### 2. **Auto-detected Application Details**
```
Web Application Details:
- Base URL: https://example.com
- Authentication Requirements: Login page detected at /login. Use BasicAuth...
- Behavior Pattern: HumanBehavior(base_delay=2.0, typing_delay=0.1)
```

### 3. **Organized HTML Structures**
```
HTML Structures of Key Pages:
- /login HTML:
<form action="/login" method="post">
  <input type="text" name="username" id="username">
  <input type="password" name="password" id="password">
  <button type="submit">Login</button>
</form>

- /dashboard HTML:
<div class="dashboard">
  <h1>Welcome</h1>
  <nav>...</nav>
</div>
```

### 4. **Testing Guidelines**
Instructions for generating security tests, load tests, and workflow journeys

## 🤖 Using the Generated Prompt

### Step 1: Generate the Prompt
```bash
sketch https://your-target-app.com > scythe_prompt.md
```

### Step 2: Customize the Prompt (Optional)

Edit the generated `scythe_prompt.md` file to:

- **Add specific test requirements** in the "Tests to Implement" section
- **Modify authentication details** if auto-detection missed something
- **Add custom credentials** for multi-user testing
- **Specify proxy configurations** for distributed testing

Example customizations:
```markdown
Tests to Implement:
1. Security Test: SQL injection attempts on /login form, expected result: False
2. Load Test: Simulate 1000 concurrent user registrations, expected success rate >95%
3. Workflow Journey: Complete user onboarding flow from /register to /dashboard
```

### Step 3: Generate Test Code

Give the prompt to an AI assistant (ChatGPT, Claude, etc.):

```
Please generate Python code using the Scythe framework based on this prompt:

[paste the generated prompt content]
```

### Step 4: Run Your Tests

```bash
# Install dependencies
pip install -r requirements.txt

# Run the generated test code
python generated_scythe_tests.py
```

## 🔧 Performance Tuning

### Crawling Speed vs. Server Load

- **Conservative** (gentle on servers): `--batch-size 5 --max-pages 25`
- **Balanced** (default): `--batch-size 10 --max-pages 50`
- **Aggressive** (fast crawling): `--batch-size 20 --max-pages 100`

### Built-in Protections

- ⏱️ **Rate Limiting**: 100ms delay between requests
- 🎯 **Domain Filtering**: Automatically stays within target domain
- 🔒 **Respectful Headers**: Identifies itself with proper User-Agent
- 📏 **Queue Management**: Prevents memory issues with large sites

## 🧪 Examples

### E-commerce Security Testing
```bash
sketch --batch-size 15 --max-pages 75 https://shop.example.com > ecommerce_security_prompt.md
# Generated prompt will include login, cart, checkout, and payment pages
```

### API Documentation Analysis
```bash
sketch --batch-size 20 --max-pages 100 https://api-docs.example.com > api_test_prompt.md
# Perfect for generating API endpoint testing scenarios
```

### Corporate Application Assessment
```bash
sketch --batch-size 8 --max-pages 40 https://internal-app.company.com > corporate_app_prompt.md
# Conservative settings for internal applications
```

## 🛠️ Development

### Prerequisites
- Rust 1.70+
- Cargo

### Building
```bash
cargo build --release
```

### Dependencies
- `clap` - CLI argument parsing
- `reqwest` - HTTP client
- `tokio` - Async runtime
- `kuchiki` - HTML parsing
- `url` - URL handling

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions welcome! Please read our contributing guidelines and submit pull requests.

## 🔗 Related Projects

- [Scythe Framework](https://github.com/EpykLab/scythe) - Advanced Python testing framework
- [Web Security Testing Guide](https://owasp.org/www-project-web-security-testing-guide/) - OWASP testing methodology

---

<p align="center">
  <strong>Transform web crawling into actionable testing strategies with Sketch + Scythe</strong>
</p>
