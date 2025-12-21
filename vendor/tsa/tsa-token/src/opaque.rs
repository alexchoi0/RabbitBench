use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use tsa_core::Result;

pub struct OpaqueToken;

impl OpaqueToken {
    pub fn generate(length: usize) -> String {
        let mut bytes = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    pub fn generate_with_hash(length: usize) -> Result<(String, String)> {
        let token = Self::generate(length);
        let hash = Self::hash(&token);
        Ok((token, hash))
    }

    pub fn hash(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        URL_SAFE_NO_PAD.encode(result)
    }

    pub fn verify(token: &str, hash: &str) -> bool {
        Self::hash(token) == hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_length() {
        let token = OpaqueToken::generate(32);
        assert!(!token.is_empty());
        assert!(token.len() >= 32);
    }

    #[test]
    fn test_generate_unique_tokens() {
        let token1 = OpaqueToken::generate(32);
        let token2 = OpaqueToken::generate(32);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_with_hash() {
        let (token, hash) = OpaqueToken::generate_with_hash(32).unwrap();
        assert!(!token.is_empty());
        assert!(!hash.is_empty());
        assert_ne!(token, hash);
    }

    #[test]
    fn test_hash_deterministic() {
        let token = "test_token_123";
        let hash1 = OpaqueToken::hash(token);
        let hash2 = OpaqueToken::hash(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_tokens() {
        let hash1 = OpaqueToken::hash("token1");
        let hash2 = OpaqueToken::hash("token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_valid_token() {
        let token = "my_secret_token";
        let hash = OpaqueToken::hash(token);
        assert!(OpaqueToken::verify(token, &hash));
    }

    #[test]
    fn test_verify_invalid_token() {
        let token = "my_secret_token";
        let hash = OpaqueToken::hash(token);
        assert!(!OpaqueToken::verify("wrong_token", &hash));
    }

    #[test]
    fn test_token_is_url_safe() {
        let token = OpaqueToken::generate(64);
        assert!(!token.contains('+'));
        assert!(!token.contains('/'));
        assert!(!token.contains('='));
    }
}
