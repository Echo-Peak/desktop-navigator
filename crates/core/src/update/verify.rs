use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn sha256_file(path: &Path) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_hex(&bytes))
}

pub fn parse_checksum(contents: &str) -> Option<String> {
    contents
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .and_then(|line| line.split_whitespace().next())
        .map(|hash| hash.to_lowercase())
}

pub fn verify_bytes(bytes: &[u8], checksum_contents: &str) -> bool {
    match parse_checksum(checksum_contents) {
        Some(expected) => sha256_hex(bytes) == expected,
        None => false,
    }
}

pub fn verify_file(path: &Path, checksum_contents: &str) -> io::Result<bool> {
    let actual = sha256_file(path)?;
    Ok(match parse_checksum(checksum_contents) {
        Some(expected) => actual == expected,
        None => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_of_known_input() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn parses_checksum_line() {
        assert_eq!(
            parse_checksum("ba7816bf  app.zip\n"),
            Some("ba7816bf".to_string())
        );
        assert_eq!(parse_checksum("\n\n  DEADBEEF file"), Some("deadbeef".into()));
    }

    #[test]
    fn verify_matches() {
        let bytes = b"abc";
        let checksum = format!("{}  app.zip", sha256_hex(bytes));
        assert!(verify_bytes(bytes, &checksum));
    }

    #[test]
    fn verify_mismatch() {
        assert!(!verify_bytes(b"abc", "00  app.zip"));
    }

    #[test]
    fn verify_file_mismatch_then_delete_semantics() {
        let dir = std::env::temp_dir().join(format!("dn-verify-{}", crate::data::now_ms()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("artifact.zip");
        std::fs::write(&file, b"abc").unwrap();
        let ok = verify_file(&file, "00  artifact.zip").unwrap();
        assert!(!ok);
        if !ok {
            std::fs::remove_file(&file).unwrap();
        }
        assert!(!file.exists());
        std::fs::remove_dir_all(&dir).ok();
    }
}
