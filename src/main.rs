use anyhow::Result;
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod db;
mod entities;
mod llm;
mod metachain;
mod models;
mod monitoring;
mod server;
mod session;
mod threading;
mod tools;

use server::LuxServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging to stderr only
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .init();

    info!(
        "Starting Lux MCP Server v{} - Illuminating your thinking...",
        env!("CARGO_PKG_VERSION")
    );

    // Check API keys (non-empty)
    let openai_available = std::env::var("OPENAI_API_KEY")
        .map(|key| !key.is_empty())
        .unwrap_or(false);
    let openrouter_available = std::env::var("OPENROUTER_API_KEY")
        .map(|key| !key.is_empty())
        .unwrap_or(false);

    info!("API Configuration:");
    info!(
        "  OpenAI API key: {}",
        if openai_available {
            "✓ Available"
        } else {
            "✗ Not found"
        }
    );
    info!(
        "  OpenRouter API key: {}",
        if openrouter_available {
            "✓ Available"
        } else {
            "✗ Not found"
        }
    );

    if !openai_available && !openrouter_available {
        eprintln!("No API keys found! Please set OPENAI_API_KEY or OPENROUTER_API_KEY");
        std::process::exit(1);
    }

    // Load config to show model defaults
    let config = llm::config::LLMConfig::from_env()?;
    info!("Default Models:");
    info!("  Chat (confer): {}", config.default_chat_model);
    info!(
        "  Reasoning (traced_reasoning): {}",
        config.default_reasoning_model
    );
    info!(
        "  Bias Checker (biased_reasoning): {}",
        config.default_bias_checker_model
    );

    // Create the server
    let server = LuxServer::new().await?;
    info!("Lux server initialized successfully");

    // Spawn session cleanup task
    let session_manager = server.session_manager();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            let removed = session_manager.cleanup_expired_sessions();
            if removed > 0 {
                tracing::info!("Session cleanup: removed {} expired sessions", removed);
            }
        }
    });
    info!("Session cleanup task started (5 minute interval)");

    // Create transport using stdin/stdout
    let transport = (stdin(), stdout());

    // Serve the handler over the transport
    info!("Starting MCP server on stdio transport");
    let service = server.serve(transport).await?;

    // Wait for the server to complete
    service.waiting().await?;

    Ok(())
}
