use super::stages::SOCStage;

/// Mermaid flowchart generator
///
/// Generate Mermaid diagram code for SOC processes
pub struct MermaidGenerator {
    /// Whether to include sub-processes
    include_subprocesses: bool,
    /// Whether to include styles
    include_styles: bool,
}

impl MermaidGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self {
            include_subprocesses: false,
            include_styles: true,
        }
    }

    /// Set whether to include sub-processes
    pub fn with_subprocesses(mut self, include: bool) -> Self {
        self.include_subprocesses = include;
        self
    }

    /// Set whether to include styles
    pub fn with_styles(mut self, include: bool) -> Self {
        self.include_styles = include;
        self
    }

    /// Generate complete SOC flowchart
    pub fn generate_flowchart(&self) -> String {
        let mut output = String::new();

        output.push_str("```mermaid\n");
        output.push_str("graph TD\n");

        // Node definitions
        output.push_str("    Start([Task Start])\n");
        output.push_str("    End([Task End])\n");

        // Stage nodes
        for stage in SOCStage::all_stages() {
            let node_id = Self::stage_to_node_id(stage);
            let node_label = stage.name();
            output.push_str(&format!("    {}[{}]\n", node_id, node_label));
        }

        // Connection relationships
        output.push_str("    Start --> Collect\n");
        output.push_str("    Collect --> Analyze\n");
        output.push_str("    Analyze --> Decide\n");
        output.push_str("    Decide --> Execute\n");
        output.push_str("    Execute --> Verify\n");
        output.push_str("    Verify -->|Not Met| Adjust\n");
        output.push_str("    Adjust --> Collect\n");
        output.push_str("    Verify -->|Met| Report\n");
        output.push_str("    Report --> Archive\n");
        output.push_str("    Archive --> End\n");

        // Styles
        if self.include_styles {
            output.push('\n');
            output.push_str("    style Start fill:#e1f5e1\n");
            output.push_str("    style End fill:#ffe1e1\n");
            output.push_str("    style Verify fill:#fff4e1\n");
            output.push_str("    style Adjust fill:#ffe1f4\n");
        }

        output.push_str("```\n");

        output
    }

    /// Generate stage sub-flowchart
    pub fn generate_stage_subprocess(&self, stage: SOCStage) -> String {
        let mut output = String::new();

        output.push_str("```mermaid\n");
        output.push_str("graph TD\n");

        let node_id = Self::stage_to_node_id(stage);
        let activities = stage.key_activities();

        // Generate nodes
        output.push_str(&format!("    Start{}([Start])\n", node_id));
        for (i, activity) in activities.iter().enumerate() {
            output.push_str(&format!("    {}_{}[{}]\n", node_id, i, activity));
        }
        output.push_str(&format!("    End{}([Complete])\n", node_id));

        // Generate connections
        output.push_str(&format!("    Start{} --> {}_0\n", node_id, node_id));
        if let Some(last) = activities.len().checked_sub(1) {
            for i in 0..last {
                output.push_str(&format!(
                    "    {}_{} --> {}_{}\n",
                    node_id,
                    i,
                    node_id,
                    i + 1
                ));
            }
            output.push_str(&format!("    {}_{} --> End{}\n", node_id, last, node_id));
        }

        // Styles
        if self.include_styles {
            output.push('\n');
            output.push_str(&format!("    style Start{} fill:#e1f5e1\n", node_id));
            output.push_str(&format!("    style End{} fill:#e1f5e1\n", node_id));
        }

        output.push_str("```\n");

        output
    }

    /// Generate exception handling flowchart
    pub fn generate_exception_flow(&self) -> String {
        let mut output = String::new();

        output.push_str("```mermaid\n");
        output.push_str("graph TD\n");

        output.push_str("    Start([Anomaly Detected])\n");
        output.push_str("    Classify[Classify Anomaly]\n");
        output.push_str("    Severity{Severity?}\n");
        output.push_str("    HandleLow[Low Priority]\n");
        output.push_str("    HandleMedium[Medium Priority]\n");
        output.push_str("    HandleHigh[High Priority]\n");
        output.push_str("    Escalate[Escalate]\n");
        output.push_str("    Retry{Retry?}\n");
        output.push_str("    ExecuteRetry[Execute Retry]\n");
        output.push_str("    Log[Log]\n");
        output.push_str("    Notify[Notify Stakeholders]\n");
        output.push_str("    End([Exception Handled])\n");

        output.push_str("    Start --> Classify\n");
        output.push_str("    Classify --> Severity\n");
        output.push_str("    Severity -->|Low| HandleLow\n");
        output.push_str("    Severity -->|Medium| HandleMedium\n");
        output.push_str("    Severity -->|High| HandleHigh\n");
        output.push_str("    HandleLow --> Retry\n");
        output.push_str("    HandleMedium --> Retry\n");
        output.push_str("    HandleHigh --> Escalate\n");
        output.push_str("    Escalate --> Notify\n");
        output.push_str("    Retry -->|Yes| ExecuteRetry\n");
        output.push_str("    ExecuteRetry --> Log\n");
        output.push_str("    Retry -->|No| Log\n");
        output.push_str("    Log --> Notify\n");
        output.push_str("    Notify --> End\n");

        if self.include_styles {
            output.push('\n');
            output.push_str("    style Start fill:#ffe1e1\n");
            output.push_str("    style End fill:#e1f5e1\n");
            output.push_str("    style Severity fill:#fff4e1\n");
            output.push_str("    style Retry fill:#fff4e1\n");
            output.push_str("    style Escalate fill:#ffe1f4\n");
        }

        output.push_str("```\n");

        output
    }

    /// Convert stage to node ID
    fn stage_to_node_id(stage: SOCStage) -> &'static str {
        match stage {
            SOCStage::InformationCollection => "Collect",
            SOCStage::ThreatAnalysis => "Analyze",
            SOCStage::DecisionMaking => "Decide",
            SOCStage::OperationExecution => "Execute",
            SOCStage::ResultVerification => "Verify",
            SOCStage::ReportGeneration => "Report",
            SOCStage::KnowledgeArchiving => "Archive",
        }
    }

    /// Generate all sub-flowcharts
    pub fn generate_all_subprocesses(&self) -> String {
        let mut output = String::new();

        for stage in SOCStage::all_stages() {
            output.push_str(&format!("### {} Sub-process\n\n", stage.name()));
            output.push_str(&self.generate_stage_subprocess(stage));
            output.push('\n');
        }

        output
    }
}

