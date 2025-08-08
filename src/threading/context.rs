use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub circular_reasoning_score: f32,
    pub distractor_fixation_score: f32,
    pub coherence_score: f32,
    pub depth_score: f32,
    pub perplexity: f32,
}

impl QualityMetrics {
    pub fn new() -> Self {
        Self {
            circular_reasoning_score: 0.0,
            distractor_fixation_score: 0.0,
            coherence_score: 1.0,
            depth_score: 0.5,
            perplexity: 20.0,
        }
    }

    pub fn from_monitor(_monitor: &crate::monitoring::MetacognitiveMonitor) -> Self {
        // TODO: Extract actual metrics from monitor when methods are available
        // For now, return default values
        Self {
            circular_reasoning_score: 0.0,
            distractor_fixation_score: 0.0,
            coherence_score: 1.0,
            depth_score: 0.5,
            perplexity: 20.0,
        }
    }
}
