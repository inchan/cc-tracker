//! CLI end-to-end tests

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn get_cmd() -> Command {
    Command::cargo_bin("prompt-tracking").unwrap()
}

fn setup_test_db() -> TempDir {
    TempDir::new().unwrap()
}

fn create_test_config(db_path: &std::path::Path, auto_analyze: bool) -> String {
    format!(
        r#"
database:
  path: "{}"
  auto_backup: false
  backup_interval: 24
capture:
  watch_directory: "/tmp"
  auto_capture: false
  deduplicate: false
  similarity_threshold: 0.9
analysis:
  auto_analyze: {}
  quality_weights:
    clarity: 0.3
    completeness: 0.3
    specificity: 0.2
    guidance: 0.2
reporting:
  auto_report: false
  formats: ["markdown"]
  output_dir: "/tmp"
categories:
  - code-generation
  - documentation
  - testing
"#,
        db_path.display(),
        auto_analyze
    )
}

#[test]
fn test_help() {
    let mut cmd = get_cmd();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Enterprise-grade prompt tracking system"));
}

#[test]
fn test_version() {
    let mut cmd = get_cmd();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("prompt-tracking"));
}

#[test]
fn test_status_command() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Prompt Tracking System"))
        .stdout(predicate::str::contains("Total Prompts:"));
}

#[test]
fn test_capture_and_list() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Capture a prompt
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Write a function to calculate fibonacci numbers")
        .arg("--category")
        .arg("code-generation")
        .assert()
        .success()
        .stdout(predicate::str::contains("Prompt captured successfully"));

    // List prompts
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("code-generation"));
}

#[test]
fn test_capture_with_tags() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Capture with tags
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Build a REST API in Python using Flask")
        .arg("--tags")
        .arg("python,flask,api")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tags: python, flask, api"));
}

#[test]
fn test_search_command() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Capture prompts
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("How to implement binary search in Rust")
        .assert()
        .success();

    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Explain quicksort algorithm")
        .assert()
        .success();

    // Search for "binary"
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("search")
        .arg("binary")
        .assert()
        .success()
        .stdout(predicate::str::contains("binary search"));
}

#[test]
fn test_analyze_command() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Capture a prompt
    let output = get_cmd()
        .arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Write a detailed function to parse JSON data with error handling")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .lines()
        .find(|line| line.starts_with("ID:"))
        .and_then(|line| line.strip_prefix("ID: "))
        .expect("Should find ID in output")
        .trim();

    // Analyze the prompt
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("analyze")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Quality Analysis"))
        .stdout(predicate::str::contains("Total Score"))
        .stdout(predicate::str::contains("Efficiency Metrics"));
}

#[test]
fn test_report_command() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, true)).unwrap();

    // Capture a prompt
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Test prompt for report generation")
        .assert()
        .success();

    // Generate weekly report
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("report")
        .arg("--report-type")
        .arg("weekly")
        .arg("--format")
        .arg("markdown")
        .assert()
        .success()
        .stdout(predicate::str::contains("Weekly Prompt Report"));
}

#[test]
fn test_trends_command() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, true)).unwrap();

    // Capture some prompts
    for i in 0..3 {
        let mut cmd = get_cmd();
        cmd.arg("--config")
            .arg(&config_path)
            .arg("capture")
            .arg(format!("Test prompt number {}", i))
            .assert()
            .success();
    }

    // Show trends
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("trends")
        .arg("--days")
        .arg("7")
        .assert()
        .success()
        .stdout(predicate::str::contains("Daily Trends"));

    // Show categories
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("trends")
        .arg("--categories")
        .assert()
        .success()
        .stdout(predicate::str::contains("Category Distribution"));
}

#[test]
fn test_export_import() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let export_path = temp_dir.path().join("export.json");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Capture prompts
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("First prompt to export")
        .assert()
        .success();

    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Second prompt to export")
        .assert()
        .success();

    // Export
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("export")
        .arg(&export_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Data exported to"));

    // Verify export file exists
    assert!(export_path.exists());

    // Create new database for import
    let db_path2 = temp_dir.path().join("test2.db");
    let config_path2 = temp_dir.path().join("config2.yaml");
    fs::write(&config_path2, create_test_config(&db_path2, false)).unwrap();

    // Import
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path2)
        .arg("import")
        .arg(&export_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully imported"));
}

#[test]
fn test_update_and_history() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Capture a prompt
    let output = get_cmd()
        .arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Original prompt content")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .lines()
        .find(|line| line.starts_with("ID:"))
        .and_then(|line| line.strip_prefix("ID: "))
        .expect("Should find ID in output")
        .trim();

    // Update the prompt
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("update")
        .arg(id)
        .arg("--category")
        .arg("updated-category")
        .assert()
        .success()
        .stdout(predicate::str::contains("updated successfully"));

    // Check history
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("history")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("Version history"));
}

#[test]
fn test_archive_unarchive() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Capture a prompt
    let output = get_cmd()
        .arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Prompt to archive")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .lines()
        .find(|line| line.starts_with("ID:"))
        .and_then(|line| line.strip_prefix("ID: "))
        .expect("Should find ID in output")
        .trim();

    // Archive
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("archive")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("archived successfully"));

    // Unarchive
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("unarchive")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("unarchived successfully"));
}

#[test]
fn test_delete_command() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("test.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Capture a prompt
    let output = get_cmd()
        .arg("--config")
        .arg(&config_path)
        .arg("capture")
        .arg("Prompt to delete")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .lines()
        .find(|line| line.starts_with("ID:"))
        .and_then(|line| line.strip_prefix("ID: "))
        .expect("Should find ID in output")
        .trim();

    // Delete
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("delete")
        .arg(id)
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted successfully"));

    // Verify prompt is gone
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("get")
        .arg(id)
        .assert()
        .failure();
}

#[test]
fn test_init_command() {
    let temp_dir = setup_test_db();
    let db_path = temp_dir.path().join("new_db.db");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, create_test_config(&db_path, false)).unwrap();

    // Init
    let mut cmd = get_cmd();
    cmd.arg("--config")
        .arg(&config_path)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Database initialized"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = get_cmd();
    cmd.arg("invalid-command").assert().failure();
}

#[test]
fn test_missing_required_arg() {
    let mut cmd = get_cmd();
    cmd.arg("get").assert().failure();
}
