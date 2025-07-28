use clap::Parser;
use kuchiki::{parse_html, NodeRef};
use kuchiki::traits::TendrilSink;
use reqwest::header::USER_AGENT;
use std::collections::{HashMap, HashSet, VecDeque};


use std::time::Duration;
use tokio::runtime::Runtime;
use url::Url;

const DEFAULT_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_PAGES: usize = 50;
const REQUEST_DELAY_MS: u64 = 100;

const PROMPT_TEMPLATE: &str = r#"## System Prompt Template

You are an expert Python developer and web application tester specializing in
adverse conditions testing using the Scythe framework. Scythe is an open-source
Python-based framework (from https://github.com/EpykLab/scythe) designed for
comprehensively evaluating web applications under stress, including security
assessments via Tactics, Techniques, and Procedures (TTPs), load testing,
functional workflow validation, distributed simulations, and edge case
exploration. It emphasizes resilience testing by simulating adversarial
behaviors, high loads, complex user interactions, and failure scenarios.

Key components of Scythe include:
- **TTPs (Tactics, Techniques, Procedures)**: Modular classes for specific
tests, like LoginBruteforceTTP for security checks. Extend the base TTP class
from scythe.core.ttp for custom tests. TTPs support payloads, execution steps,
result verification, and expected outcomes (True for success, False for
expected failure in security contexts).
- **Journeys**: Multi-step workflows using Journey and Step classes from
scythe.journeys.base. Add actions like NavigateAction, FillFormAction,
ClickAction, AssertAction from scythe.journeys.actions. Execute via
JourneyExecutor from scythe.journeys.executor.
- **Orchestrators**: For scaling and distribution. Use ScaleOrchestrator from
scythe.orchestrators.scale for concurrent runs (e.g., parallel strategy,
max_workers, replications). Use DistributedOrchestrator from
scythe.orchestrators.distributed for geographic simulations with proxies and
credentials.
- **Authentication**: Handles sessions with classes like BasicAuth from
scythe.auth.basic or BearerTokenAuth from scythe.auth.bearer. Pre-execute
authentication before tests.
- **Behaviors**: Control execution patterns with HumanBehavior (realistic
delays, typing), MachineBehavior (fast, consistent), or StealthBehavior (avoid
detection) from scythe.behaviors.
- **Executors**: TTPExecutor from scythe.core.executor for running TTPs;
integrate with behaviors and auth.
- **Reporting**: Built-in metrics like success rates, execution times, errors.
Use analyze_test_results-style functions for custom analysis.
- **Dependencies**: Requires Python 3.8+, Selenium (with Google Chrome), and
libraries like requests, beautifulsoup4 (installed via pip install -r
requirements.txt).
- **Best Practices**: Define expected results clearly (e.g., False for security
tests expecting blocks). Use realistic data. Handle retries, errors gracefully.
Test in non-production environments. Follow MIT License guidelines.

Your task is to generate complete, standalone, runnable Python code that uses
Scythe to implement the specified tests on the target web application. The code
must:
- Include all necessary imports.
- Define TTPs, Journeys, or Orchestrators as needed.
- Incorporate authentication if required.
- Apply appropriate behaviors (e.g., HumanBehavior for realistic simulations).
- Set expected results and verify outcomes.
- Execute the tests and print basic results (e.g., success rates, metrics).
- Be modular, readable, and follow Python best practices (PEP 8 style, comments, error handling).
- Handle web elements using CSS selectors or IDs based on provided HTML.

Think step-by-step:
1. The scythe docs are available via context7 MCP. When generating code ensure 
   that you understand how to use the library as defined in the latest version 
   of the docs. https://context7.com/epyklab/scythe
2. Analyze the web app's URL, authentication, and HTML structures to identify
   selectors (e.g., #username for inputs).
3. Map each test description to Scythe components (e.g., use LoginBruteforceTTP
   for brute-force tests, Journey for multi-step flows).
4. For security tests: Expect failures where controls should block
   (expected_result=False).
5. For load/scale tests: Use orchestrators with replications and workers.
6. For workflows: Build Journeys with sequential steps and assertions.
7. Ensure code is safe: No infinite loops, respect rate limits via behaviors.
8. If a test requires custom logic, extend base classes appropriately.
9. Output only the Python code in a single code block, without additional
   explanations.
10. Do no hallucinate classes, methods or other code that are not defined
    within the scythe codebase
   

Web Application Details:
- Base URL: {BASE_URL}
- Authentication Requirements: {AUTH_DETAILS}
- Proxies for Distributed Testing (if applicable): {PROXIES_LIST}
- Credentials for Multi-User Simulation (if applicable): {CREDENTIALS_LIST}
- Behavior Pattern: {BEHAVIOR_PATTERN}

HTML Structures of Key Pages (use these to derive selectors for actions like
FillFormAction or ClickAction):
{PAGE_SECTIONS}

Tests to Implement: Provide a numbered or bulleted list of tests. For each,
specify:
- Type (e.g., Security TTP, Load Test, Workflow Journey).
- Description (e.g., "Brute-force login with common passwords, expect failure
due to lockout").
- Expected Result (True/False).
- Scale (e.g., replications=100 for load tests).
- Any custom parameters.

Example List (replace with your specifics):
1. Security Test: Brute-force login attempts on /login page using usernames
   ['admin'] and passwords ['password', '123456'], selectors:
username_selector='#username', password_selector='#password',
submit_selector='#submit'. Expected result: False (should be blocked).
2. Workflow Journey: User registration flow – navigate to /register, fill form
   with email='test@example.com', password='Secure123!', click submit, assert
URL contains 'verification'.
3. Load Test: Simulate 500 concurrent user logins using ScaleOrchestrator,
   max_workers=20, expected success rate >90%.
4. [ADD_MORE_TESTS_AS_NEEDED] (e.g., Edge Case: File upload with large files,
   expect handling without errors.)

Generate the Python code accordingly.
"#;

fn is_same_domain(url: &str, base_domain: &str) -> bool {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h == base_domain))
        .unwrap_or(false)
}

fn extract_main_content(document: &NodeRef) -> String {
    if let Ok(body) = document.select_first("body") {
        let body_node = body.as_node();
        if let Ok(iter) = body_node.select("script, style") {
            let to_remove: Vec<_> = iter.collect();
            for node in to_remove {
                node.as_node().detach();
            }
        }
        body.as_node().to_string()
    } else {
        document.to_string()
    }
}

async fn fetch_url(client: &reqwest::Client, url: &str) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    let response = client
        .get(url)
        .header(USER_AGENT, "Sketch/1.0 (for testing purposes; contact: your.email@example.com)")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let text = response.text().await?;
    let document: NodeRef = parse_html().one(text);

    let title = document
        .select_first("title")
        .ok()
        .map(|t| t.text_contents())
        .unwrap_or(url.to_string());

    let main_content = extract_main_content(&document);

    Ok((title, main_content))
}

