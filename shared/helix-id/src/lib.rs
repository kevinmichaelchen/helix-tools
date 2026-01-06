use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdError {
    #[error("Invalid ID format: {0}")]
    InvalidFormat(String),
    #[error("Invalid hex in ID: {0}")]
    InvalidHex(String),
}

const DEFAULT_HASH_BYTES: usize = 3;

pub fn id_from_key(prefix: &str, key: &str) -> String {
    id_from_key_with_length(prefix, key, DEFAULT_HASH_BYTES)
}

pub fn id_from_key_with_length(prefix: &str, key: &str, bytes: usize) -> String {
    let hash = blake3::hash(key.as_bytes());
    let hex = hex::encode(&hash.as_bytes()[..bytes]);
    format!("{prefix}-{hex}")
}

pub fn id_from_parts(prefix: &str, parts: &[&str]) -> String {
    id_from_parts_with_length(prefix, parts, DEFAULT_HASH_BYTES)
}

pub fn id_from_parts_with_length(prefix: &str, parts: &[&str], bytes: usize) -> String {
    let key = parts.join(":");
    id_from_key_with_length(prefix, &key, bytes)
}

pub fn id_random(prefix: &str) -> String {
    id_random_with_length(prefix, DEFAULT_HASH_BYTES)
}

pub fn id_random_with_length(prefix: &str, bytes: usize) -> String {
    let uuid = uuid::Uuid::new_v4();
    let hash = blake3::hash(uuid.as_bytes());
    let hex = hex::encode(&hash.as_bytes()[..bytes]);
    format!("{prefix}-{hex}")
}

#[deprecated(
    since = "0.2.0",
    note = "Use id_random() for random IDs or id_from_key() for deterministic IDs"
)]
pub fn generate_id(prefix: &str) -> String {
    id_random(prefix)
}

#[deprecated(
    since = "0.2.0",
    note = "Use id_random_with_length() for random IDs or id_from_key_with_length() for deterministic IDs"
)]
pub fn generate_id_with_length(prefix: &str, bytes: usize) -> String {
    id_random_with_length(prefix, bytes)
}

#[deprecated(since = "0.2.0", note = "Use id_from_key() instead")]
pub fn generate_content_id(prefix: &str, content: &str) -> String {
    id_from_key(prefix, content)
}

#[deprecated(since = "0.2.0", note = "Use id_from_key_with_length() instead")]
pub fn generate_content_id_with_length(prefix: &str, content: &str, bytes: usize) -> String {
    id_from_key_with_length(prefix, content, bytes)
}

pub fn parse_id(id: &str) -> Result<(String, String), IdError> {
    let parts: Vec<&str> = id.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(IdError::InvalidFormat(id.to_string()));
    }

    let prefix = parts[0].to_string();
    let hash = parts[1].to_string();

    if hash.len() < 6 || hash.len() > 12 {
        return Err(IdError::InvalidFormat(format!(
            "Hash must be 6-12 characters, got {}",
            hash.len()
        )));
    }

    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(IdError::InvalidHex(hash));
    }

    Ok((prefix, hash))
}

