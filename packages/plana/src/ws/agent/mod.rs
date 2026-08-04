//! Agent runtime & orchestration message params — lifecycle streaming,
//! task management, state snapshots, the YOLO autonomous loop, and the
//! layer-2 / custom-agent registry.

pub mod agent_lifecycle;
pub mod layer2;
pub mod state_sync;
pub mod tasks;
pub mod yolo;
