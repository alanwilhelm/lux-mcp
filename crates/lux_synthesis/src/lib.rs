//! Core synthesis domain logic for lux-mcp
//!
//! This crate provides the event-driven synthesis system used by all reasoning tools.
//! It is database-agnostic and focuses purely on domain logic.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub mod events;

use events::{ActionItem, InsightEntry, SynthesisEvent};

/// Core synthesis state that evolves over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisState {
    pub id: Uuid,
    pub tool_name: String,
    pub session_id: String,
    pub version: u32,
    pub current_understanding: String,
    pub key_insights: Vec<InsightEntry>,
    pub action_items: Vec<ActionItem>,
    pub confidence_score: f32,
    pub clarity_score: f32,
    pub created_at: DateTime<Utc>,
    pub last_updated_step: u32,
}

impl SynthesisState {
    pub fn new(tool_name: String, session_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            tool_name,
            session_id,
            version: 0,
            current_understanding: String::new(),
            key_insights: Vec::new(),
            action_items: Vec::new(),
            confidence_score: 0.0,
            clarity_score: 0.0,
            created_at: Utc::now(),
            last_updated_step: 0,
        }
    }
}

/// Represents a change to the synthesis state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub previous_version: u32,
    pub new_version: u32,
    pub event: SynthesisEvent,
    pub timestamp: DateTime<Utc>,
}

/// Thread-safe synthesis engine that processes events
#[derive(Clone)]
pub struct EvolvingSynthesis {
    state: Arc<Mutex<SynthesisState>>,
    diffs: Arc<Mutex<Vec<StateDiff>>>,
    sink: Arc<dyn SynthesisSink>,
}

impl EvolvingSynthesis {
    /// Create a new synthesis with in-memory storage (for synchronous contexts)
    pub fn new_in_memory(tool_name: &str, session_id: &str) -> Self {
        let state = SynthesisState::new(tool_name.to_string(), session_id.to_string());
        Self {
            state: Arc::new(Mutex::new(state)),
            diffs: Arc::new(Mutex::new(Vec::new())),
            sink: Arc::new(InMemorySink::new()),
        }
    }
}

/// Core trait for synthesis processing
pub trait SynthesisEngine: Send + Sync {
    /// Apply an event to update the synthesis state
    fn apply(&self, event: SynthesisEvent) -> Result<()>;

    /// Get current state snapshot
    fn snapshot(&self) -> SynthesisState;

    /// Get all state diffs since creation
    fn history(&self) -> Vec<StateDiff>;
}

/// Trait for persisting synthesis updates
#[async_trait]
pub trait SynthesisSink: Send + Sync {
    /// Persist a state change
    async fn persist(&self, state: &SynthesisState, diff: &StateDiff) -> Result<()>;

    /// Load previous state if available
    async fn load(&self, tool_name: &str, session_id: &str) -> Result<Option<SynthesisState>>;
}

impl SynthesisEngine for EvolvingSynthesis {
    fn apply(&self, event: SynthesisEvent) -> Result<()> {
        let mut state = self.state.lock();
        let mut diffs = self.diffs.lock();

        let previous_version = state.version;
        state.version += 1;

        // Apply event to state
        match &event {
            SynthesisEvent::Understanding {
                text,
                confidence,
                clarity,
            } => {
                state.current_understanding = text.clone();
                if let Some(conf) = confidence {
                    state.confidence_score = *conf;
                }
                if let Some(clar) = clarity {
                    state.clarity_score = *clar;
                }
            }
            SynthesisEvent::Insight(insight) => {
                state.key_insights.push(insight.clone());
                // Keep only top 5 by confidence
                state
                    .key_insights
                    .sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
                state.key_insights.truncate(5);
            }
            SynthesisEvent::Action(action) => {
                state.action_items.push(action.clone());
                // Keep only top 5
                state.action_items.truncate(5);
            }
            SynthesisEvent::StepComplete { step_number } => {
                state.last_updated_step = *step_number;
            }
        }

        // Create diff record
        let diff = StateDiff {
            previous_version,
            new_version: state.version,
            event: event.clone(),
            timestamp: Utc::now(),
        };

        diffs.push(diff.clone());

        // Clone for async operation
        let state_clone = state.clone();
        let diff_clone = diff.clone();
        let sink = self.sink.clone();

        // Persist asynchronously
        tokio::spawn(async move {
            if let Err(e) = sink.persist(&state_clone, &diff_clone).await {
                tracing::error!("Failed to persist synthesis update: {}", e);
            }
        });

        Ok(())
    }

