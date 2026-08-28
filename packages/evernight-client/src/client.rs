//! The evernight dispatch client: guard → idempotency → transport → parse →
//! truncate → digest (spec §4.1/§4.2/§4.3, module boundary §5.2).
//!
//! The client is intentionally unaware of OreXis: audit emission happens one
//! layer up in the entelecheia adapter (spec §4.3 hook points), which wraps
//! [`EvernightClient::dispatch`] and records the pre-dispatch / post-exec
//! pair using [`ExecOutput::result_digest`].

use std::fmt;
use std::sync::Mutex;

use plana_jsonrpc::{error_codes, Id, JsonRpcRequest};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::auth::{AUTH_PARAM_KEY, METHOD_COMMAND_EXEC};
use crate::classify::{classify_rpc, classify_transport, ClassifiedError, ErrorKind};
use crate::envelope::DispatchEnvelope;
use crate::guard::{Endpoint, TargetScopeGuard};
use crate::idempotency::IdempotencyWindow;
use crate::transport::{IpcSocketTransport, Transport};

/// Per-stream output cap: 1 MiB (spec §4.1 read-back rule 3 — the adapter
/// must bound output and mark truncation explicitly).
pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

/// Successful execution output, read back as lossy-UTF-8 text (spec §4.1).
///
/// `stdout` / `stderr` are already truncated to [`MAX_OUTPUT_BYTES`] per
/// stream (char-boundary safe); `truncated` is set when either stream was
/// cut. The digest is computed over the truncated, as-held text — exactly
/// what the audit layer receives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecOutput {
    /// Process exit code reported by the broker.
    pub exit_code: i32,
    /// Captured stdout (possibly truncated).
    pub stdout: String,
    /// Captured stderr (possibly truncated).
    pub stderr: String,
    /// `true` when stdout or stderr exceeded the cap and was cut.
    pub truncated: bool,
}

impl ExecOutput {
    /// Post-exec result digest (spec §4.3): lowercase hex
    /// `sha256(stdout_bytes || b"\n" || stderr_bytes || exit_code_decimal_ascii)`
    /// over the held (post-truncation) output.
    pub fn result_digest(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.stdout.as_bytes());
        hasher.update(b"\n");
        hasher.update(self.stderr.as_bytes());
        hasher.update(self.exit_code.to_string().as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// Raw broker result object for `Command.Exec` (`{exit_code, stdout, stderr}`).
#[derive(Debug, Deserialize)]
struct RawExecOutput {
    #[serde(default)]
    exit_code: i32,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
}

/// Char-boundary-safe truncation to `limit` bytes.
///
/// Returns the (possibly shortened) owned string and whether it was cut.
/// When the limit lands inside a multi-byte character the boundary is walked
/// back to the nearest char boundary.
fn truncate_stream(input: &str, limit: usize) -> (String, bool) {
    if input.len() <= limit {
        return (input.to_string(), false);
    }
    let mut end = limit;
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }
    (input[..end].to_string(), true)
}

/// Client configuration.
///
/// The [`fmt::Debug`] projection redacts `token` so a stray log line can
/// never carry the shared secret (spec §3.2: the token never enters logs).
#[derive(Clone)]
pub struct EvernightClientConfig {
    /// Broker endpoint (IPC baseline; TCP reserved for the deferred ws form).
    pub endpoint: Endpoint,
    /// Designated single-node alias the guard enforces (spec §3.1).
    pub designated_alias: String,
    /// Static shared token, injected from the environment by the caller
    /// (convention: [`crate::auth::ENV_TOKEN_VAR`]). Held in memory only —
    /// never persisted, logged, or embedded in the wire params body beyond
    /// the request-scoped `auth` field the broker strips.
    pub token: Option<String>,
}

impl fmt::Debug for EvernightClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EvernightClientConfig")
            .field("endpoint", &self.endpoint)
            .field("designated_alias", &self.designated_alias)
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

impl EvernightClientConfig {
    /// Creates a configuration without a token (anonymous local IPC).
    pub fn new(endpoint: Endpoint, designated_alias: impl Into<String>) -> Self {
        Self {
            endpoint,
            designated_alias: designated_alias.into(),
            token: None,
        }
    }

