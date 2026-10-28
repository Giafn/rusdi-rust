use crate::self_update::UpdateError;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub fn parse_checksums(text: &str) -> Result<HashMap<String, String>, UpdateError> {
    let mut result = HashMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split_whitespace();
        let hash = parts
            .next()
            .ok_or_else(|| UpdateError::InvalidChecksum(line.into()))?;
        let name = parts
            .next()
            .ok_or_else(|| UpdateError::InvalidChecksum(line.into()))?
            .trim_start_matches('*');
        if parts.next().is_some()
            || hash.len() != 64
            || !hash.chars().all(|c| c.is_ascii_hexdigit())
            || result
                .insert(name.into(), hash.to_ascii_lowercase())
                .is_some()
        {
            return Err(UpdateError::InvalidChecksum(line.into()));
        }
    }
    Ok(result)
}

pub fn verify_sha256(bytes: &[u8], expected: &str) -> Result<(), UpdateError> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(UpdateError::ChecksumMismatch {
            expected: expected.into(),
            actual,
        })
    }
}
