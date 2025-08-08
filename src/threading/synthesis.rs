use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;

use lux_synthesis::EvolvingSynthesis;

use super::manager::ThreadManager;

/// Links synthesis states to conversation threads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisLink {
    pub thread_id: String,
    pub synthesis_id: String,
    pub created_at: DateTime<Utc>,
    pub insights: Vec<InsightSnapshot>,
    pub actions: Vec<ActionSnapshot>,
    pub confidence_level: f32,
    pub clarity_level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightSnapshot {
    pub content: String,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
    pub source_tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSnapshot {
    pub description: String,
    pub priority: String,
    pub completed: bool,
    pub timestamp: DateTime<Utc>,
}

/// Manages the insight graph linking synthesis states
pub struct InsightGraph {
    links: Arc<Mutex<HashMap<String, Vec<SynthesisLink>>>>,
    synthesis_cache: Arc<Mutex<HashMap<String, Arc<Mutex<EvolvingSynthesis>>>>>,
}

impl InsightGraph {
    pub fn new() -> Self {
        Self {
            links: Arc::new(Mutex::new(HashMap::new())),
            synthesis_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create or get synthesis for a thread
    pub fn get_or_create_synthesis(
        &self,
        thread_id: &str,
        tool_name: &str,
    ) -> Arc<Mutex<EvolvingSynthesis>> {
        let mut cache = self.synthesis_cache.lock().unwrap();

        let synthesis_id = format!("{}_{}", thread_id, tool_name);

        cache
            .entry(synthesis_id.clone())
            .or_insert_with(|| {
                Arc::new(Mutex::new(EvolvingSynthesis::new_in_memory(
                    tool_name, thread_id,
                )))
            })
            .clone()
    }

    /// Link synthesis state to thread
    pub fn link_synthesis(
        &self,
        thread_id: &str,
        synthesis: &EvolvingSynthesis,
        tool_name: &str,
    ) -> Result<()> {
        let synthesis_id = format!("{}_{}", thread_id, tool_name);

        // Extract insights and actions from synthesis
        let insights = self.extract_insights(synthesis);
        let actions = self.extract_actions(synthesis);

        // Calculate confidence and clarity from synthesis state
        let (confidence, clarity) = self.calculate_metrics(synthesis);

        let link = SynthesisLink {
            thread_id: thread_id.to_string(),
            synthesis_id: synthesis_id.clone(),
            created_at: Utc::now(),
            insights,
            actions,
            confidence_level: confidence,
            clarity_level: clarity,
        };

        let mut links = self.links.lock().unwrap();
        links
            .entry(thread_id.to_string())
            .or_insert_with(Vec::new)
            .push(link);

        info!("Linked synthesis {} to thread {}", synthesis_id, thread_id);
        Ok(())
    }

    /// Get all synthesis links for a thread
    pub fn get_thread_synthesis(&self, thread_id: &str) -> Vec<SynthesisLink> {
        let links = self.links.lock().unwrap();
        links.get(thread_id).cloned().unwrap_or_default()
    }

    /// Merge synthesis states when threads are connected
    pub fn merge_synthesis(&self, source_thread: &str, target_thread: &str) -> Result<()> {
        let links = self.links.lock().unwrap();

        // Get synthesis from source thread
        if let Some(source_links) = links.get(source_thread) {
            let mut target_links = links.get(target_thread).cloned().unwrap_or_default();

            // Merge insights and actions
            for link in source_links {
                let mut merged_link = link.clone();
                merged_link.thread_id = target_thread.to_string();
                target_links.push(merged_link);
            }

            // Update target thread links
            drop(links);
            let mut links = self.links.lock().unwrap();
            links.insert(target_thread.to_string(), target_links);
        }

        info!(
            "Merged synthesis from thread {} to {}",
            source_thread, target_thread
        );
        Ok(())
    }

    /// Extract insights from synthesis
    fn extract_insights(&self, synthesis: &EvolvingSynthesis) -> Vec<InsightSnapshot> {
        // In a real implementation, this would extract from synthesis.insights
        // For now, create a placeholder
        vec![InsightSnapshot {
            content: "Key insight from synthesis".to_string(),
            confidence: 0.85,
            timestamp: Utc::now(),
            source_tool: "traced_reasoning".to_string(),
        }]
    }

    /// Extract actions from synthesis
    fn extract_actions(&self, synthesis: &EvolvingSynthesis) -> Vec<ActionSnapshot> {
        // In a real implementation, this would extract from synthesis.actions
        // For now, create a placeholder
        vec![ActionSnapshot {
            description: "Next action from synthesis".to_string(),
            priority: "high".to_string(),
            completed: false,
            timestamp: Utc::now(),
        }]
    }

    /// Calculate confidence and clarity metrics
    fn calculate_metrics(&self, synthesis: &EvolvingSynthesis) -> (f32, f32) {
        // In a real implementation, this would calculate from synthesis state
        // For now, return placeholder values
        (0.75, 0.80)
    }

    /// Get synthesis summary for a thread
    pub fn get_synthesis_summary(&self, thread_id: &str) -> Option<SynthesisSummary> {
        let links = self.get_thread_synthesis(thread_id);
        if links.is_empty() {
            return None;
        }

        let total_insights: usize = links.iter().map(|l| l.insights.len()).sum();
        let total_actions: usize = links.iter().map(|l| l.actions.len()).sum();
        let avg_confidence: f32 =
            links.iter().map(|l| l.confidence_level).sum::<f32>() / links.len() as f32;
        let avg_clarity: f32 =
            links.iter().map(|l| l.clarity_level).sum::<f32>() / links.len() as f32;

        Some(SynthesisSummary {
            thread_id: thread_id.to_string(),
            synthesis_count: links.len(),
            total_insights,
            total_actions,
            average_confidence: avg_confidence,
            average_clarity: avg_clarity,
            last_updated: links.last().map(|l| l.created_at).unwrap_or_else(Utc::now),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisSummary {
    pub thread_id: String,
    pub synthesis_count: usize,
    pub total_insights: usize,
    pub total_actions: usize,
    pub average_confidence: f32,
    pub average_clarity: f32,
    pub last_updated: DateTime<Utc>,
}

/// Integration helper to connect synthesis with thread manager
pub struct SynthesisThreadIntegration {
    thread_manager: Arc<ThreadManager>,
    insight_graph: Arc<InsightGraph>,
}

impl SynthesisThreadIntegration {
    pub fn new(thread_manager: Arc<ThreadManager>) -> Self {
        Self {
            thread_manager,
            insight_graph: Arc::new(InsightGraph::new()),
        }
    }

    /// Process synthesis for a tool call within a thread
    pub fn process_synthesis(
        &self,
        thread_id: &str,
        tool_name: &str,
        synthesis: &EvolvingSynthesis,
    ) -> Result<()> {
        // Link synthesis to thread
        self.insight_graph
            .link_synthesis(thread_id, synthesis, tool_name)?;

        // Update thread context with synthesis info
        if let Some(_summary) = self.insight_graph.get_synthesis_summary(thread_id) {
            // TODO: Implement update_thread_metadata in ThreadManager
            // self.thread_manager.update_thread_metadata(
            //     thread_id,
            //     "synthesis_summary",
            //     &serde_json::to_value(summary)?,
            // )?;
        }

        Ok(())
    }

    /// Get synthesis context for reconstruction
    pub fn get_synthesis_context(&self, thread_id: &str) -> Option<String> {
        let summary = self.insight_graph.get_synthesis_summary(thread_id)?;

        Some(format!(
            "Synthesis Context:\n\
             - Total insights: {}\n\
             - Total actions: {}\n\
             - Average confidence: {:.2}\n\
             - Average clarity: {:.2}\n\
             - Synthesis states: {}",
            summary.total_insights,
            summary.total_actions,
            summary.average_confidence,
            summary.average_clarity,
            summary.synthesis_count
        ))
    }

    /// Merge synthesis when continuing from another thread
    pub fn continue_synthesis(&self, from_thread: &str, to_thread: &str) -> Result<()> {
        self.insight_graph.merge_synthesis(from_thread, to_thread)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insight_graph_creation() {
        let graph = InsightGraph::new();
        let synthesis = graph.get_or_create_synthesis("test-thread", "test-tool");
        assert!(synthesis.lock().is_ok());
    }

    #[test]
    fn test_synthesis_linking() {
        let graph = InsightGraph::new();
        let synthesis = EvolvingSynthesis::new_in_memory("test-tool", "test-thread");

        let result = graph.link_synthesis("test-thread", &synthesis, "test-tool");
        assert!(result.is_ok());

        let links = graph.get_thread_synthesis("test-thread");
        assert_eq!(links.len(), 1);
    }
}
