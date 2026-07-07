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
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct ReportHumanParams {
    pub summary: String,
    pub body: Option<String>,
    pub text: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct StandardCheckParams {
    pub standard_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_data: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct ComplianceReportParams {
    pub standard_id: Uuid,
    pub check_results: Vec<CheckResultItem>,
    pub device_id: Option<String>,
    pub summary: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct AuditAlignmentParams {
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_data: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct AuditLegalityParams {
    pub target: String,
    pub jurisdiction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_data: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct AgentIntegrityParams {
    pub verbose: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct SecurityAuditParams {
    pub deep: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct BlockToolParams {
    pub agent: String,
    pub tool: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct UnblockToolParams {
    pub agent: String,
    pub tool: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct SetSecurityPolicyParams {
    pub emergency_lockdown: Option<bool>,
    pub audit_only: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct SetRiskThresholdParams {
    pub level: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct InspectToolCallParams {
    pub agent: String,
    pub tool: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct SecurityStatusParams {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct SetNetworkPolicyParams {
    pub allow_hosts: Option<Vec<String>>,
    pub allow_cidrs: Option<Vec<String>>,
    pub block_private: Option<bool>,
    pub block_metadata: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
pub struct SecuritySuggestionsParams {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "mcp/orexis.ts")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ConsultationStatus;
    use serde_json::json;

    #[test]
    fn check_result_item_round_trip() {
        let item = CheckResultItem {
            standard: "ISO-27001".into(),
            status: "pass".into(),
            message: "encryption enabled".into(),
            details: None,
        };
        let v = serde_json::to_value(&item).unwrap();
        assert_eq!(v["status"], "pass");
        assert!(v.get("details").is_none(), "details skipped when None");
        let back: CheckResultItem = serde_json::from_value(v).unwrap();
        assert_eq!(back.standard, "ISO-27001");
    }

    #[test]
    fn check_result_item_with_details() {
        let item = CheckResultItem {
            standard: "ISO-27001".into(),
            status: "fail".into(),
            message: "weak cipher".into(),
            details: Some("TLS_RSA_WITH_3DES_EDE_CBC_SHA detected".into()),
        };
        let v = serde_json::to_value(&item).unwrap();
        assert_eq!(v["details"], "TLS_RSA_WITH_3DES_EDE_CBC_SHA detected");
    }

    #[test]
    fn sensitivity_rule_round_trip() {
        let r = SensitivityRule {
            tool: "neikos.container.exec".into(),
            agent: Some("skeMma".into()),
            sensitivity: "high".into(),
            reason: None,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["sensitivity"], "high");
        assert_eq!(v["agent"], "skeMma");
        assert!(v.get("reason").is_none());
    }

    #[test]
    fn ask_result_round_trip() {
        let id = Uuid::new_v4();
        let r = AskResult {
            consultation_id: id,
            question: "Proceed with deletion?".into(),
            context: "3 files selected".into(),
            options: vec!["yes".into(), "no".into()],
            recommended: "no".into(),
            status: ConsultationStatus::WaitingHuman,
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["consultation_id"], id.to_string());
        // ConsultationStatus serializes as PascalCase variant name.
        assert_eq!(v["status"], "WaitingHuman");
        assert_eq!(v["options"].as_array().unwrap().len(), 2);
        let back: AskResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.status, ConsultationStatus::WaitingHuman);
    }

    #[test]
    fn audit_alignment_result_round_trip() {
        let id = Uuid::new_v4();
        let r = AuditAlignmentResult {
            audit_id: id,
            target: "container-runtime".into(),
            total_rules: 10,
            passed: 8,
            failed: 2,
            findings: vec![AuditFinding {
                rule_id: "R-001".into(),
                severity: "high".into(),
                description: "Missing seccomp profile".into(),
                evidence: "no seccomp annotation".into(),
                recommendation: "Apply default seccomp profile".into(),
            }],
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["passed"], 8);
        assert_eq!(v["failed"], 2);
        assert_eq!(v["findings"][0]["severity"], "high");
        let back: AuditAlignmentResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.findings.len(), 1);
    }

    #[test]
    fn compliance_summary_round_trip() {
        let s = ComplianceSummary {
            total_rules: 50,
            passed: 45,
            failed: 5,
            critical_failures: 1,
            high_failures: 2,
            medium_failures: 2,
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["critical_failures"], 1);
        let back: ComplianceSummary = serde_json::from_value(v).unwrap();
        assert_eq!(back.total_rules, 50);
    }

    #[test]
    fn compliance_report_tool_result_round_trip() {
        let report_id = Uuid::new_v4();
        let standard_id = Uuid::new_v4();
        let r = ComplianceReportToolResult {
            report_id,
            standard_id,
            device_id: Some("dev-001".into()),
            overall_status: "non_compliant".into(),
            summary: ComplianceSummary {
                total_rules: 10,
                passed: 7,
                failed: 3,
                critical_failures: 1,
                high_failures: 1,
                medium_failures: 1,
            },
            details: vec![ReportDetail {
                rule_id: "R-005".into(),
                clause: "A.10".into(),
                description: "Access control".into(),
                status: "fail".into(),
                severity: "critical".into(),
                deviation: Some(0.85),
                recommendation: "Enable MFA".into(),
            }],
            generated_at: "2026-07-07T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["overall_status"], "non_compliant");
        assert_eq!(v["device_id"], "dev-001");
        assert_eq!(v["details"][0]["deviation"], 0.85);
        let back: ComplianceReportToolResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.details.len(), 1);
    }

    #[test]
    fn compliance_report_tool_result_no_device() {
        let r = ComplianceReportToolResult {
            report_id: Uuid::new_v4(),
            standard_id: Uuid::new_v4(),
            device_id: None,
            overall_status: "compliant".into(),
            summary: ComplianceSummary {
                total_rules: 1,
                passed: 1,
                failed: 0,
                critical_failures: 0,
                high_failures: 0,
                medium_failures: 0,
            },
            details: vec![],
            generated_at: "2026-01-01T00:00:00Z".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert!(
            v.get("device_id").is_some(),
            "Option without skip → null on wire"
        );
        assert_eq!(v["device_id"], serde_json::Value::Null);
    }

    #[test]
    fn rule_check_result_round_trip() {
        let r = RuleCheckResult {
            rule_id: "R-001".into(),
            clause: "§5.2".into(),
            description: "Check X".into(),
            status: "fail".into(),
            actual_value: Some(json!({"value": 80})),
            expected: Some(json!({"min": 100})),
            deviation: Some(0.2),
            severity: "medium".into(),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["actual_value"]["value"], 80);
        assert_eq!(v["deviation"], 0.2);
        let back: RuleCheckResult = serde_json::from_value(v).unwrap();
        assert_eq!(back.deviation, Some(0.2));
    }

    #[test]
    fn block_tool_params_round_trip() {
        let p = BlockToolParams {
            agent: "neikos".into(),
            tool: "container.remove".into(),
            reason: Some("destructive in production".into()),
        };
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["tool"], "container.remove");
    }
}
