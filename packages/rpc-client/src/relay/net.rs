//! Unified network context for download lanes — the outbound half of
//! the relay's proxy policy, generalized for any tool that fetches
//! artifacts (the chest container lane's gateway downloads, model
//! fetchers, installer shims).
//!
//! Two pieces, deliberately small:
//!
//! * `download_client` — a reqwest client built **from a
//!   `ProxyDecision`** so every desktop dialer consumes the same
//!   explicit traffic policy: rustls TLS, https-only redirects, bounded
//!   connect/read waits, proxy applied verbatim. `Direct` disables
//!   ambient `HTTP(S)_PROXY` exactly like the WebSocket dialers — a
//!   factory tool's traffic policy is explicit configuration, never
//!   ambient environment.
//! * `mirror_ladder` — the ordered candidate-URL ladder a resilient
//!   download walks (the WoWSP ordering: the user's own mirror first,
//!   the source second, the manifest's region-tagged mirrors after).
//!   Pure string work; host matching stays with the caller.
//!
//! Both halves are the *join* the artifact lane was missing: proxy.rs
//! resolved decisions for dialers, the config crate keeps a mirror
//! table for embeddings, but nothing offered a shared "fetch this file
//! from a ladder, through this proxy" context. This module is that
//! context, host-agnostic on purpose (chest feeds it the gateway
//! manifest's mirrors; the builtin embedding mirror table is
//! untouched).

use std::time::Duration;

use super::proxy::{ProxyConfig, ProxyDecision};

/// Bounded TCP+TLS handshake wait. A download lane would rather fail
/// over to the next ladder entry than sit on a half-open connection.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Bounded **gap between reads**, not a total-request cap: a multi-GB
/// body legitimately streams for minutes, so a whole-request timeout
/// would abort healthy transfers. Byte budgets stay with the caller
/// (the lane knows the manifest's declared size).
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Redirect ceiling, matching reqwest's own default — the policy below
/// is custom, so the stock cap does not apply and must be re-stated.
const MAX_REDIRECTS: usize = 10;

/// Build the download client a fetch lane dials through.
///
/// * **TLS** rides the workspace's rustls/ring line (no native TLS, no
///   aws-lc). reqwest refuses to even BUILD a client without a
///   process-wide CryptoProvider, so this installs the
///   workspace-standard ring provider first (idempotent — an
///   already-installed provider, ring or otherwise, stays).
/// * **Redirects** follow only while every hop target stays `https`;
///   a downgrade hop (to `http` or any other scheme) is an error, not
///   a follow — mirrors and CDNs redirect liberally, and a silent
///   plaintext hop would strip the lane's transport guarantees.
/// * **Proxy** comes from the decision: `Through` applies the proxy
///   verbatim (credentials included, `socks5` supported); `Direct`
///   calls `no_proxy` so ambient `HTTP(S)_PROXY` never leaks into a
///   lane that was configured explicit-direct.
///
/// Callers that want the layered env chain resolve it first:
/// `download_client(&resolve(Some(&desktop_proxy)))` — see
/// `relay::proxy::resolve`.
///
/// # Panics
///
/// Panics when the decision's proxy URL cannot parse (a malformed
/// `host`/`scheme`). Config errors must fail loudly; silently
/// falling back to direct would send traffic around the configured
/// proxy — the one behavior a traffic policy must never exhibit.
pub fn download_client(decision: &ProxyDecision) -> reqwest::Client {
    ensure_ring_provider();
    let mut builder = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .redirect(https_only_redirect_policy());
    match decision {
        ProxyDecision::Direct => builder = builder.no_proxy(),
        ProxyDecision::Through(config) => builder = builder.proxy(for_reqwest(config)),
    }
    builder
        .build()
        .expect("a validated proxy decision builds a reqwest client")
}

/// Install the workspace-standard ring CryptoProvider as the process
/// default if none is present yet. reqwest (built with
/// `rustls-no-provider`) panics at client BUILD time otherwise, so a
/// fetch lane must not depend on some other dialer having installed
/// one first. Idempotent by construction: an already-installed
/// provider (ring or not) stays untouched.
fn ensure_ring_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Translate one `ProxyConfig` into reqwest's proxy type.
fn for_reqwest(config: &ProxyConfig) -> reqwest::Proxy {
    let proxy = reqwest::Proxy::all(config.url()).unwrap_or_else(|error| {
        panic!(
            "the configured proxy URL '{}' does not parse: {}",
            config.url(),
            error
        )
    });
    match &config.username {
        // Basic auth has no "absent password" form — an unset password
        // is the empty one (base64 of "user:").
        Some(username) => proxy.basic_auth(username, config.password.as_deref().unwrap_or("")),
        None => proxy,
    }
}

/// Redirect policy: follow https hops only, up to `MAX_REDIRECTS`.
fn https_only_redirect_policy() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS {
            return attempt.error("redirect limit reached");
        }
        if attempt.url().scheme() != "https" {
            return attempt.error("refusing to follow a non-https redirect");
        }
        attempt.follow()
    })
}

