use serde_json::Value;
use std::{collections::HashMap, marker::PhantomData};

use super::{mcp_tools::McpToolResult, tool_trait::ToolDescriptor};
use _domain_agent::AgentMarker;
use _domain_skills_permissions::ToolCapability;

pub struct ToolRegistry<M: AgentMarker> {
    tools: HashMap<&'static str, ToolDescriptor>,
    _marker: PhantomData<M>,
}

impl<M: AgentMarker> ToolRegistry<M> {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            _marker: PhantomData,
        }
    }

    pub fn register<T: McpTool<Agent = M>>(&mut self, tool: T) {
        let name = T::NAME;
        self.tools.insert(name, ToolDescriptor::new(tool));
    }

    pub fn register_boxed(&mut self, descriptor: ToolDescriptor) {
        self.tools.insert(descriptor.name(), descriptor);
    }

    pub async fn invoke(&self, tool_name: &str, params: Value) -> McpToolResult {
        match self.tools.get(tool_name) {
            Some(desc) => desc.invoke(params).await,
            None => {
                let agent = M::FRIENDLY_NAME;
                McpToolResult::failure(format!("{agent} does not provide tool: {tool_name}"))
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<&ToolDescriptor> {
        self.tools.get(name)
    }

    pub fn tools(&self) -> &HashMap<&'static str, ToolDescriptor> {
        &self.tools
    }

    pub fn tool_names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }

    pub fn tool_capabilities(&self) -> HashMap<&'static str, ToolCapability> {
        self.tools
            .iter()
            .map(|(name, desc)| (*name, *desc.capability()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl<M: AgentMarker> Default for ToolRegistry<M> {
    fn default() -> Self {
        Self::new()
    }
}

use super::tool_trait::McpTool;

pub struct GlobalToolRegistry {
    registries: HashMap<&'static str, ToolDescriptor>,
}

impl GlobalToolRegistry {
    pub fn new() -> Self {
        Self {
            registries: HashMap::new(),
        }
    }

    pub fn register<T: McpTool>(&mut self, tool: T) {
        let name = T::NAME;
        self.registries.insert(name, ToolDescriptor::new(tool));
    }

    pub async fn invoke(&self, tool_name: &str, params: Value) -> McpToolResult {
        match self.registries.get(tool_name) {
            Some(desc) => desc.invoke(params).await,
            None => McpToolResult::failure(format!("Tool not found: {tool_name}")),
        }
    }

    pub fn get(&self, name: &str) -> Option<&ToolDescriptor> {
        self.registries.get(name)
    }

    pub fn tool_names_by_agent(&self, agent_folder: &str) -> Vec<&'static str> {
        self.registries
            .values()
            .filter(|d| d.agent_folder() == agent_folder)
            .map(|d| d.name())
            .collect()
    }

    pub fn all_tools(&self) -> &HashMap<&'static str, ToolDescriptor> {
        &self.registries
    }

    pub fn len(&self) -> usize {
        self.registries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.registries.is_empty()
    }
}

impl Default for GlobalToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use _domain_agent::HubRisMarker;
    use std::future::Future;
    use std::pin::Pin;

    struct CreateTodo;
    impl McpTool for CreateTodo {
        type Agent = HubRisMarker;
        const NAME: &'static str = "create_todo";
        fn invoke(
            &self,
            _params: Value,
        ) -> Pin<Box<dyn Future<Output = McpToolResult> + Send + '_>> {
            Box::pin(async { McpToolResult::success_text("todo created".into()) })
        }
    }

    struct ListTodo;
    impl McpTool for ListTodo {
        type Agent = HubRisMarker;
        const NAME: &'static str = "list_todo";
        fn invoke(
            &self,
            _params: Value,
        ) -> Pin<Box<dyn Future<Output = McpToolResult> + Send + '_>> {
            Box::pin(async { McpToolResult::success_text("[]".into()) })
        }
    }

    #[tokio::test]
    async fn typed_registry_dispatch() {
        let mut reg: ToolRegistry<HubRisMarker> = ToolRegistry::new();
        reg.register(CreateTodo);
        reg.register(ListTodo);

        let result = reg.invoke("create_todo", Value::Null).await;
        assert!(result.success);

        let result = reg.invoke("unknown", Value::Null).await;
        assert!(!result.success);
    }

    #[tokio::test]
    async fn global_registry_dispatch() {
        let mut reg = GlobalToolRegistry::new();
        reg.register(CreateTodo);
        reg.register(ListTodo);

        let hubris_tools = reg.tool_names_by_agent("hubris");
        assert_eq!(hubris_tools.len(), 2);

        let result = reg.invoke("list_todo", Value::Null).await;
        assert!(result.success);
    }
}
