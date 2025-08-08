use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create sessions table
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Sessions::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Sessions::SessionType).string().not_null())
                    .col(
                        ColumnDef::new(Sessions::SessionExternalId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Sessions::Query).text().not_null())
                    .col(
                        ColumnDef::new(Sessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Sessions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Sessions::CompletedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Sessions::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(Sessions::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .to_owned(),
            )
            .await?;

        // Create session_models table
        manager
            .create_table(
                Table::create()
                    .table(SessionModels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SessionModels::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SessionModels::SessionId).uuid().not_null())
                    .col(ColumnDef::new(SessionModels::Role).string().not_null())
                    .col(ColumnDef::new(SessionModels::ModelName).string().not_null())
                    .col(ColumnDef::new(SessionModels::ModelProvider).string())
                    .col(
                        ColumnDef::new(SessionModels::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SessionModels::Table, SessionModels::SessionId)
                            .to(Sessions::Table, Sessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create reasoning_steps table
        manager
            .create_table(
                Table::create()
                    .table(ReasoningSteps::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReasoningSteps::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ReasoningSteps::SessionId).uuid().not_null())
                    .col(
                        ColumnDef::new(ReasoningSteps::StepNumber)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ReasoningSteps::StepType).string().not_null())
                    .col(ColumnDef::new(ReasoningSteps::Content).text().not_null())
                    .col(ColumnDef::new(ReasoningSteps::RawLlmResponse).text())
                    .col(ColumnDef::new(ReasoningSteps::ModelUsed).string())
                    .col(ColumnDef::new(ReasoningSteps::ConfidenceScore).float())
                    .col(ColumnDef::new(ReasoningSteps::ClarityScore).float())
                    .col(ColumnDef::new(ReasoningSteps::ThinkingTimeMs).integer())
                    .col(ColumnDef::new(ReasoningSteps::TokensUsed).integer())
                    .col(
                        ColumnDef::new(ReasoningSteps::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ReasoningSteps::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ReasoningSteps::Table, ReasoningSteps::SessionId)
                            .to(Sessions::Table, Sessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on reasoning_steps
        manager
            .create_index(
                Index::create()
                    .table(ReasoningSteps::Table)
                    .name("idx_reasoning_steps_session_step")
                    .col(ReasoningSteps::SessionId)
                    .col(ReasoningSteps::StepNumber)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create synthesis_states table
        manager
            .create_table(
                Table::create()
                    .table(SynthesisStates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SynthesisStates::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SynthesisStates::SessionId).uuid().not_null())
                    .col(
                        ColumnDef::new(SynthesisStates::Version)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SynthesisStates::StepNumber).integer())
                    .col(ColumnDef::new(SynthesisStates::CurrentUnderstanding).text())
                    .col(
                        ColumnDef::new(SynthesisStates::ConfidenceScore)
                            .float()
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(SynthesisStates::ClarityScore)
                            .float()
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(SynthesisStates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(SynthesisStates::RawUpdateCall).text())
                    .col(
                        ColumnDef::new(SynthesisStates::ParsedData)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SynthesisStates::Table, SynthesisStates::SessionId)
                            .to(Sessions::Table, Sessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on synthesis_states
        manager
            .create_index(
                Index::create()
                    .table(SynthesisStates::Table)
                    .name("idx_synthesis_states_session_version")
                    .col(SynthesisStates::SessionId)
                    .col(SynthesisStates::Version)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create insights table
        manager
            .create_table(
                Table::create()
                    .table(Insights::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Insights::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Insights::SynthesisStateId).uuid().not_null())
                    .col(ColumnDef::new(Insights::InsightText).text().not_null())
                    .col(ColumnDef::new(Insights::Confidence).float())
                    .col(ColumnDef::new(Insights::SourceStep).integer())
                    .col(
                        ColumnDef::new(Insights::SupportedByEvidence)
                            .boolean()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Insights::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Insights::Table, Insights::SynthesisStateId)
                            .to(SynthesisStates::Table, SynthesisStates::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create action_items table
        manager
            .create_table(
                Table::create()
                    .table(ActionItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ActionItems::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ActionItems::SynthesisStateId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ActionItems::ActionText).text().not_null())
                    .col(ColumnDef::new(ActionItems::Priority).string())
                    .col(ColumnDef::new(ActionItems::Rationale).text())
                    .col(ColumnDef::new(ActionItems::Dependencies).array(ColumnType::Text))
                    .col(
                        ColumnDef::new(ActionItems::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ActionItems::Table, ActionItems::SynthesisStateId)
                            .to(SynthesisStates::Table, SynthesisStates::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create bias_detections table
        manager
            .create_table(
                Table::create()
                    .table(BiasDetections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BiasDetections::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BiasDetections::SessionId).uuid().not_null())
                    .col(
                        ColumnDef::new(BiasDetections::StepNumber)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(BiasDetections::HasBias).boolean().not_null())
                    .col(ColumnDef::new(BiasDetections::Severity).string())
                    .col(ColumnDef::new(BiasDetections::BiasTypes).array(ColumnType::Text))
                    .col(ColumnDef::new(BiasDetections::Suggestions).array(ColumnType::Text))
                    .col(ColumnDef::new(BiasDetections::Confidence).float())
                    .col(
                        ColumnDef::new(BiasDetections::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(BiasDetections::Table, BiasDetections::SessionId)
                            .to(Sessions::Table, Sessions::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .table(Sessions::Table)
                    .name("idx_sessions_type")
                    .col(Sessions::SessionType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Sessions::Table)
                    .name("idx_sessions_status")
                    .col(Sessions::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Sessions::Table)
                    .name("idx_sessions_created")
                    .col(Sessions::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order due to foreign keys
        manager
            .drop_table(Table::drop().table(BiasDetections::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ActionItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Insights::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SynthesisStates::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ReasoningSteps::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SessionModels::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Define table identifiers
#[derive(Iden)]
enum Sessions {
    Table,
    Id,
    SessionType,
    SessionExternalId,
    Query,
    CreatedAt,
    UpdatedAt,
    CompletedAt,
    Status,
    Metadata,
}

#[derive(Iden)]
enum SessionModels {
    Table,
    Id,
    SessionId,
    Role,
    ModelName,
    ModelProvider,
    CreatedAt,
}

#[derive(Iden)]
enum ReasoningSteps {
    Table,
    Id,
    SessionId,
    StepNumber,
    StepType,
    Content,
    RawLlmResponse,
    ModelUsed,
    ConfidenceScore,
    ClarityScore,
    ThinkingTimeMs,
    TokensUsed,
    CreatedAt,
    Metadata,
}

#[derive(Iden)]
enum SynthesisStates {
    Table,
    Id,
    SessionId,
    Version,
    StepNumber,
    CurrentUnderstanding,
    ConfidenceScore,
    ClarityScore,
    CreatedAt,
    RawUpdateCall,
    ParsedData,
}

#[derive(Iden)]
enum Insights {
    Table,
    Id,
    SynthesisStateId,
    InsightText,
    Confidence,
    SourceStep,
    SupportedByEvidence,
    CreatedAt,
}

#[derive(Iden)]
enum ActionItems {
    Table,
    Id,
    SynthesisStateId,
    ActionText,
    Priority,
    Rationale,
    Dependencies,
    CreatedAt,
}

#[derive(Iden)]
enum BiasDetections {
    Table,
    Id,
    SessionId,
    StepNumber,
    HasBias,
    Severity,
    BiasTypes,
    Suggestions,
    Confidence,
    CreatedAt,
}
