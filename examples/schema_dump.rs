/// Schema dump tool — exports JsonSchema from arona types as JSON.
/// Run: cargo run --example schema_dump > arona_schema.json
use schemars::schema_for;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "definitions": {
            "Agent": schema_for!(arona::Agent),
            "AgentBadge": schema_for!(arona::AgentBadge),
            "AgentStatus": schema_for!(arona::AgentStatus),
            "RequestState": schema_for!(arona::RequestState),
            "CompletionOutcome": schema_for!(arona::CompletionOutcome),
            "ModelTier": schema_for!(arona::ModelTier),
            "StreamChunkKind": schema_for!(arona::StreamChunkKind),
            "ContainerStatus": schema_for!(arona::ContainerStatus),
            "TaskStatus": schema_for!(arona::TaskStatus),
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