fn extract_links(document: &NodeRef, base_url: &Url, base_domain: &str) -> Vec<String> {
    let mut links = Vec::new();

    if let Ok(iter) = document.select("a[href]") {
        for link in iter {
            if let Some(href) = link.attributes.borrow().get("href") {
                if let Ok(absolute_url) = base_url.join(&href) {
                    let absolute_url_str = absolute_url.to_string();
                    if is_same_domain(&absolute_url_str, base_domain) {
                        links.push(absolute_url_str);
                    }
                }
            }
        }
    }

    links
}

fn detect_auth_details(page_contents: &HashMap<String, String>) -> String {
    // Look for common authentication patterns
    for (path, content) in page_contents {
        let content_lower = content.to_lowercase();

        // Check for login forms
        if path.contains("login") || path.contains("signin") || path.contains("auth") {
            if content_lower.contains("password") && content_lower.contains("input") {
                return format!("Login page detected at {}. Use BasicAuth or form-based authentication. Analyze the form structure for exact selectors.", path);
            }
        }

        // Check for API endpoints that might use bearer tokens
        if content_lower.contains("bearer") || content_lower.contains("authorization") {
            return "Bearer token authentication likely required. Check API documentation or network requests.".to_string();
        }
    }

    "No specific authentication method detected. Manual analysis required.".to_string()
}

async fn process_batch(
    client: &reqwest::Client,
    urls: Vec<String>,
    base_url: Url,
    base_domain: String,
    silent: bool,
) -> (HashMap<String, (String, String)>, Vec<String>) {
    let mut results = HashMap::new();
    let mut new_urls = Vec::new();

    let tasks: Vec<_> = urls.into_iter().map(|url| {
        let client = client.clone();
        let url_clone = url.clone();
        let base_url_clone = base_url.clone();
        let base_domain_clone = base_domain.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(REQUEST_DELAY_MS)).await;
            match fetch_url(&client, &url_clone).await {
                Ok((title, content)) => {
                    let document: NodeRef = parse_html().one(content.clone());
                    let links = extract_links(&document, &base_url_clone, &base_domain_clone);
                    Some((url_clone, title, content, links))
                }
                Err(e) => {
                    if !silent {
                        eprintln!("Error fetching {}: {}", url_clone, e);
                    }
                    None
                }
            }
        })
    }).collect();

    for task in tasks {
        if let Ok(Some((url, title, content, links))) = task.await {
            results.insert(url, (title, content));
            new_urls.extend(links);
        }
    }

    (results, new_urls)
}

