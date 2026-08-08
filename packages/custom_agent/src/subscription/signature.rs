//! Ed25519 signature verification for subscribed Layer-3 agents.
//!
//! Every subscription from a third-party source must carry a detached
//! Ed25519 signature over its `agent.toml` (`agent.toml.sig`, base64). The
//! verifying public key comes from the built-in trusted-source registry; a
//! source without a registered key cannot be verified and is rejected.
//!
//! Local agents (loaded from `.amphoreus/` or the local agents directory) are
//! exempt: they never pass through the subscription path.

use anyhow::{Context, Result, anyhow};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Verifies a detached base64 signature over `agent.toml` bytes.
///
/// Pure function, unit-testable without touching the registry.
pub fn verify_detached(manifest_bytes: &[u8], sig_b64: &str, pubkey_b64: &str) -> Result<()> {
    let pubkey_bytes: [u8; 32] = decode_b64(pubkey_b64)
        .context("public key is not valid base64")?
        .try_into()
        .map_err(|_| anyhow!("public key must be 32 bytes"))?;
    let pubkey =
        VerifyingKey::from_bytes(&pubkey_bytes).context("public key is not a valid Ed25519 key")?;

    let sig = Signature::from_slice(&decode_b64(sig_b64).context("signature is not valid base64")?)
        .context("signature is not a valid Ed25519 signature")?;

    pubkey
        .verify(manifest_bytes, &sig)
        .map_err(|_| anyhow!("agent.toml signature verification failed"))
}

/// Looks up the registered Ed25519 public key for a trusted source.
///
/// Sources in `SubscribeSettings.trusted_sources` are allowed to subscribe,
/// but only sources with a registered key can actually be verified;
/// unverifiable sources are rejected when `verify_signature` is enabled.
///
/// The registry is populated out-of-band (e.g. via
/// `CELESTIA_AGENT_SIGNING_PUBKEY_<SOURCE>` env vars) until a config file
/// surface is introduced.
pub fn pubkey_for(source: &str) -> Option<String> {
    std::env::var(format!(
        "CELESTIA_AGENT_SIGNING_PUBKEY_{}",
        source.to_uppercase()
    ))
    .ok()
    .filter(|s| !s.is_empty())
}

fn decode_b64(s: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s.trim())
        .map_err(|e| anyhow!("base64 decode failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn keypair(seed: u8) -> (SigningKey, Vec<u8>) {
        let sk = SigningKey::from_bytes(&[seed; 32]);
        let pk = sk.verifying_key().to_bytes().to_vec();
        (sk, pk)
    }

    fn b64(bytes: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    #[test]
    fn verify_accepts_genuine_signature() -> Result<()> {
        let (sk, pk) = keypair(1);
        let manifest = b"[agent]\nid = \"demo\"\nlayer = 3\n";
        let sig = sk.sign(manifest);
        verify_detached(manifest, &b64(&sig.to_bytes()), &b64(&pk))?;
        Ok(())
    }

    #[test]
    fn verify_rejects_tampered_manifest() {
        let (sk, pk) = keypair(2);
        let manifest = b"[agent]\nid = \"demo\"\nlayer = 3\n";
        let sig = sk.sign(manifest);
        let tampered = b"[agent]\nid = \"evil\"\nlayer = 3\n";
        assert!(verify_detached(tampered, &b64(&sig.to_bytes()), &b64(&pk)).is_err());
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let (sk, _) = keypair(3);
        let (_, other_pk) = keypair(4);
        let manifest = b"[agent]\nid = \"demo\"\nlayer = 3\n";
        let sig = sk.sign(manifest);
        assert!(verify_detached(manifest, &b64(&sig.to_bytes()), &b64(&other_pk)).is_err());
    }

    #[test]
    fn verify_rejects_garbage_input() {
        assert!(verify_detached(b"x", "not-base64!!", "also-not").is_err());
    }
}
