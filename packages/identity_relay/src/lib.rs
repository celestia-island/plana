//! The family's HTTP transport for kirino's OAuth relying-party core.
//!
//! kirino's [`HttpFetcher`] is the one seam between the relying-party
//! protocol (PKCE, state, the token exchange, identity) and the network —
//! the crate opens no socket by design. This member implements that seam
//! with `reqwest`, so every service that signs in through a provider or a
//! relay source sends exactly the requests the protocol core builds, and
//! none of them hand-writes a token call.
//!
//! # Posture
//!
//! * **No redirects.** A token or UserInfo request carries a secret or a
//!   bearer token; following a redirect would hand it to whatever host the
//!   answer names. The built-in client refuses to follow any.
//! * **A bounded timeout.** An identity provider that hangs must not hang
//!   the sign-in with it; [`ReqwestFetcher::DEFAULT_TIMEOUT`] unless the
//!   host says otherwise.
//! * **Errors carry no secrets.** Transport errors are reported without
//!   the request URL (`reqwest`'s `without_url`), and a non-2xx answer is
//!   reported by status only — never with the body, which may echo input.
//! * **JSON in.** Every request asks for `application/json` and refuses an
//!   answer that is not JSON.
//!
//! A host with its own client policy can wrap it with
//! [`ReqwestFetcher::from_client`]; it then owns the redirect and timeout
//! posture itself.

use std::time::Duration;

use async_trait::async_trait;
use kirino::auth::passport::oauth::HttpFetcher;
use serde_json::Value;
use url::Url;

/// The default per-request timeout for identity-provider calls.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

/// A transport that could not be built.
#[derive(Debug, thiserror::Error)]
pub enum ReqwestFetcherError {
    /// The HTTP client could not be constructed.
    #[error("identity transport could not be built: {0}")]
    Build(String),
}

/// [`HttpFetcher`] over `reqwest`.
#[derive(Debug, Clone)]
pub struct ReqwestFetcher {
    client: reqwest::Client,
}

/// Install the ring-based rustls crypto provider exactly once.
///
/// The workspace `reqwest` is built with `rustls-no-provider` (pure-Rust
/// TLS), so no `reqwest::Client` can be constructed until a provider is
/// installed. [`ReqwestFetcher::new`] and [`ReqwestFetcher::with_timeout`]
/// call this themselves; a host that builds its own client for
/// [`ReqwestFetcher::from_client`] must call it first. Another crate in the
/// process may already have installed a provider — any installed provider
/// satisfies reqwest, so that is tolerated.
pub fn ensure_rustls_crypto_provider() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if rustls::crypto::CryptoProvider::get_default().is_some() {
            return;
        }
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

impl ReqwestFetcher {
    /// A client with the family posture: no redirects, [`DEFAULT_TIMEOUT`].
    ///
    /// # Errors
    ///
    /// [`ReqwestFetcherError::Build`] when the TLS stack cannot be
    /// initialised.
    pub fn new() -> Result<Self, ReqwestFetcherError> {
        Self::with_timeout(DEFAULT_TIMEOUT)
    }

    /// A client with the family posture and a custom per-request timeout.
    ///
    /// # Errors
    ///
    /// [`ReqwestFetcherError::Build`] when the TLS stack cannot be
    /// initialised.
    pub fn with_timeout(timeout: Duration) -> Result<Self, ReqwestFetcherError> {
        ensure_rustls_crypto_provider();
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout)
            .build()
            .map_err(|error| ReqwestFetcherError::Build(error.without_url().to_string()))?;
        Ok(Self { client })
    }

    /// Wrap a host-provided client. The host owns its redirect and timeout
    /// posture; see the module docs for why the built-in client refuses
    /// redirects.
    #[must_use]
    pub fn from_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Read the answer as JSON, or report the status without the body.
    async fn json(response: reqwest::Response) -> anyhow::Result<Value> {
        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("identity provider answered {status}");
        }
        response.json().await.map_err(|error| {
            anyhow::anyhow!(
                "identity provider answer unreadable: {}",
                error.without_url()
            )
        })
    }
}

