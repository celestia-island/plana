use super::{mermaid_generator::MermaidGenerator, stages::SOCStage};

/// Skills prompt template generator
///
/// Generate Skills prompt template with SOC processes
pub struct PromptTemplateGenerator {
    /// Mermaid generator
    mermaid_generator: MermaidGenerator,
}

impl PromptTemplateGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self {
            mermaid_generator: MermaidGenerator::new(),
        }
    }

    /// Generate complete Skills prompt template
    pub fn generate_skills_prompt(&self, skills_name: &str, skills_description: &str) -> String {
        let mut output = String::new();

        // Title
        output.push_str(&format!("# {} Skills Code of Conduct\n\n", skills_name));
        output.push_str(&format!("> {}\n\n", skills_description));

        // Core principles
        output.push_str("## Core Principles\n\n");
        output.push_str("You are an Agent that follows the standard SOC (Security Operations Center) process.\n\n");
        output.push_str("**Important Notes**:\n");
        output.push_str(
            "1. **SOC Process-Oriented**: All operations must follow the standard SOC process\n",
        );
        output
            .push_str("2. **Standardized Execution**: Strictly follow the defined process steps\n");
        output.push_str("3. **Documentation-Driven**: Each step must have clear documentation\n");
        output.push_str(
            "4. **Knowledge Archival**: Knowledge must be archived after task completion\n\n",
        );

        // SOC Process
        output.push_str("## SOC Process\n\n");
        output.push_str("Please strictly follow the SOC process below to execute tasks:\n\n");

        for stage in SOCStage::all_stages() {
            output.push_str(&self.generate_stage_section(stage));
            output.push('\n');
        }

        // Flow chart
        output.push_str("## Flow Chart\n\n");
        output.push_str("### Complete SOC Process\n\n");
        output.push_str(&self.mermaid_generator.generate_flowchart());
        output.push('\n');

        // Tool usage
        output.push_str("## Tool Usage\n\n");
        output.push_str("You can complete tasks by executing JavaScript scripts through the unique `exec` tool.\n");
        output
            .push_str("Use ES module imports in scripts: `import {{ tool }} from 'agent'; tool({{ params }})`.\n");
        output.push_str("- [Specific available APIs will be dynamically generated based on the current Skill's related_tools]\n\n");

        // Notes
        output.push_str("## Notes\n\n");
        output.push_str("- Always follow the SOC process, do not skip any steps\n");
        output.push_str("- Each step must have clear output and records\n");
        output.push_str("- When encountering anomalies, handle them according to the process\n");
        output.push_str("- After task completion, ensure knowledge archiving is performed\n");
        output.push_str(
            "- If assistance is needed, clearly specify which other Skills or tools are required\n",
        );

        output
    }

    /// Generate stage sections
    fn generate_stage_section(&self, stage: SOCStage) -> String {
        let mut output = String::new();

        output.push_str(&format!("### {}. {}\n\n", stage.number(), stage.name()));
        output.push_str(&format!("**Description**: {}\n\n", stage.description()));

        output.push_str("**Key Activities**:\n");
        for activity in stage.key_activities() {
            output.push_str(&format!("- {}\n", activity));
        }
        output.push('\n');

        output.push_str("**Exception Handling**:\n");
        output.push_str(&self.generate_exception_handling(stage));
        output.push('\n');

        output
    }

    /// Generate exception handling guide
    fn generate_exception_handling(&self, stage: SOCStage) -> String {
        let exceptions = match stage {
            SOCStage::InformationCollection => vec![
                "If necessary information is missing, proactively request supplementation",
                "If information sources are unreliable, perform cross-validation",
                "If system state is abnormal, record and report it",
            ],
            SOCStage::ThreatAnalysis => vec![
                "If a high-risk threat is detected, report it immediately",
                "If the threat pattern is unclear, initiate deep analysis",
                "If resources are insufficient, request support",
            ],
            SOCStage::DecisionMaking => vec![
                "If the strategy is infeasible, reassess the plan",
                "If resources are insufficient, adjust resource allocation",
                "If multiple feasible plans exist, perform trade-off analysis",
            ],
            SOCStage::OperationExecution => vec![
                "If a tool call fails, log the error and retry",
                "If execution times out, check system status",
                "If unexpected results occur, pause and analyze",
            ],
            SOCStage::ResultVerification => vec![
                "If verification fails, analyze the cause and adjust the strategy",
                "If results do not meet criteria, re-execute relevant steps",
                "If additional resources are needed, request support",
            ],
            SOCStage::ReportGeneration => vec![
                "If data is incomplete, supplement missing information",
                "If the report format has issues, adjust the format",
                "If additional analysis is needed, add analysis content",
            ],
            SOCStage::KnowledgeArchiving => vec![
                "If storage fails, check system status and retry",
                "If knowledge format has issues, adjust the format",
                "If additional knowledge is needed, perform knowledge extraction",
            ],
        };

        let mut output = String::new();
        for exception in exceptions {
            output.push_str(&format!("- {}\n", exception));
        }

        output
    }

    /// Generate dependency description template
    pub fn generate_dependencies_template(&self) -> String {
        let mut output = String::new();

        output.push_str("## Dependencies\n\n");

        output.push_str("### Required Tools\n");
        output.push_str("- [Tool Name]: [Usage Description]\n\n");

        output.push_str("### Optional Tools\n");
        output.push_str("- [Tool Name]: [Usage Description]\n\n");

        output.push_str("### Collaborating Skills\n");
        output.push_str("- [Skills Name]: [Collaboration Scenario]\n\n");

        output
    }

    /// Generate brief SOC process description (for embedding into existing prompts)
    pub fn generate_soc_brief(&self) -> String {
        let mut output = String::new();

        output.push_str("## SOC Process Overview\n\n");
        output.push_str("All your operations must follow the standard SOC (Security Operations Center) process:\n\n");

        for stage in SOCStage::all_stages() {
            output.push_str(&format!(
                "{}. **{}**: {}\n",
                stage.number(),
                stage.name(),
                stage.description()
            ));
        }

        output.push_str("\n**Flow Chart**:\n");
        output.push_str(&self.mermaid_generator.generate_flowchart());

        output
    }
}

