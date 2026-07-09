//! 状态树的域适配器 —— 把每个业务域的数据接进树。
//!
//! 每个域提供：
//! - `upsert_*` —— 把该域的实体列表/增量写进 `state.<domain>.<id>`。
//! - （可选）`load_initial` —— 首次访问该 scope 时的懒载入钩子。
//!
//! 已迁移：agents（workspace，含字段级增量）、devices（workspace）、
//! preferences（user）、conversations（user）。其余域按相同模式增量迁移。

pub mod agents;
pub mod conversations;
pub mod devices;

pub use agents::{load_initial as load_agents, remove_agent, upsert_agent_patches, upsert_agents};
pub use conversations::{
    remove_conversation, upsert_conversation, upsert_conversations,
    user_scope as user_conversation_scope,
};
pub use devices::{load_initial as load_devices, remove_device, upsert_devices};
