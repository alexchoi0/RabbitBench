use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};

const BACKUP_CODE_LENGTH: usize = 8;
const DEFAULT_BACKUP_CODE_COUNT: usize = 10;

pub struct BackupCodes;

impl BackupCodes {
    pub fn generate(count: usize) -> Vec<String> {
        (0..count).map(|_| Self::generate_code()).collect()
    }

    pub fn generate_default() -> Vec<String> {
        Self::generate(DEFAULT_BACKUP_CODE_COUNT)
    }

    pub fn hash(code: &str) -> String {
        let normalized = code.to_uppercase().replace("-", "").replace(" ", "");
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize())
    }

    pub fn hash_all(codes: &[String]) -> Vec<String> {
        codes.iter().map(|c| Self::hash(c)).collect()
    }

    pub fn verify(code: &str, hashed_codes: &[String]) -> Option<usize> {
        let hash = Self::hash(code);
        hashed_codes.iter().position(|h| h == &hash)
    }

    pub fn format_for_display(code: &str) -> String {
        let code = code.to_uppercase();
        if code.len() == 8 {
            format!("{}-{}", &code[0..4], &code[4..8])
        } else {
            code
        }
    }

    fn generate_code() -> String {
        let mut bytes = [0u8; BACKUP_CODE_LENGTH];
        rand::thread_rng().fill_bytes(&mut bytes);

        bytes
            .iter()
            .map(|b| {
                let idx = (*b as usize) % 36;
                if idx < 10 {
                    (b'0' + idx as u8) as char
                } else {
                    (b'A' + (idx - 10) as u8) as char
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_backup_codes() {
        let codes = BackupCodes::generate_default();
        assert_eq!(codes.len(), 10);
        for code in &codes {
            assert_eq!(code.len(), 8);
        }
    }

    #[test]
    fn test_verify_backup_code() {
        let codes = BackupCodes::generate(3);
        let hashed = BackupCodes::hash_all(&codes);

        assert_eq!(BackupCodes::verify(&codes[0], &hashed), Some(0));
        assert_eq!(BackupCodes::verify(&codes[1], &hashed), Some(1));
        assert_eq!(BackupCodes::verify("invalid", &hashed), None);
    }

    #[test]
    fn test_format_for_display() {
        assert_eq!(BackupCodes::format_for_display("ABCD1234"), "ABCD-1234");
    }
}
