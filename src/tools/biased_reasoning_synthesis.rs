use serde::{Deserialize, Serialize};

// Core synthesis structure that evolves throughout reasoning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvolvingSynthesis {
    pub current_understanding: String,   // ≤ 3 sentences
    pub key_insights: Vec<InsightEntry>, // max 5 entries
    pub action_items: Vec<ActionItem>,   // max 5 active
    pub confidence_score: f32,           // 0.0 to 1.0
    pub clarity_score: f32,              // 0.0 to 1.0
    pub recommendations: Vec<Recommendation>,
    pub context_factors: Vec<String>,
    pub constraints: Vec<String>,
    pub version: u32,
    pub last_updated_step: u32,
}

// Patch structure for incremental updates
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SynthesisPatch {
    pub current_understanding: Option<String>,
    pub key_insights: Option<Vec<InsightEntry>>,
    pub action_items: Option<Vec<ActionItem>>,
    pub confidence_score: Option<f32>,
    pub clarity_score: Option<f32>,
    pub recommendations: Option<Vec<Recommendation>>,
    pub context_factors: Option<Vec<String>>,
    pub constraints: Option<Vec<String>>,
    pub last_updated_step: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightEntry {
    pub insight: String, // ≤15 words
    pub confidence: f32,
    pub source_step: u32,
    pub supported_by_evidence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub action: String,
    pub priority: Priority,
    pub rationale: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation: String,
    pub strength: RecommendationStrength,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecommendationStrength {
    Strong,
    Moderate,
    Weak,
}

// Snapshot for including in responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisSnapshot {
    pub current_understanding: String,
    pub top_insights: Vec<String>, // Top 3-5
    pub next_actions: Vec<String>, // Immediate actions
    pub confidence_level: String,  // "low", "medium", "high"
    pub ready_for_decision: bool,
}

// Final comprehensive synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalSynthesis {
    pub executive_summary: String,
    pub conclusion: Conclusion,
    pub action_plan: ActionPlan,
    pub full_context: Context,
    pub meta: MetaInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conclusion {
    pub main_finding: String,
    pub supporting_points: Vec<String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    pub immediate_actions: Vec<ActionItem>,
    pub medium_term_actions: Vec<ActionItem>,
    pub considerations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub key_factors: Vec<String>,
    pub assumptions_made: Vec<String>,
    pub methodology: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaInfo {
    pub total_steps: u32,
    pub confidence_score: f32,
    pub biases_detected: Vec<String>,
    pub models_used: Vec<String>,
}

impl EvolvingSynthesis {
    pub fn merge_patch(&mut self, patch: SynthesisPatch) {
        if let Some(v) = patch.current_understanding {
            self.current_understanding = v;
        }
        if let Some(v) = patch.key_insights {
            // Keep only top 5 by confidence
            self.key_insights = v;
            self.key_insights
                .sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
            self.key_insights.truncate(5);
        }
        if let Some(v) = patch.action_items {
            // Keep only top 5 by priority
            self.action_items = v;
            self.action_items.truncate(5);
        }
        if let Some(v) = patch.confidence_score {
            self.confidence_score = v;
        }
        if let Some(v) = patch.clarity_score {
            self.clarity_score = v;
        }
        if let Some(v) = patch.recommendations {
            self.recommendations = v;
        }
        if let Some(v) = patch.context_factors {
            self.context_factors = v;
        }
        if let Some(v) = patch.constraints {
            self.constraints = v;
        }
        if let Some(v) = patch.last_updated_step {
            self.last_updated_step = v;
        }
        self.version += 1;
    }

    pub fn to_snapshot(&self) -> SynthesisSnapshot {
        let confidence_level = match self.confidence_score {
            x if x >= 0.8 => "high",
            x if x >= 0.5 => "medium",
            _ => "low",
        }
        .to_string();

        SynthesisSnapshot {
            current_understanding: self.current_understanding.clone(),
            top_insights: self
                .key_insights
                .iter()
                .take(3)
                .map(|i| i.insight.clone())
                .collect(),
            next_actions: self
                .action_items
                .iter()
                .filter(|a| matches!(a.priority, Priority::High))
                .take(3)
                .map(|a| a.action.clone())
                .collect(),
            confidence_level,
            ready_for_decision: self.confidence_score >= 0.7 && self.version >= 3,
        }
    }

    pub fn to_final_synthesis(&self, total_steps: u32, models_used: Vec<String>) -> FinalSynthesis {
        FinalSynthesis {
            executive_summary: self.current_understanding.clone(),
            conclusion: Conclusion {
                main_finding: self
                    .key_insights
                    .first()
                    .map(|i| i.insight.clone())
                    .unwrap_or_else(|| "No clear finding emerged".to_string()),
                supporting_points: self
                    .key_insights
                    .iter()
                    .skip(1)
                    .filter(|i| i.confidence > 0.7)
                    .map(|i| i.insight.clone())
                    .collect(),
                caveats: self.constraints.clone(),
            },
            action_plan: ActionPlan {
                immediate_actions: self
                    .action_items
                    .iter()
                    .filter(|a| matches!(a.priority, Priority::High))
                    .cloned()
                    .collect(),
                medium_term_actions: self
                    .action_items
                    .iter()
                    .filter(|a| matches!(a.priority, Priority::Medium))
                    .cloned()
                    .collect(),
                considerations: self
                    .action_items
                    .iter()
                    .filter(|a| matches!(a.priority, Priority::Low))
                    .map(|a| a.rationale.clone())
                    .collect(),
            },
            full_context: Context {
                key_factors: self.context_factors.clone(),
                assumptions_made: vec![], // Would be populated from session data
                methodology: "Dual-model reasoning with bias checking".to_string(),
            },
            meta: MetaInfo {
                total_steps,
                confidence_score: self.confidence_score,
                biases_detected: vec![], // Would be populated from session data
                models_used,
            },
        }
    }
}

// Event store for auditability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisEvent {
    pub step_number: u32,
    pub patch: SynthesisPatch,
    pub timestamp: String,
    pub event_type: EventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    ReasoningUpdate,
    BiasCheckRefinement,
    UserGuidance,
    FinalCompilation,
}

// Store for maintaining synthesis history
pub struct SynthesisStore {
    pub current: EvolvingSynthesis,
    pub events: Vec<SynthesisEvent>,
}

impl SynthesisStore {
    pub fn new() -> Self {
        Self {
            current: EvolvingSynthesis::default(),
            events: Vec::new(),
        }
    }

    pub fn apply_patch(&mut self, patch: SynthesisPatch, step_number: u32, event_type: EventType) {
        // Store event
        self.events.push(SynthesisEvent {
            step_number,
            patch: patch.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type,
        });

        // Apply to current
        self.current.merge_patch(patch);
    }
}
