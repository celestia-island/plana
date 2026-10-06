//! Symmetric secret sealing for at-rest credential columns.
//!
//! AES-256-GCM under a caller-supplied 32-byte key with a fresh random
//! 96-bit nonce per call. The sealed blob is `nonce || ciphertext || tag`
//! (the nonce is the leading 12 bytes) as raw bytes — it is binary, so it
//! belongs in a `bytea`/`BLOB` column, never in a text column. GCM
//! authenticates, so [`open`] with the wrong key fails loudly instead of
//! yielding garbage plaintext.
//!
//! This is the upstream home of the sealing scheme shittim-chest's
//! `llm::config` pioneered (provider API keys) — arona's model catalog
//! needed the identical capability for its provider rows, and a scheme
//! shared by two services belongs upstream (workspace §3.3 rule 4). The
//! byte format is identical to chest's so blobs move between services
//! unchanged when the key material agrees.
//!
//! No key derivation happens here: `key` is used as-is. The intended key
//! source is one base64-encoded 32-byte value per instance from a 0600
//! env file or systemd `LoadCredential` (workspace §3.7 class-A root);
//! [`decode_key`] parses that envelope.

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::{Aead, AeadCore, KeyInit};
use anyhow::{Result, anyhow, bail};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

/// Blob layout: 12-byte nonce prefix, then ciphertext+tag.
const NONCE_LEN: usize = 12;

/// Parse the per-instance key envelope: base64 of exactly 32 bytes.
///
/// Rejects shorter/longer material loudly — a truncated key would seal
/// fine under AES-128-ish semantics and then never open again, and a
/// padded key would silently differ from the operator's intent.
pub fn decode_key(encoded: &str) -> Result<[u8; 32]> {
    let key_bytes = BASE64.decode(encoded.trim())?;
    if key_bytes.len() != 32 {
        bail!(
            "encryption key must decode to exactly 32 bytes, got {}",
            key_bytes.len()
        );
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

/// Encrypt a secret for storage in a binary column.
///
/// A freshly generated nonce per call means the same plaintext seals to a
/// different blob every time — no deterministic output, and therefore no
/// equality search over stored secrets.
pub fn seal(plaintext: &str, key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!("cipher init failed: {e}"))?;
    let nonce = Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("encryption failed: {e}"))?;
    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&ciphertext);
    Ok(blob)
}

/// Decrypt a blob produced by [`seal`] (byte-format compatible with
/// chest's `llm::config::encrypt_api_key`).
///
/// Fails when the blob is under 12 bytes (no nonce), when the key is
/// wrong or the ciphertext was tampered with, or when the recovered
/// bytes are not valid UTF-8.
pub fn open(blob: &[u8], key: &[u8; 32]) -> Result<String> {
    if blob.len() < NONCE_LEN {
        bail!("encrypted blob too short");
    }
    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| anyhow!("cipher init failed: {e}"))?;
    // Fixed-size array → GenericArray via From (from_slice is deprecated
    // generic-array 0.x API and denied by the workspace lint set); the
    // try_into itself checks the 12-byte length.
    let nonce_arr: [u8; NONCE_LEN] = nonce_bytes
        .try_into()
        .map_err(|_| anyhow!("nonce is not 12 bytes"))?;
    let plaintext = cipher
        .decrypt((&nonce_arr).into(), ciphertext)
        .map_err(|e| anyhow!("decryption failed: {e}"))?;
    String::from_utf8(plaintext).map_err(|e| anyhow!("decrypted secret is not UTF-8: {e}"))
}

/// Best-effort variant of [`open`] for optional columns: an empty/absent
/// blob answers `None`, a malformed blob answers `None` with no error —
/// for call sites that render "no key stored" instead of failing the
/// request. Callers that must distinguish corruption from absence use
/// [`open`] directly.
pub fn maybe_open(blob: Option<&[u8]>, key: &[u8; 32]) -> Option<String> {
    let blob = blob?;
    if blob.is_empty() {
        return None;
    }
    open(blob, key).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 32 zero bytes, base64 — the test key. Not a secret.
    const KEY_B64: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    fn key() -> [u8; 32] {
        decode_key(KEY_B64).expect("test key parses")
    }

    #[test]
    fn decode_key_accepts_exactly_32_bytes() {
        assert!(decode_key(KEY_B64).is_ok());
        // 31 bytes: rejected loudly.
        let short = BASE64.encode([0u8; 31]);
        assert!(decode_key(&short).is_err());
        // 33 bytes: rejected loudly.
        let long = BASE64.encode([0u8; 33]);
        assert!(decode_key(&long).is_err());
        // Surrounding whitespace tolerated (env-file editors add newlines).
        assert!(decode_key(&format!("  {KEY_B64}\n")).is_ok());
    }

    #[test]
    fn seal_open_round_trips() {
        let key = key();
        let blob = seal("sk-live-1234", &key).expect("seal");
        assert_eq!(open(&blob, &key).expect("open"), "sk-live-1234");
        // Nonce prefix is present and the blob is longer than the plaintext.
        assert!(blob.len() > 12 + "sk-live-1234".len());
    }

    #[test]
    fn seal_is_non_deterministic() {
        let key = key();
        let a = seal("same", &key).expect("seal a");
        let b = seal("same", &key).expect("seal b");
        assert_ne!(a, b, "fresh nonce per call must change the blob");
    }

    #[test]
    fn open_rejects_wrong_key_and_tampering() {
        let key = key();
        let blob = seal("secret", &key).expect("seal");
        let wrong = [1u8; 32];
        assert!(open(&blob, &wrong).is_err(), "wrong key must fail loudly");
        let mut tampered = blob.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0x01;
        assert!(open(&tampered, &key).is_err(), "GCM must authenticate");
        assert!(open(&blob[..11], &key).is_err(), "short blob rejected");
    }

    #[test]
    fn maybe_open_treats_empty_as_absent() {
        let key = key();
        assert_eq!(maybe_open(None, &key), None);
        assert_eq!(maybe_open(Some(&[]), &key), None);
        let blob = seal("k", &key).expect("seal");
        assert_eq!(maybe_open(Some(&blob), &key), Some("k".to_string()));
        assert_eq!(
            maybe_open(Some(&[9u8; 3]), &key),
            None,
            "corrupt blob degrades to None"
        );
    }

    /// Byte-format compatibility with chest's `llm::config` sealers: the
    /// nonce is the leading 12 bytes and the ciphertext follows. This is
    /// the property that lets a blob sealed by one service open in
    /// another when the key material agrees — pinned so a drift here is
    /// caught as a test failure, not as a production migration surprise.
    #[test]
    fn blob_layout_is_nonce_prefixed() {
        let key = key();
        let blob = seal("layout", &key).expect("seal");
        let cipher = Aes256Gcm::new_from_slice(&key).expect("cipher");
        let nonce_arr: [u8; NONCE_LEN] = blob[..NONCE_LEN].try_into().expect("nonce slice");
        let plain = cipher
            .decrypt((&nonce_arr).into(), &blob[NONCE_LEN..])
            .expect("manual open");
        assert_eq!(plain, b"layout");
    }
}
