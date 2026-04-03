# mcp-searxng

An MCP (Model Context Protocol) server that provides web search via a [SearXNG](https://docs.searxng.org/) instance.

## Prerequisites

- Rust 1.85+ (edition 2024)
- A running SearXNG instance with JSON API enabled

## Building

```sh
cargo build --release
```

The binary will be at `target/release/mcp-searxng`.

## Configuration

| Environment Variable | Default | Description |
|---|---|---|
| `SEARXNG_URL` | `http://localhost:8080` | Base URL of your SearXNG instance |
| `RUST_LOG` | (off) | Log level (e.g. `info`, `debug`) — logs go to stderr |

## Usage with Claude Code

Add to your `.mcp.json`:

```json
{
  "mcpServers": {
    "searxng": {
      "command": "/path/to/mcp-searxng",
      "env": {
        "SEARXNG_URL": "http://localhost:8080"
      }
    }
  }
}
```

## Tool: `search`

Search the web using SearXNG. Returns a list of results with titles, URLs, and content snippets.

### Parameters

| Parameter | Type | Required | Description |
|---|---|---|---|
| `query` | string | yes | The search query string |
| `categories` | string | no | Comma-separated categories (e.g. `"general"`, `"news"`, `"images"`, `"it"`, `"science"`). Defaults to `"general"`. |
| `max_results` | integer | no | Number of results to return (1-50). Defaults to 10. |
| `pageno` | integer | no | Page number for pagination. Defaults to 1. |
| `time_range` | string | no | Time range filter: `"day"`, `"week"`, `"month"`, or `"year"`. |