    /// Sets the static shared token (request-scoped `auth` param).
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }
}

/// Dispatch client for the evernight `Command.Exec` terminal route.
///
/// Generic over [`Transport`] so tests can substitute an in-process stub;
/// production uses [`IpcSocketTransport`] (see [`Self::with_ipc`]).
pub struct EvernightClient<T: Transport> {
    transport: T,
    config: EvernightClientConfig,
    guard: TargetScopeGuard,
    idempotency: Mutex<IdempotencyWindow>,
}

impl<T: Transport> EvernightClient<T> {
    /// Builds a client around an explicit transport.
    pub fn new(transport: T, config: EvernightClientConfig) -> Self {
        let guard = TargetScopeGuard::new(config.designated_alias.clone());
        Self {
            transport,
            config,
            guard,
            idempotency: Mutex::new(IdempotencyWindow::new()),
        }
    }

    /// The client configuration.
    pub fn config(&self) -> &EvernightClientConfig {
        &self.config
    }

    /// Dispatches one terminal command execution (spec §5.3 sequence).
    ///
    /// Flow: guard.validate → idempotency `check_insert` (duplicate →
    /// [`ErrorKind::DuplicateRequest`] with **no** transport call and no
    /// duplicate audit) → `Command.Exec` request built from
    /// [`DispatchEnvelope::wire_params`] (request-scoped `auth` param
    /// attached when a token is configured) → transport → classify →
    /// parse `{exit_code, stdout, stderr}` → truncate → digest.
    ///
    /// The envelope's `request_id` doubles as the JSON-RPC id so audit
    /// records correlate with the wire message end to end.
    pub async fn dispatch(
        &self,
        envelope: DispatchEnvelope,
    ) -> Result<ExecOutput, ClassifiedError> {
        // 1. Target-scope gate (spec §4.2: reject before any byte is sent).
        self.guard.validate(&envelope.target_scope)?;

        // 2. Idempotency gate: duplicates never reach the broker.
        let duplicate = {
            let mut window = self
                .idempotency
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            !window.check_insert(&envelope.request_id)
        };
        if duplicate {
            return Err(ClassifiedError::new(
                ErrorKind::DuplicateRequest,
                error_codes::INVALID_REQUEST,
                format!(
                    "duplicate request_id {}: an identical dispatch was already recorded",
                    envelope.request_id
                ),
            )
            .with_reason("duplicate_request"));
        }

        // 3. Build the wire request: envelope → params (+ request-scoped
        //    auth field when configured). The broker strips `auth`.
        let mut params = envelope.wire_params();
        if let Some(token) = self.config.token.as_deref() {
            if let Some(object) = params.as_object_mut() {
                object.insert(AUTH_PARAM_KEY.to_string(), Value::String(token.to_string()));
            }
        }
        let request = JsonRpcRequest::new_raw(METHOD_COMMAND_EXEC, Some(params))
            .with_id(Id::String(envelope.request_id.clone()));

        // 4. Exchange and classify (spec §4.2 mapping).
        let response = self
            .transport
            .call(request)
            .await
            .map_err(classify_transport)?;
        if let Some(error) = response.error {
            return Err(classify_rpc(error));
        }
        let result = response.result.ok_or_else(|| {
            ClassifiedError::new(
                ErrorKind::Other,
                error_codes::INTERNAL_ERROR,
                "evernight response carried neither result nor error",
            )
        })?;

        // 5. Parse the three-field result object.
        let raw: RawExecOutput = serde_json::from_value(result).map_err(|e| {
            ClassifiedError::new(
                ErrorKind::Other,
                error_codes::INTERNAL_ERROR,
                format!("evernight exec result did not match the contract shape: {e}"),
            )
        })?;

        // 6. Bound both streams and mark truncation (spec §4.1 rule 3).
        let (stdout, stdout_cut) = truncate_stream(&raw.stdout, MAX_OUTPUT_BYTES);
        let (stderr, stderr_cut) = truncate_stream(&raw.stderr, MAX_OUTPUT_BYTES);
        Ok(ExecOutput {
            exit_code: raw.exit_code,
            stdout,
            stderr,
            truncated: stdout_cut || stderr_cut,
        })
    }
}

