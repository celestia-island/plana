//! `tls` feature internals: the rustls connector `wss://` dials through.
//!
//! Default dials need no custom connector — tokio-tungstenite builds one
//! from the webpki root store with full verification, which is the safe
//! default this crate ships. Only the explicit
//! [`RpcClientBuilder::danger_accept_invalid_certs`](crate::RpcClientBuilder::danger_accept_invalid_certs)
//! test hook reaches this module.
//!
//! [`RpcClientBuilder::install_ring_tls_provider`](crate::RpcClientBuilder::install_ring_tls_provider)
//! installs the process-wide CryptoProvider the *default* connector needs;
//! the danger connector below pins the ring provider explicitly so the hook
//! behaves the same whether or not that was called.

use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::DigitallySignedStruct;
use tokio_tungstenite::Connector;

/// Build the TLS connector for a `wss://` dial.
///
/// `None` (the default) keeps tokio-tungstenite's upstream connector:
/// webpki roots, full certificate and server-name verification. The danger
/// hook swaps in a connector whose certificate verifier accepts anything.
pub(crate) fn connector(danger_accept_invalid_certs: bool) -> Option<Connector> {
    if !danger_accept_invalid_certs {
        return None;
    }
    let config = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("the ring provider supports the safe default protocol versions")
    .dangerous()
    .with_custom_certificate_verifier(Arc::new(AcceptAnyServerCert))
    .with_no_client_auth();
    Some(Connector::Rustls(Arc::new(config)))
}

/// A [`ServerCertVerifier`] that accepts any certificate, chain, or
/// hostname — the definition of the test hook that installs it.
#[derive(Debug)]
struct AcceptAnyServerCert;

impl ServerCertVerifier for AcceptAnyServerCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        // The danger verifier asserts signatures valid without inspecting
        // them, but the scheme list must still intersect what rustls
        // negotiates or the handshake never gets that far. These are the
        // schemes the ring provider can produce or consume.
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
        ]
    }
}
