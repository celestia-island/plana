#[macro_export]
macro_rules! define_tool {
    (
        $(#[$meta:meta])*
        $name:ident($agent:ty) => $tool_name:literal {
            $($field:ident : $ty:ty),* $(,)?
        }
        invoke($params:ident) $body:block
    ) => {
        $(#[$meta])*
        pub struct $name {
            $($field: $ty,)*
        }

        impl $name {
            pub fn new($($field: $ty),*) -> Self {
                Self { $($field),* }
            }
        }

        impl $crate::tool_trait::McpTool for $name {
            type Agent = $agent;
            const NAME: &'static str = $tool_name;

            fn invoke(
                &self,
                $params: serde_json::Value,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = $crate::mcp_tools::McpToolResult> + Send + '_>> {
                Box::pin(async move { $body })
            }
        }
    };

    (
        $(#[$meta:meta])*
        $name:ident($agent:ty) => $tool_name:literal
        invoke($params:ident) $body:block
    ) => {
        $(#[$meta])*
        pub struct $name;

        impl $crate::tool_trait::McpTool for $name {
            type Agent = $agent;
            const NAME: &'static str = $tool_name;

            fn invoke(
                &self,
                $params: serde_json::Value,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = $crate::mcp_tools::McpToolResult> + Send + '_>> {
                Box::pin(async move { $body })
            }
        }
    };
}

#[macro_export]
macro_rules! register_agent_tools {
    ($registry:expr, $($tool:expr),* $(,)?) => {
        $(
            $registry.register($tool);
        )*
    };
}