impl EvernightClient<IpcSocketTransport> {
    /// Builds the production IPC client from the endpoint configuration.
    ///
    /// Fails with [`ErrorKind::Other`] when the configured endpoint is
    /// [`Endpoint::Tcp`] — the remote form needs the deferred ws transport.
    pub fn with_ipc(config: EvernightClientConfig) -> Result<Self, ClassifiedError> {
        match &config.endpoint {
            Endpoint::Ipc(socket_path) => Ok(Self::new(
                IpcSocketTransport::new(socket_path.clone()),
                config,
            )),
            Endpoint::Tcp { .. } => Err(ClassifiedError::new(
                ErrorKind::Other,
                error_codes::INTERNAL_ERROR,
                "remote tcp endpoints require the deferred ws transport; use Endpoint::Ipc",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::ErrorKind;
    use crate::envelope::TargetScope;
    use crate::transport::TransportError;
    use plana_jsonrpc::JsonRpcResponse;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn scope(alias: &str) -> TargetScope {
        TargetScope::DesignatedSingleNode {
            alias: alias.to_string(),
        }
    }

    fn envelope(request_id: &str) -> DispatchEnvelope {
        DispatchEnvelope {
            request_id: request_id.to_string(),
            origin_agent: "PoleMos".to_string(),
            target_scope: scope("hydro-lab"),
            command: "uname -a".to_string(),
            cwd: Some("/workspace".to_string()),
            timeout_secs: 60,
        }
    }

    fn success_response(result: Value) -> JsonRpcResponse {
        JsonRpcResponse::success(Id::String("unused".to_string()), result)
    }

    /// Records every call and the last observed params; answers canned success.
    struct RecordingTransport {
        calls: AtomicUsize,
        last_params: Mutex<Option<Value>>,
        last_id: Mutex<Option<Id>>,
        canned: Value,
    }

    impl RecordingTransport {
        fn new(canned: Value) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                last_params: Mutex::new(None),
                last_id: Mutex::new(None),
                canned,
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }

        fn last_params(&self) -> Option<Value> {
            self.last_params.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl Transport for RecordingTransport {
        async fn call(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.last_params.lock().unwrap() = req.params.clone();
            *self.last_id.lock().unwrap() = req.id.clone();
            Ok(success_response(self.canned.clone()))
        }
    }

    /// Always fails with the given transport error (peer-unreachable probe).
    struct FailingTransport(TransportError);

    #[async_trait::async_trait]
    impl Transport for FailingTransport {
        async fn call(&self, _req: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError> {
            Err(self.0.clone())
        }
    }

    fn client_with(transport: RecordingTransport) -> EvernightClient<RecordingTransport> {
        EvernightClient::new(
            transport,
            EvernightClientConfig::new(Endpoint::Ipc("/tmp/broker.sock".to_string()), "hydro-lab"),
        )
    }

    #[test]
    fn digest_matches_the_spec_formula() {
        // §4.3: sha256(stdout || "\n" || stderr || exit_code decimal ASCII).
        // Fixed vector: "hi\n" || "\n" || "" || "0" → sha256 over b"hi\n\n0".
        let output = ExecOutput {
            exit_code: 0,
            stdout: "hi\n".to_string(),
            stderr: String::new(),
            truncated: false,
        };
        let mut expected = Sha256::new();
        expected.update(b"hi\n\n0");
        assert_eq!(output.result_digest(), hex::encode(expected.finalize()));

        // Second vector with unicode stderr and a non-zero exit code.
        let output = ExecOutput {
            exit_code: 2,
            stdout: "oki".to_string(),
            stderr: "erré".to_string(),
            truncated: false,
        };
        let mut expected = Sha256::new();
        expected.update(b"oki");
        expected.update(b"\n");
        expected.update("erré".as_bytes());
        expected.update(b"2");
        assert_eq!(output.result_digest(), hex::encode(expected.finalize()));
    }

    #[test]
    fn ascii_truncation_cuts_at_the_cap_and_marks() {
        let big = "a".repeat(MAX_OUTPUT_BYTES + 4242);
        let (cut, truncated) = truncate_stream(&big, MAX_OUTPUT_BYTES);
        assert!(truncated);
        assert_eq!(cut.len(), MAX_OUTPUT_BYTES);
        // Under the cap: untouched, not marked.
        let small = "a".repeat(MAX_OUTPUT_BYTES);
        let (kept, truncated) = truncate_stream(&small, MAX_OUTPUT_BYTES);
        assert!(!truncated);
        assert_eq!(kept, small);
    }

    #[test]
    fn multibyte_truncation_never_splits_a_character() {
        // '日' is 3 bytes; 350_000 chars = 1_050_000 bytes > 1 MiB. The raw
        // limit lands one byte into a character and must be walked back.
        let big = "日".repeat(350_000);
        assert!(big.len() > MAX_OUTPUT_BYTES);
        let (cut, truncated) = truncate_stream(&big, MAX_OUTPUT_BYTES);
        assert!(truncated);
        assert_eq!(cut.chars().count() * 3, cut.len(), "must hold whole chars");
        assert!(cut.len() <= MAX_OUTPUT_BYTES);
        assert!(cut.ends_with('日'));
    }

    #[tokio::test]
    async fn dispatch_parses_exec_output_and_correlates_the_id() {
        let transport = RecordingTransport::new(json!({
            "exit_code": 0,
            "stdout": "Linux broker 6.1\n",
            "stderr": ""
        }));
        let client = client_with(transport);
        let envelope = envelope("req-1");
        let expected_id = envelope.request_id.clone();
        let output = client.dispatch(envelope).await.unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, "Linux broker 6.1\n");
        assert_eq!(output.stderr, "");
        assert!(!output.truncated);
        // The envelope request_id rides the wire as the JSON-RPC id.
        assert_eq!(
            client.transport.last_id.lock().unwrap().clone(),
            Some(Id::String(expected_id))
        );
    }

    #[tokio::test]
    async fn dispatch_without_token_sends_no_auth_field() {
        let transport = RecordingTransport::new(json!({
            "exit_code": 0,
            "stdout": "",
            "stderr": ""
        }));
        let client = client_with(transport);
        client.dispatch(envelope("req-2")).await.unwrap();
        let params = client.transport.last_params().unwrap();
        assert!(
            params.get("auth").is_none(),
            "no token configured, no auth field"
        );
        assert_eq!(params["command"], "uname -a");
        assert_eq!(params["cwd"], "/workspace");
        assert_eq!(params["timeout"], 60);
    }

    #[tokio::test]
    async fn dispatch_with_token_attaches_request_scoped_auth() {
        let transport = RecordingTransport::new(json!({
            "exit_code": 0,
            "stdout": "",
            "stderr": ""
        }));
        let client = EvernightClient::new(
            transport,
            EvernightClientConfig::new(Endpoint::Ipc("/tmp/broker.sock".to_string()), "hydro-lab")
                .with_token("<your-evernight-token>"),
        );
        client.dispatch(envelope("req-3")).await.unwrap();
        let params = client.transport.last_params().unwrap();
        assert_eq!(params["auth"], "<your-evernight-token>");
        // The auth field is additive; the wire subset stays intact.
        assert_eq!(params["command"], "uname -a");
        assert_eq!(params["timeout"], 60);
        // Token stays in the in-memory config only.
        assert_eq!(
            client.config().token.as_deref(),
            Some("<your-evernight-token>")
        );
    }

    #[tokio::test]
    async fn duplicate_request_id_makes_zero_transport_calls() {
        let transport = RecordingTransport::new(json!({
            "exit_code": 0,
            "stdout": "once",
            "stderr": ""
        }));
        let client = client_with(transport);
        let first = client.dispatch(envelope("dup-1")).await.unwrap();
        assert_eq!(first.stdout, "once");

        let second = client.dispatch(envelope("dup-1")).await;
        let error = second.expect_err("duplicate must be rejected");
        assert_eq!(error.kind, ErrorKind::DuplicateRequest);
        assert_eq!(error.data, Some(json!({"reason": "duplicate_request"})));
        // Exactly one broker exchange happened; the duplicate was stopped
        // locally (no second call, no duplicated audit feed).
        assert_eq!(client.transport.calls(), 1);
    }

    #[tokio::test]
    async fn out_of_scope_target_is_rejected_before_transport() {
        let transport = RecordingTransport::new(json!({
            "exit_code": 0,
            "stdout": "",
            "stderr": ""
        }));
        let client = client_with(transport);
        let mut envelope = envelope("scope-1");
        envelope.target_scope = scope("elsewhere");
        let error = client.dispatch(envelope).await.expect_err("must reject");
        assert_eq!(error.kind, ErrorKind::TargetOutOfScope);
        assert_eq!(error.code, -32005);
        assert_eq!(error.data, Some(json!({"reason": "target_out_of_scope"})));
        assert_eq!(client.transport.calls(), 0);
    }

    #[tokio::test]
    async fn transport_failure_classifies_as_peer_unreachable() {
        let client = EvernightClient::new(
            FailingTransport(TransportError::Connect("connection refused".to_string())),
            EvernightClientConfig::new(Endpoint::Ipc("/tmp/broker.sock".to_string()), "hydro-lab"),
        );
        let error = client
            .dispatch(envelope("down-1"))
            .await
            .expect_err("must fail");
        assert_eq!(error.kind, ErrorKind::PeerUnreachable);
        assert_eq!(error.code, -32603);
        assert_eq!(error.data, Some(json!({"reason": "peer_unreachable"})));
    }

    #[tokio::test]
    async fn broker_error_response_is_classified() {
        struct ErroringTransport;

        #[async_trait::async_trait]
        impl Transport for ErroringTransport {
            async fn call(&self, _req: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError> {
                Ok(JsonRpcResponse::error(
                    Id::String("x".to_string()),
                    plana_jsonrpc::JsonRpcError::internal_error("Command timed out after 60s")
                        .with_data(json!({"timeout": true})),
                ))
            }
        }

        let client = EvernightClient::new(
            ErroringTransport,
            EvernightClientConfig::new(Endpoint::Ipc("/tmp/broker.sock".to_string()), "hydro-lab"),
        );
        let error = client
            .dispatch(envelope("timeout-1"))
            .await
            .expect_err("must fail");
        assert_eq!(error.kind, ErrorKind::ExecutionTimeout);
        assert_eq!(error.code, -32603);
    }

    #[tokio::test]
    async fn oversize_output_is_truncated_and_marked() {
        let big_stdout = "a".repeat(MAX_OUTPUT_BYTES + 1);
        let transport = RecordingTransport::new(json!({
            "exit_code": 0,
            "stdout": big_stdout,
            "stderr": ""
        }));
        let client = client_with(transport);
        let output = client.dispatch(envelope("trunc-1")).await.unwrap();
        assert!(output.truncated);
        assert_eq!(output.stdout.len(), MAX_OUTPUT_BYTES);
        // Digest is computed over the held (truncated) text.
        let mut expected = Sha256::new();
        expected.update(output.stdout.as_bytes());
        expected.update(b"\n");
        expected.update(b"");
        expected.update(b"0");
        assert_eq!(output.result_digest(), hex::encode(expected.finalize()));
    }

    #[tokio::test]
    async fn malformed_result_classifies_as_other() {
        let transport = RecordingTransport::new(json!({"exit_code": "not-a-number"}));
        let client = client_with(transport);
        let error = client
            .dispatch(envelope("bad-1"))
            .await
            .expect_err("must fail");
        assert_eq!(error.kind, ErrorKind::Other);
        assert_eq!(error.code, -32603);
        assert!(error.message.contains("contract shape"));
    }

    #[test]
    fn with_ipc_rejects_tcp_endpoints() {
        let config = EvernightClientConfig::new(
            Endpoint::Tcp {
                host: "192.0.2.10".to_string(),
                port: 7777,
            },
            "hydro-lab",
        );
        let error = EvernightClient::with_ipc(config)
            .err()
            .expect("tcp is deferred");
        assert_eq!(error.kind, ErrorKind::Other);
    }

    #[test]
    fn config_debug_never_leaks_the_token() {
        let config =
            EvernightClientConfig::new(Endpoint::Ipc("/tmp/broker.sock".to_string()), "hydro-lab")
                .with_token("<your-evernight-token>");
        let rendered = format!("{config:?}");
        assert!(rendered.contains("<redacted>"), "got: {rendered}");
        assert!(
            !rendered.contains("<your-evernight-token>"),
            "got: {rendered}"
        );
    }

    /// In-process stub broker: one UnixListener, one canned response.
    /// Answers a single newline-delimited JSON-RPC response whose id echoes
    /// the request id, then stops.
    #[cfg(unix)]
    #[tokio::test]
    async fn ipc_round_trip_against_a_stub_broker() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("broker.sock");
        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request: Value = serde_json::from_str(line.trim()).unwrap();

            // Contract assertions on what actually crossed the socket.
            assert_eq!(request["jsonrpc"], "2.0");
            assert_eq!(request["method"], "Command.Exec");
            let params = request["params"].as_object().unwrap();
            assert_eq!(params.len(), 3, "exactly command/cwd/timeout on the wire");
            assert_eq!(params["command"], "uname -a");
            assert_eq!(params["cwd"], "/workspace");
            assert_eq!(params["timeout"], 60);
            assert!(params.get("auth").is_none());

            let response = json!({
                "jsonrpc": "2.0",
                "id": request["id"].clone(),
                "result": {
                    "exit_code": 0,
                    "stdout": "Linux hydro-lab 6.1.0\n",
                    "stderr": ""
                }
            });
            reader
                .get_mut()
                .write_all(format!("{response}\n").as_bytes())
                .await
                .unwrap();
        });

        let transport = IpcSocketTransport::new(socket_path.to_string_lossy().to_string());
        let client = EvernightClient::new(
            transport,
            EvernightClientConfig::new(
                Endpoint::Ipc(socket_path.to_string_lossy().to_string()),
                "hydro-lab",
            ),
        );
        let output = client.dispatch(envelope("ipc-1")).await.unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout, "Linux hydro-lab 6.1.0\n");
        assert_eq!(output.stderr, "");
        assert!(!output.truncated);
        assert_eq!(output.result_digest().len(), 64);
        server.await.unwrap();
    }

    /// A broker error travelling over a real socket must classify through the
    /// same taxonomy as the in-process path.
    #[cfg(unix)]
    #[tokio::test]
    async fn ipc_round_trip_surfaces_broker_auth_error() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("broker.sock");
        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request: Value = serde_json::from_str(line.trim()).unwrap();
            let response = json!({
                "jsonrpc": "2.0",
                "id": request["id"].clone(),
                "error": {
                    "code": -32005,
                    "message": "static token rejected"
                }
            });
            reader
                .get_mut()
                .write_all(format!("{response}\n").as_bytes())
                .await
                .unwrap();
        });

        let transport = IpcSocketTransport::new(socket_path.to_string_lossy().to_string());
        let client = EvernightClient::new(
            transport,
            EvernightClientConfig::new(
                Endpoint::Ipc(socket_path.to_string_lossy().to_string()),
                "hydro-lab",
            ),
        );
        let error = client
            .dispatch(envelope("ipc-2"))
            .await
            .expect_err("auth error must propagate");
        assert_eq!(error.kind, ErrorKind::Auth);
        assert_eq!(error.code, -32005);
        server.await.unwrap();
    }
}
