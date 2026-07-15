use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ReviewStatus, ValidationResult};

/// Metadata review record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataReview {
    pub id: Uuid,
    pub provider_id: String,
    pub status: ReviewStatus,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewer: Option<String>,
    pub comments: Vec<String>,
    pub validation_result: ValidationResult,
}

impl MetadataReview {
    pub fn new(provider_id: String, validation_result: ValidationResult) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            provider_id,
            status: ReviewStatus::Pending,
            submitted_at: now,
            reviewed_at: None,
            reviewer: None,
            comments: Vec::new(),
            validation_result,
        }
    }

    pub fn approve(&mut self, reviewer: String, comment: Option<String>) {
        self.status = ReviewStatus::Approved;
        self.reviewed_at = Some(Utc::now());
        self.reviewer = Some(reviewer);
        if let Some(c) = comment {
            self.comments.push(c);
        }
    }

    pub fn reject(&mut self, reviewer: String, reason: String) {
        self.status = ReviewStatus::Rejected;
        self.reviewed_at = Some(Utc::now());
        self.reviewer = Some(reviewer);
        self.comments.push(reason);
    }

    pub fn request_changes(&mut self, reviewer: String, changes: Vec<String>) {
        self.status = ReviewStatus::NeedsChanges;
        self.reviewed_at = Some(Utc::now());
        self.reviewer = Some(reviewer);
        self.comments.extend(changes);
    }
}
