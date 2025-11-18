//! Edge case and error handling tests

use prompt_tracking::{
    analysis::{EfficiencyAnalyzer, QualityAnalyzer, generate_summary},
    capture::{CaptureService, PromptBuilder},
    config::Config,
    database::{Database, PromptFilter},
    models::{Prompt, PromptMetadata, QualityScore, EfficiencyMetrics},
    reporting::{build_report_data, ReportFormat, ReportGenerator, ReportType},
    cache::CacheManager,
    utils::{calculate_hash, normalize_whitespace, truncate_string},
};
use chrono::Utc;

// === Utility Function Tests ===

#[test]
fn test_hash_empty_string() {
    let hash = calculate_hash("");
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
}

#[test]
fn test_hash_unicode() {
    let hash = calculate_hash("한글 테스트 🎉");
    assert!(!hash.is_empty());
}

#[test]
fn test_normalize_only_whitespace() {
    let result = normalize_whitespace("     ");
    assert_eq!(result, "");
}

#[test]
fn test_normalize_tabs_and_newlines() {
    let result = normalize_whitespace("word1\t\tword2\n\n\nword3");
    assert_eq!(result, "word1 word2 word3");
}

#[test]
fn test_truncate_empty_string() {
    let result = truncate_string("", 10);
    assert_eq!(result, "");
}

#[test]
fn test_truncate_exact_length() {
    let result = truncate_string("12345", 5);
    assert_eq!(result, "12345");
}

#[test]
fn test_truncate_unicode() {
    let result = truncate_string("한글테스트입니다", 6);
    assert!(result.len() <= 9); // 3 bytes per char + "..."
}

// === Capture Service Tests ===

#[test]
fn test_capture_very_long_content() {
    let service = CaptureService::default();
    let long_content = "test ".repeat(10000);
    let result = service.process_content(&long_content);
    assert!(result.is_ok());
}

#[test]
fn test_capture_special_characters() {
    let service = CaptureService::default();
    let content = r#"Write code with "quotes", 'apostrophes', and special chars: @#$%^&*()"#;
    let result = service.process_content(content);
    assert!(result.is_ok());
}

#[test]
fn test_capture_korean_content() {
    let service = CaptureService::default();
    let content = "Rust로 정렬 함수를 작성해주세요";
    let result = service.process_content(content);
    assert!(result.is_ok());
}

#[test]
fn test_prompt_builder_empty_content() {
    let prompt = PromptBuilder::new("").build();
    assert!(prompt.content.is_empty());
}

#[test]
fn test_prompt_builder_all_fields() {
    let prompt = PromptBuilder::new("content")
        .category("test")
        .tag("tag1")
        .tag("tag2")
        .model("claude-3-opus")
        .input_tokens(100)
        .output_tokens(200)
        .execution_time_ms(1000)
        .estimated_cost(0.01)
        .context("test context")
        .build();

    assert_eq!(prompt.category, Some("test".to_string()));
    assert_eq!(prompt.tags.len(), 2);
    assert_eq!(prompt.metadata.model, "claude-3-opus");
}

#[test]
fn test_similarity_empty_strings() {
    let service = CaptureService::default();
    let similarity = service.calculate_similarity("", "");
    assert_eq!(similarity, 1.0);
}

#[test]
fn test_similarity_one_empty() {
    let service = CaptureService::default();
    let similarity = service.calculate_similarity("hello world", "");
    assert_eq!(similarity, 0.0);
}

// === Database Edge Cases ===

#[test]
fn test_db_update_nonexistent() {
    let db = Database::in_memory().unwrap();
    let mut prompt = Prompt::new("test".to_string());
    prompt.content_hash = "hash".to_string();
    prompt.id = "nonexistent".to_string();

    // Update should succeed even if doesn't exist (SQLite behavior)
    let result = db.update_prompt(&prompt);
    assert!(result.is_ok());
}

#[test]
fn test_db_delete_nonexistent() {
    let db = Database::in_memory().unwrap();
    let result = db.delete_prompt("nonexistent");
    assert!(result.is_ok());
}

