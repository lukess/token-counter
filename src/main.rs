use anyhow::{Context, Result};
use clap::Parser;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE};
use serde::Serialize;
use std::time::Duration;
use tiktoken_rs::{cl100k_base, o200k_base, p50k_base, r50k_base};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Parser, Debug)]
#[command(name = "token-counter")]
#[command(about = "Count tokens in URL responses using OpenAI's tiktoken")]
struct Args {
    /// URL to fetch (REST API or MCP server)
    #[arg(value_name = "URL")]
    url: Option<String>,

    /// Local file to read input from instead of a URL
    #[arg(short = 'f', long, value_name = "PATH")]
    file: Option<String>,

    /// Tokenizer encoding:
    /// - o200k_base: GPT-4o, GPT-4o mini (omni models)
    /// - cl100k_base: GPT-4, GPT-3.5-turbo, text-embedding-ada-002
    /// - p50k_base: Codex models, text-davinci-002/003
    /// - r50k_base: GPT-3 models (davinci, curie, babbage, ada)
    #[arg(short, long, default_value = "o200k_base")]
    model: String,

    /// Treat URL as MCP server endpoint (sends tools/list request)
    #[arg(short = 'M', long)]
    mcp: bool,

    /// Request timeout in seconds
    #[arg(short, long, default_value_t = DEFAULT_TIMEOUT_SECS)]
    timeout: u64,

    /// Skip TLS certificate verification
    #[arg(short = 'k', long)]
    insecure: bool,

    /// Output result as JSON
    #[arg(short, long)]
    json: bool,

    /// Custom headers (format: 'Key1:Value1,Key2:Value2')
    #[arg(long)]
    headers: Option<String>,

    /// X-FROM header value for HTTP requests
    #[arg(long, default_value = "token-counter")]
    x_from: String,
}

#[derive(Serialize)]
struct MCPRequest {
    jsonrpc: &'static str,
    id: i32,
    method: &'static str,
}

#[derive(Serialize)]
struct TokenCountResult {
    source: String,
    token_count: usize,
    char_count: usize,
    model: String,
    is_mcp: bool,
    content_type: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.url.is_some() && args.file.is_some() {
        anyhow::bail!("Provide either a URL or --file, not both");
    }

    let (source, content, content_type) = if let Some(ref path) = args.file {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;
        (path.clone(), content, "file".to_string())
    } else if args.url.is_some() {
        let (content, content_type) = fetch_url(&args)?;
        (args.url.clone().unwrap(), content, content_type)
    } else {
        anyhow::bail!("No input provided. Specify a URL or use --file <PATH>");
    };

    let token_count = count_tokens(&content, &args.model)?;

    let result = TokenCountResult {
        source,
        token_count,
        char_count: content.len(),
        model: args.model.clone(),
        is_mcp: args.mcp,
        content_type,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Source: {}", result.source);
        println!("Content-Type: {}", result.content_type);
        println!("Character count: {}", result.char_count);
        println!("Token count ({}): {}", result.model, result.token_count);
        if result.is_mcp {
            println!("Mode: MCP (tools/list)");
        }
    }

    Ok(())
}

fn fetch_url(args: &Args) -> Result<(String, String)> {
    let url = args
        .url
        .as_ref()
        .context("URL is required for fetching")?;

    let client = Client::builder()
        .timeout(Duration::from_secs(args.timeout))
        .danger_accept_invalid_certs(args.insecure)
        .build()
        .context("Failed to build HTTP client")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-from"),
        HeaderValue::from_str(&args.x_from)?,
    );

    // Parse custom headers
    if let Some(ref custom_headers) = args.headers {
        for h in custom_headers.split(',') {
            let parts: Vec<&str> = h.trim().splitn(2, ':').collect();
            if parts.len() == 2 {
                let name = HeaderName::from_bytes(parts[0].trim().as_bytes())?;
                let value = HeaderValue::from_str(parts[1].trim())?;
                headers.insert(name, value);
            }
        }
    }

    let response = if args.mcp {
        let mcp_request = MCPRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "tools/list",
        };

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json, text/event-stream"),
        );

        client
            .post(url)
            .headers(headers)
            .json(&mcp_request)
            .send()
            .context("MCP request failed")?
    } else {
        client
            .get(url)
            .headers(headers)
            .send()
            .context("GET request failed")?
    };

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("Unexpected status code: {}", status);
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let body = response.text().context("Failed to read response body")?;

    Ok((body, content_type))
}

fn count_tokens(text: &str, model: &str) -> Result<usize> {
    let bpe = match model {
        "o200k_base" | "o200k_harmony" => o200k_base()?,
        "cl100k_base" => cl100k_base()?,
        "p50k_base" => p50k_base()?,
        "r50k_base" => r50k_base()?,
        _ => anyhow::bail!(
            "Unknown model: {}. Supported: o200k_base, o200k_harmony, cl100k_base, p50k_base, r50k_base",
            model
        ),
    };

    let tokens = bpe.encode_with_special_tokens(text);
    Ok(tokens.len())
}
