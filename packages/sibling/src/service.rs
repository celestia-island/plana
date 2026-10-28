//! The built, ready-to-call sibling handle.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use secrecy::SecretString;
use serde::Serialize;

use crate::builder::SiblingServiceBuilder;
use crate::call::SiblingCall;
use crate::provider::EndpointProvider;
use crate::retry::Retry;

/// One slot in the endpoint list, preserving builder declaration order:
/// static URLs resolved at build time, dynamic providers queried per call.
#[derive(Clone)]
pub(crate) enum EndpointSlot {
    Static(String),
    Dynamic(Arc<dyn EndpointProvider>),
}

/// A declared sibling service endpoint — one instance per target sibling per
/// consumer process. Cheap to clone (shares the underlying connection pool).
#[derive(Clone)]
pub struct SiblingService {
    pub(crate) name: String,
    pub(crate) client: reqwest::Client,
    pub(crate) endpoints: Vec<EndpointSlot>,
    pub(crate) token: Option<SecretString>,
    pub(crate) total_timeout: Duration,
}

impl fmt::Debug for SiblingService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SiblingService")
            .field("name", &self.name)
            .field("static_endpoints", &self.endpoints.len())
            .field("has_bearer_token", &self.token.is_some())
            .field("total_timeout", &self.total_timeout)
            .finish()
    }
}

impl SiblingService {
    /// Start building a sibling named `name` (the tracing/log identity).
    ///
    /// Returns the builder, not `Self` — the declarative one-liner
    /// (`SiblingService::new("arona").endpoints_env(..).build()?`) is the
    /// whole point of the facility, so this lint deviation is by design.
    #[must_use]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(name: impl Into<String>) -> SiblingServiceBuilder {
        SiblingServiceBuilder::new(name)
    }

    /// Alias of [`SiblingService::new`].
    #[must_use]
    pub fn builder(name: impl Into<String>) -> SiblingServiceBuilder {
        SiblingServiceBuilder::new(name)
    }

    /// The sibling identity used in spans and every error variant.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Begin a request-then-response call. `path` is appended to the
    /// endpoint root; `body` is JSON-serialized (omitted for GET/HEAD —
    /// pass `&()` there).
    pub fn call<'req, B: Serialize + ?Sized>(
        &self,
        method: http::Method,
        path: &str,
        body: &'req B,
    ) -> SiblingCall<'_, 'req, B> {
        SiblingCall {
            service: self,
            method,
            path: path.to_string(),
            body: Some(body),
            retry: Retry::None,
            total_timeout: None,
            idempotency_key: None,
        }
    }

    pub(crate) fn client(&self) -> &reqwest::Client {
        &self.client
    }

    pub(crate) fn token(&self) -> Option<&SecretString> {
        self.token.as_ref()
    }

    pub(crate) fn total_timeout(&self) -> Duration {
        self.total_timeout
    }

    /// Ordered endpoint list for one call: static slots in declaration
    /// order, each dynamic provider queried in place (its endpoints splice
    /// in at the slot's position).
    pub(crate) fn resolve_endpoints(&self) -> Vec<String> {
        let mut out = Vec::new();
        for slot in &self.endpoints {
            match slot {
                EndpointSlot::Static(url) => out.push(url.clone()),
                EndpointSlot::Dynamic(provider) => out.extend(provider.endpoints()),
            }
        }
        out
    }
}