impl Default for MermaidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_generate_flowchart() -> Result<()> {
        let generator = MermaidGenerator::new();
        let flowchart = generator.generate_flowchart();

        assert!(flowchart.contains("graph TD"));
        assert!(flowchart.contains("Task Start"));
        assert!(flowchart.contains("Task End"));
        assert!(flowchart.contains("Information Collection"));
        assert!(flowchart.contains("Threat Analysis"));
        Ok(())
    }

    #[test]
    fn test_generate_stage_subprocess() -> Result<()> {
        let generator = MermaidGenerator::new();
        let subprocess = generator.generate_stage_subprocess(SOCStage::InformationCollection);

        assert!(subprocess.contains("graph TD"));
        assert!(subprocess.contains("Receive task input"));
        assert!(subprocess.contains("Query historical records"));
        Ok(())
    }

    #[test]
    fn test_generate_exception_flow() -> Result<()> {
        let generator = MermaidGenerator::new();
        let flow = generator.generate_exception_flow();

        assert!(flow.contains("Anomaly Detected"));
        assert!(flow.contains("Classify Anomaly"));
        assert!(flow.contains("Severity"));
        Ok(())
    }

    #[test]
    fn test_without_styles() -> Result<()> {
        let generator = MermaidGenerator::new().with_styles(false);
        let flowchart = generator.generate_flowchart();

        assert!(!flowchart.contains("style Start"));
        Ok(())
    }

    #[test]
    fn test_generate_all_subprocesses() -> Result<()> {
        let generator = MermaidGenerator::new();
        let all = generator.generate_all_subprocesses();

        assert!(all.contains("Information Collection Sub-process"));
        assert!(all.contains("Threat Analysis Sub-process"));
        assert!(all.contains("Knowledge Archiving Sub-process"));
        Ok(())
    }
}