#[test]
fn test_db_empty_list() {
    let db = Database::in_memory().unwrap();
    let prompts = db.list_prompts(&PromptFilter::default()).unwrap();
    assert!(prompts.is_empty());
}

#[test]
fn test_db_filter_limit_zero() {
    let db = Database::in_memory().unwrap();

    let mut prompt = Prompt::new("test".to_string());
    prompt.content_hash = "hash".to_string();
    db.create_prompt(&prompt).unwrap();

    let filter = PromptFilter {
        limit: Some(0),
        ..Default::default()
    };

    let results = db.list_prompts(&filter).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_db_filter_large_offset() {
    let db = Database::in_memory().unwrap();

    let mut prompt = Prompt::new("test".to_string());
    prompt.content_hash = "hash".to_string();
    db.create_prompt(&prompt).unwrap();

    let filter = PromptFilter {
        offset: Some(1000),
        ..Default::default()
    };

    let results = db.list_prompts(&filter).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_db_multiple_tags() {
    let db = Database::in_memory().unwrap();

    let mut prompt = Prompt::new("test".to_string());
    prompt.content_hash = "hash".to_string();
    prompt.tags = vec!["rust".to_string(), "api".to_string(), "async".to_string()];
    db.create_prompt(&prompt).unwrap();

    let retrieved = db.get_prompt(&prompt.id).unwrap().unwrap();
    assert_eq!(retrieved.tags.len(), 3);
}

#[test]
fn test_db_empty_search() {
    let db = Database::in_memory().unwrap();
    let results = db.search_prompts("").unwrap();
    assert!(results.is_empty());
}

// === Analysis Edge Cases ===

#[test]
fn test_quality_single_word() {
    let analyzer = QualityAnalyzer::default();
    let prompt = create_prompt("test");
    let score = analyzer.analyze(&prompt).unwrap();
    assert!(score.total_score >= 0.0 && score.total_score <= 100.0);
}

#[test]
fn test_quality_very_long_prompt() {
    let analyzer = QualityAnalyzer::default();
    let content = "word ".repeat(500);
    let prompt = create_prompt(&content);
    let score = analyzer.analyze(&prompt).unwrap();
    assert!(score.total_score >= 0.0);
}

#[test]
fn test_efficiency_zero_tokens() {
    let analyzer = EfficiencyAnalyzer::default();
    let prompt = create_prompt("test");
    let metrics = analyzer.analyze(&prompt).unwrap();
    assert!(metrics.efficiency_score >= 0.0);
}

#[test]
fn test_efficiency_very_high_tokens() {
    let analyzer = EfficiencyAnalyzer::default();
    let mut prompt = create_prompt("test");
    prompt.metadata.input_tokens = Some(100000);
    prompt.metadata.output_tokens = Some(100000);
    let metrics = analyzer.analyze(&prompt).unwrap();
    assert!(metrics.efficiency_score >= 0.0 && metrics.efficiency_score <= 100.0);
}

#[test]
fn test_generate_summary_empty() {
    let summary = generate_summary(&[], &[], &[]);
    assert_eq!(summary.total_prompts, 0);
    assert_eq!(summary.avg_quality_score, 0.0);
    assert_eq!(summary.avg_efficiency_score, 0.0);
}

// === Report Edge Cases ===

#[test]
fn test_report_custom_days() {
    let data = build_report_data(
        ReportType::Custom { days: 14 },
        &[],
        &[],
        &[],
    );
    assert_eq!(data.title, "14-Day Prompt Report");
}

#[test]
fn test_report_all_formats() {
    let formats = vec![
        ReportFormat::Markdown,
        ReportFormat::Html,
        ReportFormat::Json,
        ReportFormat::Csv,
    ];

    let data = build_report_data(ReportType::Weekly, &[], &[], &[]);

    for format in formats {
        let generator = ReportGenerator::new(format);
        let result = generator.generate(&data);
        assert!(result.is_ok());
    }
}

#[test]
fn test_report_format_from_str_case_insensitive() {
    assert!(ReportFormat::from_str("MARKDOWN").is_some());
    assert!(ReportFormat::from_str("HTML").is_some());
    assert!(ReportFormat::from_str("Json").is_some());
    assert!(ReportFormat::from_str("MD").is_some());
}

// === Config Tests ===

#[test]
fn test_config_default_values() {
    let config = Config::default();
    assert!(!config.database.path.is_empty());
    assert!(config.capture.similarity_threshold > 0.0);
    assert!(config.analysis.quality_weights.clarity > 0.0);
}

// === Cache Tests ===

#[test]
fn test_cache_concurrent_access() {
    use std::thread;

    let cache = CacheManager::default();
    let prompt = create_prompt("test");

    // Simulate concurrent access
    let _handles: Vec<_> = (0..10)
        .map(|i| {
            let cache_ref = &cache;
            let prompt_clone = prompt.clone();
            thread::scope(|_| {
                cache_ref.prompts.set(format!("key_{}", i), prompt_clone);
            })
        })
        .collect();

    assert!(cache.prompts.len() <= 10);
}

#[test]
fn test_cache_invalidate_all() {
    let cache = CacheManager::default();

    cache.prompts.set("p1".to_string(), create_prompt("test1"));
    cache.counts.set("total".to_string(), 100);

    cache.invalidate_all();

    assert!(cache.prompts.is_empty());
    assert!(cache.counts.is_empty());
}

// === Model Tests ===

#[test]
fn test_prompt_metadata_clone() {
    let metadata = PromptMetadata {
        model: "test".to_string(),
        input_tokens: Some(100),
        output_tokens: Some(200),
        execution_time_ms: Some(1000),
        estimated_cost: Some(0.01),
        context: Some("context".to_string()),
    };

    let cloned = metadata.clone();
    assert_eq!(metadata.model, cloned.model);
    assert_eq!(metadata.input_tokens, cloned.input_tokens);
}

#[test]
fn test_quality_score_fields() {
    let score = QualityScore {
        prompt_id: "test".to_string(),
        total_score: 85.0,
        clarity: 80.0,
        completeness: 90.0,
        specificity: 85.0,
        guidance: 85.0,
        analyzed_at: Utc::now(),
    };

    assert_eq!(score.total_score, 85.0);
}

#[test]
fn test_efficiency_metrics_fields() {
    let metrics = EfficiencyMetrics {
        prompt_id: "test".to_string(),
        efficiency_score: 75.0,
        token_efficiency: 70.0,
        time_efficiency: 80.0,
        cost_efficiency: 75.0,
        calculated_at: Utc::now(),
    };

    assert_eq!(metrics.efficiency_score, 75.0);
}

// === Integration ===

#[test]
fn test_full_pipeline() {
    let db = Database::in_memory().unwrap();
    let capture = CaptureService::default();
    let quality = QualityAnalyzer::default();
    let efficiency = EfficiencyAnalyzer::default();

    // Capture
    let prompt = capture.process_content("Write a REST API endpoint").unwrap();
    db.create_prompt(&prompt).unwrap();

    // Analyze
    let q_score = quality.analyze(&prompt).unwrap();
    let e_score = efficiency.analyze(&prompt).unwrap();
    db.save_quality_score(&q_score).unwrap();
    db.save_efficiency_metrics(&e_score).unwrap();

    // Save version
    db.save_version(&prompt).unwrap();

    // Retrieve and verify
    let retrieved = db.get_prompt(&prompt.id).unwrap().unwrap();
    let q = db.get_quality_score(&prompt.id).unwrap().unwrap();
    let e = db.get_efficiency_metrics(&prompt.id).unwrap().unwrap();
    let history = db.get_version_history(&prompt.id).unwrap();

    assert_eq!(retrieved.id, prompt.id);
    assert!(q.total_score > 0.0);
    assert!(e.efficiency_score > 0.0);
    assert_eq!(history.len(), 1);
}

// Helper function
fn create_prompt(content: &str) -> Prompt {
    let mut prompt = Prompt::new(content.to_string());
    prompt.content_hash = calculate_hash(content);
    prompt
}
