use serde::{Deserialize, Serialize};

use plana_state_sync::ModelTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSubcallResult {
    pub success: bool,
    pub content: String,
    pub model_name: Option<String>,
    pub token_usage: Option<(u32, u32)>,
}

/// Identity a ghost-subagent call runs under. All fields optional so the
/// legacy anonymous shape stays constructible; `None` everywhere is
/// exactly the pre-extension behavior.
///
/// The ghost-subagent program (workspace PLAN §2.16) upgrades anonymous
/// `LlmSubcallService` calls into identity-carrying inline sub-agents:
/// no downstream node, no container, but a badge, a UUID, and a record
/// the parent's raw-record view can show.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubagentCallContext {
    /// Full badge of the ghost sub-agent, e.g. `demiurge.001.002`.
    /// The caller (allocator) guarantees uniqueness and depth ≤ 5.
    pub badge: Option<String>,
    /// UUID minted for this invocation (string form; this crate stays
    /// uuid-dependency-free).
    pub uuid: Option<String>,
    /// Functional kind driving the call, e.g. `rag_search` or
    /// `sensitive_preflight`. Fixed prompt per kind.
    pub kind: Option<String>,
    /// Badge of the parent the sub-agent nests under (for attribution
    /// when the recorder is not the allocator).
    pub parent_badge: Option<String>,
}

#[async_trait::async_trait]
pub trait LlmSubcallService: Send + Sync {
    async fn llm_chat(
        &self,
        tier: ModelTier,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> LlmSubcallResult;

    /// Identity-carrying variant for ghost sub-agents.
    ///
    /// Default implementation delegates to the anonymous [`Self::llm_chat`]
    /// (ignoring the context), so every existing implementer keeps
    /// compiling and behaving identically; implementers that record
    /// sub-agent calls override this.
    async fn llm_chat_as(
        &self,
        ctx: &SubagentCallContext,
        tier: ModelTier,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> LlmSubcallResult {
        let _ = ctx;
        self.llm_chat(tier, prompt, system_prompt, max_tokens, temperature)
            .await
    }

    fn record_token_usage(
        &self,
        agent_id: &str,
        agent_type: plana_state_sync::Agent,
        model_name: Option<&str>,
        input_tokens: u32,
        output_tokens: u32,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingSvc {
        anon_calls: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl LlmSubcallService for CountingSvc {
        async fn llm_chat(
            &self,
            _tier: ModelTier,
            _prompt: &str,
            _system_prompt: Option<&str>,
            _max_tokens: Option<u32>,
            _temperature: Option<f32>,
        ) -> LlmSubcallResult {
            self.anon_calls.fetch_add(1, Ordering::SeqCst);
            LlmSubcallResult {
                success: true,
                content: "anon".into(),
                model_name: None,
                token_usage: None,
            }
        }

        fn record_token_usage(
            &self,
            _agent_id: &str,
            _agent_type: plana_state_sync::Agent,
            _model_name: Option<&str>,
            _input_tokens: u32,
            _output_tokens: u32,
        ) {
        }
    }

    #[tokio::test]
    async fn llm_chat_as_defaults_to_anonymous_call() {
        let svc = CountingSvc {
            anon_calls: AtomicUsize::new(0),
        };
        let ctx = SubagentCallContext {
            badge: Some("demiurge.001.002".into()),
            uuid: Some("0192aaaa-bbbb-7ccc-8ddd-0123456789ab".into()),
            kind: Some("rag_search".into()),
            parent_badge: Some("demiurge.001".into()),
        };
        let result = svc
            .llm_chat_as(&ctx, ModelTier::Basic, "q", None, Some(16), Some(0.2))
            .await;
        assert!(result.success);
        // The default impl must route through llm_chat exactly once.
        assert_eq!(svc.anon_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn subagent_call_context_defaults_to_anonymous() {
        let ctx = SubagentCallContext::default();
        assert!(ctx.badge.is_none());
        assert!(ctx.uuid.is_none());
        assert!(ctx.kind.is_none());
        assert!(ctx.parent_badge.is_none());
    }
}
