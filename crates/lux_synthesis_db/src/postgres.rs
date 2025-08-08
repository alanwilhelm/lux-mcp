//! PostgreSQL implementation of SynthesisSink

use anyhow::Result;
use async_trait::async_trait;
use lux_synthesis::{
    events::{ActionItem, InsightEntry, SynthesisEvent},
    StateDiff, SynthesisSink, SynthesisState,
};
use sea_orm::{sea_query::Expr, *};
use tracing::info;
use uuid::Uuid;

// Import entities from main crate (we'll need to move these or re-export)
// For now, we'll define the ActiveModel structures we need

/// PostgreSQL sink for synthesis persistence
pub struct PostgresSink {
    db: DatabaseConnection,
}

impl PostgresSink {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SynthesisSink for PostgresSink {
    async fn persist(&self, state: &SynthesisState, diff: &StateDiff) -> Result<()> {
        use sea_orm::ActiveValue::*;

        // Create synthesis state record
        let synthesis_record = synthesis_states::ActiveModel {
            id: Set(state.id),
            session_id: Set(state
                .session_id
                .parse::<Uuid>()
                .unwrap_or_else(|_| Uuid::new_v4())),
            version: Set(state.version as i32),
            step_number: Set(Some(state.last_updated_step as i32)),
            current_understanding: Set(Some(state.current_understanding.clone())),
            confidence_score: Set(state.confidence_score),
            clarity_score: Set(state.clarity_score),
            created_at: Set(state.created_at.into()),
            raw_update_call: Set(Some(serde_json::to_string(&diff.event)?)),
            parsed_data: Set(serde_json::json!({
                "tool_name": state.tool_name,
                "event": diff.event,
            })),
        };

        // Insert synthesis state
        synthesis_record.insert(&self.db).await?;

        // Handle event-specific data
        match &diff.event {
            SynthesisEvent::Insight(insight) => {
                self.persist_insight(state.id, insight).await?;
            }
            SynthesisEvent::Action(action) => {
                self.persist_action(state.id, action).await?;
            }
            _ => {}
        }

        info!(
            "Persisted synthesis update for {} v{}",
            state.tool_name, state.version
        );
        Ok(())
    }

    async fn load(&self, tool_name: &str, session_id: &str) -> Result<Option<SynthesisState>> {
        use sea_orm::QueryFilter;

        // Try to parse session_id as UUID
        let session_uuid = match session_id.parse::<Uuid>() {
            Ok(uuid) => uuid,
            Err(_) => return Ok(None),
        };

        // Find the latest synthesis state for this tool/session
        // We'll use a custom condition to filter JSON data
        let json_filter = format!("parsed_data->>'tool_name' = '{}'", tool_name);

        let latest = synthesis_states::Entity::find()
            .filter(synthesis_states::Column::SessionId.eq(session_uuid))
            .filter(Condition::all().add(Expr::cust(&json_filter)))
            .order_by_desc(synthesis_states::Column::Version)
            .one(&self.db)
            .await?;

        if let Some(record) = latest {
            // Load related insights and actions
            let insights = insights::Entity::find()
                .filter(insights::Column::SynthesisStateId.eq(record.id))
                .all(&self.db)
                .await?;

            let actions = action_items::Entity::find()
                .filter(action_items::Column::SynthesisStateId.eq(record.id))
                .all(&self.db)
                .await?;

            // Reconstruct SynthesisState
            Ok(Some(SynthesisState {
                id: record.id,
                tool_name: tool_name.to_string(),
                session_id: session_id.to_string(),
                version: record.version as u32,
                current_understanding: record.current_understanding.unwrap_or_default(),
                key_insights: insights
                    .into_iter()
                    .map(|i| InsightEntry {
                        insight: i.insight_text,
                        confidence: i.confidence.unwrap_or(0.5),
                        source_step: i.source_step.unwrap_or(0) as u32,
                        supported_by_evidence: i.supported_by_evidence.unwrap_or(true),
                    })
                    .collect(),
                action_items: actions
                    .into_iter()
                    .map(|a| ActionItem {
                        action: a.action_text,
                        priority: match a.priority.as_deref() {
                            Some("high") => lux_synthesis::events::Priority::High,
                            Some("low") => lux_synthesis::events::Priority::Low,
                            _ => lux_synthesis::events::Priority::Medium,
                        },
                        rationale: a.rationale.unwrap_or_default(),
                        dependencies: a
                            .dependencies
                            .and_then(|d| serde_json::from_value::<Vec<String>>(d).ok())
                            .unwrap_or_default(),
                    })
                    .collect(),
                confidence_score: record.confidence_score,
                clarity_score: record.clarity_score,
                created_at: record.created_at.into(),
                last_updated_step: record.step_number.unwrap_or(0) as u32,
            }))
        } else {
            Ok(None)
        }
    }
}

impl PostgresSink {
    async fn persist_insight(&self, synthesis_id: Uuid, insight: &InsightEntry) -> Result<()> {
        use sea_orm::ActiveValue::*;

        let insight_record = insights::ActiveModel {
            id: Set(Uuid::new_v4()),
            synthesis_state_id: Set(synthesis_id),
            insight_text: Set(insight.insight.clone()),
            confidence: Set(Some(insight.confidence)),
            source_step: Set(Some(insight.source_step as i32)),
            supported_by_evidence: Set(Some(insight.supported_by_evidence)),
            created_at: Set(chrono::Utc::now().into()),
        };

        insight_record.insert(&self.db).await?;
        Ok(())
    }

    async fn persist_action(&self, synthesis_id: Uuid, action: &ActionItem) -> Result<()> {
        use sea_orm::ActiveValue::*;

        let action_record = action_items::ActiveModel {
            id: Set(Uuid::new_v4()),
            synthesis_state_id: Set(synthesis_id),
            action_text: Set(action.action.clone()),
            priority: Set(Some(format!("{:?}", action.priority).to_lowercase())),
            rationale: Set(Some(action.rationale.clone())),
            dependencies: Set(Some(serde_json::to_value(&action.dependencies)?)),
            created_at: Set(chrono::Utc::now().into()),
        };

        action_record.insert(&self.db).await?;
        Ok(())
    }
}

// Temporary entity definitions - these should come from the main crate
mod synthesis_states {
    use sea_orm::entity::prelude::*;
    use serde_json::Value as Json;
    use uuid::Uuid;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "synthesis_states")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        pub session_id: Uuid,
        pub version: i32,
        pub step_number: Option<i32>,
        pub current_understanding: Option<String>,
        pub confidence_score: f32,
        pub clarity_score: f32,
        pub created_at: DateTimeWithTimeZone,
        pub raw_update_call: Option<String>,
        pub parsed_data: Json,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod insights {
    use sea_orm::entity::prelude::*;
    use uuid::Uuid;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "insights")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        pub synthesis_state_id: Uuid,
        pub insight_text: String,
        pub confidence: Option<f32>,
        pub source_step: Option<i32>,
        pub supported_by_evidence: Option<bool>,
        pub created_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod action_items {
    use sea_orm::entity::prelude::*;
    use serde_json::Value as Json;
    use uuid::Uuid;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "action_items")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: Uuid,
        pub synthesis_state_id: Uuid,
        pub action_text: String,
        pub priority: Option<String>,
        pub rationale: Option<String>,
        pub dependencies: Option<Json>,
        pub created_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
