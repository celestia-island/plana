//! 全局状态树同步状态机 —— 联机游戏式的客户端/服务端状态同步设施。
//!
//! arona 的通信协调核心。提供两套同步机制：
//!
//! - **同步上下文（高频状态同步）**：服务端权威的状态树，客户端声明视口
//!   （path 前缀），服务端推送增量 patch（`PatchOp`）+ 周期全量快照兜底。
//!   适合状态类数据（agents、devices、conversations 等）。
//!
//! - **推送上下文（低频按需查询）**：topic 订阅制的事件流频道，
//!   适合时间序列数据（logs、reports、streaming chunks 等）。
//!
//! ## 结构
//!
//! - [`patch`] —— JSON-Merge-Patch 合并 + diff 生成（RFC 7396 变体）。
//! - [`snapshot`] —— 视口路径前缀匹配 + 子树裁剪。
//! - [`tree`] —— 单个隔离树实例（scope = workspace 全局 / user 私有）。
//! - [`registry`] —— 树实例表，懒加载 + 空闲回收 reaper。
//! - [`store`] —— 叶子持久化 trait（NoopStore / MemoryStore / 消费方自定义）。
//! - [`session`] —— 客户端会话（视口 + 频道订阅 + 双 scope）。
//! - [`channel`] —— 推送上下文：topic → method 映射表 + 事件包装。
//! - [`state_bridge`] —— 同步上下文的服务端推送 writer。
//! - [`domains`] —— 域适配器（agents / devices / conversations）。

pub mod domains;
pub mod patch;
pub mod registry;
pub mod snapshot;
pub mod store;
pub mod tree;

// ═══════════════════════════════════════════════════════════════
//  Re-exports
// ═══════════════════════════════════════════════════════════════

pub use patch::{PatchKind, PatchOp};
pub use registry::StateTreeRegistry;
pub use store::{MemoryStore, NoopStore, StateStore};
pub use tree::{PatchEvent, ScopeKey, ScopeOwner, StateTree};

// ═══════════════════════════════════════════════════════════════
//  Wire protocol constants
// ═══════════════════════════════════════════════════════════════

/// JSON-RPC method name for state patch notifications.
pub const METHOD_STATE_PATCH: &str = "Sync.StatePatch";
/// JSON-RPC method name for state snapshot notifications.
pub const METHOD_STATE_SNAPSHOT: &str = "Sync.StateSnapshot";
/// JSON-RPC method name for channel event notifications.
pub const METHOD_CHANNEL_EVENT: &str = "Sync.ChannelEvent";

/// RPC method for subscribing to state viewports.
pub const RPC_STATE_SUBSCRIBE: &str = "state.subscribe";
/// RPC method for unsubscribing from state viewports.
pub const RPC_STATE_UNSUBSCRIBE: &str = "state.unsubscribe";
/// RPC method for subscribing to channels.
pub const RPC_CHANNEL_SUBSCRIBE: &str = "channel.subscribe";
/// RPC method for unsubscribing from channels.
pub const RPC_CHANNEL_UNSUBSCRIBE: &str = "channel.unsubscribe";

/// Periodic full-snapshot fallback interval (server pushes viewport snapshot
/// every this duration as a self-healing mechanism).
pub const SNAPSHOT_TICK_SECS: u64 = 3;

/// Topic → JSON-RPC method mapping for the push context (channel) layer.
///
/// Maps server-push notification method names to snake_case topic names.
/// State-snapshot methods are deliberately excluded (they belong to the
/// sync context, not the push context).
pub const ALL_TOPICS: &[(&str, &str)] = &[
    ("Sync.AgentStreamingChunk", "agent_streaming"),
    ("Sync.AgentThinkingStep", "agent_thinking"),
    ("Sync.AgentToolCall", "agent_tool_call"),
    ("Sync.McpToolResult", "mcp_tool_result"),
    ("Sync.SkillChainStart", "skill_chain"),
    ("Sync.SkillChainStep", "skill_chain"),
    ("Sync.SkillChainComplete", "skill_chain"),
    ("Sync.YoloCycleStep", "yolo_cycle"),
    ("Sync.YoloCycleComplete", "yolo_cycle"),
    ("Sync.TaskCreated", "task"),
    ("Sync.TaskStatusUpdate", "task"),
    ("Sync.ServerLogEntry", "server_logs"),
    ("Sync.ContainerLogEntry", "container_logs"),
    ("Sync.AgentReport", "reports"),
    ("Sync.IndustrialTelemetryPush", "industrial_telemetry"),
    ("Sync.IndustrialAlarmPush", "industrial_alarm"),
    (
        "Sync.IndustrialWriteApprovalPush",
        "industrial_write_approval",
    ),
    ("Sync.HumanReviewRequest", "human_review"),
    ("Sync.SystemMessage", "system_notification"),
    ("Sync.AudioPullProgress", "audio_pull_progress"),
];

/// Look up the channel topic name for a given JSON-RPC method.
/// Returns `None` if the method is not an event-stream method.
pub fn topic_for_method(method: &str) -> Option<&'static str> {
    ALL_TOPICS
        .iter()
        .find(|(m, _)| *m == method)
        .map(|(_, t)| *t)
}

#[cfg(test)]
mod topic_tests {
    use super::*;

    #[test]
    fn system_message_maps_to_system_notification_topic() {
        assert_eq!(
            topic_for_method("Sync.SystemMessage"),
            Some("system_notification")
        );
    }
}
