pub mod handler;

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::llm::LLMConfig;
use crate::tools::{ChatTool, TracedReasoningTool, BiasedReasoningTool, PlannerTool};
use crate::metachain::MetachainEngine;
use crate::session::SessionManager;

#[derive(Clone)]
pub struct LuxServer {
    pub chat_tool: Arc<ChatTool>,
    pub traced_reasoning_tool: Arc<Mutex<TracedReasoningTool>>,
    pub biased_reasoning_tool: Arc<BiasedReasoningTool>,
    pub planner_tool: Arc<Mutex<PlannerTool>>,
    pub metachain: Arc<MetachainEngine>,
    pub session_manager: Arc<SessionManager>,
}

impl LuxServer {
    pub fn new() -> anyhow::Result<Self> {
        let config = LLMConfig::from_env()?;
        
        let session_manager = Arc::new(SessionManager::new(30)); // 30 minute TTL
        let chat_tool = Arc::new(ChatTool::new(config.clone())?);
        let traced_reasoning_tool = Arc::new(Mutex::new(TracedReasoningTool::new(config.clone(), session_manager.clone())?));
        let biased_reasoning_tool = Arc::new(BiasedReasoningTool::new(config.clone(), session_manager.clone())?);
        let planner_tool = Arc::new(Mutex::new(PlannerTool::new(config.clone(), session_manager.clone())?));
        let metachain = Arc::new(MetachainEngine::new());
        
        Ok(Self {
            chat_tool,
            traced_reasoning_tool,
            biased_reasoning_tool,
            planner_tool,
            metachain,
            session_manager,
        })
    }
}