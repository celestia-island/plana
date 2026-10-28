//! [`SiblingService`] construction: endpoint sources, credentials, timeouts,
//! proxy hygiene — resolved eagerly at `build()` so misconfiguration fails
//! at startup instead of on the first call.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use secrecy::SecretString;
use thiserror::Error;

use crate::provider::EndpointProvider;
use crate::service::SiblingService;

/// Total-deadline default (design §3.4: services in the wild currently
/// scatter 2–120 s; migration PRs must pin each call site's original value).
pub(crate) const DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(10);

/// Builder-time failure: the facility is misconfigured and refuses to start.
#[derive(Debug, Error)]
pub enum SiblingBuildError {
    /// The sibling name is the tracing/log identity; an empty one would make
    /// every span useless.
    #[error("sibling name must not be empty")]
    NameEmpty,
    /// No endpoint source (`endpoints_env` / `endpoint_provider` /
    /// `endpoint_fixed`) was configured.
    #[error("sibling `{0}` has no endpoint source configured")]
    NoEndpointSource(String),
    /// The named environment variable is not set.
    #[error("environment variable `{name}` is not set")]
    EnvMissing { name: String },
    /// The named environment variable is set but yields no usable endpoint.
    #[error("environment variable `{name}` is set but yields no usable endpoints")]
    EnvEmpty { name: String },
    /// An endpoint URL failed validation (parse or scheme).
    #[error("endpoint `{url}` is invalid: {cause}")]
    InvalidEndpoint { url: String, cause: String },
    /// A bearer credential source is missing or empty.
    #[error("credential environment variable `{name}` is not set or empty")]
    TokenMissing { name: String },
    /// The underlying `reqwest::Client` could not be built.
    #[error("failed to build HTTP client: {0}")]
    HttpClient(String),
}

/// One endpoint source. Sources are tried in declaration order.
#[derive(Clone)]
pub(crate) enum EndpointSource {
    Env(String),
    Provider(Arc<dyn EndpointProvider>),
    Fixed(String),
}

/// Builder for [`SiblingService`] — start from
/// [`SiblingService::new`](crate::SiblingService::new).
#[allow(clippy::module_name_repetitions)]
pub struct SiblingServiceBuilder {
    name: String,
    sources: Vec<EndpointSource>,
    bearer_env: Option<String>,
    bearer_secret: Option<SecretString>,
    total_timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    via_proxy: bool,
}

impl fmt::Debug for SiblingServiceBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SiblingServiceBuilder")
            .field("name", &self.name)
            .field(
                "sources",
                &self
                    .sources
                    .iter()
                    .map(|s| match s {
                        EndpointSource::Env(v) => format!("env({v})"),
                        EndpointSource::Provider(_) => "provider()".to_string(),
                        EndpointSource::Fixed(u) => format!("fixed({u})"),
                    })
                    .collect::<Vec<_>>(),
            )
            .field("bearer_env", &self.bearer_env)
            .field("has_bearer_secret", &self.bearer_secret.is_some())
            .field("total_timeout", &self.total_timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("read_timeout", &self.read_timeout)
            .field("via_proxy", &self.via_proxy)
            .finish()
    }
}