#[macro_export]
macro_rules! define_id {
    ($name:ident, $prefix:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name(String);

        impl $name {
            /// Create ID from a natural key (deterministic - same key = same ID).
            /// Use this for entities with unique identifiers like URLs, paths, or names.
            ///
            /// Example: `SourceId::from_key("facebook/react")`
            pub fn from_key(key: &str) -> Self {
                Self($crate::id_from_key($prefix, key))
            }

            /// Create ID from a natural key with custom hash length.
            pub fn from_key_with_length(key: &str, bytes: usize) -> Self {
                Self($crate::id_from_key_with_length($prefix, key, bytes))
            }

            /// Create ID from multiple key parts joined with `:` separator.
            /// Use this for composite keys.
            ///
            /// Example: `DocId::from_parts(&[source_id.as_str(), "docs/intro.md"])`
            pub fn from_parts(parts: &[&str]) -> Self {
                Self($crate::id_from_parts($prefix, parts))
            }

            /// Create ID from multiple key parts with custom hash length.
            pub fn from_parts_with_length(parts: &[&str], bytes: usize) -> Self {
                Self($crate::id_from_parts_with_length($prefix, parts, bytes))
            }

            /// Generate a random ID (non-deterministic).
            /// Use this only for entities without natural keys, like user-created items
            /// where duplicates are intentionally allowed.
            ///
            /// Example: `IssueId::random()` for issues where same title is allowed.
            pub fn random() -> Self {
                Self($crate::id_random($prefix))
            }

            /// Generate a random ID with custom hash length.
            pub fn random_with_length(bytes: usize) -> Self {
                Self($crate::id_random_with_length($prefix, bytes))
            }

            /// Wrap an existing ID string. No validation performed.
            pub fn from_string(s: impl Into<String>) -> Self {
                Self(s.into())
            }

            /// Get the ID as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Get the prefix for this ID type.
            pub fn prefix() -> &'static str {
                $prefix
            }

            // Deprecated methods for backwards compatibility

            #[deprecated(
                since = "0.2.0",
                note = "Use random() for random IDs or from_key() for deterministic IDs"
            )]
            pub fn generate() -> Self {
                Self::random()
            }

            #[deprecated(since = "0.2.0", note = "Use random_with_length() instead")]
            pub fn generate_with_length(bytes: usize) -> Self {
                Self::random_with_length(bytes)
            }

            #[deprecated(since = "0.2.0", note = "Use from_key() instead")]
            pub fn from_content(content: &str) -> Self {
                Self::from_key(content)
            }

            #[deprecated(since = "0.2.0", note = "Use from_key_with_length() instead")]
            pub fn from_content_with_length(content: &str, bytes: usize) -> Self {
                Self::from_key_with_length(content, bytes)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_from_key_deterministic() {
        let id1 = id_from_key("src", "facebook/react");
        let id2 = id_from_key("src", "facebook/react");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_id_from_key_different_keys() {
        let id1 = id_from_key("src", "facebook/react");
        let id2 = id_from_key("src", "vercel/next.js");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_from_key_format() {
        let id = id_from_key("src", "facebook/react");
        assert!(id.starts_with("src-"));
        let expected_len = "src-".len() + 6;
        assert_eq!(id.len(), expected_len);
    }

    #[test]
    fn test_id_from_parts() {
        let source_id = "src-abc123";
        let path = "docs/intro.md";
        let id = id_from_parts("doc", &[source_id, path]);
        assert!(id.starts_with("doc-"));
    }

    #[test]
    fn test_id_from_parts_deterministic() {
        let id1 = id_from_parts("doc", &["src-abc123", "docs/intro.md"]);
        let id2 = id_from_parts("doc", &["src-abc123", "docs/intro.md"]);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_id_from_parts_order_matters() {
        let id1 = id_from_parts("doc", &["a", "b"]);
        let id2 = id_from_parts("doc", &["b", "a"]);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_random_unique() {
        let id1 = id_random("bd");
        let id2 = id_random("bd");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_random_format() {
        let id = id_random("bd");
        assert!(id.starts_with("bd-"));
        let expected_len = "bd-".len() + 6;
        assert_eq!(id.len(), expected_len);
    }

    #[test]
    fn test_id_with_length() {
        let id = id_from_key_with_length("src", "test", 4);
        let expected_len = "src-".len() + 8;
        assert_eq!(id.len(), expected_len);
    }

    #[test]
    fn test_parse_id() {
        let (prefix, hash) = parse_id("bd-a1b2c3").unwrap();
        assert_eq!(prefix, "bd");
        assert_eq!(hash, "a1b2c3");
    }

    #[test]
    fn test_parse_id_invalid_no_separator() {
        assert!(parse_id("invalid").is_err());
    }

    #[test]
    fn test_parse_id_invalid_hash_too_short() {
        assert!(parse_id("bd-abc").is_err());
    }

    #[test]
    fn test_parse_id_invalid_non_hex_chars() {
        assert!(parse_id("bd-xyz123").is_err());
    }

    define_id!(SourceId, "src");
    define_id!(DocId, "doc");
    define_id!(IssueId, "bd");

    #[test]
    fn test_typed_id_from_key() {
        let id = SourceId::from_key("facebook/react");
        assert!(id.as_str().starts_with("src-"));
    }

    #[test]
    fn test_typed_id_from_key_deterministic() {
        let id1 = SourceId::from_key("facebook/react");
        let id2 = SourceId::from_key("facebook/react");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_typed_id_from_parts() {
        let source = SourceId::from_key("facebook/react");
        let doc = DocId::from_parts(&[source.as_str(), "docs/hooks.md"]);
        assert!(doc.as_str().starts_with("doc-"));
    }

    #[test]
    fn test_typed_id_random() {
        let id1 = IssueId::random();
        let id2 = IssueId::random();
        assert!(id1.as_str().starts_with("bd-"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_typed_id_prefix() {
        assert_eq!(SourceId::prefix(), "src");
        assert_eq!(DocId::prefix(), "doc");
        assert_eq!(IssueId::prefix(), "bd");
    }
}
