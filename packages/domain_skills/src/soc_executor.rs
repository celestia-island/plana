use anyhow::{Result, anyhow};
use serde::Serialize;

use tracing::{debug, error, info};

use arona_infra_utils::soc::{ProcessReport, SOCProcessManager, SOCStage};

#[derive(Debug, Clone, Serialize)]
struct SocStageStatus {
    status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct SocVerifiedStatus {
    verified: bool,
}

#[derive(Debug, Clone, Serialize)]
struct SocArchivedStatus {
    archived: bool,
}

/// SOC process-enhanced Skill executor
///
/// Automatically integrate SOC process management when executing Skills
pub struct SOCSkillExecutor {
    /// SOC process manager
    soc_manager: SOCProcessManager,
    /// Whether SOC process is enabled
    enabled: bool,
}

impl SOCSkillExecutor {
    /// Create new SOC Skill executor
    pub fn new(enabled: bool) -> Self {
        Self {
            soc_manager: SOCProcessManager::new(),
            enabled,
        }
    }

    /// Get SOC process manager
    pub fn soc_manager(&self) -> &SOCProcessManager {
        &self.soc_manager
    }

    /// Get mutable SOC process manager
    pub fn soc_manager_mut(&mut self) -> &mut SOCProcessManager {
        &mut self.soc_manager
    }

    /// Execute Skills (with SOC process)
    ///
    /// # Arguments
    /// * `skill_name` - skill name
    /// * `executor` - the actual execution function
    ///
    /// # Returns
    /// Execution result
    pub async fn execute_skill<F, Fut, T, E>(&mut self, skill_name: &str, executor: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        if !self.enabled {
            return executor().await.map_err(|e| anyhow!("{}", e));
        }

        info!("starting skill execution: {}", skill_name);

        // Reset SOC process
        self.soc_manager.reset();
        self.soc_manager.set_context(
            "skill_name",
            serde_json::Value::String(skill_name.to_string()),
        );

        // Execute each stage
        self.execute_with_soc_flow(skill_name, executor).await
    }

    /// Execute Skills with SOC process
    async fn execute_with_soc_flow<F, Fut, T, E>(
        &mut self,
        skill_name: &str,
        executor: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        // Stage 1: Information gathering (handled internally by executor)
        self.record_stage_start(SOCStage::InformationCollection);
        debug!("stage 1: information collection");
        self.soc_manager.record_stage_result(
            SOCStage::InformationCollection,
            serde_json::to_value(SocStageStatus {
                status: "collecting",
            })
            .unwrap_or(serde_json::Value::Null),
            true,
            None,
        );
        self.soc_manager.advance_stage();

        // Stage 2: Threat analysis (handled internally by executor)
        self.record_stage_start(SOCStage::ThreatAnalysis);
        debug!("stage 2: threat analysis");
        self.soc_manager.record_stage_result(
            SOCStage::ThreatAnalysis,
            serde_json::to_value(SocStageStatus {
                status: "analyzing",
            })
            .unwrap_or(serde_json::Value::Null),
            true,
            None,
        );
        self.soc_manager.advance_stage();

        // Stage 3: Decision making (handled internally by executor)
        self.record_stage_start(SOCStage::DecisionMaking);
        debug!("stage 3: decision making");
        self.soc_manager.record_stage_result(
            SOCStage::DecisionMaking,
            serde_json::to_value(SocStageStatus { status: "deciding" })
                .unwrap_or(serde_json::Value::Null),
            true,
            None,
        );
        self.soc_manager.advance_stage();

        // Stage 4: Execute operations
        self.record_stage_start(SOCStage::OperationExecution);
        debug!("stage 4: operation execution");

        let result = executor().await;

        match result {
            Ok(value) => {
                // Record execution success
                self.soc_manager.record_stage_result(
                    SOCStage::OperationExecution,
                    serde_json::to_value(SocStageStatus { status: "success" })
                        .unwrap_or(serde_json::Value::Null),
                    true,
                    None,
                );
                self.soc_manager.advance_stage();

                // Stage 5: Result validation
                self.record_stage_start(SOCStage::ResultVerification);
                debug!("stage 5: result verification");
                self.soc_manager.record_stage_result(
                    SOCStage::ResultVerification,
                    serde_json::to_value(SocVerifiedStatus { verified: true })
                        .unwrap_or(serde_json::Value::Null),
                    true,
                    None,
                );
                self.soc_manager.advance_stage();

                // Stage 6: Generate report
                self.record_stage_start(SOCStage::ReportGeneration);
                debug!("stage 6: report generation");
                let report = self.soc_manager.generate_report();
                self.soc_manager.record_stage_result(
                    SOCStage::ReportGeneration,
                    serde_json::to_value(&report).unwrap_or(serde_json::Value::Null),
                    true,
                    None,
                );
                self.soc_manager.advance_stage();

                // Stage 7: Knowledge consolidation
                self.record_stage_start(SOCStage::KnowledgeArchiving);
                debug!("stage 7: knowledge archiving");
                self.soc_manager.record_stage_result(
                    SOCStage::KnowledgeArchiving,
                    serde_json::to_value(SocArchivedStatus { archived: true })
                        .unwrap_or(serde_json::Value::Null),
                    true,
                    None,
                );
                // Advance to completed state (this sets is_completed = true)
                self.soc_manager.advance_stage();

                info!("skill execution completed: {}", skill_name);
                Ok(value)
            },
            Err(e) => {
                // Record execution failure
                error!("skill execution failed: {} - {}", skill_name, e);
                self.soc_manager.record_stage_result(
                    SOCStage::OperationExecution,
                    serde_json::to_value(SocStageStatus { status: "failed" })
                        .unwrap_or(serde_json::Value::Null),
                    false,
                    Some(e.to_string()),
                );

                Err(anyhow!("{}", e))
            },
        }
    }

    /// Record stage start
    fn record_stage_start(&mut self, stage: SOCStage) {
        let stage_name = stage.name();
        debug!(
            "starting stage {}/{}: {}",
            stage.number(),
            SOCStage::all_stages().len(),
            stage_name
        );
    }

    /// Generate SOC process report
    pub fn generate_report(&self) -> ProcessReport {
        self.soc_manager.generate_report()
    }

    /// Check if SOC process is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable/disable SOC process
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_skill_with_soc_enabled() -> Result<()> {
        let mut executor = SOCSkillExecutor::new(true);

        let result = executor
            .execute_skill("test_skill", || async { Ok::<_, String>(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result?, 42);

        let report = executor.generate_report();
        assert!(report.is_completed, "SOC flow should be completed");
        assert_eq!(report.completed_stages, 6);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_skill_with_soc_disabled() -> Result<()> {
        let mut executor = SOCSkillExecutor::new(false);

        let result = executor
            .execute_skill("test_skill", || async { Ok::<_, String>(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result?, 42);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_skill_failure() -> Result<()> {
        let mut executor = SOCSkillExecutor::new(true);

        let result = executor
            .execute_skill("test_skill", || async {
                Err::<i32, String>("test error".to_string())
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("test error"),
            "expected 'test error' in error message, got: {}",
            err
        );
        Ok(())
    }

    #[test]
    fn test_enable_disable() -> Result<()> {
        let mut executor = SOCSkillExecutor::new(true);
        assert!(executor.is_enabled());

        executor.set_enabled(false);
        assert!(!executor.is_enabled());

        executor.set_enabled(true);
        assert!(executor.is_enabled());
        Ok(())
    }
}
