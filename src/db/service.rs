//! Database service layer using SeaORM

use super::connection::DatabaseConnection;
use anyhow::Result;
use chrono::Utc;
use sea_orm::*;
use serde_json::json;
use uuid::Uuid;

use crate::entities::{
    action_items as action_item, bias_detections as bias_detection, insights as insight,
    reasoning_steps as reasoning_step, session_models as session_model, sessions as session,
    synthesis_states as synthesis_state,
};
use crate::tools::biased_reasoning::{
    BiasedReasoningRequest, BiasedReasoningResponse, NextAction, Severity, StepType,
};
use crate::tools::biased_reasoning_synthesis::{
    ActionItem as SynthesisAction, InsightEntry, SynthesisPatch,
};

pub struct DatabaseService {
    db: DatabaseConnection,
}

impl DatabaseService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get a reference to the underlying database connection
    pub fn pool(&self) -> sea_orm::DatabaseConnection {
        self.db.get_connection().clone()
    }

    /// Create or get a session for biased reasoning
    pub async fn create_or_get_session(
        &self,
        session_id: &str,
        request: &BiasedReasoningRequest,
    ) -> Result<session::Model> {
        // Try to find existing session
        let existing = session::Entity::find()
            .filter(session::Column::SessionExternalId.eq(session_id))
            .one(self.db.get_connection())
            .await?;

        if let Some(session) = existing {
            return Ok(session);
        }

        // Create new session
        let new_session = session::ActiveModel {
            id: Set(Uuid::new_v4()),
            session_type: Set("biased_reasoning".to_string()),
            session_external_id: Set(session_id.to_string()),
            query: Set(request.query.clone()),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            status: Set("active".to_string()),
            metadata: Set(json!({
                "max_analysis_rounds": request.max_analysis_rounds,
            })),
            ..Default::default()
        };

        Ok(new_session.insert(self.db.get_connection()).await?)
    }

    /// Log models used in a session
    pub async fn log_session_models(
        &self,
        session_id: Uuid,
        primary_model: &str,
        verifier_model: &str,
    ) -> Result<()> {
        let models = vec![
            session_model::ActiveModel {
                id: Set(Uuid::new_v4()),
                session_id: Set(session_id),
                role: Set("primary".to_string()),
                model_name: Set(primary_model.to_string()),
                model_provider: Set(self.detect_provider(primary_model)),
                created_at: Set(Utc::now().into()),
            },
            session_model::ActiveModel {
                id: Set(Uuid::new_v4()),
                session_id: Set(session_id),
                role: Set("verifier".to_string()),
                model_name: Set(verifier_model.to_string()),
                model_provider: Set(self.detect_provider(verifier_model)),
                created_at: Set(Utc::now().into()),
            },
        ];

        session_model::Entity::insert_many(models)
            .exec(self.db.get_connection())
            .await?;

        Ok(())
    }

    /// Log a reasoning step
    pub async fn log_reasoning_step(
        &self,
        session_id: Uuid,
        response: &BiasedReasoningResponse,
        raw_llm_response: Option<&str>,
        thinking_time_ms: Option<i32>,
    ) -> Result<reasoning_step::Model> {
        let step = reasoning_step::ActiveModel {
            id: Set(Uuid::new_v4()),
            session_id: Set(session_id),
            step_number: Set(response.step_number as i32),
            step_type: Set(format!("{:?}", response.step_type)),
            content: Set(response.content.clone()),
            raw_llm_response: Set(raw_llm_response.map(|s| s.to_string())),
            model_used: Set(Some(response.model_used.clone())),
            confidence_score: Set(response
                .reasoning_metadata
                .as_ref()
                .map(|m| m.confidence_level as f32)),
            thinking_time_ms: Set(thinking_time_ms),
            created_at: Set(Utc::now().into()),
            metadata: Set(json!({
                "next_action": format!("{:?}", response.next_action),
                "session_status": response.session_status,
            })),
            ..Default::default()
        };

        Ok(step.insert(self.db.get_connection()).await?)
    }

    /// Log a synthesis state update
    pub async fn log_synthesis_update(
        &self,
        session_id: Uuid,
        version: i32,
        step_number: i32,
        patch: &SynthesisPatch,
        raw_update_call: &str,
    ) -> Result<synthesis_state::Model> {
        let synthesis = synthesis_state::ActiveModel {
            id: Set(Uuid::new_v4()),
            session_id: Set(session_id),
            version: Set(version),
            step_number: Set(Some(step_number)),
            current_understanding: Set(patch.current_understanding.clone()),
            confidence_score: Set(patch.confidence_score.unwrap_or(0.0)),
            clarity_score: Set(patch.clarity_score.unwrap_or(0.0)),
            created_at: Set(Utc::now().into()),
            raw_update_call: Set(Some(raw_update_call.to_string())),
            parsed_data: Set(serde_json::to_value(patch).unwrap_or(json!({}))),
        };

        let synthesis_model = synthesis.insert(self.db.get_connection()).await?;

        // Log insights if provided
        if let Some(insights) = &patch.key_insights {
            self.log_insights(synthesis_model.id, insights).await?;
        }

        // Log action items if provided
        if let Some(actions) = &patch.action_items {
            self.log_action_items(synthesis_model.id, actions).await?;
        }

        Ok(synthesis_model)
    }

    /// Log insights for a synthesis state
    async fn log_insights(
        &self,
        synthesis_state_id: Uuid,
        insights: &[InsightEntry],
    ) -> Result<()> {
        let insight_models: Vec<insight::ActiveModel> = insights
            .iter()
            .map(|i| insight::ActiveModel {
                id: Set(Uuid::new_v4()),
                synthesis_state_id: Set(synthesis_state_id),
                insight_text: Set(i.insight.clone()),
                confidence: Set(Some(i.confidence)),
                source_step: Set(Some(i.source_step as i32)),
                supported_by_evidence: Set(Some(i.supported_by_evidence)),
                created_at: Set(Utc::now().into()),
            })
            .collect();

        if !insight_models.is_empty() {
            insight::Entity::insert_many(insight_models)
                .exec(self.db.get_connection())
                .await?;
        }

        Ok(())
    }

    /// Log action items for a synthesis state
    async fn log_action_items(
        &self,
        synthesis_state_id: Uuid,
        actions: &[SynthesisAction],
    ) -> Result<()> {
        let action_models: Vec<action_item::ActiveModel> = actions
            .iter()
            .map(|a| action_item::ActiveModel {
                id: Set(Uuid::new_v4()),
                synthesis_state_id: Set(synthesis_state_id),
                action_text: Set(a.action.clone()),
                priority: Set(Some(format!("{:?}", a.priority).to_lowercase())),
                rationale: Set(Some(a.rationale.clone())),
                dependencies: Set(Some(a.dependencies.clone())),
                created_at: Set(Utc::now().into()),
            })
            .collect();

        if !action_models.is_empty() {
            action_item::Entity::insert_many(action_models)
                .exec(self.db.get_connection())
                .await?;
        }

        Ok(())
    }

    /// Log bias detection result
    pub async fn log_bias_detection(
        &self,
        session_id: Uuid,
        step_number: i32,
        bias_result: &crate::tools::biased_reasoning::BiasCheckResult,
    ) -> Result<()> {
        let bias = bias_detection::ActiveModel {
            id: Set(Uuid::new_v4()),
            session_id: Set(session_id),
            step_number: Set(step_number),
            has_bias: Set(bias_result.has_bias),
            severity: Set(Some(format!("{:?}", bias_result.severity).to_lowercase())),
            bias_types: Set(Some(
                bias_result
                    .bias_types
                    .iter()
                    .map(|b| format!("{:?}", b))
                    .collect(),
            )),
            suggestions: Set(Some(bias_result.suggestions.clone())),
            confidence: Set(Some(bias_result.confidence)),
            created_at: Set(Utc::now().into()),
        };

        bias.insert(self.db.get_connection()).await?;
        Ok(())
    }

    /// Get session with all related data
    pub async fn get_session_with_data(
        &self,
        session_id: &str,
    ) -> Result<
        Option<(
            session::Model,
            Vec<reasoning_step::Model>,
            Vec<synthesis_state::Model>,
        )>,
    > {
        let session = session::Entity::find()
            .filter(session::Column::SessionExternalId.eq(session_id))
            .one(self.db.get_connection())
            .await?;

        if let Some(session) = session {
            let steps = reasoning_step::Entity::find()
                .filter(reasoning_step::Column::SessionId.eq(session.id))
                .order_by_asc(reasoning_step::Column::StepNumber)
                .all(self.db.get_connection())
                .await?;

            let synthesis_states = synthesis_state::Entity::find()
                .filter(synthesis_state::Column::SessionId.eq(session.id))
                .order_by_asc(synthesis_state::Column::Version)
                .all(self.db.get_connection())
                .await?;

            Ok(Some((session, steps, synthesis_states)))
        } else {
            Ok(None)
        }
    }

    /// Mark session as completed
    pub async fn complete_session(&self, session_id: Uuid) -> Result<()> {
        let mut session: session::ActiveModel = session::Entity::find_by_id(session_id)
            .one(self.db.get_connection())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?
            .into();

        session.status = Set("completed".to_string());
        session.completed_at = Set(Some(Utc::now().into()));
        session.update(self.db.get_connection()).await?;

        Ok(())
    }

    fn detect_provider(&self, model: &str) -> Option<String> {
        if model.starts_with("gpt") || model.starts_with("o3") || model.starts_with("o4") {
            Some("openai".to_string())
        } else if model.contains("/") {
            Some("openrouter".to_string())
        } else {
            None
        }
    }

    /// High-level method to log a complete biased reasoning step
    pub async fn log_biased_reasoning_step(
        &self,
        request: &BiasedReasoningRequest,
        response: &BiasedReasoningResponse,
        thinking_time_ms: i32,
    ) -> Result<()> {
        // Create or get session
        let session = self
            .create_or_get_session(&response.session_id, request)
            .await?;

        // Log reasoning step
        let _step = self
            .log_reasoning_step(
                session.id,
                response,
                None, // We could add raw LLM response later
                Some(thinking_time_ms),
            )
            .await?;

        // If synthesis state is present, log it
        if let Some(synthesis) = &response.synthesis_snapshot {
            // Convert SynthesisSnapshot to a simplified patch
            // Map confidence level string to numeric score
            let confidence_score = match synthesis.confidence_level.as_str() {
                "high" => 0.9,
                "medium" => 0.6,
                "low" => 0.3,
                _ => 0.5,
            };

            // Convert top insights to InsightEntry format
            let insights: Vec<InsightEntry> = synthesis
                .top_insights
                .iter()
                .enumerate()
                .map(|(i, insight)| InsightEntry {
                    insight: insight.clone(),
                    confidence: confidence_score,
                    source_step: response.step_number,
                    supported_by_evidence: true,
                })
                .collect();

            // Convert next actions to ActionItem format
            let actions: Vec<SynthesisAction> = synthesis
                .next_actions
                .iter()
                .enumerate()
                .map(|(i, action)| SynthesisAction {
                    action: action.clone(),
                    priority: crate::tools::biased_reasoning_synthesis::Priority::High,
                    rationale: String::new(),
                    dependencies: vec![],
                })
                .collect();

            let patch = SynthesisPatch {
                current_understanding: Some(synthesis.current_understanding.clone()),
                confidence_score: Some(confidence_score),
                clarity_score: Some(confidence_score), // Use same value for now
                key_insights: if insights.is_empty() {
                    None
                } else {
                    Some(insights)
                },
                action_items: if actions.is_empty() {
                    None
                } else {
                    Some(actions)
                },
                recommendations: None,
                context_factors: None,
                constraints: None,
                last_updated_step: Some(response.step_number),
            };

            // Use step number as version since SynthesisSnapshot doesn't have version
            self.log_synthesis_update(
                session.id,
                response.step_number as i32, // Use step number as version
                response.step_number as i32,
                &patch,
                &format!("Step {}: {:?}", response.step_number, response.step_type),
            )
            .await?;
        }

        // Log bias detection if present
        if response.step_type == StepType::BiasAnalysis {
            // Extract bias check result from content
            // For now, create a basic bias check result
            let bias_result = crate::tools::biased_reasoning::BiasCheckResult {
                has_bias: response.content.contains("bias detected")
                    || response.content.contains("potential bias"),
                severity: Severity::Low,
                bias_types: vec![],
                explanation: String::new(),
                suggestions: vec![],
                confidence: 0.8,
            };

            self.log_bias_detection(session.id, response.step_number as i32, &bias_result)
                .await?;
        }

        // Mark session as completed if needed
        if response.next_action == NextAction::Complete {
            self.complete_session(session.id).await?;
        }

        Ok(())
    }
}
