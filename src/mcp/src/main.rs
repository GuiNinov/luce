use anyhow::Result;
use luce_mcp::{HttpServer, McpServer};
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Default database path
    let db_path = env::var("LUCE_DB_PATH").unwrap_or_else(|_| "./luce.db".to_string());

    // Ensure the directory exists
    if let Some(parent) = Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    if args.len() > 1 && args[1] == "--http" {
        // HTTP transport
        let addr = args.get(2).unwrap_or(&"127.0.0.1:3000".to_string()).clone();
        println!("Starting MCP server with HTTP transport on {}", addr);
        println!("Database path: {}", db_path);

        let server = HttpServer::new(&db_path);
        server.run(&addr).await
    } else {
        // stdio transport (default)
        println!("Starting MCP server with stdio transport");
        println!("Database path: {}", db_path);

        let server = McpServer::new(&db_path).await?;
        server.run_stdio().await
    }
}
