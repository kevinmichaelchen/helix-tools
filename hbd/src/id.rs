#![allow(dead_code)]

use uuid::Uuid;

const ID_PREFIX: &str = "bd";
const HASH_LEN: usize = 6;

pub fn generate_issue_id() -> String {
    let uuid = Uuid::new_v4();
    let hash = blake3::hash(uuid.as_bytes());
    let hex = hex::encode(&hash.as_bytes()[..3]);
    format!("{ID_PREFIX}-{hex}")
}

pub fn generate_comment_id() -> String {
    let uuid = Uuid::new_v4();
    let hash = blake3::hash(uuid.as_bytes());
    hex::encode(&hash.as_bytes()[..8])
}

pub fn is_valid_issue_id(id: &str) -> bool {
    id.starts_with(&format!("{ID_PREFIX}-")) && id.len() == ID_PREFIX.len() + 1 + HASH_LEN
}

pub fn normalize_id(id: &str) -> String {
    id.to_lowercase()
}

pub fn matches_partial(full_id: &str, partial: &str) -> bool {
    let partial = partial.to_lowercase();
    let full = full_id.to_lowercase();
    full.contains(&partial)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_issue_id_format() {
        let id = generate_issue_id();
        assert!(id.starts_with("bd-"));
        assert_eq!(id.len(), 9);
    }

    #[test]
    fn test_generate_unique_ids() {
        let id1 = generate_issue_id();
        let id2 = generate_issue_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_is_valid_issue_id() {
        assert!(is_valid_issue_id("bd-a1b2c3"));
        assert!(!is_valid_issue_id("bd-a1b2"));
        assert!(!is_valid_issue_id("xx-a1b2c3"));
        assert!(!is_valid_issue_id("bda1b2c3"));
    }

    #[test]
    fn test_matches_partial() {
        assert!(matches_partial("bd-a1b2c3", "a1b2"));
        assert!(matches_partial("bd-a1b2c3", "bd-a1"));
        assert!(matches_partial("bd-A1B2C3", "a1b2"));
        assert!(!matches_partial("bd-a1b2c3", "xyz"));
    }
}
