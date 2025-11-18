//! Utility functions and helpers

use sha2::{Digest, Sha256};

/// Calculate SHA-256 hash of content
pub fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Truncate string to specified length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Normalize whitespace in text
pub fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash_consistency() {
        let content = "test content";
        let hash1 = calculate_hash(content);
        let hash2 = calculate_hash(content);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_calculate_hash_different_content() {
        let hash1 = calculate_hash("content1");
        let hash2 = calculate_hash("content2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_truncate_string_short() {
        let result = truncate_string("short", 10);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_string_long() {
        let result = truncate_string("this is a very long string", 10);
        assert_eq!(result, "this is...");
    }

    #[test]
    fn test_normalize_whitespace() {
        let result = normalize_whitespace("  multiple   spaces   here  ");
        assert_eq!(result, "multiple spaces here");
    }

    #[test]
    fn test_normalize_whitespace_with_newlines() {
        let result = normalize_whitespace("line1\n\nline2\tline3");
        assert_eq!(result, "line1 line2 line3");
    }
}
