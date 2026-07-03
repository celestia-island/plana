/// Schema dump tool — exports JsonSchema from arona types as JSON.
/// Run: cargo run --example schema_dump > arona_schema.json
use schemars::schema_for;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "definitions": {
            "Agent": schema_for!(arona::Agent),
            "AgentBadge": schema_for!(arona::AgentBadge),
            "AgentErrorCode": schema_for!(arona::AgentErrorCode),
            "AgentStatus": schema_for!(arona::AgentStatus),
            "CompletionOutcome": schema_for!(arona::CompletionOutcome),
            "ContainerStatus": schema_for!(arona::ContainerStatus),
            "EmbeddingModel": schema_for!(arona::EmbeddingModel),
            "KnowledgeBaseStatus": schema_for!(arona::KnowledgeBaseStatus),
            "LlmStream": schema_for!(arona::LlmStream),
            "ModelTier": schema_for!(arona::ModelTier),
            "PeriodType": schema_for!(arona::PeriodType),
            "ReportSelection": schema_for!(arona::ReportSelection),
            "ReportType": schema_for!(arona::ReportType),
            "RequestState": schema_for!(arona::RequestState),
            "RetryReason": schema_for!(arona::RetryReason),
            "RouteInfo": schema_for!(arona::RouteInfo),
            "SkillStage": schema_for!(arona::SkillStage),
            "StreamChunkKind": schema_for!(arona::StreamChunkKind),
            "StreamSegment": schema_for!(arona::StreamSegment),
            "StructuredAgentError": schema_for!(arona::StructuredAgentError),
            "TaskStatus": schema_for!(arona::TaskStatus),
            "WorkStatus": schema_for!(arona::WorkStatus),
            "YoloTaskTier": schema_for!(arona::YoloTaskTier),
            "ModelCategory": schema_for!(arona::model::ModelCategory),
            "ModelBackend": schema_for!(arona::model::ModelBackend),
            "ModelDescriptor": schema_for!(arona::model::ModelDescriptor),
            "ModelServerStatus": schema_for!(arona::model::ModelServerStatus),
            "ModelServerInfo": schema_for!(arona::model::ModelServerInfo),
            "ModelServerKind": schema_for!(arona::model::ModelServerKind),
            "ModelServerAction": schema_for!(arona::model::ModelServerAction),
            "ModelInferenceRequest": schema_for!(arona::model::ModelInferenceRequest),
            "ModelInferenceResult": schema_for!(arona::model::ModelInferenceResult),
            "AgentStreamingChunkParams": schema_for!(arona::AgentStreamingChunkParams),
            "McpToolResultParams": schema_for!(arona::McpToolResultParams),
            "TaskCreatedParams": schema_for!(arona::TaskCreatedParams),
            "TaskStatusUpdateParams": schema_for!(arona::TaskStatusUpdateParams),
            "TuiAgentInfo": schema_for!(arona::TuiAgentInfo),
        },
    });
    let json = serde_json::to_string_pretty(&schemas)?;
    println!("{json}");
    Ok(())
}
