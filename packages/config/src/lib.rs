//! Configuration system crate
//!
//! Provides TOML config caching, provider config, RBAC, environment variables, hot-reload.
//!
//! ## Design documentation
//!
//! See [`docs/design/en/agent-config-system.md`](https://github.com/celestia-island/entelecheia/blob/main/docs/design/en/agent-config-system.md)
//! for detailed design docs covering Agent Configuration System architecture.
#![allow(clippy::type_complexity)]

mod agent_config;
mod app_config;
pub mod async_protocol;
pub mod context_config;
pub mod dangerous_flags;
pub mod embedding_models;
pub mod errors;
mod event_bus;
pub mod gen_protocol;
pub mod model_category;
pub mod model_preference;
mod provider_config;
mod provider_config_validator;
mod provider_config_watcher;
mod provider_crud;
mod provider_metadata;
mod rbac_config;
pub mod toml_cache;

// Re-export everything that config/mod.rs exports
pub use agent_config::LlmProviderConfig;
pub use app_config::{AppConfig, DatabaseConfig, LlmConfig, UiConfig, UiMode, UserConfig};
pub use async_protocol::{
    AsyncGenRequest, AsyncGenResult, AsyncProtocolError, AsyncProtocolHandler, AsyncTaskStatus,
    ReferenceInput, ReferenceKind,
};
pub use context_config::{ConnectionContext, ContextStore, ContextType};
pub use event_bus::{
    ConfigChangeAudit, ConfigEvent, ConfigEventBus, ConfigEventListener, ConfigHotReloadManager,
    add_global_listener, get_global_audit_log, publish_global_event, subscribe_global_events,
};
pub use gen_protocol::{
    AudioKind, Capability, GenProtocol, ProtocolCapability, ThreeDKind, UnknownCapabilityError,
    UnknownProtocolError,
};
pub use model_category::{GenerationModality, GenerationParams, ModelCategory};
pub use model_preference::{
    AgentModelPreferenceEntry, AgentModelPreferences, ModelPreference, QualifiedModelId,
    QualifiedModelIdParseError, ResolvedPreference, SkillModelPreferenceEntry,
    find_prefs_config_path, load_agent_model_preferences,
};
pub use provider_config::{
    AutonomousMemory, AvailablePeriodConfig, BillingType, ModelEntry, PeriodUnit,
    PriorityOrderEntry, ProviderConfigData, ProviderEntry, ResolvedModelCard, StatsType,
    build_provider_order_map, ensure_provider_config_from_env, find_provider_config_path,
    load_provider_config, select_model_card_for_tier, select_model_entry_for_tier,
};
pub use provider_config_validator::{
    AvailableModel, AvailableProvider, DefaultProviderConfigValidator, ProviderConfigValidator,
    ProviderValidationError, ProviderValidationResult, ValidationErrorDetail,
    ValidationWarningDetail, validate_and_log_providers,
};
pub use provider_config_watcher::{
    ConfigUpdateAuditLog, ProviderConfigEvent, ProviderConfigWatcher, get_global_config,
    get_global_watcher, spawn_global_watcher, subscribe_global_events as subscribe_provider_events,
};
pub use provider_crud::{
    ConfigChangeEvent, ConfigChangeSource, ConfiguredProvider, ProviderConfigManager,
    ProviderCrudConfig, ProviderCrudPreferences, ProviderUpdates,
};
pub use provider_metadata::{
    MetadataReview, MetadataReviewManager, MetadataSource, ModelMetadata, PricingModel,
    ProviderMetadata, ProviderMetadataValidator, ProviderQuotaInfo, RateLimit, ReviewStatus,
    UsageType, ValidationError, ValidationErrorType, ValidationResult, ValidationWarning,
};
pub use rbac_config::RbacConfig;
pub use toml_cache::TomlConfigCache;
