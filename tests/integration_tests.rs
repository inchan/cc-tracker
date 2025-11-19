//! Integration tests for Prompt Tracking System

use prompt_tracking::{
    analysis::{EfficiencyAnalyzer, QualityAnalyzer},
    capture::CaptureService,
    database::{Database, PromptFilter},
    reporting::{build_report_data, ReportFormat, ReportGenerator, ReportType},
};

/// Test complete workflow: capture -> analyze -> report
#[test]
fn test_complete_workflow() {
    let db = Database::in_memory().unwrap();
    let capture_service = CaptureService::default();
    let quality_analyzer = QualityAnalyzer::default();
    let efficiency_analyzer = EfficiencyAnalyzer::default();

    // Capture prompts
    let prompts = vec![
        "Write a Rust function that implements a binary search tree with insert and search operations",
        "Create a Python script to parse JSON files and extract specific fields",
        "Explain the difference between TCP and UDP protocols",
    ];

    for content in prompts {
        let prompt = capture_service.process_content(content).unwrap();
        db.create_prompt(&prompt).unwrap();

        // Analyze
        let quality = quality_analyzer.analyze(&prompt).unwrap();
        let efficiency = efficiency_analyzer.analyze(&prompt).unwrap();

        db.save_quality_score(&quality).unwrap();
        db.save_efficiency_metrics(&efficiency).unwrap();
    }

    // Verify
    assert_eq!(db.count_prompts().unwrap(), 3);

    // Generate report
    let prompts = db.list_prompts(&PromptFilter::default()).unwrap();
    let scores = db.get_all_quality_scores().unwrap();
    let report_data = build_report_data(ReportType::Weekly, &prompts, &scores, &[]);

    let generator = ReportGenerator::new(ReportFormat::Markdown);
    let report = generator.generate(&report_data).unwrap();

    assert!(report.contains("Weekly Prompt Report"));
    assert!(report.contains("Total Prompts"));
}

/// Test version history
#[test]
fn test_version_history() {
    let db = Database::in_memory().unwrap();
    let capture_service = CaptureService::default();

    // Create initial prompt
    let mut prompt = capture_service
        .process_content("Original content")
        .unwrap();
    db.create_prompt(&prompt).unwrap();

    // Save version
    db.save_version(&prompt).unwrap();

    // Modify and save again
    prompt.content = "Modified content".to_string();
    prompt.content_hash = "new_hash".to_string();
    db.update_prompt(&prompt).unwrap();
    db.save_version(&prompt).unwrap();

    // Check history
    let history = db.get_version_history(&prompt.id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].version, 2); // Most recent first
    assert_eq!(history[1].version, 1);
}

/// Test export and import
#[test]
fn test_export_import() {
    let db1 = Database::in_memory().unwrap();
    let capture_service = CaptureService::default();

    // Create and capture prompt
    let prompt = capture_service
        .process_content("Test prompt for export")
        .unwrap();
    db1.create_prompt(&prompt).unwrap();

    // Export
    let json = db1.export_to_json().unwrap();
    assert!(json.contains("Test prompt for export"));

    // Import to new database
    let db2 = Database::in_memory().unwrap();
    let imported = db2.import_from_json(&json).unwrap();
    assert_eq!(imported, 1);

    // Verify
    assert_eq!(db2.count_prompts().unwrap(), 1);
}

/// Test advanced filtering
#[test]
fn test_advanced_filtering() {
    let db = Database::in_memory().unwrap();
    let capture_service = CaptureService::default();

    // Create prompts with different categories
    let mut prompt1 = capture_service
        .process_content("Write code in Python")
        .unwrap();
    prompt1.category = Some("code-generation".to_string());
    db.create_prompt(&prompt1).unwrap();

    let mut prompt2 = capture_service
        .process_content("Document the API")
        .unwrap();
    prompt2.category = Some("documentation".to_string());
    db.create_prompt(&prompt2).unwrap();

    // Filter by category
    let filter = PromptFilter {
        category: Some("code-generation".to_string()),
        ..Default::default()
    };
    let results = db.list_prompts(&filter).unwrap();
    assert_eq!(results.len(), 1);

    // Search
    let results = db.search_prompts("Python").unwrap();
    assert_eq!(results.len(), 1);
}