fn build_page_sections(page_contents: &HashMap<String, (String, String)>, _base_url: &Url) -> String {
    let mut sections = Vec::new();

    // Sort pages by path for consistent output
    let mut sorted_pages: Vec<_> = page_contents.iter().collect();
    sorted_pages.sort_by_key(|(url, _)| *url);

    for (url, (_title, content)) in sorted_pages {
        if let Ok(parsed_url) = Url::parse(url) {
            let path = if parsed_url.path() == "/" {
                "/ (Home Page)".to_string()
            } else {
                parsed_url.path().to_string()
            };

            sections.push(format!(
                "- {} HTML:\n```html\n{}\n```",
                path,
                content
            ));
        }
    }

    if sections.is_empty() {
        "[No pages scraped]".to_string()
    } else {
        sections.join("\n")
    }
}

async fn crawl_and_generate_prompt_async(start_url: &str, batch_size: usize, max_pages: usize, silent: bool) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let base_url = Url::parse(start_url)?;
    let base_domain = base_url.host_str().unwrap().to_string();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut all_page_contents: HashMap<String, (String, String)> = HashMap::new();

    queue.push_back(start_url.to_string());

    while !queue.is_empty() && visited.len() < max_pages {
        let mut batch = Vec::new();

        // Collect a batch of URLs to process
        for _ in 0..batch_size {
            if let Some(url) = queue.pop_front() {
                if !visited.contains(&url) {
                    visited.insert(url.clone());
                    batch.push(url);
                }
            } else {
                break;
            }
        }

        if batch.is_empty() {
            break;
        }

        if !silent {
            println!("Processing batch of {} URLs... (Total visited: {})", batch.len(), visited.len());
        }

        // Process the batch
        let (results, new_urls) = process_batch(&client, batch, base_url.clone(), base_domain.clone(), silent).await;

        all_page_contents.extend(results);

        // Add new URLs to the queue
        for url in new_urls {
            if !visited.contains(&url) && !queue.contains(&url) {
                queue.push_back(url);
            }
        }

        // Limit the queue size to prevent memory issues
        while queue.len() > 1000 {
            queue.pop_back();
        }
    }

    // Build the populated prompt
    let page_sections = build_page_sections(&all_page_contents, &base_url);
    let auth_details = detect_auth_details(&all_page_contents.iter().map(|(k, (_, v))| (k.clone(), v.clone())).collect());

    let populated_prompt = PROMPT_TEMPLATE
        .replace("{BASE_URL}", start_url)
        .replace("{AUTH_DETAILS}", &auth_details)
        .replace("{PROXIES_LIST}", "[Leave blank if not needed]")
        .replace("{CREDENTIALS_LIST}", "[Leave blank if not needed]")
        .replace("{BEHAVIOR_PATTERN}", "HumanBehavior(base_delay=2.0, typing_delay=0.1)")
        .replace("{PAGE_SECTIONS}", &page_sections);

    Ok(populated_prompt)
}

fn crawl_and_generate_prompt(start_url: &str, batch_size: usize, max_pages: usize, output_file: Option<String>, silent: bool) {
    let rt = Runtime::new().unwrap();

    match rt.block_on(crawl_and_generate_prompt_async(start_url, batch_size, max_pages, silent)) {
        Ok(prompt) => {
            if let Some(filename) = output_file {
                match std::fs::write(&filename, &prompt) {
                    Ok(_) => {
                        if !silent {
                            eprintln!("Output saved to: {}", filename);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error writing to file {}: {}", filename, e);
                    }
                }
            } else {
                println!("{}", prompt);
            }
        }
        Err(e) => {
            eprintln!("Error during crawling: {}", e);
        }
    }
}

#[derive(Parser, Debug)]
#[command(about = "Multithreaded web scraper to crawl same-domain pages and generate Scythe testing prompt.")]
struct Args {
    url: String,
    #[arg(short, long, default_value_t = DEFAULT_BATCH_SIZE)]
    batch_size: usize,
    #[arg(short, long, default_value_t = DEFAULT_MAX_PAGES)]
    max_pages: usize,
    #[arg(short, long, help = "Output file to save scan content")]
    output: Option<String>,
    #[arg(short, long, help = "Silent mode - suppress runtime logging")]
    silent: bool,
}

fn main() {
    let args = Args::parse();
    crawl_and_generate_prompt(&args.url, args.batch_size, args.max_pages, args.output, args.silent);
}
