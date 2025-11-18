//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::PromptTrackingError;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub capture: CaptureConfig,
    pub analysis: AnalysisConfig,
    pub reporting: ReportingConfig,
    pub categories: Vec<String>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub auto_backup: bool,
    pub backup_interval: u32,
}

/// Capture configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub watch_directory: String,
    pub auto_capture: bool,
    pub deduplicate: bool,
    pub similarity_threshold: f64,
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub auto_analyze: bool,
    pub quality_weights: QualityWeights,
}

/// Quality score weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityWeights {
    pub clarity: f64,
    pub completeness: f64,
    pub specificity: f64,
    pub guidance: f64,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    pub auto_report: bool,
    pub weekly: ScheduleConfig,
    pub monthly: MonthlyScheduleConfig,
    pub formats: Vec<String>,
    pub output_dir: String,
}

/// Schedule configuration for weekly reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub day: String,
    pub time: String,
}

/// Schedule configuration for monthly reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyScheduleConfig {
    pub enabled: bool,
    pub day: u8,
    pub time: String,
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &PathBuf) -> Result<Self, PromptTrackingError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            PromptTrackingError::ConfigError(format!("Failed to read config: {}", e))
        })?;

        serde_yaml::from_str(&content).map_err(|e| {
            PromptTrackingError::ConfigError(format!("Failed to parse config: {}", e))
        })
    }

    /// Get default configuration path
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("prompt-tracking")
            .join("config.yaml")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                path: "~/.local/share/prompt-tracking/prompts.db".to_string(),
                auto_backup: true,
                backup_interval: 24,
            },
            capture: CaptureConfig {
                watch_directory: "$HOME/.claude-code-data".to_string(),
                auto_capture: true,
                deduplicate: true,
                similarity_threshold: 0.95,
            },
            analysis: AnalysisConfig {
                auto_analyze: true,
                quality_weights: QualityWeights {
                    clarity: 0.3,
                    completeness: 0.3,
                    specificity: 0.2,
                    guidance: 0.2,
                },
            },
            reporting: ReportingConfig {
                auto_report: true,
                weekly: ScheduleConfig {
                    enabled: true,
                    day: "monday".to_string(),
                    time: "09:00".to_string(),
                },
                monthly: MonthlyScheduleConfig {
                    enabled: true,
                    day: 1,
                    time: "09:00".to_string(),
                },
                formats: vec!["markdown".to_string(), "html".to_string()],
                output_dir: "~/Documents/Prompt Reports".to_string(),
            },
            categories: vec![
                "code-generation".to_string(),
                "documentation".to_string(),
                "analysis".to_string(),
                "testing".to_string(),
                "debugging".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert!(config.database.auto_backup);
        assert_eq!(config.database.backup_interval, 24);
        assert!(config.capture.auto_capture);
        assert_eq!(config.capture.similarity_threshold, 0.95);
    }

    #[test]
    fn test_quality_weights_sum() {
        let config = Config::default();
        let weights = &config.analysis.quality_weights;

        let sum = weights.clarity + weights.completeness + weights.specificity + weights.guidance;
        assert!((sum - 1.0).abs() < 0.001); // Should sum to 1.0
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();

        assert!(yaml.contains("database:"));
        assert!(yaml.contains("capture:"));
        assert!(yaml.contains("analysis:"));
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
database:
  path: "/test/path"
  auto_backup: false
  backup_interval: 12
capture:
  watch_directory: "/watch"
  auto_capture: false
  deduplicate: true
  similarity_threshold: 0.9
analysis:
  auto_analyze: true
  quality_weights:
    clarity: 0.25
    completeness: 0.25
    specificity: 0.25
    guidance: 0.25
reporting:
  auto_report: false
  weekly:
    enabled: false
    day: "friday"
    time: "10:00"
  monthly:
    enabled: true
    day: 15
    time: "08:00"
  formats: ["json"]
  output_dir: "/reports"
categories:
  - custom
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.database.path, "/test/path");
        assert!(!config.database.auto_backup);
        assert_eq!(config.database.backup_interval, 12);
    }
}
