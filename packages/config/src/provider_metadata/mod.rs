//! Provider metadata management
//!
//! Implements structured validation and review workflow:
//! - Format validation: data types, field formats
//! - Logic validation: value ranges, enum values
//! - Completeness validation: required fields present
//! - Consistency validation: cross-field relationships correct

pub mod parsing;
pub mod validation;

use anyhow::{Error, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use parsing::MetadataReview;
pub use validation::ProviderMetadataValidator;

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub filled_defaults: HashMap<String, serde_json::Value>,
    pub needs_review: bool,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            filled_defaults: HashMap::new(),
            needs_review: false,
        }
    }

    pub fn with_error(mut self, error: ValidationError) -> Self {
        self.passed = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: ValidationWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn with_filled_default(mut self, key: String, value: serde_json::Value) -> Self {
        self.filled_defaults.insert(key, value);
        self
    }

    pub fn mark_needs_review(mut self) -> Self {
        self.needs_review = true;
        self
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_type: ValidationErrorType,
    pub field: String,
    pub message: String,
}

/// Validation error type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationErrorType {
    Format,
    Logic,
    Completeness,
    Consistency,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

/// Provider metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub provider_id: String,
    pub display_name: String,
    pub api_endpoint: String,
    pub pricing_model: PricingModel,
    pub usage_type: UsageType,
    pub available_models: Vec<ModelMetadata>,
    pub default_model: Option<String>,
    pub rate_limit: Option<RateLimit>,
    pub quota: Option<ProviderQuotaInfo>,
    pub source: MetadataSource,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub model_name: String,
    pub context_window: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
    pub supports_streaming: bool,
    pub price_input: Option<f64>,
    pub price_output: Option<f64>,
}

/// Pricing model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    OneTime,
    Periodic,
    PayAsYouGo,
}

/// Usage type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageType {
    Metered,
    Quota,
    Unlimited,
}

/// Rate limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: Option<u32>,
    pub tokens_per_day: Option<u64>,
    pub concurrent_requests: Option<u32>,
}

/// Quota information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderQuotaInfo {
    pub limit: u64,
    pub period_hours: u32,
    pub reset_time: Option<DateTime<Utc>>,
}

/// Metadata source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataSource {
    Official,
    Community,
    UserOverride,
}

/// Review status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    NeedsChanges,
}

/// Metadata review manager
#[derive(Debug, Clone)]
pub struct MetadataReviewManager {
    pending_reviews: HashMap<Uuid, MetadataReview>,
    completed_reviews: Vec<MetadataReview>,
}

impl MetadataReviewManager {
    pub fn new() -> Self {
        Self {
            pending_reviews: HashMap::new(),
            completed_reviews: Vec::new(),
        }
    }

    pub fn submit_review(&mut self, review: MetadataReview) -> Result<(), Error> {
        let id = review.id;
        if self.pending_reviews.contains_key(&id) {
            return Err(anyhow!("Review record already exists"));
        }
        self.pending_reviews.insert(id, review);
        Ok(())
    }

    pub fn pending_reviews(&self) -> Vec<&MetadataReview> {
        self.pending_reviews.values().collect()
    }

    pub fn approve(
        &mut self,
        id: Uuid,
        reviewer: String,
        comment: Option<String>,
    ) -> Result<(), Error> {
        let review = self
            .pending_reviews
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Review record not found"))?;
        review.approve(reviewer, comment);

        let review = self
            .pending_reviews
            .remove(&id)
            .ok_or_else(|| anyhow!("Review record not found"))?;
        self.completed_reviews.push(review);
        Ok(())
    }

    pub fn reject(&mut self, id: Uuid, reviewer: String, reason: String) -> Result<(), Error> {
        let review = self
            .pending_reviews
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Review record not found"))?;
        review.reject(reviewer, reason);

        let review = self
            .pending_reviews
            .remove(&id)
            .ok_or_else(|| anyhow!("Review record not found"))?;
        self.completed_reviews.push(review);
        Ok(())
    }

    pub fn request_changes(
        &mut self,
        id: Uuid,
        reviewer: String,
        changes: Vec<String>,
    ) -> Result<(), Error> {
        let review = self
            .pending_reviews
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Review record not found"))?;
        review.request_changes(reviewer, changes);

        let review = self
            .pending_reviews
            .remove(&id)
            .ok_or_else(|| anyhow!("Review record not found"))?;
        self.completed_reviews.push(review);
        Ok(())
    }

    pub fn review_history(&self, provider_id: &str) -> Vec<&MetadataReview> {
        self.completed_reviews
            .iter()
            .filter(|r| r.provider_id == provider_id)
            .collect()
    }
}

impl Default for MetadataReviewManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result() -> Result<()> {
        let result = ValidationResult::new()
            .with_error(ValidationError {
                error_type: ValidationErrorType::Format,
                field: "test".to_string(),
                message: "test error".to_string(),
            })
            .with_warning(ValidationWarning {
                field: "test2".to_string(),
                message: "test warning".to_string(),
            })
            .mark_needs_review();

        assert!(!result.passed);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.needs_review);
        Ok(())
    }

    #[test]
    fn test_metadata_review() -> Result<()> {
        let mut manager = MetadataReviewManager::new();
        let validation_result = ValidationResult::new().mark_needs_review();
        let review = MetadataReview::new("test-provider".to_string(), validation_result);

        manager.submit_review(review.clone())?;
        assert_eq!(manager.pending_reviews().len(), 1);

        manager.approve(review.id, "admin".to_string(), Some("Good".to_string()))?;
        assert_eq!(manager.pending_reviews().len(), 0);
        assert_eq!(manager.completed_reviews.len(), 1);
        Ok(())
    }
}
