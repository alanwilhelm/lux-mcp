//! Event definitions for synthesis system

use serde::{Deserialize, Serialize};

/// Events that can update synthesis state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SynthesisEvent {
    /// Update the current understanding
    Understanding {
        text: String,
        confidence: Option<f32>,
        clarity: Option<f32>,
    },

    /// Add a new insight
    Insight(InsightEntry),

    /// Add a new action item
    Action(ActionItem),

    /// Mark step completion
    StepComplete { step_number: u32 },
}

/// An insight discovered during reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightEntry {
    pub insight: String, // ≤15 words
    pub confidence: f32, // 0.0 to 1.0
    pub source_step: u32,
    pub supported_by_evidence: bool,
}

/// An actionable item to pursue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub action: String, // ≤20 words
    pub priority: Priority,
    pub rationale: String,
    pub dependencies: Vec<String>,
}

/// Priority levels for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// Tool-specific event types (examples for each tool)
pub mod tool_events {

    /// Events from biased_reasoning tool
    #[derive(Debug, Clone)]
    pub enum BiasedReasoningEvent {
        ReasoningStep {
            content: String,
            confidence: f32,
        },
        BiasDetected {
            bias_type: String,
            severity: String,
            suggestions: Vec<String>,
        },
        CorrectionMade {
            original: String,
            corrected: String,
        },
    }

    /// Events from traced_reasoning tool  
    #[derive(Debug, Clone)]
    pub enum TracedReasoningEvent {
        ThoughtCompleted {
            thought: String,
            depth: u32,
            is_revision: bool,
        },
        CircularReasoningDetected {
            similarity: f32,
        },
        DistractorDetected {
            relevance: f32,
        },
    }

    /// Events from planner tool
    #[derive(Debug, Clone)]
    pub enum PlannerEvent {
        StepPlanned {
            step: String,
            dependencies: Vec<String>,
        },
        BranchCreated {
            from_step: u32,
            branch_id: String,
        },
        PlanRevised {
            step_number: u32,
            new_content: String,
        },
    }
}