    fn snapshot(&self) -> SynthesisState {
        self.state.lock().clone()
    }

    fn history(&self) -> Vec<StateDiff> {
        self.diffs.lock().clone()
    }
}

/// Builder for creating synthesis instances
pub struct SynthesisBuilder {
    tool_name: String,
    session_id: String,
    sink: Option<Arc<dyn SynthesisSink>>,
}

impl SynthesisBuilder {
    pub fn new(tool_name: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            session_id: session_id.into(),
            sink: None,
        }
    }

    pub fn with_sink(mut self, sink: Arc<dyn SynthesisSink>) -> Self {
        self.sink = Some(sink);
        self
    }

    pub async fn build(self) -> Result<EvolvingSynthesis> {
        let sink = self.sink.unwrap_or_else(|| Arc::new(InMemorySink::new()));

        // Try to load existing state
        let state = if let Some(loaded) = sink.load(&self.tool_name, &self.session_id).await? {
            loaded
        } else {
            SynthesisState::new(self.tool_name, self.session_id)
        };

        Ok(EvolvingSynthesis {
            state: Arc::new(Mutex::new(state)),
            diffs: Arc::new(Mutex::new(Vec::new())),
            sink,
        })
    }
}

/// Default in-memory sink for testing
pub struct InMemorySink {
    storage: Arc<Mutex<Vec<(SynthesisState, StateDiff)>>>,
}

impl InMemorySink {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl SynthesisSink for InMemorySink {
    async fn persist(&self, state: &SynthesisState, diff: &StateDiff) -> Result<()> {
        self.storage.lock().push((state.clone(), diff.clone()));
        Ok(())
    }

    async fn load(&self, tool_name: &str, session_id: &str) -> Result<Option<SynthesisState>> {
        let storage = self.storage.lock();
        Ok(storage
            .iter()
            .rev()
            .find(|(s, _)| s.tool_name == tool_name && s.session_id == session_id)
            .map(|(s, _)| s.clone()))
    }
}

/// Trait for adapting tool-specific events to synthesis events
pub trait ToolAdaptor: Send + Sync {
    /// Convert tool-specific event to synthesis event(s)
    fn handle_tool_event(&self, event: Self::Event, synthesis: &dyn SynthesisEngine) -> Result<()>
    where
        Self: Sized;

    type Event;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_synthesis_evolution() {
        let synthesis = SynthesisBuilder::new("test_tool", "test_session")
            .build()
            .await
            .unwrap();

        // Apply understanding
        synthesis
            .apply(SynthesisEvent::Understanding {
                text: "Initial understanding".to_string(),
                confidence: Some(0.5),
                clarity: Some(0.6),
            })
            .unwrap();

        // Check state
        let state = synthesis.snapshot();
        assert_eq!(state.version, 1);
        assert_eq!(state.current_understanding, "Initial understanding");
        assert_eq!(state.confidence_score, 0.5);

        // Add insight
        synthesis
            .apply(SynthesisEvent::Insight(InsightEntry {
                insight: "Key finding".to_string(),
                confidence: 0.8,
                source_step: 1,
                supported_by_evidence: true,
            }))
            .unwrap();

        let state = synthesis.snapshot();
        assert_eq!(state.version, 2);
        assert_eq!(state.key_insights.len(), 1);
    }
}
