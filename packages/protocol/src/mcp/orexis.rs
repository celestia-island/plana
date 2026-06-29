use serde_json::Value;
use uuid::Uuid;

use crate::enums::ConsultationStatus;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct CheckResultItem {
    pub standard: String,
    pub status: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct SensitivityRule {
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub sensitivity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct AskResult {
    pub consultation_id: Uuid,
    pub question: String,
    pub context: String,
    pub options: Vec<String>,
    pub recommended: String,
    pub status: ConsultationStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct ReplyResult {
    pub consultation_id: Uuid,
    pub answer: String,
    pub selected_options: Vec<String>,
    pub status: ConsultationStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct ReportHumanResult {
    pub report_id: Uuid,
    pub report_type: String,
    #[ts(type = "Record<string, unknown>")]
    pub content: Value,
    pub consultation_id: Option<Uuid>,
    pub status: ConsultationStatus,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct AuditAlignmentResult {
    pub audit_id: Uuid,
    pub target: String,
    pub total_rules: usize,
    pub passed: usize,
    pub failed: usize,
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct AuditLegalityResult {
    pub audit_id: Uuid,
    pub target: String,
    pub jurisdiction: String,
    pub total_requirements: usize,
    pub compliant: usize,
    pub non_compliant: usize,
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct AuditFinding {
    pub rule_id: String,
    pub severity: String,
    pub description: String,
    pub evidence: String,
    pub recommendation: String,
}

// ── Tool parameter structs (for .d.ts API signature generation) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ReportHumanParams {
    pub summary: String,
    pub body: Option<String>,
    pub text: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct StandardCheckParams {
    pub standard_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_data: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ComplianceReportParams {
    pub standard_id: Uuid,
    pub check_results: Vec<CheckResultItem>,
    pub device_id: Option<String>,
    pub summary: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct AuditAlignmentParams {
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_data: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct AuditLegalityParams {
    pub target: String,
    pub jurisdiction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_data: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct AgentIntegrityParams {
    pub verbose: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SecurityAuditParams {
    pub deep: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct BlockToolParams {
    pub agent: String,
    pub tool: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct UnblockToolParams {
    pub agent: String,
    pub tool: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SetSecurityPolicyParams {
    pub emergency_lockdown: Option<bool>,
    pub audit_only: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SetRiskThresholdParams {
    pub level: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct InspectToolCallParams {
    pub agent: String,
    pub tool: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SecurityStatusParams {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SetNetworkPolicyParams {
    pub allow_hosts: Option<Vec<String>>,
    pub allow_cidrs: Option<Vec<String>>,
    pub block_private: Option<bool>,
    pub block_metadata: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct SecuritySuggestionsParams {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct ManageSensitivityRulesParams {
    pub action: String,
    pub rules: Option<Vec<SensitivityRule>>,
}

// ── Tool result structs (compliance) ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct ComplianceRule {
    pub id: String,
    pub standard: String,
    pub clause: String,
    pub description: String,
    pub check_type: String,
    #[ts(type = "Record<string, unknown>")]
    pub parameters: Value,
    pub severity: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct StandardRegisterResult {
    pub standard_id: Uuid,
    pub rules_registered: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct RuleCheckResult {
    pub rule_id: String,
    pub clause: String,
    pub description: String,
    pub status: String,
    #[ts(type = "Record<string, unknown> | null")]
    pub actual_value: Option<Value>,
    #[ts(type = "Record<string, unknown> | null")]
    pub expected: Option<Value>,
    pub deviation: Option<f64>,
    pub severity: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct StandardCheckResult {
    pub standard_id: Uuid,
    pub total_rules: usize,
    pub passed: usize,
    pub failed: usize,
    pub not_applicable: usize,
    pub results: Vec<RuleCheckResult>,
    pub overall_status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct ComplianceSummary {
    pub total_rules: usize,
    pub passed: usize,
    pub failed: usize,
    pub critical_failures: usize,
    pub high_failures: usize,
    pub medium_failures: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct ReportDetail {
    pub rule_id: String,
    pub clause: String,
    pub description: String,
    pub status: String,
    pub severity: String,
    pub deviation: Option<f64>,
    pub recommendation: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct ComplianceReportToolResult {
    pub report_id: Uuid,
    pub standard_id: Uuid,
    pub device_id: Option<String>,
    pub overall_status: String,
    pub summary: ComplianceSummary,
    pub details: Vec<ReportDetail>,
    pub generated_at: String,
}
