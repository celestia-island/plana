//! Symmetric secret sealing for at-rest credential columns.
//!
//! AES-256-GCM under a caller-supplied 32-byte key with a fresh random
//! 96-bit nonce per call, in two encodings over the same `nonce ||
//! ciphertext || tag` layout (the nonce is the leading 12 bytes):
//!
//! - **raw bytes** — [`seal`] / [`open`] / [`maybe_open`] for `bytea`/BLOB
//!   columns (chest's `llm::config` lineage; binary, never a text column);
//! - **text envelope** — [`seal_text`] / [`open_text`] /
//!   [`maybe_seal_text`] / [`maybe_open_text`] for text columns: standard
//!   (padded) base64 of the same layout, byte-compatible with chest's
//!   `crypto.rs` and scepter's `infra/crypto.rs` text columns.
//!
//! The two encodings are not interchangeable — a raw blob is not base64
//! text and vice versa. GCM authenticates, so [`open`] / [`open_text`]
//! with the wrong key fail loudly instead of yielding garbage plaintext.
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

// ── text envelope: base64(nonce ‖ ct ‖ tag) for text columns ───────────

/// Seal a secret into the **text envelope**: standard (padded) base64 of
/// `nonce ‖ ciphertext ‖ tag`.
///
/// Byte-format compatible with chest's `crypto::encrypt_secret` and
/// scepter's `infra::crypto::encrypt` — a value written by either service
/// opens here when the key material agrees, and vice versa, so adopting
/// this crate migrates no stored value. The raw [`seal`] variant stays
/// the `bytea`-column form; the encodings are not interchangeable.
///
/// Fails only where [`seal`] fails — unreachable with a correctly sized
/// key — and the error carries no key material.
pub fn seal_text(plaintext: &str, key: &[u8; 32]) -> Result<String> {
    Ok(BASE64.encode(seal(plaintext, key)?))
}

/// Open a [`seal_text`] envelope, or a value written by chest's
/// `crypto::decrypt_secret` / scepter's `infra::crypto::decrypt` — the
/// text formats are identical.
///
/// Fails on invalid base64 (including the URL-safe alphabet and missing
/// padding), on a decoded blob too short to carry a nonce, and — via GCM
/// authentication — indistinguishably on a wrong key, a tampered value and
/// a value this module never wrote; and on non-UTF-8 plaintext, which can
/// only be a blob this module did not write. Use [`maybe_open_text`] on
/// columns that may hold a legacy plaintext.
pub fn open_text(encoded: &str, key: &[u8; 32]) -> Result<String> {
    let blob = BASE64.decode(encoded)?;
    open(&blob, key)
}

/// Best-effort [`seal_text`] for a value about to be stored.
///
/// `None` stays `None` and an empty string stays empty, so both survive a
/// [`maybe_open_text`] round-trip unchanged. A sealing failure is logged
/// at `error` and collapses to `None` — a trap for write paths that
/// assign the result straight into a column (the credential is dropped as
/// SQL NULL); propagate [`seal_text`] where the value must not be lost.
pub fn maybe_seal_text(value: &Option<String>, key: &[u8; 32]) -> Option<String> {
    match value.as_ref() {
        None => None,
        Some(v) if v.is_empty() => Some(String::new()),
        Some(v) => seal_text(v, key)
            .map_err(|e| {
                tracing::error!(error = %e, "text-envelope sealing failed for secret value");
                e
            })
            .ok(),
    }
}

