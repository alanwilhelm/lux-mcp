use anyhow::Result;
use lux_mcp::llm::config::LLMConfig;
use lux_mcp::session::SessionManager;
use lux_mcp::tools::{BiasedReasoningRequest, BiasedReasoningTool};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up environment
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    // Initialize components
    let config = LLMConfig::from_env()?;
    let session_manager = Arc::new(SessionManager::new(60)); // 60 minute TTL
    let tool = BiasedReasoningTool::new(config, session_manager)?;

    println!("🧪 Testing biased reasoning directly...\n");

    // Test 1: Initial query
    let request1 = BiasedReasoningRequest {
        query: "Should small startups use microservices architecture?".to_string(),
        session_id: None,
        new_session: None,
        primary_model: None,
        verifier_model: None,
        max_analysis_rounds: 2,
    };

    println!("📝 Step 1: Initial Query");
    println!("========================");
    match tool.process_step(request1).await {
        Ok(response) => {
            println!("✅ Success!");
            println!("Step Type: {:?}", response.step_type);
            println!("Session ID: {}", response.session_id);
            println!("Content:\n{}", response.content);
            println!("\nNext Action: {:?}", response.next_action);

            if let Some(synthesis) = &response.synthesis_snapshot {
                println!("\n📊 Synthesis Snapshot:");
                println!("  Understanding: {}", synthesis.current_understanding);
                println!("  Confidence: {}", synthesis.confidence_level);
                println!("  Ready for Decision: {}", synthesis.ready_for_decision);
                if !synthesis.top_insights.is_empty() {
                    println!("  Top Insights: {:?}", synthesis.top_insights);
                }
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }

    Ok(())
}
