use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{ServerCapabilities, ServerInfo},
    schemars, tool,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// --- Config ---

#[derive(Debug, Deserialize)]
struct Config {
    searxng_url: String,
}

fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(p) = std::env::var_os("MCP_SEARXNG_CONFIG") {
        return Ok(PathBuf::from(p));
    }
    let home = std::env::var_os("HOME").ok_or("HOME not set")?;
    Ok(PathBuf::from(home).join(".config/mcp-searxng/config.toml"))
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = config_path()?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read config at {}: {e}", path.display()))?;
    let config: Config = toml::from_str(&text)
        .map_err(|e| format!("failed to parse config at {}: {e}", path.display()))?;
    Ok(config)
}

// --- SearXNG API types ---

#[derive(Debug, Deserialize)]
struct SearxngResponse {
    results: Vec<SearxngResult>,
}

#[derive(Debug, Deserialize)]
struct SearxngResult {
    title: String,
    url: String,
    content: Option<String>,
}

// --- Tool arguments ---

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
struct SearchArgs {
    /// The search query string
    query: String,

    /// Comma-separated search categories (e.g. "general", "images", "news", "it", "science").
    /// Defaults to "general" if not specified.
    #[serde(default)]
    categories: Option<String>,

    /// Number of results to return (1-50). Defaults to 10.
    #[serde(default)]
    max_results: Option<u32>,

    /// Page number for pagination. Defaults to 1.
    #[serde(default)]
    pageno: Option<u32>,

    /// Time range filter: "day", "week", "month", or "year".
    #[serde(default)]
    time_range: Option<String>,
}

// --- MCP Server ---

#[derive(Clone)]
struct SearxngServer {
    tool_router: ToolRouter<Self>,
    client: reqwest::Client,
    base_url: String,
}

#[rmcp::tool_router]
impl SearxngServer {
    fn new(base_url: String) -> Self {
        Self {
            tool_router: Self::tool_router(),
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// Search the web using SearXNG. Returns a list of results with titles, URLs, and content snippets.
    #[tool(name = "search")]
    async fn search(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<String, rmcp::ErrorData> {
        let max_results = args.max_results.unwrap_or(10).min(50) as usize;

        let mut params = vec![
            ("q".to_string(), args.query),
            ("format".to_string(), "json".to_string()),
        ];
        if let Some(cats) = args.categories {
            params.push(("categories".to_string(), cats));
        }
        if let Some(page) = args.pageno {
            params.push(("pageno".to_string(), page.to_string()));
        }
        if let Some(time_range) = args.time_range {
            params.push(("time_range".to_string(), time_range));
        }

        let response = self
            .client
            .get(format!("{}/search", self.base_url))
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("HTTP request failed: {e}"), None)
            })?;

        let body: SearxngResponse = response.json().await.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to parse response: {e}"), None)
        })?;

        let results: Vec<String> = body
            .results
            .into_iter()
            .take(max_results)
            .enumerate()
            .map(|(i, r)| {
                format!(
                    "{}. {}\n   URL: {}\n   {}",
                    i + 1,
                    r.title,
                    r.url,
                    r.content.unwrap_or_default()
                )
            })
            .collect();

        if results.is_empty() {
            Ok("No results found.".to_string())
        } else {
            Ok(results.join("\n\n"))
        }
    }
}

#[rmcp::tool_handler]
impl ServerHandler for SearxngServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("A search tool powered by SearXNG")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let config = load_config()?;
    let base_url = config.searxng_url.trim_end_matches('/').to_string();

    let server = SearxngServer::new(base_url);
    let transport = rmcp::transport::io::stdio();

    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
