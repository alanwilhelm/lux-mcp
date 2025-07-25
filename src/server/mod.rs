pub mod handler;

use std::sync::{Arc, Mutex};
use crate::llm::LLMConfig;
use crate::tools::{ChatTool, TracedReasoningTool, BiasedReasoningTool};
use crate::metachain::MetachainEngine;
use crate::monitoring::MetacognitiveMonitor;

#[derive(Clone)]
pub struct LuxServer {
    pub chat_tool: Arc<ChatTool>,
    pub traced_reasoning_tool: Arc<TracedReasoningTool>,
    pub biased_reasoning_tool: Arc<BiasedReasoningTool>,
    pub metachain: Arc<MetachainEngine>,
    pub monitor: Arc<Mutex<MetacognitiveMonitor>>,
}

impl LuxServer {
    pub fn new() -> anyhow::Result<Self> {
        let config = LLMConfig::from_env()?;
        
        let monitor = Arc::new(Mutex::new(MetacognitiveMonitor::new()));
        let chat_tool = Arc::new(ChatTool::new(config.clone())?);
        let traced_reasoning_tool = Arc::new(TracedReasoningTool::new(config.clone(), monitor.clone())?);
        let biased_reasoning_tool = Arc::new(BiasedReasoningTool::new(config.clone(), monitor.clone())?);
        let metachain = Arc::new(MetachainEngine::new());
        
        Ok(Self {
            chat_tool,
            traced_reasoning_tool,
            biased_reasoning_tool,
            metachain,
            monitor,
        })
    }
}