/// Best-effort [`open_text`] for optional stored credentials.
///
/// `None` stays `None` and an empty string stays empty — "unset"
/// round-trips without touching the cipher. Every other failure (wrong
/// key, tampered or truncated value, a legacy plaintext, non-base64 text)
/// is logged at `warn` and becomes `None`.
///
/// `None` is therefore ambiguous: "nothing stored", "unreadable with the
/// configured key" and "corrupt" all look alike. Use [`open_text`] where
/// the distinction matters.
pub fn maybe_open_text(value: &Option<String>, key: &[u8; 32]) -> Option<String> {
    match value.as_ref() {
        None => None,
        Some(v) if v.is_empty() => Some(String::new()),
        Some(v) => open_text(v, key)
            .map_err(|e| {
                tracing::warn!(error = %e, "text-envelope opening failed for secret value");
                e
            })
            .ok(),
    }
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
    // ── text envelope (T3-S1) ──────────────────────────────────────────

    #[test]
    fn text_round_trips_and_is_standard_base64_with_padding() {
        let key = key();
        let sealed = seal_text("sk-live-1234", &key).expect("seal_text");
        // Standard alphabet with padding: only [A-Za-z0-9+/] and '=' fillers.
        assert!(!sealed.contains('-'), "URL-safe alphabet leaked: {sealed}");
        assert!(!sealed.contains('_'), "URL-safe alphabet leaked: {sealed}");
        assert!(sealed.contains('='), "padding missing: {sealed}");
        assert_eq!(open_text(&sealed, &key).expect("open_text"), "sk-live-1234");
    }

    #[test]
    fn open_text_rejects_wrong_key_without_garbage() {
        let key = key();
        let sealed = seal_text("secret", &key).expect("seal_text");
        let wrong = [1u8; 32];
        assert!(open_text(&sealed, &wrong).is_err(), "wrong key must fail");
        let mut tampered = sealed.clone();
        let last = tampered.len() - 1;
        tampered.replace_range(last.., if &tampered[last..] == "A" { "B" } else { "A" });
        assert!(open_text(&tampered, &key).is_err(), "tampering must fail");
    }

    #[test]
    fn open_text_rejects_non_base64_and_url_safe_alphabet() {
        let key = key();
        assert!(open_text("not base64 at all!", &key).is_err());
        assert!(
            open_text("", &key).is_err(),
            "empty string is not an envelope"
        );
        // A URL-safe alphabet character (-) is invalid in the standard engine.
        assert!(open_text("a-b_c", &key).is_err());
    }

    #[test]
    fn maybe_text_preserves_none_and_empty_semantics() {
        let key = key();
        assert_eq!(maybe_open_text(&None, &key), None);
        assert_eq!(
            maybe_open_text(&Some(String::new()), &key),
            Some(String::new())
        );
        assert_eq!(maybe_seal_text(&None, &key), None);
        assert_eq!(
            maybe_seal_text(&Some(String::new()), &key),
            Some(String::new())
        );
        // Round-trip through the maybe pair.
        let stored = maybe_seal_text(&Some("value".to_string()), &key).expect("seal");
        assert_eq!(
            maybe_open_text(&Some(stored), &key),
            Some("value".to_string())
        );
        // A legacy plaintext column degrades to None on read.
        assert_eq!(
            maybe_open_text(&Some("legacy plaintext".to_string()), &key),
            None
        );
    }

    #[test]
    fn text_envelope_handles_long_secrets() {
        let key = key();
        let long = "x".repeat(10_240);
        let sealed = seal_text(&long, &key).expect("seal_text");
        assert_eq!(open_text(&sealed, &key).expect("open_text"), long);
    }

    #[test]
    fn text_blob_layout_is_twelve_byte_nonce_prefixed() {
        let key = key();
        let sealed = seal_text("layout", &key).expect("seal_text");
        let decoded = BASE64.decode(sealed.as_bytes()).expect("standard base64");
        let nonce_arr: [u8; NONCE_LEN] = decoded[..NONCE_LEN].try_into().expect("nonce slice");
        let cipher = Aes256Gcm::new_from_slice(&key).expect("cipher");
        let plain = cipher
            .decrypt((&nonce_arr).into(), &decoded[NONCE_LEN..])
            .expect("manual open");
        assert_eq!(plain, b"layout");
    }

    #[test]
    fn text_and_raw_variants_are_not_interchangeable() {
        let key = key();
        // The raw blob is binary — it is not valid base64 text, so the text
        // opener rejects it instead of silently "decrypting" noise.
        let raw = seal("raw-form", &key).expect("seal");
        let raw_as_text = String::from_utf8_lossy(&raw).into_owned();
        assert!(
            open_text(&raw_as_text, &key).is_err(),
            "raw blob is not a text envelope"
        );
        // And the text envelope is base64 text — feeding those bytes to the
        // raw opener fails the nonce/GCM layout instead of returning data.
        let text = seal_text("text-form", &key).expect("seal_text");
        assert!(
            open(text.as_bytes(), &key).is_err(),
            "text envelope is not a raw blob"
        );
        assert!(open(&String::into_bytes(text), &key).is_err());
    }

    /// Byte-format compatibility with chest's `crypto::encrypt_secret` /
    /// scepter's `infra::crypto::encrypt`: standard (padded) base64 of
    /// AES-256-GCM `nonce(12) || ciphertext-with-tag`, non-deterministic
    /// per call. The sealed-here side is produced the way those services
    /// produce theirs (explicit nonce + STANDARD engine), so this pins the
    /// exact wire property S2's real-ciphertext round-trip will exercise.
    #[test]
    fn opens_a_chest_and_scepter_shaped_envelope() {
        use aes_gcm::aead::{Aead, KeyInit};
        use base64::engine::general_purpose::STANDARD as BASE64;

        let key = key();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("cipher");
        let nonce: [u8; NONCE_LEN] = [0x42; NONCE_LEN]; // explicit, like the reference impls' split
        let ciphertext = cipher
            .encrypt((&nonce).into(), b"migrated-from-chest".as_slice())
            .expect("encrypt");
        let mut blob = nonce.to_vec();
        blob.extend_from_slice(&ciphertext);
        let foreign = BASE64.encode(&blob);

        assert_eq!(
            open_text(&foreign, &key).expect("cross-service open"),
            "migrated-from-chest"
        );
        // And the reverse direction: sealed here, openable by the
        // reference algorithm (split_at(12) + GCM verify).
        let ours = seal_text("written-here", &key).expect("seal_text");
        let decoded = BASE64.decode(ours.as_bytes()).expect("decode");
        let (n, rest) = decoded.split_at(NONCE_LEN);
        let nonce_arr: [u8; NONCE_LEN] = n.try_into().expect("nonce slice");
        let plain = cipher
            .decrypt((&nonce_arr).into(), rest)
            .expect("reference open");
        assert_eq!(plain, b"written-here");
    }

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
