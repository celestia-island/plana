/// Schema dump tool — exports JsonSchema from plana types as JSON.
/// Run: cargo run --example schema_dump > plana_schema.json
use schemars::schema_for;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "definitions": {
            "Agent": schema_for!(plana_types::Agent),
            "AgentBadge": schema_for!(plana_types::AgentBadge),
            "AgentErrorCode": schema_for!(plana_types::AgentErrorCode),
            "AgentStatus": schema_for!(plana_types::AgentStatus),
            "CompletionOutcome": schema_for!(plana_types::CompletionOutcome),
            "ContainerStatus": schema_for!(plana_types::ContainerStatus),
            "EmbeddingModel": schema_for!(plana_types::EmbeddingModel),
            "KnowledgeBaseStatus": schema_for!(plana_types::KnowledgeBaseStatus),
            "LlmStream": schema_for!(plana_types::LlmStream),
            "ModelTier": schema_for!(plana_types::ModelTier),
            "PeriodType": schema_for!(plana_types::PeriodType),
            "ReportSelection": schema_for!(plana_types::ReportSelection),
            "ReportType": schema_for!(plana_types::ReportType),
            "RequestState": schema_for!(plana_types::RequestState),
            "RetryReason": schema_for!(plana_types::RetryReason),
            "RouteInfo": schema_for!(plana_types::RouteInfo),
            "SkillStage": schema_for!(plana_types::SkillStage),
            "StreamChunkKind": schema_for!(plana_types::StreamChunkKind),
            "StreamSegment": schema_for!(plana_types::StreamSegment),
            "StructuredAgentError": schema_for!(plana_types::StructuredAgentError),
            "TaskStatus": schema_for!(plana_types::TaskStatus),
            "WorkStatus": schema_for!(plana_types::WorkStatus),
            "YoloTaskTier": schema_for!(plana_types::YoloTaskTier),
            "ModelCategory": schema_for!(plana_types::model::ModelCategory),
            "ModelBackend": schema_for!(plana_types::model::ModelBackend),
            "ModelDescriptor": schema_for!(plana_types::model::ModelDescriptor),
            "ModelServerStatus": schema_for!(plana_types::model::ModelServerStatus),
            "ModelServerInfo": schema_for!(plana_types::model::ModelServerInfo),
            "ModelServerKind": schema_for!(plana_types::model::ModelServerKind),
            "ModelServerAction": schema_for!(plana_types::model::ModelServerAction),
            "ModelInferenceRequest": schema_for!(plana_types::model::ModelInferenceRequest),
            "ModelInferenceResult": schema_for!(plana_types::model::ModelInferenceResult),
            "AgentStreamingChunkParams": schema_for!(plana_types::AgentStreamingChunkParams),
            "McpToolResultParams": schema_for!(plana_types::McpToolResultParams),
            "TaskCreatedParams": schema_for!(plana_types::TaskCreatedParams),
            "TaskStatusUpdateParams": schema_for!(plana_types::TaskStatusUpdateParams),
            "TuiAgentInfo": schema_for!(plana_types::TuiAgentInfo),
        },
    });
    let json = serde_json::to_string_pretty(&schemas)?;
    println!("{json}");
    Ok(())
}
