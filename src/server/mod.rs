pub mod handler;

use crate::db::{DatabaseConnection, DatabaseService};
use crate::llm::LLMConfig;
use crate::metachain::MetachainEngine;
use crate::session::SessionManager;
use crate::threading::{QualityThreadIntegration, SynthesisThreadIntegration, ThreadManager};
use crate::tools::{
    BiasedReasoningTool, ChatTool, HybridBiasedReasoningTool, PlannerTool,
    SequentialThinkingExternalTool, SequentialThinkingTool, TracedReasoningTool,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct LuxServer {
    chat_tool: Arc<ChatTool>,
    traced_reasoning_tool: Arc<Mutex<TracedReasoningTool>>,
    biased_reasoning_tool: Arc<BiasedReasoningTool>,
    planner_tool: Arc<Mutex<PlannerTool>>,
    sequential_thinking_tool: Arc<SequentialThinkingTool>,
    sequential_thinking_external_tool: Arc<SequentialThinkingExternalTool>,
    hybrid_biased_reasoning_tool: Arc<HybridBiasedReasoningTool>,
    metachain: Arc<MetachainEngine>,
    session_manager: Arc<SessionManager>,
    thread_manager: Arc<ThreadManager>,
    synthesis_integration: Arc<SynthesisThreadIntegration>,
    quality_integration: Arc<QualityThreadIntegration>,
    db_service: Option<Arc<DatabaseService>>,
}

impl LuxServer {
    pub async fn new() -> anyhow::Result<Self> {
        let config = LLMConfig::from_env()?;

        let session_manager = Arc::new(SessionManager::new(30)); // 30 minute TTL
        let thread_manager = Arc::new(ThreadManager::new()); // 3 hour TTL by default
        let synthesis_integration =
            Arc::new(SynthesisThreadIntegration::new(thread_manager.clone()));
        let quality_integration = Arc::new(QualityThreadIntegration::new(thread_manager.clone()));
        let chat_tool = Arc::new(ChatTool::new(config.clone())?);
        let traced_reasoning_tool = Arc::new(Mutex::new(TracedReasoningTool::new(
            config.clone(),
            session_manager.clone(),
        )?));
        let biased_reasoning_tool = Arc::new(BiasedReasoningTool::new(
            config.clone(),
            session_manager.clone(),
        )?);
        let planner_tool = Arc::new(Mutex::new(PlannerTool::new(
            config.clone(),
            session_manager.clone(),
        )?));
        let sequential_thinking_tool = Arc::new(SequentialThinkingTool::new());
        let sequential_thinking_external_tool = Arc::new(SequentialThinkingExternalTool::new());
        let hybrid_biased_reasoning_tool = Arc::new(HybridBiasedReasoningTool::new());
        let metachain = Arc::new(MetachainEngine::new());

        // Initialize database service if DATABASE_URL is set
        let db_service = if std::env::var("DATABASE_URL").is_ok() {
            match DatabaseConnection::new().await {
                Ok(db_conn) => {
                    let service = Arc::new(DatabaseService::new(db_conn));
                    tracing::info!("Database service initialized successfully");
                    Some(service)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize database service: {}. Continuing without database logging.", e);
                    None
                }
            }
        } else {
            tracing::info!("DATABASE_URL not set, running without database logging");
            None
        };

        Ok(Self {
            chat_tool,
            traced_reasoning_tool,
            biased_reasoning_tool,
            planner_tool,
            sequential_thinking_tool,
            sequential_thinking_external_tool,
            hybrid_biased_reasoning_tool,
            metachain,
            session_manager,
            thread_manager,
            synthesis_integration,
            quality_integration,
            db_service,
        })
    }

    pub fn session_manager(&self) -> Arc<SessionManager> {
        self.session_manager.clone()
    }
}
