use clap::Parser;
use kuchiki::{parse_html, NodeRef};
use kuchiki::traits::TendrilSink;
use reqwest::header::USER_AGENT;
use std::collections::{HashSet, VecDeque};

use std::time::Duration;
use tokio::runtime::Runtime;
use url::Url;

const DEFAULT_BATCH_SIZE: usize = 10;

const DEFAULT_MAX_PAGES: usize = 50;
const REQUEST_DELAY_MS: u64 = 100;

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
        .header(USER_AGENT, "ScytheTestScraper/1.0 (for testing purposes; contact: your.email@example.com)")
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

async fn process_batch(
    client: &reqwest::Client,
    urls: Vec<String>,
    base_url: Url,
    base_domain: String,
) -> (Vec<String>, Vec<String>) {
    let mut results = Vec::new();
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
                    Some((title, content, links))
                }
                Err(e) => {
                    eprintln!("Error fetching {}: {}", url_clone, e);
                    None
                }
            }
        })
    }).collect();

    for task in tasks {
        if let Ok(Some((title, content, links))) = task.await {
            results.push(format!("# {}\n\n```html\n{}\n```\n", title, content));
            new_urls.extend(links);
        }
    }

    (results, new_urls)
}

async fn crawl_and_generate_markdown_async(start_url: &str, batch_size: usize, max_pages: usize) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let base_url = Url::parse(start_url)?;
    let base_domain = base_url.host_str().unwrap().to_string();
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut all_results: Vec<String> = Vec::new();

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

        println!("Processing batch of {} URLs... (Total visited: {})", batch.len(), visited.len());

        // Process the batch
        let (results, new_urls) = process_batch(&client, batch, base_url.clone(), base_domain.clone()).await;
        
        all_results.extend(results);

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

    Ok(all_results)
}

fn crawl_and_generate_markdown(start_url: &str, batch_size: usize, max_pages: usize) {
    let rt = Runtime::new().unwrap();
    
    match rt.block_on(crawl_and_generate_markdown_async(start_url, batch_size, max_pages)) {
        Ok(results) => {
            println!("{}", results.join("\n"));
        }
        Err(e) => {
            eprintln!("Error during crawling: {}", e);
        }
    }
}

#[derive(Parser, Debug)]
#[command(about = "Multithreaded web scraper to crawl same-domain pages and output Markdown with HTML content.")]
struct Args {
    url: String,
    #[arg(short, long, default_value_t = DEFAULT_BATCH_SIZE)]
    batch_size: usize,
    #[arg(short, long, default_value_t = DEFAULT_MAX_PAGES)]
    max_pages: usize,
}

fn main() {
    let args = Args::parse();
    crawl_and_generate_markdown(&args.url, args.batch_size, args.max_pages);
}