use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::stages::{SOCProcessState, SOCStage, StageResult};

/// SOC process manager
///
/// Manages execution and state tracking for standard SOC processes
pub struct SOCProcessManager {
    /// Process status
    state: SOCProcessState,
    /// Process context data
    context: HashMap<String, serde_json::Value>,
}

impl SOCProcessManager {
    /// Create new SOC process manager
    pub fn new() -> Self {
        Self {
            state: SOCProcessState::default(),
            context: HashMap::new(),
        }
    }

    /// Get current stage
    pub fn current_stage(&self) -> SOCStage {
        self.state.current_stage
    }

    /// Get process status
    pub fn state(&self) -> &SOCProcessState {
        &self.state
    }

    /// Set context data
    pub fn set_context(&mut self, key: &str, value: serde_json::Value) {
        self.context.insert(key.to_string(), value);
    }

    /// Get context data
    pub fn get_context(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get(key)
    }

    /// Advance to next stage
    ///
    /// # Returns
    /// Returns true if successfully advanced; returns false if already at the last stage
    pub fn advance_stage(&mut self) -> bool {
        let stages = SOCStage::all_stages();
        let current_index = stages.iter().position(|s| *s == self.state.current_stage);

        if let Some(index) = current_index {
            if index < stages.len() - 1 {
                // Mark current stage as complete
                self.state.completed_stages.push(self.state.current_stage);

                // Advance to next stage
                self.state.current_stage = stages[index + 1];
                return true;
            } else {
                // Already at the last stage
                self.state.is_completed = true;
                return false;
            }
        }

        false
    }

    /// Record stage execution result
    pub fn record_stage_result(
        &mut self,
        stage: SOCStage,
        data: serde_json::Value,
        success: bool,
        error: Option<String>,
    ) {
        let result = StageResult {
            stage,
            timestamp: Utc::now(),
            data,
            success,
            error,
        };
        self.state.stage_results.push(result);
    }

    /// Check if process is complete
    pub fn is_completed(&self) -> bool {
        self.state.is_completed
    }

    /// Reset process
    pub fn reset(&mut self) {
        self.state = SOCProcessState::default();
        self.context.clear();
    }

    /// Generate process execution report
    pub fn generate_report(&self) -> ProcessReport {
        ProcessReport {
            total_stages: SOCStage::all_stages().len(),
            completed_stages: self.state.completed_stages.len(),
            current_stage: self.state.current_stage,
            is_completed: self.state.is_completed,
            stage_results: self.state.stage_results.clone(),
        }
    }
}

impl Default for SOCProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Process execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessReport {
    /// Total stage count
    pub total_stages: usize,
    /// Completed stage count
    pub completed_stages: usize,
    /// Current stage
    pub current_stage: SOCStage,
    /// Whether complete
    pub is_completed: bool,
    /// Stage execution result
    pub stage_results: Vec<StageResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use serde_json::json;

    #[test]
    fn test_new_manager() -> Result<()> {
        let manager = SOCProcessManager::new();
        assert_eq!(manager.current_stage(), SOCStage::InformationCollection);
        assert!(!manager.is_completed());
        Ok(())
    }

    #[test]
    fn test_advance_stage() -> Result<()> {
        let mut manager = SOCProcessManager::new();

        assert!(manager.advance_stage());
        assert_eq!(manager.current_stage(), SOCStage::ThreatAnalysis);

        assert!(manager.advance_stage());
        assert_eq!(manager.current_stage(), SOCStage::DecisionMaking);
        Ok(())
    }

    #[test]
    fn test_advance_to_completion() -> Result<()> {
        let mut manager = SOCProcessManager::new();

        // Advance 6 times (from stage 1 to stage 7)
        for _ in 0..6 {
            assert!(manager.advance_stage());
        }

        // 7th time should return false (already at last stage)
        assert!(!manager.advance_stage());
        assert!(manager.is_completed());
        Ok(())
    }

    #[test]
    fn test_context_data() -> Result<()> {
        let mut manager = SOCProcessManager::new();

        manager.set_context("task", json!("test task"));
        assert_eq!(manager.get_context("task"), Some(&json!("test task")));
        assert_eq!(manager.get_context("nonexistent"), None);
        Ok(())
    }

    #[test]
    fn test_record_stage_result() -> Result<()> {
        let mut manager = SOCProcessManager::new();

        manager.record_stage_result(
            SOCStage::InformationCollection,
            json!({"data": "test"}),
            true,
            None,
        );

        let report = manager.generate_report();
        assert_eq!(report.stage_results.len(), 1);
        assert!(report.stage_results[0].success);
        Ok(())
    }

    #[test]
    fn test_reset() -> Result<()> {
        let mut manager = SOCProcessManager::new();

        manager.advance_stage();
        manager.set_context("test", json!("value"));

        manager.reset();

        assert_eq!(manager.current_stage(), SOCStage::InformationCollection);
        assert_eq!(manager.get_context("test"), None);
        Ok(())
    }

    #[test]
    fn test_generate_report() -> Result<()> {
        let mut manager = SOCProcessManager::new();

        manager.record_stage_result(
            SOCStage::InformationCollection,
            json!({"collected": "data"}),
            true,
            None,
        );
        manager.advance_stage();

        let report = manager.generate_report();
        assert_eq!(report.total_stages, 7);
        assert_eq!(report.completed_stages, 1);
        assert_eq!(report.current_stage, SOCStage::ThreatAnalysis);
        assert!(!report.is_completed);
        Ok(())
    }
}
