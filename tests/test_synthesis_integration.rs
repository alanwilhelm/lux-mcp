use anyhow::Result;
use lux_mcp::db::{DatabaseConnection, DatabaseService};
use lux_mcp::llm::config::LLMConfig;
use lux_mcp::server::LuxServer;
use lux_mcp::session::SessionManager;
use lux_mcp::tools::{BiasedReasoningRequest, BiasedReasoningTool, StepType};
use std::sync::Arc;

#[tokio::test]
async fn test_biased_reasoning_synthesis_evolution() -> Result<()> {
    // Set up test environment
    std::env::set_var("OPENAI_API_KEY", "test-key");
    std::env::set_var(
        "DATABASE_URL",
        "postgres://lux_user:lux_password@localhost/lux_mcp",
    );

    // Create config
    let config = LLMConfig::from_env()?;

    // Create session manager with 30 minute TTL
    let session_manager = Arc::new(SessionManager::new(30));

    // Create biased reasoning tool
    let tool = BiasedReasoningTool::new(config, session_manager)?;

    // Test 1: Initial query
    let request1 = BiasedReasoningRequest {
        query: "Should we use microservices architecture?".to_string(),
        session_id: None,
        new_session: None,
        primary_model: None,
        verifier_model: None,
        max_analysis_rounds: 3,
    };

    println!("Test 1: Initial query");
    let response1 = tool.process_step(request1).await?;
    assert_eq!(response1.step_type, StepType::Query);
    assert_eq!(response1.step_number, 1);
    println!("Session ID: {}", response1.session_id);
    println!("Synthesis snapshot: {:?}", response1.synthesis_snapshot);

    // Test 2: Continue reasoning
    let request2 = BiasedReasoningRequest {
        query: "Should we use microservices architecture?".to_string(),
        session_id: Some(response1.session_id.clone()),
        new_session: None,
        primary_model: None,
        verifier_model: None,
        max_analysis_rounds: 3,
    };

    println!("\nTest 2: Continue reasoning");
    let response2 = tool.process_step(request2).await?;
    assert_eq!(response2.step_type, StepType::Reasoning);
    assert_eq!(response2.step_number, 2);
    assert!(response2.synthesis_snapshot.is_some());

    if let Some(synthesis) = &response2.synthesis_snapshot {
        println!("Understanding: {}", synthesis.current_understanding);
        println!("Confidence: {}", synthesis.confidence_level);
        println!("Insights: {:?}", synthesis.top_insights);
    }

    // Test 3: Bias check
    let request3 = BiasedReasoningRequest {
        query: "Should we use microservices architecture?".to_string(),
        session_id: Some(response1.session_id.clone()),
        new_session: None,
        primary_model: None,
        verifier_model: None,
        max_analysis_rounds: 3,
    };

    println!("\nTest 3: Bias check");
    let response3 = tool.process_step(request3).await?;
    assert_eq!(response3.step_type, StepType::BiasAnalysis);
    assert_eq!(response3.step_number, 3);

    // Test 4: Final synthesis
    let request4 = BiasedReasoningRequest {
        query: "Should we use microservices architecture?".to_string(),
        session_id: Some(response1.session_id),
        new_session: None,
        primary_model: None,
        verifier_model: None,
        max_analysis_rounds: 3,
    };

    println!("\nTest 4: Final synthesis");
    let response4 = tool.process_step(request4).await?;

    if response4.step_type == StepType::Synthesis {
        println!("Final synthesis reached!");
        if let Some(synthesis) = &response4.synthesis_snapshot {
            println!("Final understanding: {}", synthesis.current_understanding);
            println!("Ready for decision: {}", synthesis.ready_for_decision);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_synthesis_database_persistence() -> Result<()> {
    // Set up database connection
    std::env::set_var(
        "DATABASE_URL",
        "postgres://lux_user:lux_password@localhost/lux_mcp",
    );

    let db_conn = DatabaseConnection::new().await?;
    let db_service = DatabaseService::new(db_conn);

    // Create a test session
    let session_id = "test_synthesis_123";
    let request = BiasedReasoningRequest {
        query: "Test query for synthesis".to_string(),
        session_id: Some(session_id.to_string()),
        new_session: None,
        primary_model: None,
        verifier_model: None,
        max_analysis_rounds: 1,
    };

    // Create or get session
    let session = db_service
        .create_or_get_session(session_id, &request)
        .await?;
    println!("Created session: {:?}", session.id);

    // TODO: Add more database persistence tests once synthesis logging is integrated

    Ok(())
}
