//! Data models for the prompt tracking system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Status of a prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PromptStatus {
    #[default]
    Active,
    Archived,
    Deprecated,
}

impl fmt::Display for PromptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptStatus::Active => write!(f, "active"),
            PromptStatus::Archived => write!(f, "archived"),
            PromptStatus::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl FromStr for PromptStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            "deprecated" => Ok(Self::Deprecated),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

/// Represents a captured prompt with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub content: String,
    pub content_hash: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub status: PromptStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: PromptMetadata,
}

/// Metadata associated with a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    pub model: String,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub execution_time_ms: Option<u64>,
    pub estimated_cost: Option<f64>,
    pub context: Option<String>,
}

/// Quality analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub prompt_id: String,
    pub total_score: f64,
    pub clarity: f64,
    pub completeness: f64,
    pub specificity: f64,
    pub guidance: f64,
    pub analyzed_at: DateTime<Utc>,
}

/// Efficiency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub prompt_id: String,
    pub efficiency_score: f64,
    pub token_efficiency: f64,
    pub time_efficiency: f64,
    pub cost_efficiency: f64,
    pub calculated_at: DateTime<Utc>,
}

impl Prompt {
    /// Create a new prompt with generated ID and timestamps
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            content_hash: String::new(), // Will be calculated
            content,
            category: None,
            tags: Vec::new(),
            status: PromptStatus::default(),
            created_at: now,
            updated_at: now,
            metadata: PromptMetadata::default(),
        }
    }
}

impl Default for PromptMetadata {
    fn default() -> Self {
        Self {
            model: String::from("claude-3-5-sonnet"),
            input_tokens: None,
            output_tokens: None,
            execution_time_ms: None,
            estimated_cost: None,
            context: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_new_creates_valid_prompt() {
        let content = "Test prompt content".to_string();
        let prompt = Prompt::new(content.clone());

        assert_eq!(prompt.content, content);
        assert!(!prompt.id.is_empty());
        assert!(prompt.content_hash.is_empty()); // Not calculated yet
        assert!(prompt.category.is_none());
        assert!(prompt.tags.is_empty());
    }

    #[test]
    fn test_prompt_new_generates_unique_ids() {
        let prompt1 = Prompt::new("First".to_string());
        let prompt2 = Prompt::new("Second".to_string());

        assert_ne!(prompt1.id, prompt2.id);
    }

    #[test]
    fn test_prompt_metadata_default() {
        let metadata = PromptMetadata::default();

        assert_eq!(metadata.model, "claude-3-5-sonnet");
        assert!(metadata.input_tokens.is_none());
        assert!(metadata.output_tokens.is_none());
        assert!(metadata.execution_time_ms.is_none());
        assert!(metadata.estimated_cost.is_none());
        assert!(metadata.context.is_none());
    }

    #[test]
    fn test_prompt_serialization() {
        let prompt = Prompt::new("Test content".to_string());
        let json = serde_json::to_string(&prompt).unwrap();
        let deserialized: Prompt = serde_json::from_str(&json).unwrap();

        assert_eq!(prompt.id, deserialized.id);
        assert_eq!(prompt.content, deserialized.content);
    }

    #[test]
    fn test_quality_score_creation() {
        let score = QualityScore {
            prompt_id: "test-id".to_string(),
            total_score: 85.0,
            clarity: 20.0,
            completeness: 25.0,
            specificity: 20.0,
            guidance: 20.0,
            analyzed_at: Utc::now(),
        };

        assert_eq!(score.total_score, 85.0);
        assert!(score.total_score <= 100.0);
    }

    #[test]
    fn test_efficiency_metrics_creation() {
        let metrics = EfficiencyMetrics {
            prompt_id: "test-id".to_string(),
            efficiency_score: 75.0,
            token_efficiency: 80.0,
            time_efficiency: 70.0,
            cost_efficiency: 75.0,
            calculated_at: Utc::now(),
        };

        assert_eq!(metrics.efficiency_score, 75.0);
    }
}
