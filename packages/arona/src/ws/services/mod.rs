//! Service-facing message params — authentication, LLM provider configuration,
//! knowledge-base (RAG) lifecycle, and industrial-control telemetry.

pub mod auth;
pub mod industrial;
pub mod knowledge_base;
pub mod llm_provider;