#[async_trait]
impl HttpFetcher for ReqwestFetcher {
    async fn post_form(&self, url: &Url, params: &[(&str, &str)]) -> anyhow::Result<Value> {
        let response = self
            .client
            .post(url.as_str())
            .form(params)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| {
                anyhow::anyhow!("identity provider unreachable: {}", error.without_url())
            })?;
        Self::json(response).await
    }

    async fn post_json(&self, url: &Url, body: &Value) -> anyhow::Result<Value> {
        let response = self
            .client
            .post(url.as_str())
            .json(body)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| {
                anyhow::anyhow!("identity provider unreachable: {}", error.without_url())
            })?;
        Self::json(response).await
    }

    async fn get_bearer(&self, url: &Url, access_token: &str) -> anyhow::Result<Value> {
        let response = self
            .client
            .get(url.as_str())
            .bearer_auth(access_token)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|error| {
                anyhow::anyhow!("identity provider unreachable: {}", error.without_url())
            })?;
        Self::json(response).await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{body_string_contains, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn base(server: &MockServer, route: &str) -> Url {
        Url::parse(&format!("{}{route}", server.uri())).expect("url")
    }

    #[tokio::test]
    async fn the_form_call_sends_exactly_the_fields_and_parses_json() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            // MUT:form-fields — the body must be the form encoding of the given fields.
            .and(body_string_contains("grant_type=authorization_code"))
            .and(body_string_contains("code_verifier=the-verifier"))
            .and(header("Accept", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": "at-1"})))
            .expect(1)
            .mount(&server)
            .await;
        let fetcher = ReqwestFetcher::new().expect("fetcher");
        let answer = fetcher
            .post_form(
                &base(&server, "/oauth/token"),
                &[
                    ("grant_type", "authorization_code"),
                    ("code_verifier", "the-verifier"),
                ],
            )
            .await
            .expect("ok");
        assert_eq!(answer["access_token"], "at-1");
    }

    #[tokio::test]
    async fn the_json_call_sends_the_body_and_the_bearer_call_the_token() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("\"client_secret\":\"s3cr3t\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": "at-2"})))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/oauth/userinfo"))
            // MUT:bearer-header — the access token travels as a Bearer credential.
            .and(header("Authorization", "Bearer at-2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"sub": "sub-1"})))
            .expect(1)
            .mount(&server)
            .await;
        let fetcher = ReqwestFetcher::new().expect("fetcher");
        let token = fetcher
            .post_json(
                &base(&server, "/oauth/token"),
                &json!({"client_secret": "s3cr3t"}),
            )
            .await
            .expect("ok");
        let claims = fetcher
            .get_bearer(
                &base(&server, "/oauth/userinfo"),
                token["access_token"].as_str().expect("token"),
            )
            .await
            .expect("ok");
        assert_eq!(claims["sub"], "sub-1");
    }

    #[tokio::test]
    async fn a_non_2xx_answer_is_reported_by_status_and_never_by_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_string("error: invalid_grant for client s3cr3t-value"),
            )
            .mount(&server)
            .await;
        let fetcher = ReqwestFetcher::new().expect("fetcher");
        let err = fetcher
            .post_form(
                &base(&server, "/oauth/token"),
                &[("grant_type", "authorization_code")],
            )
            .await
            .unwrap_err()
            .to_string();
        // MUT:no-body-in-errors — the body may echo the request's secrets.
        assert_eq!(err, "identity provider answered 400 Bad Request");
    }

    #[tokio::test]
    async fn redirects_are_not_followed() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(ResponseTemplate::new(302).insert_header("Location", "/elsewhere"))
            .mount(&server)
            .await;
        Mock::given(path("/elsewhere"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .expect(0)
            .mount(&server)
            .await;
        let fetcher = ReqwestFetcher::new().expect("fetcher");
        // MUT:no-redirects — a 3xx is an answer, not a hop: the secret never leaves the issuer.
        assert_eq!(
            fetcher
                .post_form(
                    &base(&server, "/oauth/token"),
                    &[("client_secret", "s3cr3t")]
                )
                .await
                .unwrap_err()
                .to_string(),
            "identity provider answered 302 Found"
        );
    }

    #[tokio::test]
    async fn the_core_finishes_a_login_through_this_transport() {
        use kirino::auth::passport::oauth::{
            Callback, CallbackQuery, ClientAuth, InMemoryLoginAttemptStore, ProviderMetadata,
            TokenBody, authorize_url, begin, finish,
        };

        let server = MockServer::start().await;
        let issuer = server.uri();
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=authorization_code"))
            .and(body_string_contains("code_verifier="))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"access_token": "at-9", "token_type": "Bearer"})),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/oauth/userinfo"))
            .and(header("Authorization", "Bearer at-9"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"sub": "sub-9", "iss": issuer, "name": "Ada"})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let meta = ProviderMetadata {
            provider: "example-idp".to_owned(),
            issuer: Some(issuer.clone()),
            authorize_url: base(&server, "/oauth/authorize"),
            token_url: base(&server, "/oauth/token"),
            userinfo_url: Some(base(&server, "/oauth/userinfo")),
            token_body: TokenBody::Form,
            scopes: vec!["openid".to_owned()],
        };
        let client = ClientAuth::Public {
            client_id: "cid".to_owned(),
        };
        let store = InMemoryLoginAttemptStore::new();
        let redirect = Url::parse("https://app.example/api/auth/relay/callback").expect("url");
        let attempt = begin(
            &store,
            "example-idp",
            Some("/inbox".to_owned()),
            chrono::Duration::seconds(60),
        )
        .await
        .expect("begin");
        let authorize = authorize_url(&meta, &client, &redirect, &attempt);
        assert!(
            authorize
                .as_str()
                .starts_with(&format!("{}/oauth/authorize?", server.uri()))
        );

        let query = CallbackQuery {
            code: Some("code-1".to_owned()),
            state: Some(attempt.state.clone()),
            error: None,
            error_description: None,
        };
        let login = finish(
            &ReqwestFetcher::new().expect("fetcher"),
            &store,
            &meta,
            &client,
            &redirect,
            Callback::new(&query, None, &[]),
        )
        .await
        .expect("finish");
        assert_eq!(login.identity.subject, "sub-9");
        assert_eq!(login.profile.display_name.as_deref(), Some("Ada"));
        assert_eq!(login.return_to.as_deref(), Some("/inbox"));
    }
}