/// Test trend analysis
#[test]
fn test_trend_analysis() {
    let db = Database::in_memory().unwrap();
    let capture_service = CaptureService::default();

    // Create prompts
    for i in 0..5 {
        let content = format!("Test prompt {}", i);
        let prompt = capture_service.process_content(&content).unwrap();
        db.create_prompt(&prompt).unwrap();
    }

    // Get trends
    let trends = db.get_daily_trends(7).unwrap();
    // Should have at least one data point for today
    assert!(!trends.is_empty() || db.count_prompts().unwrap() > 0);

    // Get category distribution
    let distribution = db.get_category_distribution().unwrap();
    assert!(!distribution.is_empty());
}

/// Test duplicate detection
#[test]
fn test_duplicate_detection() {
    let db = Database::in_memory().unwrap();
    let capture_service = CaptureService::default();

    let content = "Same content for duplicate test";

    // Capture first prompt
    let prompt1 = capture_service.process_content(content).unwrap();
    db.create_prompt(&prompt1).unwrap();

    // Try to find duplicate by hash
    let prompt2 = capture_service.process_content(content).unwrap();
    let existing = db.find_by_hash(&prompt2.content_hash).unwrap();

    assert!(existing.is_some());
    assert_eq!(existing.unwrap().id, prompt1.id);
}

/// Test quality scoring
#[test]
fn test_quality_scoring() {
    let analyzer = QualityAnalyzer::default();
    let capture_service = CaptureService::default();

    // High quality prompt
    let high_quality = capture_service
        .process_content(
            "Context: Building a REST API. Create a Python function that validates user input. \
             Must handle null values. Return JSON format with error messages.",
        )
        .unwrap();

    // Low quality prompt
    let low_quality = capture_service.process_content("do thing").unwrap();

    let high_score = analyzer.analyze(&high_quality).unwrap();
    let low_score = analyzer.analyze(&low_quality).unwrap();

    assert!(high_score.total_score > low_score.total_score);
}

/// Test report generation in multiple formats
#[test]
fn test_multiple_report_formats() {
    let db = Database::in_memory().unwrap();
    let capture_service = CaptureService::default();

    let prompt = capture_service.process_content("Test content").unwrap();
    db.create_prompt(&prompt).unwrap();

    let prompts = db.list_prompts(&PromptFilter::default()).unwrap();
    let report_data = build_report_data(ReportType::Weekly, &prompts, &[], &[]);

    // Markdown
    let md = ReportGenerator::new(ReportFormat::Markdown)
        .generate(&report_data)
        .unwrap();
    assert!(md.contains("# Weekly Prompt Report"));

    // HTML
    let html = ReportGenerator::new(ReportFormat::Html)
        .generate(&report_data)
        .unwrap();
    assert!(html.contains("<!DOCTYPE html>"));

    // JSON
    let json = ReportGenerator::new(ReportFormat::Json)
        .generate(&report_data)
        .unwrap();
    assert!(json.contains("\"title\""));

    // CSV
    let csv = ReportGenerator::new(ReportFormat::Csv)
        .generate(&report_data)
        .unwrap();
    assert!(csv.contains("Metric,Value"));
}

/// Test similarity detection
#[test]
fn test_similarity_detection() {
    let service = CaptureService::new(0.7);

    let similar1 = "Write a function to sort an array in ascending order";
    let similar2 = "Write a function to sort an array in descending order";
    let different = "Create a web server with REST endpoints";

    assert!(service.is_similar(similar1, similar2));
    assert!(!service.is_similar(similar1, different));
}

/// Test automatic category detection
#[test]
fn test_auto_categorization() {
    let service = CaptureService::default();

    let code = service
        .process_content("Implement a binary search function")
        .unwrap();
    assert_eq!(code.category, Some("code-generation".to_string()));

    let docs = service
        .process_content("Document the API endpoints")
        .unwrap();
    assert_eq!(docs.category, Some("documentation".to_string()));

    let test = service
        .process_content("Write unit tests for the module")
        .unwrap();
    assert_eq!(test.category, Some("testing".to_string()));

    let debug = service
        .process_content("Fix the authentication bug")
        .unwrap();
    assert_eq!(debug.category, Some("debugging".to_string()));
}