/// The ordered candidate-URL ladder a resilient download walks: the
/// user's mirror first, the source URL second, then the manifest's
/// mirrors in the order given — the WoWSP ordering (`user → source →
/// mirrors`).
///
/// * `mirrors` is the manifest's list verbatim: `(prefix, regions)`
///   pairs where `prefix` is a URL prefix ending in `/` and
///   `regions` names the regions it serves (`["cn"]`, `["intl"]`).
///   Each surviving mirror contributes `prefix + url` — the prefix is
///   prepended **verbatim** to the source URL, the manifest's
///   documented composition (the gh-proxy shape:
///   `https://gh-proxy.com/` + `https://github.com/…`).
/// * A mirror whose `regions` is empty is **dead configuration** —
///   the gateway's own validation rejects it as "a mirror no region
///   resolves to" — and is skipped, not dialed.
/// * `user_mirror` uses the same prefix convention and outranks
///   everything: an explicit user choice beats both the source and any
///   manifest mirror. `None` (or a blank string) simply drops the
///   rung.
///
/// The function is pure on purpose: it does not resolve the client's
/// region (callers pre-filter the list they pass when they need to)
/// and does not match hosts (foreign-host no-op behavior is the
/// caller's concern — this ladder builder never inspects the URL).
pub fn mirror_ladder(
    url: &str,
    mirrors: &[(&str, &[&str])],
    user_mirror: Option<&str>,
) -> Vec<String> {
    let user_mirror = user_mirror
        .map(str::trim)
        .filter(|prefix| !prefix.is_empty());
    let mut ladder = Vec::with_capacity(mirrors.len() + 1 + usize::from(user_mirror.is_some()));
    if let Some(prefix) = user_mirror {
        ladder.push(format!("{prefix}{url}"));
    }
    ladder.push(url.to_string());
    for (prefix, regions) in mirrors {
        if regions.is_empty() {
            continue;
        }
        ladder.push(format!("{prefix}{url}"));
    }
    ladder
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── mirror_ladder ────────────────────────────────────────────────────

    /// The manifest shape chest feeds: (prefix, regions) pairs, straight
    /// from the gateway's nine-key manifest.
    const MANIFEST_MIRRORS: &[(&str, &[&str])] = &[
        ("https://cn.example.test/", &["cn"]),
        ("https://intl.example.test/", &["intl"]),
        ("https://dead.example.test/", &[]),
    ];

    const SOURCE: &str = "https://gateway.example.test/downloads/arona-image/latest";

    #[test]
    fn ladder_orders_user_source_then_mirrors() {
        let ladder = mirror_ladder(SOURCE, MANIFEST_MIRRORS, Some("https://mine.example.test/"));
        assert_eq!(
            ladder,
            vec![
                // The user's own mirror outranks everything (WoWSP order).
                format!("https://mine.example.test/{}", SOURCE),
                // The source itself is tried before any public mirror.
                SOURCE.to_string(),
                // Manifest mirrors follow, in the order the manifest gave.
                format!("https://cn.example.test/{}", SOURCE),
                format!("https://intl.example.test/{}", SOURCE),
            ]
        );
    }

    #[test]
    fn ladder_skips_dead_mirrors_and_blank_user_rungs() {
        // The empty-regions mirror is dead configuration — it must not
        // appear anywhere in the ladder (region filtering).
        let ladder = mirror_ladder(SOURCE, MANIFEST_MIRRORS, None);
        assert_eq!(
            ladder,
            vec![
                SOURCE.to_string(),
                format!("https://cn.example.test/{}", SOURCE),
                format!("https://intl.example.test/{}", SOURCE),
            ]
        );
        // A blank user mirror is not a rung at all.
        assert_eq!(
            mirror_ladder(SOURCE, &[], Some("   ")),
            vec![SOURCE.to_string()]
        );
        // No user mirror, no manifest mirrors: the source alone.
        assert_eq!(mirror_ladder(SOURCE, &[], None), vec![SOURCE.to_string()]);
    }

    #[test]
    fn ladder_is_pure_string_work_for_any_url_shape() {
        // Host-agnostic by contract: a gh-proxy-shaped URL composes the
        // same way as the gateway URL, and no host ever gets filtered
        // out (that is the caller's concern).
        let github = "https://github.com/celestia-island/arona/releases/download/v1/arona.tar.gz";
        assert_eq!(
            mirror_ladder(github, &[("https://gh-proxy.com/", &["cn"])], None),
            vec![
                github.to_string(),
                format!("https://gh-proxy.com/{}", github),
            ]
        );
    }

    // ── download_client: proxy decision ──────────────────────────────────

    /// What one plain-HTTP request through a forward proxy looks like:
    /// an absolute-form request line plus (when configured) the
    /// Proxy-Authorization header.
    #[tokio::test]
    async fn a_through_decision_dials_the_proxy_with_credentials() {
        use tokio::io::AsyncWriteExt;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let port = listener.local_addr().expect("addr").port();

        let serve = async {
            let (mut socket, _) = listener.accept().await.expect("accept");
            let raw = read_one_request(&mut socket).await;
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                .await
                .expect("respond");
            raw
        };

        let decision = ProxyDecision::Through(ProxyConfig {
            scheme: "http".into(),
            host: format!("127.0.0.1:{}", port),
            username: Some("factory".into()),
            password: Some("hunter2".into()),
        });
        let client = download_client(&decision);
        let (proxy_saw, answer) = tokio::join!(serve, async {
            client
                .get("http://target.example.test/file")
                .send()
                .await
                .expect("the proxy answers")
                .text()
                .await
                .expect("body")
        });

        // Absolute form on the wire proves the request went THROUGH the
        // proxy rather than to the origin; the credentials must ride
        // the proxy hop (base64("factory:hunter2")). Header names are
        // matched case-insensitively — hyper writes them lowercase.
        assert!(
            proxy_saw.starts_with("GET http://target.example.test/file HTTP/1.1"),
            "the proxy saw: {}",
            proxy_saw
        );
        let wire = proxy_saw.to_ascii_lowercase();
        let credentials_riding = wire.contains("proxy-authorization: basic ")
            && proxy_saw.contains("ZmFjdG9yeTpodW50ZXIy");
        assert!(
            credentials_riding,
            "credentials must ride the proxy hop: {}",
            proxy_saw
        );
        assert_eq!(answer, "ok");
    }

    /// Serializes ambient-env mutation: reqwest snapshots the proxy
    /// environment at client build time, and no other test in this
    /// crate builds a reqwest client.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// `Direct` is explicit-direct: even with `HTTP_PROXY` pointing
    /// at a live listener, the lane must not leak through it.
    #[tokio::test]
    async fn a_direct_decision_ignores_ambient_proxy_env() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let port = listener.local_addr().expect("addr").port();

        // reqwest snapshots the env at build time, so the mutation can
        // be scoped to the build itself (env changes are process-local:
        // the host shell is unaffected).
        let client = {
            let _guard = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
            std::env::set_var("HTTP_PROXY", format!("http://127.0.0.1:{}", port));
            std::env::set_var("http_proxy", format!("http://127.0.0.1:{}", port));
            let client = download_client(&ProxyDecision::Direct);
            std::env::remove_var("HTTP_PROXY");
            std::env::remove_var("http_proxy");
            client
        };

        let attempt = async { client.get("http://target.example.test/file").send().await };
        let probe = async {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};

            // Anything at all arriving on the listener is the leak.
            let (mut socket, _) = listener.accept().await.expect("accept");
            let mut chunk = [0u8; 128];
            let _ = socket.read(&mut chunk).await;
            let _ = socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                .await;
        };
        let (outcome, leaked) =
            tokio::join!(attempt, tokio::time::timeout(Duration::from_secs(2), probe));

        assert!(leaked.is_err(), "direct must not touch the ambient proxy");
        assert!(
            outcome.is_err(),
            "the unreachable origin fails the request instead of proxying it"
        );
    }

    // ── download_client: redirect policy ─────────────────────────────────

    /// One 302 whose target is plaintext http: the policy must refuse
    /// the hop (a loud error), not follow it and not silently return
    /// the redirect response as if it were the artifact.
    #[tokio::test]
    async fn a_non_https_redirect_is_refused_not_followed() {
        use tokio::io::AsyncWriteExt;

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let port = listener.local_addr().expect("addr").port();

        let serve = async {
            let (mut socket, _) = listener.accept().await.expect("accept");
            let saw_request = read_one_request(&mut socket).await.len();
            socket
                .write_all(
                    b"HTTP/1.1 302 Found\r\nLocation: http://downgrade.example.test/next\r\nContent-Length: 0\r\n\r\n",
                )
                .await
                .expect("respond");
            saw_request
        };

        let client = download_client(&ProxyDecision::Direct);
        let (saw_request, outcome) = tokio::join!(serve, async {
            client
                .get(format!("http://127.0.0.1:{}/artifact", port))
                .send()
                .await
        });

        // The origin was reached (the refusal happened at the redirect,
        // not before the request) and the downgrade target was never
        // dialed. reqwest's Display does not carry the policy message,
        // so the assertion walks the source chain where it lives.
        assert!(saw_request > 0, "the origin saw the first request");
        let error = outcome.expect_err("a plaintext redirect hop must fail");
        assert!(
            error.is_redirect(),
            "must be a redirect-policy failure: {error}"
        );
        let mut chain = error.to_string();
        let mut source = std::error::Error::source(&error);
        while let Some(cause) = source {
            chain.push_str(" <- ");
            chain.push_str(&cause.to_string());
            source = cause.source();
        }
        assert!(
            chain.contains("non-https redirect"),
            "the error must name the https-only policy: {chain}"
        );
    }

    /// Read one HTTP request off the socket (up to the header
    /// terminator, or EOF when the peer hangs up first).
    async fn read_one_request(socket: &mut tokio::net::TcpStream) -> String {
        use tokio::io::AsyncReadExt;

        let mut raw = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            let n = socket.read(&mut chunk).await.expect("read");
            raw.extend_from_slice(&chunk[..n]);
            if n == 0 || raw.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
        }
        String::from_utf8_lossy(&raw).to_string()
    }
}
