//! Prompt capture and processing
//!
//! Handles prompt parsing, metadata extraction, and duplicate detection.

use std::path::Path;

use crate::models::{Prompt, PromptMetadata};
use crate::utils::{calculate_hash, normalize_whitespace};
use crate::{PromptTrackingError, Result};

/// Prompt capture service
pub struct CaptureService {
    /// Similarity threshold for duplicate detection (0.0 - 1.0)
    similarity_threshold: f64,
}

/// Result of prompt processing
#[derive(Debug)]
pub struct CaptureResult {
    pub prompt: Prompt,
    pub is_duplicate: bool,
    pub similar_hash: Option<String>,
}

impl Default for CaptureService {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.95,
        }
    }
}

impl CaptureService {
    /// Create a new capture service with custom similarity threshold
    pub fn new(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold: similarity_threshold.clamp(0.0, 1.0),
        }
    }

    /// Process prompt content and create a Prompt struct
    pub fn process_content(&self, content: &str) -> Result<Prompt> {
        let normalized = normalize_whitespace(content);

        if normalized.is_empty() {
            return Err(PromptTrackingError::InvalidFormat);
        }

        let mut prompt = Prompt::new(normalized.clone());
        prompt.content_hash = calculate_hash(&normalized);

        // Extract metadata from content
        prompt.metadata = self.extract_metadata(&normalized);

        // Auto-detect category
        prompt.category = self.detect_category(&normalized);

        // Extract tags
        prompt.tags = self.extract_tags(&normalized);

        Ok(prompt)
    }

    /// Capture prompt from file
    pub fn capture_from_file(&self, path: &Path) -> Result<Prompt> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            PromptTrackingError::FileNotFound(format!("Failed to read file: {}", e))
        })?;

        self.process_content(&content)
    }

    /// Extract metadata from prompt content
    fn extract_metadata(&self, content: &str) -> PromptMetadata {
        let mut metadata = PromptMetadata::default();

        // Estimate input tokens (rough approximation: ~4 chars per token)
        let estimated_tokens = (content.len() as f64 / 4.0).ceil() as u32;
        metadata.input_tokens = Some(estimated_tokens);

        // Detect model hints in content
        if content.contains("claude-3-opus") || content.contains("opus") {
            metadata.model = "claude-3-opus".to_string();
        } else if content.contains("claude-3-haiku") || content.contains("haiku") {
            metadata.model = "claude-3-haiku".to_string();
        } else if content.contains("claude-3-5-sonnet") || content.contains("sonnet") {
            metadata.model = "claude-3-5-sonnet".to_string();
        }

        // Extract context if present (look for common patterns)
        if let Some(context) = self.extract_context(content) {
            metadata.context = Some(context);
        }

        metadata
    }

    /// Auto-detect category based on content analysis
    fn detect_category(&self, content: &str) -> Option<String> {
        let lower = content.to_lowercase();

        // Code generation patterns
        if lower.contains("write a function")
            || lower.contains("implement")
            || lower.contains("create a class")
            || lower.contains("code that")
            || lower.contains("프로그램")
            || lower.contains("함수")
        {
            return Some("code-generation".to_string());
        }

        // Documentation patterns
        if lower.contains("document")
            || lower.contains("readme")
            || lower.contains("explain")
            || lower.contains("description")
            || lower.contains("문서")
            || lower.contains("설명")
        {
            return Some("documentation".to_string());
        }

        // Testing patterns
        if lower.contains("test")
            || lower.contains("unit test")
            || lower.contains("테스트")
            || lower.contains("spec")
        {
            return Some("testing".to_string());
        }

        // Debugging patterns
        if lower.contains("debug")
            || lower.contains("fix")
            || lower.contains("error")
            || lower.contains("bug")
            || lower.contains("버그")
            || lower.contains("수정")
        {
            return Some("debugging".to_string());
        }

        // Analysis patterns
        if lower.contains("analyze")
            || lower.contains("review")
            || lower.contains("분석")
            || lower.contains("검토")
        {
            return Some("analysis".to_string());
        }

        None
    }

    /// Extract tags from content
    fn extract_tags(&self, content: &str) -> Vec<String> {
        let mut tags = Vec::new();
        let lower = content.to_lowercase();

        // Programming language detection
        let languages = [
            ("rust", "rust"),
            ("python", "python"),
            ("javascript", "javascript"),
            ("typescript", "typescript"),
            ("java", "java"),
            ("go", "go"),
            ("c++", "cpp"),
            ("ruby", "ruby"),
            ("php", "php"),
            ("swift", "swift"),
            ("kotlin", "kotlin"),
        ];

        for (pattern, tag) in languages {
            if lower.contains(pattern) {
                tags.push(tag.to_string());
            }
        }

        // Framework detection
        let frameworks = [
            ("react", "react"),
            ("vue", "vue"),
            ("angular", "angular"),
            ("django", "django"),
            ("flask", "flask"),
            ("express", "express"),
            ("spring", "spring"),
            ("rails", "rails"),
        ];

        for (pattern, tag) in frameworks {
            if lower.contains(pattern) {
                tags.push(tag.to_string());
            }
        }

        // Topic detection
        if lower.contains("api") || lower.contains("rest") || lower.contains("graphql") {
            tags.push("api".to_string());
        }
        if lower.contains("database") || lower.contains("sql") || lower.contains("db") {
            tags.push("database".to_string());
        }
        if lower.contains("async") || lower.contains("concurrent") || lower.contains("parallel") {
            tags.push("async".to_string());
        }
        if lower.contains("security") || lower.contains("auth") || lower.contains("encrypt") {
            tags.push("security".to_string());
        }

        tags
    }

    /// Extract context from content
    fn extract_context(&self, content: &str) -> Option<String> {
        // Look for context patterns
        let patterns = [
            "context:",
            "background:",
            "situation:",
            "given:",
            "scenario:",
        ];

        for pattern in patterns {
            if let Some(start) = content.to_lowercase().find(pattern) {
                let context_start = start + pattern.len();
                let remaining = &content[context_start..];

                // Extract until newline or end
                let context = remaining
                    .lines()
                    .next()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());

                if context.is_some() {
                    return context;
                }
            }
        }

        None
    }

    /// Calculate similarity between two strings using Jaccard similarity
    pub fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = s1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = s2.split_whitespace().collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Check if content is similar to existing content
    pub fn is_similar(&self, content1: &str, content2: &str) -> bool {
        self.calculate_similarity(content1, content2) >= self.similarity_threshold
    }

    /// Get similarity threshold
    pub fn similarity_threshold(&self) -> f64 {
        self.similarity_threshold
    }
}

