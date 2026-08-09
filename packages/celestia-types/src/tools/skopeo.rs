use crate::enums::{GoalStatus, GoalTaskStatus, TrackStatus};

// ═══════════════════════════════════════════════════════════════════════════════
// Goal / Track / GoalTask — replaces the old OKR system
// ═══════════════════════════════════════════════════════════════════════════════

// ── Goal result types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalEntry {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: GoalStatus,
    pub priority: u32,
    pub owner: String,
    pub parent_goal_id: Option<String>,
    pub workspace_ids: Vec<String>,
    pub alignment_score: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalCreateResult {
    pub goal_id: String,
    pub title: String,
    pub status: GoalStatus,
    pub workspace_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalUpdateResult {
    pub goal_id: String,
    pub title: String,
    pub status: GoalStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalCloseResult {
    pub goal_id: String,
    pub status: GoalStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalListResult {
    pub count: usize,
    pub goals: Vec<GoalEntry>,
}

// ── Track result types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct TrackEntry {
    pub id: String,
    pub goal_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TrackStatus,
    pub target_value: Option<f64>,
    pub current_value: f64,
    pub unit: Option<String>,
    pub weight: f64,
    pub due_at: Option<String>,
    pub workspace_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct TrackCreateResult {
    pub track_id: String,
    pub goal_id: String,
    pub title: String,
    pub status: TrackStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct TrackUpdateResult {
    pub track_id: String,
    pub title: String,
    pub status: TrackStatus,
    pub current_value: f64,
    pub target_value: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct TrackCloseResult {
    pub track_id: String,
    pub status: TrackStatus,
}

// ── GoalTask result types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskEntry {
    pub id: String,
    pub track_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: GoalTaskStatus,
    pub assignee: Option<String>,
    pub effort_estimate: Option<String>,
    pub due_at: Option<String>,
    pub workspace_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskCreateResult {
    pub task_id: String,
    pub track_id: String,
    pub title: String,
    pub status: GoalTaskStatus,
    pub workspace_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskUpdateResult {
    pub task_id: String,
    pub title: String,
    pub status: GoalTaskStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskCompleteResult {
    pub task_id: String,
    pub status: GoalTaskStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskListResult {
    pub count: usize,
    pub tasks: Vec<GoalTaskEntry>,
}

// ── Alignment check ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct AlignmentCheckResult {
    pub goal_id: String,
    pub alignment_status: String,
    pub score: f64,
    pub recommendations: String,
}

// ── Param types ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalCreateParams {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<u32>,
    pub owner: Option<String>,
    pub parent_goal_id: Option<String>,
    pub workspace_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalUpdateParams {
    pub goal_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<u32>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalCloseParams {
    pub goal_id: String,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalListParams {
    pub workspace_id: Option<String>,
    pub status: Option<String>,
    pub parent_goal_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct TrackCreateParams {
    pub goal_id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_value: Option<f64>,
    pub current_value: Option<f64>,
    pub unit: Option<String>,
    pub weight: Option<f64>,
    pub due_at: Option<String>,
    pub workspace_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct TrackUpdateParams {
    pub track_id: String,
    pub title: Option<String>,
    pub target_value: Option<f64>,
    pub current_value: Option<f64>,
    pub status: Option<String>,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct TrackCloseParams {
    pub track_id: String,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskCreateParams {
    pub track_id: String,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub effort_estimate: Option<String>,
    pub due_at: Option<String>,
    pub workspace_ids: Option<Vec<String>>,
    pub parent_task_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskUpdateParams {
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    #[ts(type = "Record<string, unknown> | null")]
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskCompleteParams {
    pub task_id: String,
    #[ts(type = "Record<string, unknown> | null")]
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct GoalTaskListParams {
    pub track_id: Option<String>,
    pub goal_id: Option<String>,
    pub workspace_id: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "tools/skopeo.ts")]
pub struct AlignmentCheckParams {
    pub goal_id: String,
}
