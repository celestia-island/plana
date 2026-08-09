use serde_json::Value;
use std::{future::Future, pin::Pin};

use super::tools::ToolResult;
use _domain_agent::AgentMarker;
use _domain_skills_permissions::ToolCapability;

pub trait Tool: Send + Sync + 'static {
    type Agent: AgentMarker;

    const NAME: &'static str;

    const CAPABILITY: ToolCapability = ToolCapability {
        access_mode: _domain_skills_permissions::AccessMode::Read,
        risk_level: _domain_skills_permissions::RiskLevel::Info,
        scope: _domain_skills_permissions::ToolScope::Any,
    };

    fn invoke(&self, params: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>>;

    fn schema(&self) -> ToolSchema {
        ToolSchema::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolSchema {
    pub description: &'static str,
    pub required: &'static [&'static str],
}

pub trait ErasedTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn agent_folder(&self) -> &'static str;
    fn capability(&self) -> ToolCapability;
    fn invoke_erased(&self, params: Value)
    -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>>;
    fn schema(&self) -> ToolSchema;
}

impl<T: Tool> ErasedTool for T {
    fn name(&self) -> &'static str {
        T::NAME
    }

    fn agent_folder(&self) -> &'static str {
        <T::Agent as AgentMarker>::FOLDER_NAME
    }

    fn capability(&self) -> ToolCapability {
        T::CAPABILITY
    }

    fn invoke_erased(
        &self,
        params: Value,
    ) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        self.invoke(params)
    }

    fn schema(&self) -> ToolSchema {
        <Self as Tool>::schema(self)
    }
}

pub struct ToolDescriptor {
    name: &'static str,
    agent_folder: &'static str,
    capability: ToolCapability,
    schema: ToolSchema,
    tool: Box<dyn ErasedTool>,
}

impl ToolDescriptor {
    pub fn new<T: Tool>(tool: T) -> Self {
        Self {
            name: T::NAME,
            agent_folder: <T::Agent as AgentMarker>::FOLDER_NAME,
            capability: T::CAPABILITY,
            schema: tool.schema(),
            tool: Box::new(tool),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn agent_folder(&self) -> &'static str {
        self.agent_folder
    }

    pub fn capability(&self) -> &ToolCapability {
        &self.capability
    }

    pub async fn invoke(&self, params: Value) -> ToolResult {
        self.tool.invoke_erased(params).await
    }

    pub fn schema(&self) -> &ToolSchema {
        &self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use _domain_agent::HubRisMarker;

    struct DummyTool;

    impl Tool for DummyTool {
        type Agent = HubRisMarker;
        const NAME: &'static str = "dummy_test_tool";

        fn invoke(&self, _params: Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
            Box::pin(async { ToolResult::success_text("ok".into()) })
        }
    }

    #[tokio::test]
    async fn erased_dispatch_works() {
        let tool = DummyTool;
        let erased: &dyn ErasedTool = &tool;
        assert_eq!(erased.name(), "dummy_test_tool");
        assert_eq!(erased.agent_folder(), "hubris");
        let result = erased.invoke_erased(Value::Null).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn tool_descriptor_dispatch() {
        let desc = ToolDescriptor::new(DummyTool);
        assert_eq!(desc.name(), "dummy_test_tool");
        assert_eq!(desc.agent_folder(), "hubris");
        let result = desc.invoke(Value::Null).await;
        assert!(result.success);
    }

    #[test]
    fn typed_tool_has_zero_overhead() {
        assert_eq!(std::mem::size_of::<DummyTool>(), 0);
    }
}