impl Default for PromptTemplateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_generate_skills_prompt() -> Result<()> {
        let generator = PromptTemplateGenerator::new();
        let prompt = generator.generate_skills_prompt("test_skills", "This is a test Skills");

        assert!(prompt.contains("test_skills Skills Code of Conduct"));
        assert!(prompt.contains("This is a test Skills"));
        assert!(prompt.contains("SOC Process-Oriented"));
        assert!(prompt.contains("Information Collection"));
        assert!(prompt.contains("Threat Analysis"));
        Ok(())
    }

    #[test]
    fn test_generate_dependencies_template() -> Result<()> {
        let generator = PromptTemplateGenerator::new();
        let template = generator.generate_dependencies_template();

        assert!(template.contains("Dependencies"));
        assert!(template.contains("Required Tools"));
        assert!(template.contains("Collaborating Skills"));
        Ok(())
    }

    #[test]
    fn test_generate_soc_brief() -> Result<()> {
        let generator = PromptTemplateGenerator::new();
        let brief = generator.generate_soc_brief();

        assert!(brief.contains("SOC Process Overview"));
        assert!(brief.contains("graph TD"));
        Ok(())
    }

    #[test]
    fn test_stage_section_contains_key_activities() -> Result<()> {
        let generator = PromptTemplateGenerator::new();
        let prompt = generator.generate_skills_prompt("test", "test");

        assert!(prompt.contains("Receive task input"));
        assert!(prompt.contains("Pattern recognition"));
        assert!(prompt.contains("Strategy selection"));
        Ok(())
    }

    #[test]
    fn test_stage_section_contains_exception_handling() -> Result<()> {
        let generator = PromptTemplateGenerator::new();
        let prompt = generator.generate_skills_prompt("test", "test");

        assert!(prompt.contains("Exception Handling"));
        assert!(prompt.contains("If necessary information is missing"));
        Ok(())
    }
}