impl SiblingServiceBuilder {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sources: Vec::new(),
            bearer_env: None,
            bearer_secret: None,
            total_timeout: None,
            connect_timeout: None,
            read_timeout: None,
            via_proxy: false,
        }
    }

    /// Read the endpoint list from an environment variable
    /// (`scheme://host:port,scheme://host:port,...`, order preserved,
    /// whitespace tolerated). Resolved once at `build()` — fail-fast.
    #[must_use]
    pub fn endpoints_env(mut self, var: impl Into<String>) -> Self {
        self.sources.push(EndpointSource::Env(var.into()));
        self
    }

    /// Dynamic endpoint callback (e.g. backed by a consumer-owned DB table).
    /// Queried on every call; see [`EndpointProvider`].
    #[must_use]
    pub fn endpoint_provider(mut self, provider: Arc<dyn EndpointProvider>) -> Self {
        self.sources.push(EndpointSource::Provider(provider));
        self
    }

    /// Pin one endpoint root (`scheme://host[:port]`); call repeatedly for a
    /// static list. Resolved once at `build()` — fail-fast.
    #[must_use]
    pub fn endpoint_fixed(mut self, url: impl Into<String>) -> Self {
        self.sources.push(EndpointSource::Fixed(url.into()));
        self
    }

    /// Bearer token read from an environment variable at `build()`
    /// (fail-fast). Stored in a redacting [`SecretString`]; never logged.
    #[must_use]
    pub fn bearer_env(mut self, var: impl Into<String>) -> Self {
        self.bearer_env = Some(var.into());
        self
    }

    /// Bearer token injected directly (already-resolved secret). Takes
    /// precedence over [`bearer_env`](Self::bearer_env) when both are set.
    #[must_use]
    pub fn bearer_secret(mut self, secret: SecretString) -> Self {
        self.bearer_secret = Some(secret);
        self
    }

    /// Total wall-clock deadline for one call — shared across all failover
    /// attempts (the worst case is not attempts × deadline); per-call
    /// `.timeout()` overrides it. Defaults to 10 s.
    #[must_use]
    pub fn total_timeout(mut self, d: Duration) -> Self {
        self.total_timeout = Some(d);
        self
    }

    /// Connect-phase timeout (client-level).
    #[must_use]
    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = Some(d);
        self
    }

    /// Read-phase timeout — maximum idle gap between reads (client-level).
    #[must_use]
    pub fn read_timeout(mut self, d: Duration) -> Self {
        self.read_timeout = Some(d);
        self
    }

    /// Opt in to proxy support (system/env proxies). Sibling endpoints live
    /// on the internal network; the default is `.no_proxy()` and this is the
    /// only way to leave it.
    #[must_use]
    pub fn via_proxy(mut self) -> Self {
        self.via_proxy = true;
        self
    }

    /// Resolve every eagerly-resolvable source and build the client.
    ///
    /// Fails loudly (returns [`SiblingBuildError`]) rather than degrading:
    /// a sibling that starts without its endpoint/token is a deployment
    /// bug, not a runtime surprise.
    pub fn build(self) -> Result<SiblingService, SiblingBuildError> {
        if self.name.is_empty() {
            return Err(SiblingBuildError::NameEmpty);
        }
        if self.sources.is_empty() {
            return Err(SiblingBuildError::NoEndpointSource(self.name.clone()));
        }

        let mut slots: Vec<crate::service::EndpointSlot> = Vec::with_capacity(self.sources.len());
        for source in &self.sources {
            match source {
                EndpointSource::Env(var) => {
                    let raw = std::env::var(var)
                        .map_err(|_| SiblingBuildError::EnvMissing { name: var.clone() })?;
                    let mut any = false;
                    for url in split_endpoint_list(&raw) {
                        validate_endpoint(&url).map_err(|cause| {
                            SiblingBuildError::InvalidEndpoint {
                                url: url.clone(),
                                cause,
                            }
                        })?;
                        slots.push(crate::service::EndpointSlot::Static(url));
                        any = true;
                    }
                    if !any {
                        return Err(SiblingBuildError::EnvEmpty { name: var.clone() });
                    }
                }
                EndpointSource::Fixed(url) => {
                    let url = url.trim().to_string();
                    validate_endpoint(&url).map_err(|cause| {
                        SiblingBuildError::InvalidEndpoint {
                            url: url.clone(),
                            cause,
                        }
                    })?;
                    slots.push(crate::service::EndpointSlot::Static(url));
                }
                EndpointSource::Provider(provider) => {
                    slots.push(crate::service::EndpointSlot::Dynamic(Arc::clone(provider)));
                }
            }
        }

        // Credentials: direct secret wins (documented); env resolves
        // fail-fast when no direct secret was given.
        let token = match self.bearer_secret {
            Some(secret) => Some(secret),
            None => match &self.bearer_env {
                Some(var) => {
                    let raw = std::env::var(var)
                        .map_err(|_| SiblingBuildError::TokenMissing { name: var.clone() })?;
                    if raw.trim().is_empty() {
                        return Err(SiblingBuildError::TokenMissing { name: var.clone() });
                    }
                    Some(SecretString::new(raw))
                }
                None => None,
            },
        };

        crate::ensure_rustls_crypto_provider();
        // No client-level total timeout: the call-level wall-clock deadline
        // lives in call::execute (shared across failover attempts). Only the
        // connect/read layers belong on the client.
        let mut client = reqwest::Client::builder();
        if let Some(d) = self.connect_timeout {
            client = client.connect_timeout(d);
        }
        if let Some(d) = self.read_timeout {
            client = client.read_timeout(d);
        }
        if !self.via_proxy {
            client = client.no_proxy();
        }
        let client = client
            .build()
            .map_err(|e| SiblingBuildError::HttpClient(e.to_string()))?;

        Ok(SiblingService {
            name: self.name,
            client,
            endpoints: slots,
            token,
            total_timeout: self.total_timeout.unwrap_or(DEFAULT_TOTAL_TIMEOUT),
        })
    }
}

/// Split an env endpoint list on commas, trimming whitespace and dropping
/// empty entries.
pub(crate) fn split_endpoint_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Endpoint URLs must parse, be http(s), and carry no userinfo — siblings
/// are network services; anything else is configuration noise. Userinfo is
/// rejected because endpoint strings flow into tracing spans and error
/// contexts, where embedded `user:password@` would leak credentials.
pub(crate) fn validate_endpoint(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("empty URL".to_string());
    }
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("parse failed: {e}"))?;
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("userinfo (`user:password@`) is not allowed in endpoint URLs".to_string());
    }
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        other => Err(format!("unsupported scheme `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_endpoint_list_trims_and_drops_empties() {
        assert_eq!(
            split_endpoint_list(" http://a ,http://b,  , http://c "),
            vec!["http://a", "http://b", "http://c"]
        );
        assert!(split_endpoint_list(" , , ").is_empty());
        assert!(split_endpoint_list("").is_empty());
    }

    #[test]
    fn validate_endpoint_rejects_bad_urls() {
        assert!(validate_endpoint("").is_err());
        assert!(validate_endpoint("ftp://192.0.2.1").is_err());
        assert!(validate_endpoint("not a url").is_err());
        assert!(validate_endpoint("http://192.0.2.1:3002").is_ok());
        assert!(validate_endpoint("https://192.0.2.1").is_ok());
    }

    #[test]
    fn validate_endpoint_rejects_userinfo() {
        // Credentials embedded in endpoint URLs would reach tracing spans
        // and error contexts (R2 S3-1).
        assert!(validate_endpoint("http://user:pass@192.0.2.1").is_err());
        assert!(validate_endpoint("http://user@192.0.2.1").is_err());
        assert!(validate_endpoint("http://:pass@192.0.2.1").is_err());
        assert!(validate_endpoint("http://192.0.2.1").is_ok());
    }
}
