use anyhow::Result;
use luce_mcp::{HttpServer, McpServer};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--http" {
        // HTTP transport
        let addr = args.get(2).unwrap_or(&"127.0.0.1:3000".to_string()).clone();
        println!("Starting MCP server with HTTP transport on {}", addr);

        let server = HttpServer::new();
        server.run(&addr).await
    } else {
        // stdio transport (default)
        println!("Starting MCP server with stdio transport");

        let server = McpServer::new();
        server.run_stdio().await
    }
}
