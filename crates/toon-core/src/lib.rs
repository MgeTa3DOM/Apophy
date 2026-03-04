//! TOON v3 — Token-Oriented Object Notation
//!
//! Sérialisation compacte pour LLMs (-60% tokens vs JSON).
//! Encode → zstd-12 compress → decode roundtrip.
//! Règle CLAUDE.md : TOON obligatoire pour toute sérialisation > 100 tokens.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tiktoken_rs::cl100k_base;

/// Erreurs TOON.
#[derive(Debug, Error)]
pub enum ToonError {
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("compression failed: {0}")]
    Compress(String),
    #[error("decompression failed: {0}")]
    Decompress(String),
}

/// Résultat TOON typé.
pub type ToonResult<T> = Result<T, ToonError>;

/// Seuil au-delà duquel TOON est obligatoire (règle CLAUDE.md).
pub const TOON_THRESHOLD_TOKENS: usize = 100;

/// Encode une valeur sérialisable en TOON (JSON compact + zstd-12).
pub fn encode<T: Serialize>(value: &T) -> ToonResult<Vec<u8>> {
    let json = serde_json::to_vec(value)?;
    zstd::encode_all(json.as_slice(), 12)
        .map_err(|e| ToonError::Compress(e.to_string()))
}

/// Décode un buffer TOON en valeur typée.
pub fn decode<T: for<'de> Deserialize<'de>>(data: &[u8]) -> ToonResult<T> {
    let json = zstd::decode_all(data)
        .map_err(|e| ToonError::Decompress(e.to_string()))?;
    serde_json::from_slice(&json).map_err(ToonError::Serialize)
}

/// Compte les tokens d'une chaîne via tiktoken (cl100k_base).
pub fn count_tokens(text: &str) -> usize {
    let bpe = cl100k_base().expect("tiktoken init failed");
    bpe.encode_with_special_tokens(text).len()
}

/// Vérifie si un texte dépasse le seuil TOON (> 100 tokens).
pub fn requires_toon(text: &str) -> bool {
    count_tokens(text) > TOON_THRESHOLD_TOKENS
}

/// Statistiques de compression TOON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToonStats {
    pub json_bytes: usize,
    pub toon_bytes: usize,
    pub json_tokens: usize,
    pub ratio: f64,
}

/// Calcule les statistiques de compression pour une valeur.
pub fn stats<T: Serialize>(value: &T) -> ToonResult<ToonStats> {
    let json = serde_json::to_string(value)?;
    let compressed = encode(value)?;
    let json_tokens = count_tokens(&json);
    let ratio = if json.is_empty() {
        1.0
    } else {
        compressed.len() as f64 / json.len() as f64
    };
    Ok(ToonStats {
        json_bytes: json.len(),
        toon_bytes: compressed.len(),
        json_tokens,
        ratio,
    })
}

/// Encode un batch de valeurs en un seul buffer TOON.
pub fn encode_batch<T: Serialize>(values: &[T]) -> ToonResult<Vec<u8>> {
    let json = serde_json::to_vec(values)?;
    zstd::encode_all(json.as_slice(), 12)
        .map_err(|e| ToonError::Compress(e.to_string()))
}

/// Décode un batch TOON en vecteur de valeurs.
pub fn decode_batch<T: for<'de> Deserialize<'de>>(data: &[u8]) -> ToonResult<Vec<T>> {
    let json = zstd::decode_all(data)
        .map_err(|e| ToonError::Decompress(e.to_string()))?;
    serde_json::from_slice(&json).map_err(ToonError::Serialize)
}

/// Sérialise en JSON ou TOON selon le seuil de tokens.
///
/// Si le contenu dépasse `TOON_THRESHOLD_TOKENS`, compresse en TOON.
/// Sinon retourne le JSON brut. Retourne `(data, is_toon)`.
pub fn smart_encode<T: Serialize>(value: &T) -> ToonResult<(Vec<u8>, bool)> {
    let json = serde_json::to_string(value)?;
    if requires_toon(&json) {
        let toon = encode(value)?;
        Ok((toon, true))
    } else {
        Ok((json.into_bytes(), false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestPayload {
        name: String,
        tags: Vec<String>,
        score: f64,
    }

    fn small_payload() -> TestPayload {
        TestPayload {
            name: "small".to_string(),
            tags: vec!["a".to_string()],
            score: 1.0,
        }
    }

    fn large_payload() -> TestPayload {
        TestPayload {
            name: "x".repeat(500),
            tags: vec!["repeat".to_string(); 100],
            score: 99.9,
        }
    }

    #[test]
    fn roundtrip_encode_decode() {
        let payload = TestPayload {
            name: "apophy-test".to_string(),
            tags: vec!["rust".to_string(), "llm".to_string(), "sovereign".to_string()],
            score: 42.0,
        };
        let encoded = encode(&payload).expect("encode failed");
        let decoded: TestPayload = decode(&encoded).expect("decode failed");
        assert_eq!(payload, decoded);
    }

    #[test]
    fn compression_reduces_size() {
        let payload = large_payload();
        let s = stats(&payload).expect("stats failed");
        assert!(
            s.toon_bytes < s.json_bytes,
            "TOON ({}) should be smaller than JSON ({})",
            s.toon_bytes,
            s.json_bytes
        );
        assert!(s.ratio < 1.0);
    }

    #[test]
    fn token_counting() {
        let tokens = count_tokens("Hello world, this is a test");
        assert!(tokens > 0);
        assert!(tokens < 20);
    }

    #[test]
    fn threshold_check() {
        assert!(!requires_toon("short text"));
        let long = "word ".repeat(200);
        assert!(requires_toon(&long));
    }

    #[test]
    fn batch_roundtrip() {
        let items = vec![small_payload(), large_payload()];
        let encoded = encode_batch(&items).expect("batch encode failed");
        let decoded: Vec<TestPayload> = decode_batch(&encoded).expect("batch decode failed");
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0], items[0]);
        assert_eq!(decoded[1], items[1]);
    }

    #[test]
    fn smart_encode_small_is_json() {
        let payload = small_payload();
        let (data, is_toon) = smart_encode(&payload).expect("smart_encode failed");
        assert!(!is_toon);
        // Should be valid JSON
        let _: TestPayload = serde_json::from_slice(&data).expect("not valid json");
    }

    #[test]
    fn smart_encode_large_is_toon() {
        let payload = large_payload();
        let (data, is_toon) = smart_encode(&payload).expect("smart_encode failed");
        assert!(is_toon);
        // Should decode as TOON
        let decoded: TestPayload = decode(&data).expect("toon decode failed");
        assert_eq!(decoded, payload);
    }
}
