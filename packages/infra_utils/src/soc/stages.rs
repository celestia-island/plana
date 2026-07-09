use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// SOC process stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SOCStage {
    /// Stage 1: Information gathering
    InformationCollection,
    /// Stage 2: Threat analysis
    ThreatAnalysis,
    /// Stage 3: Decision making
    DecisionMaking,
    /// Stage 4: Execution
    OperationExecution,
    /// Stage 5: Result validation
    ResultVerification,
    /// Stage 6: Report generation
    ReportGeneration,
    /// Stage 7: Knowledge consolidation
    KnowledgeArchiving,
}

impl SOCStage {
    /// Get stage name
    pub fn name(&self) -> &'static str {
        match self {
            SOCStage::InformationCollection => "Information Collection",
            SOCStage::ThreatAnalysis => "Threat Analysis",
            SOCStage::DecisionMaking => "Decision Making",
            SOCStage::OperationExecution => "Operation Execution",
            SOCStage::ResultVerification => "Result Verification",
            SOCStage::ReportGeneration => "Report Generation",
            SOCStage::KnowledgeArchiving => "Knowledge Archiving",
        }
    }

    /// Get stage number (1-7)
    pub fn number(&self) -> u8 {
        match self {
            SOCStage::InformationCollection => 1,
            SOCStage::ThreatAnalysis => 2,
            SOCStage::DecisionMaking => 3,
            SOCStage::OperationExecution => 4,
            SOCStage::ResultVerification => 5,
            SOCStage::ReportGeneration => 6,
            SOCStage::KnowledgeArchiving => 7,
        }
    }

    /// Get all stages (in order)
    pub fn all_stages() -> Vec<SOCStage> {
        vec![
            SOCStage::InformationCollection,
            SOCStage::ThreatAnalysis,
            SOCStage::DecisionMaking,
            SOCStage::OperationExecution,
            SOCStage::ResultVerification,
            SOCStage::ReportGeneration,
            SOCStage::KnowledgeArchiving,
        ]
    }

    /// Get stage description
    pub fn description(&self) -> &'static str {
        match self {
            SOCStage::InformationCollection => {
                "Collect all task-related information, analyze the current context, assess system state and resources, and consolidate into a complete information view"
            }
            SOCStage::ThreatAnalysis => {
                "Identify potential threat patterns, evaluate risk levels and impact, prioritize threats, and generate a complete threat profile"
            }
            SOCStage::DecisionMaking => {
                "Select appropriate response strategies, design detailed execution plans, allocate necessary resources, and formulate an execution plan"
            }
            SOCStage::OperationExecution => {
                "Invoke appropriate tools, execute specific operational tasks, monitor execution status in real-time, and handle potential anomalies"
            }
            SOCStage::ResultVerification => {
                "Check operation results, evaluate execution effectiveness, determine if objectives are met, and adjust strategy and re-execute if necessary"
            }
            SOCStage::ReportGeneration => {
                "Organize all relevant data, perform analysis and summarization, generate structured reports, and deliver reports to stakeholders"
            }
            SOCStage::KnowledgeArchiving => {
                "Extract valuable lessons learned, encode experience as reusable knowledge, store in the knowledge base, and share with other Agents"
            }
        }
    }

    /// Get key activity list for the stage
    pub fn key_activities(&self) -> Vec<&'static str> {
        match self {
            SOCStage::InformationCollection => vec![
                "Receive task input",
                "Query historical records",
                "Collect environmental information",
                "Get system status",
                "Summarize information",
                "Verify information completeness",
            ],
            SOCStage::ThreatAnalysis => vec![
                "Pattern recognition",
                "Risk assessment",
                "Priority ranking",
                "Threat profiling",
                "Anomaly identification",
            ],
            SOCStage::DecisionMaking => vec![
                "Strategy selection",
                "Solution design",
                "Resource allocation",
                "Execution plan formulation",
                "Risk assessment",
            ],
            SOCStage::OperationExecution => {
                vec![
                    "Tool invocation",
                    "Operation execution",
                    "Status monitoring",
                    "Exception handling",
                    "Progress tracking",
                ]
            }
            SOCStage::ResultVerification => vec![
                "Result check",
                "Effectiveness evaluation",
                "Compliance judgment",
                "Strategy adjustment",
                "Re-execute (if needed)",
            ],
            SOCStage::ReportGeneration => {
                vec![
                    "Data organization",
                    "Analysis summary",
                    "Report generation",
                    "Report output",
                    "Report verification",
                ]
            }
            SOCStage::KnowledgeArchiving => {
                vec![
                    "Experience extraction",
                    "Knowledge encoding",
                    "Knowledge storage",
                    "Knowledge sharing",
                    "Index update",
                ]
            }
        }
    }
}

/// SOC process state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SOCProcessState {
    /// Current stage
    pub current_stage: SOCStage,
    /// Completed stages
    pub completed_stages: Vec<SOCStage>,
    /// Stage execution result
    pub stage_results: Vec<StageResult>,
    /// Whether complete
    pub is_completed: bool,
}

impl Default for SOCProcessState {
    fn default() -> Self {
        Self {
            current_stage: SOCStage::InformationCollection,
            completed_stages: Vec::new(),
            stage_results: Vec::new(),
            is_completed: false,
        }
    }
}

/// Stage execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    /// Stage
    pub stage: SOCStage,
    /// Execution time
    pub timestamp: DateTime<Utc>,
    /// Result data
    pub data: serde_json::Value,
    /// Whether successful
    pub success: bool,
    /// Error info (if any)
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_stage_number() -> Result<()> {
        assert_eq!(SOCStage::InformationCollection.number(), 1);
        assert_eq!(SOCStage::ThreatAnalysis.number(), 2);
        assert_eq!(SOCStage::DecisionMaking.number(), 3);
        assert_eq!(SOCStage::OperationExecution.number(), 4);
        assert_eq!(SOCStage::ResultVerification.number(), 5);
        assert_eq!(SOCStage::ReportGeneration.number(), 6);
        assert_eq!(SOCStage::KnowledgeArchiving.number(), 7);
        Ok(())
    }

    #[test]
    fn test_all_stages_count() -> Result<()> {
        let stages = SOCStage::all_stages();
        assert_eq!(stages.len(), 7);
        Ok(())
    }

    #[test]
    fn test_stage_name() -> Result<()> {
        assert_eq!(
            SOCStage::InformationCollection.name(),
            "Information Collection"
        );
        assert_eq!(SOCStage::ThreatAnalysis.name(), "Threat Analysis");
        Ok(())
    }

    #[test]
    fn test_key_activities() -> Result<()> {
        let activities = SOCStage::InformationCollection.key_activities();
        assert!(!activities.is_empty());
        assert!(activities.contains(&"Receive task input"));
        Ok(())
    }

    #[test]
    fn test_default_process_state() -> Result<()> {
        let state = SOCProcessState::default();
        assert_eq!(state.current_stage, SOCStage::InformationCollection);
        assert!(state.completed_stages.is_empty());
        assert!(!state.is_completed);
        Ok(())
    }
}
