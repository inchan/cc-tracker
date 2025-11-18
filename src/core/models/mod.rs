//! Data models for the prompt tracking system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a captured prompt with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub content: String,
    pub content_hash: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
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
