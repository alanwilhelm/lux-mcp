use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Simple sequential thinking tool - pure state tracking without LLM
/// Inspired by Anthropic's sequential-thinking-mcp but in Rust

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtData {
    pub thought: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,

    #[serde(default)]
    pub is_revision: bool,
    #[serde(default)]
    pub revises_thought: Option<u32>,
    #[serde(default)]
    pub branch_from_thought: Option<u32>,
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub needs_more_thoughts: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SequentialThinkingRequest {
    pub thought: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,

    #[serde(default)]
    pub is_revision: bool,
    #[serde(default)]
    pub revises_thought: Option<u32>,
    #[serde(default)]
    pub branch_from_thought: Option<u32>,
    #[serde(default)]
    pub branch_id: Option<String>,
    #[serde(default)]
    pub needs_more_thoughts: bool,

    // Optional session support
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SequentialThinkingResponse {
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,
    pub branches: Vec<String>,
    pub thought_history_length: usize,

    // Echo back the session if provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    // Simple status field
    pub status: String, // "recorded", "revision", "branch", "complete"
}

#[derive(Debug, Default)]
struct SessionState {
    thought_history: Vec<ThoughtData>,
    branches: HashMap<String, Vec<ThoughtData>>,
}

/// Tool for managing sequential thinking sessions
pub struct SequentialThinkingTool {
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
}

impl SequentialThinkingTool {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn process_thought(
        &self,
        request: SequentialThinkingRequest,
    ) -> Result<SequentialThinkingResponse> {
        let session_id = request
            .session_id
            .clone()
            .unwrap_or_else(|| "default".to_string());

        let mut sessions = self.sessions.lock();
        let session = sessions
            .entry(session_id.clone())
            .or_insert_with(SessionState::default);

        // Create thought data
        let mut thought_data = ThoughtData {
            thought: request.thought,
            thought_number: request.thought_number,
            total_thoughts: request.total_thoughts,
            next_thought_needed: request.next_thought_needed,
            is_revision: request.is_revision,
            revises_thought: request.revises_thought,
            branch_from_thought: request.branch_from_thought,
            branch_id: request.branch_id.clone(),
            needs_more_thoughts: request.needs_more_thoughts,
        };

        // Auto-adjust total_thoughts if needed
        if thought_data.thought_number > thought_data.total_thoughts {
            thought_data.total_thoughts = thought_data.thought_number;
        }

        // Add to history
        session.thought_history.push(thought_data.clone());

        // Track branches
        if let (Some(_), Some(branch_id)) =
            (thought_data.branch_from_thought, &thought_data.branch_id)
        {
            session
                .branches
                .entry(branch_id.clone())
                .or_insert_with(Vec::new)
                .push(thought_data.clone());
        }

        // Determine status
        let status = if !thought_data.next_thought_needed {
            "complete"
        } else if thought_data.is_revision {
            "revision"
        } else if thought_data.branch_from_thought.is_some() {
            "branch"
        } else {
            "recorded"
        };

        // Log if enabled (similar to Anthropic's DISABLE_THOUGHT_LOGGING)
        if std::env::var("DISABLE_THOUGHT_LOGGING")
            .unwrap_or_default()
            .to_lowercase()
            != "true"
        {
            Self::log_thought(&thought_data);
        }

        let branches: Vec<String> = session.branches.keys().cloned().collect();

        info!(
            "Sequential thought {}/{} recorded - Status: {}, Branches: {}, History: {}",
            thought_data.thought_number,
            thought_data.total_thoughts,
            status,
            branches.len(),
            session.thought_history.len()
        );

        Ok(SequentialThinkingResponse {
            thought_number: thought_data.thought_number,
            total_thoughts: thought_data.total_thoughts,
            next_thought_needed: thought_data.next_thought_needed,
            branches,
            thought_history_length: session.thought_history.len(),
            session_id: if session_id == "default" {
                None
            } else {
                Some(session_id)
            },
            status: status.to_string(),
        })
    }

    fn log_thought(thought_data: &ThoughtData) {
        let prefix = if thought_data.is_revision {
            format!(
                "🔄 Revision (revising thought {})",
                thought_data.revises_thought.unwrap_or(0)
            )
        } else if let Some(branch_from) = thought_data.branch_from_thought {
            format!(
                "🌿 Branch (from thought {}, ID: {})",
                branch_from,
                thought_data
                    .branch_id
                    .as_ref()
                    .unwrap_or(&"unknown".to_string())
            )
        } else {
            "💭 Thought".to_string()
        };

        debug!(
            "\n┌─────────────────────────────────────────┐\n\
             │ {} {}/{}                                 │\n\
             ├─────────────────────────────────────────┤\n\
             │ {}                                       │\n\
             └─────────────────────────────────────────┘",
            prefix, thought_data.thought_number, thought_data.total_thoughts, thought_data.thought
        );
    }

    // Helper function to clear a session
    pub fn clear_session(&self, session_id: Option<String>) -> Result<()> {
        let session_id = session_id.unwrap_or_else(|| "default".to_string());
        let mut sessions = self.sessions.lock();
        sessions.remove(&session_id);
        info!("Cleared session: {}", session_id);
        Ok(())
    }

    // Helper function to get session summary
    pub fn get_session_summary(&self, session_id: Option<String>) -> Result<String> {
        let session_id = session_id.unwrap_or_else(|| "default".to_string());
        let sessions = self.sessions.lock();

        if let Some(session) = sessions.get(&session_id) {
            let summary = format!(
                "Session '{}': {} thoughts, {} branches",
                session_id,
                session.thought_history.len(),
                session.branches.len()
            );
            Ok(summary)
        } else {
            Ok(format!("No session found with ID: {}", session_id))
        }
    }
}

impl Default for SequentialThinkingTool {
    fn default() -> Self {
        Self::new()
    }
}