/// Builder for creating prompts with custom settings
pub struct PromptBuilder {
    content: String,
    category: Option<String>,
    tags: Vec<String>,
    metadata: PromptMetadata,
}

impl PromptBuilder {
    /// Create a new prompt builder
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            category: None,
            tags: Vec::new(),
            metadata: PromptMetadata::default(),
        }
    }

    /// Set the category
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set multiple tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set the model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.metadata.model = model.into();
        self
    }

    /// Set input tokens
    pub fn input_tokens(mut self, tokens: u32) -> Self {
        self.metadata.input_tokens = Some(tokens);
        self
    }

    /// Set output tokens
    pub fn output_tokens(mut self, tokens: u32) -> Self {
        self.metadata.output_tokens = Some(tokens);
        self
    }

    /// Set execution time
    pub fn execution_time_ms(mut self, time: u64) -> Self {
        self.metadata.execution_time_ms = Some(time);
        self
    }

    /// Set estimated cost
    pub fn estimated_cost(mut self, cost: f64) -> Self {
        self.metadata.estimated_cost = Some(cost);
        self
    }

    /// Set context
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.metadata.context = Some(context.into());
        self
    }

    /// Build the prompt
    pub fn build(self) -> Prompt {
        let normalized = normalize_whitespace(&self.content);
        let mut prompt = Prompt::new(normalized.clone());

        prompt.content_hash = calculate_hash(&normalized);
        prompt.category = self.category;
        prompt.tags = self.tags;
        prompt.metadata = self.metadata;

        // Estimate tokens if not set
        if prompt.metadata.input_tokens.is_none() {
            let estimated = (normalized.len() as f64 / 4.0).ceil() as u32;
            prompt.metadata.input_tokens = Some(estimated);
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_service_default() {
        let service = CaptureService::default();
        assert_eq!(service.similarity_threshold(), 0.95);
    }

    #[test]
    fn test_process_content() {
        let service = CaptureService::default();
        let prompt = service
            .process_content("Write a function in Rust that calculates fibonacci")
            .unwrap();

        assert!(!prompt.id.is_empty());
        assert!(!prompt.content_hash.is_empty());
        assert_eq!(prompt.category, Some("code-generation".to_string()));
        assert!(prompt.tags.contains(&"rust".to_string()));
    }

    #[test]
    fn test_process_empty_content() {
        let service = CaptureService::default();
        let result = service.process_content("   ");

        assert!(result.is_err());
    }

    #[test]
    fn test_detect_category_code_generation() {
        let service = CaptureService::default();
        let prompt = service
            .process_content("Write a function that sorts an array")
            .unwrap();

        assert_eq!(prompt.category, Some("code-generation".to_string()));
    }

    #[test]
    fn test_detect_category_documentation() {
        let service = CaptureService::default();
        let prompt = service
            .process_content("Document this API endpoint")
            .unwrap();

        assert_eq!(prompt.category, Some("documentation".to_string()));
    }

    #[test]
    fn test_detect_category_testing() {
        let service = CaptureService::default();
        let prompt = service
            .process_content("Write unit tests for this module")
            .unwrap();

        assert_eq!(prompt.category, Some("testing".to_string()));
    }

    #[test]
    fn test_detect_category_debugging() {
        let service = CaptureService::default();
        let prompt = service
            .process_content("Fix this bug in the authentication")
            .unwrap();

        assert_eq!(prompt.category, Some("debugging".to_string()));
    }

    #[test]
    fn test_extract_tags_languages() {
        let service = CaptureService::default();
        let prompt = service
            .process_content("Convert this Python code to TypeScript")
            .unwrap();

        assert!(prompt.tags.contains(&"python".to_string()));
        assert!(prompt.tags.contains(&"typescript".to_string()));
    }

    #[test]
    fn test_extract_tags_frameworks() {
        let service = CaptureService::default();
        let prompt = service
            .process_content("Create a React component with Django backend")
            .unwrap();

        assert!(prompt.tags.contains(&"react".to_string()));
        assert!(prompt.tags.contains(&"django".to_string()));
    }

    #[test]
    fn test_similarity_identical() {
        let service = CaptureService::default();
        let similarity = service.calculate_similarity("hello world", "hello world");

        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_similarity_different() {
        let service = CaptureService::default();
        let similarity = service.calculate_similarity("hello world", "foo bar");

        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_similarity_partial() {
        let service = CaptureService::default();
        let similarity =
            service.calculate_similarity("hello world foo", "hello world bar");

        // 2 common words (hello, world) out of 4 unique (hello, world, foo, bar)
        assert!((similarity - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_is_similar() {
        let service = CaptureService::new(0.5);

        assert!(service.is_similar("hello world foo", "hello world bar"));
        assert!(!service.is_similar("hello", "world"));
    }

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new("Test content")
            .category("testing")
            .tag("rust")
            .tag("unit-test")
            .model("claude-3-opus")
            .input_tokens(100)
            .output_tokens(200)
            .execution_time_ms(500)
            .estimated_cost(0.01)
            .context("Testing context")
            .build();

        assert_eq!(prompt.category, Some("testing".to_string()));
        assert_eq!(prompt.tags.len(), 2);
        assert_eq!(prompt.metadata.model, "claude-3-opus");
        assert_eq!(prompt.metadata.input_tokens, Some(100));
        assert_eq!(prompt.metadata.output_tokens, Some(200));
        assert_eq!(prompt.metadata.execution_time_ms, Some(500));
        assert_eq!(prompt.metadata.estimated_cost, Some(0.01));
        assert_eq!(
            prompt.metadata.context,
            Some("Testing context".to_string())
        );
    }

    #[test]
    fn test_prompt_builder_auto_estimate_tokens() {
        let prompt = PromptBuilder::new("This is a test prompt").build();

        // Should auto-estimate tokens
        assert!(prompt.metadata.input_tokens.is_some());
    }

    #[test]
    fn test_extract_context() {
        let service = CaptureService::default();
        // Note: content is normalized (newlines become spaces)
        let prompt = service
            .process_content("Context: Building a web API\nWrite a REST endpoint")
            .unwrap();

        // After normalization, context extracts until end since no newline exists
        assert!(prompt.metadata.context.is_some());
        assert!(prompt.metadata.context.unwrap().contains("Building a web API"));
    }

    #[test]
    fn test_content_normalization() {
        let service = CaptureService::default();
        let prompt = service
            .process_content("  Multiple   spaces   here  ")
            .unwrap();

        assert_eq!(prompt.content, "Multiple spaces here");
    }
}
