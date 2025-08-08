use anyhow::Result;
use lux_mcp::llm::config::LLMConfig;
use lux_mcp::tools::{BiasedReasoningRequest, BiasedReasoningTool, StepType};
use lux_mcp::session::SessionManager;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up environment
    std::env::set_var("RUST_LOG", "debug");
    
    // Initialize components
    let config = LLMConfig::from_env()?;
    let session_manager = Arc::new(SessionManager::new());
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
            
            // Continue with step 2
            let session_id = response.session_id;
            
            let request2 = BiasedReasoningRequest {
                query: "Should small startups use microservices architecture?".to_string(),
                session_id: Some(session_id.clone()),
                new_session: None,
                primary_model: None,
                verifier_model: None,
                max_analysis_rounds: 2,
            };
            
            println!("\n\n📝 Step 2: Continue Reasoning");
            println!("=============================");
            match tool.process_step(request2).await {
                Ok(response2) => {
                    println!("✅ Success!");
                    println!("Step Type: {:?}", response2.step_type);
                    println!("Step Number: {}", response2.step_number);
                    println!("Content Preview:\n{}", &response2.content[..200.min(response2.content.len())]);
                }
                Err(e) => {
                    println!("❌ Error in step 2: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    Ok(())
}