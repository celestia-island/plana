use super::{
    enums::{AnnotationType, FileOpStatus},
    haplotes::{AgentReference, ConflictInfo},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileListResult {
    pub path: String,
    pub total_count: usize,
    #[serde(rename = "items")]
    pub entries: Vec<FileEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: super::enums::FileType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct MkDirResult {
    pub path: String,
    pub status: FileOpStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileDeleteResult {
    pub path: String,
    pub status: FileOpStatus,
    #[serde(default)]
    pub conflicts: Vec<ConflictInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileEditResult {
    pub path: String,
    pub status: FileOpStatus,
    pub occurrences: usize,
    #[serde(default)]
    pub conflicts: Vec<ConflictInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileReadResult {
    pub path: String,
    pub size_bytes: usize,
    pub content: String,
    #[serde(default)]
    pub conflicts: Vec<ConflictInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileExistsResult {
    pub path: String,
    pub exists: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileWriteResult {
    pub path: String,
    pub size_bytes: usize,
    pub status: FileOpStatus,
    #[serde(default)]
    pub conflicts: Vec<ConflictInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileInfoResult {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: super::enums::FileType,
    pub size_bytes: u64,
    pub modified_unix: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileTreeEntry>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct FileTreeListResult {
    pub path: String,
    pub total_count: usize,
    pub tree: Vec<FileTreeEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Annotation {
    pub id: String,
    pub file_path: String,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub content: String,
    pub annotation_type: AnnotationType,
    pub author: Option<AgentReference>,
    pub created_at: String,
    pub resolved: bool,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ListAnnotationsResult {
    pub file_path: String,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ResolveAnnotationResult {
    pub annotation_id: String,
    pub resolved: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct FileReadParams {
    pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct FileWriteParams {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct FileEditParams {
    pub path: String,
    pub old_content: String,
    pub new_content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct FileDeleteParams {
    pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct FileExistsParams {
    pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct FileListParams {
    pub path: String,
    #[serde(default)]
    pub recursive: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct FileGetInfoParams {
    pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
pub struct FileCreateDirParams {
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_file_read_result_with_conflicts() -> Result<()> {
        let r = FileReadResult {
            path: "src/main.rs".to_string(),
            size_bytes: 100,
            content: "fn main() {}".to_string(),
            conflicts: vec![ConflictInfo {
                conflict_id: "cflict-001".to_string(),
                file_path: "src/main.rs".to_string(),
                line_range: None,
                conflicting_agent: super::super::haplotes::AgentReference {
                    agent_type: "WebAutomation".to_string(),
                    instance_badge: None,
                },
                operation_type: super::super::enums::FileOperationType::Editing,
                since: "2026-05-11T10:00:00Z".to_string(),
            }],
        };
        let json = serde_json::to_string(&r)?;
        let de: FileReadResult = serde_json::from_str(&json)?;
        assert_eq!(de.conflicts.len(), 1);
        assert_eq!(de.conflicts[0].conflict_id, "cflict-001");
        Ok(())
    }

    #[test]
    fn test_file_edit_result_with_empty_conflicts() -> Result<()> {
        let r = FileEditResult {
            path: "src/lib.rs".to_string(),
            status: FileOpStatus::Edited,
            occurrences: 1,
            conflicts: vec![],
        };
        let json = serde_json::to_string(&r)?;
        assert!(json.contains("conflicts"));
        let de: FileEditResult = serde_json::from_str(&json)?;
        assert!(de.conflicts.is_empty());
        Ok(())
    }

    #[test]
    fn test_backward_compat_missing_conflicts() -> Result<()> {
        let mut map = serde_json::Map::new();
        map.insert(
            "path".to_string(),
            serde_json::Value::String("src/lib.rs".to_string()),
        );
        map.insert(
            "status".to_string(),
            serde_json::Value::String("Edited".to_string()),
        );
        map.insert(
            "occurrences".to_string(),
            serde_json::Value::Number(1.into()),
        );
        let val = serde_json::Value::Object(map);
        let json = serde_json::to_string(&val)?;
        let de: FileEditResult = serde_json::from_str(&json)?;
        assert!(de.conflicts.is_empty());
        Ok(())
    }
}
