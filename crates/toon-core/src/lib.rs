//! TOON v3 — Token-Oriented Object Notation
//!
//! Sérialisation compacte pour LLMs (-60% tokens vs JSON).
//! Encode → zstd-12 compress → decode roundtrip.

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

/// Encode une valeur sérialisable en TOON (JSON compact + zstd).
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

/// Compte les tokens économisés (JSON brut vs TOON compressé).
pub fn token_savings<T: Serialize>(value: &T) -> ToonResult<(usize, usize)> {
    let json = serde_json::to_string(value)?;
    let compressed = encode(value)?;
    Ok((json.len(), compressed.len()))
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
        let payload = TestPayload {
            name: "a]".repeat(500),
            tags: vec!["repeat".to_string(); 100],
            score: 99.9,
        };

        let (json_size, toon_size) = token_savings(&payload).expect("token_savings failed");
        assert!(
            toon_size < json_size,
            "TOON ({toon_size}) should be smaller than JSON ({json_size})"
        );
    }
}
