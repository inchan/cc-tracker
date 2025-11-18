//! Prompt Tracking CLI
//!
//! Command-line interface for the Prompt Tracking System.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use prompt_tracking::{
    analysis::{EfficiencyAnalyzer, QualityAnalyzer},
    capture::CaptureService,
    config::Config,
    database::{Database, PromptFilter},
    reporting::{build_report_data, ReportFormat, ReportGenerator, ReportType},
    utils::truncate_string,
};

#[derive(Parser)]
#[command(name = "prompt-tracking")]
#[command(author, version, about = "Enterprise-grade prompt tracking system for Claude Code")]
struct Cli {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture a new prompt
    Capture {
        /// Prompt content to capture
        content: Option<String>,

        /// Read prompt from file
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Category for the prompt
        #[arg(short = 'c', long)]
        category: Option<String>,

        /// Tags for the prompt (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// List stored prompts
    List {
        /// Maximum number of prompts to display
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Filter by tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Get details of a specific prompt
    Get {
        /// Prompt ID
        id: String,
    },

    /// Analyze prompt quality
    Analyze {
        /// Prompt ID to analyze (or 'all' for all prompts)
        id: String,
    },

    /// Generate reports
    Report {
        /// Report type: weekly, monthly
        #[arg(short = 't', long, default_value = "weekly")]
        report_type: String,

        /// Output format: markdown, html, json, csv
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Search prompts
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show system status and statistics
    Status,

    /// Delete a prompt
    Delete {
        /// Prompt ID to delete
        id: String,
    },
}

fn main() {
    // Initialize logger
    env_logger::init();

    let cli = Cli::parse();

    // Load configuration
    let config = if let Some(path) = &cli.config {
        Config::load(path).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
            Config::default()
        })
    } else {
        Config::default()
    };

    // Initialize database
    let db = match Database::new(&config.database.path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error: Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Execute command
    let result = match cli.command {
        Commands::Capture {
            content,
            file,
            category,
            tags,
        } => cmd_capture(&db, &config, content, file, category, tags),

        Commands::List {
            limit,
            category,
            tags,
        } => cmd_list(&db, limit, category, tags),

        Commands::Get { id } => cmd_get(&db, &id),

        Commands::Analyze { id } => cmd_analyze(&db, &config, &id),

        Commands::Report {
            report_type,
            format,
            output,
        } => cmd_report(&db, &report_type, &format, output),

        Commands::Search { query, limit } => cmd_search(&db, &query, limit),

        Commands::Status => cmd_status(&db),

        Commands::Delete { id } => cmd_delete(&db, &id),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_capture(
    db: &Database,
    config: &Config,
    content: Option<String>,
    file: Option<PathBuf>,
    category: Option<String>,
    tags: Option<String>,
) -> Result<(), String> {
    let capture_service = CaptureService::new(config.capture.similarity_threshold);

    // Get content from argument or file
    let prompt_content = if let Some(c) = content {
        c
    } else if let Some(f) = file {
        std::fs::read_to_string(&f)
            .map_err(|e| format!("Failed to read file: {}", e))?
    } else {
        return Err("Either content or --file must be provided".to_string());
    };

    // Process the prompt
    let mut prompt = capture_service
        .process_content(&prompt_content)
        .map_err(|e| format!("Failed to process content: {}", e))?;

    // Override category if provided
    if let Some(cat) = category {
        prompt.category = Some(cat);
    }

    // Add tags if provided
    if let Some(tag_str) = tags {
        let additional_tags: Vec<String> = tag_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        prompt.tags.extend(additional_tags);
    }

    // Check for duplicates
    if config.capture.deduplicate {
        if let Ok(Some(existing)) = db.find_by_hash(&prompt.content_hash) {
            println!("Duplicate detected! Existing prompt ID: {}", existing.id);
            return Ok(());
        }
    }

    // Save to database
    db.create_prompt(&prompt)
        .map_err(|e| format!("Failed to save prompt: {}", e))?;

    println!("Prompt captured successfully!");
    println!("ID: {}", prompt.id);
    if let Some(cat) = &prompt.category {
        println!("Category: {}", cat);
    }
    if !prompt.tags.is_empty() {
        println!("Tags: {}", prompt.tags.join(", "));
    }

    // Auto-analyze if enabled
    if config.analysis.auto_analyze {
        let quality_analyzer = QualityAnalyzer::new(config.analysis.quality_weights.clone());
        let efficiency_analyzer = EfficiencyAnalyzer::default();

        if let Ok(quality_score) = quality_analyzer.analyze(&prompt) {
            let _ = db.save_quality_score(&quality_score);
            println!("Quality Score: {:.1}", quality_score.total_score);
        }

        if let Ok(efficiency) = efficiency_analyzer.analyze(&prompt) {
            let _ = db.save_efficiency_metrics(&efficiency);
            println!("Efficiency Score: {:.1}", efficiency.efficiency_score);
        }
    }

    Ok(())
}

fn cmd_list(
    db: &Database,
    limit: usize,
    category: Option<String>,
    tags: Option<String>,
) -> Result<(), String> {
    let filter = PromptFilter {
        category,
        tags: tags
            .map(|t| {
                t.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        limit: Some(limit),
        ..Default::default()
    };

    let prompts = db
        .list_prompts(&filter)
        .map_err(|e| format!("Failed to list prompts: {}", e))?;

    if prompts.is_empty() {
        println!("No prompts found.");
        return Ok(());
    }

    println!("{:<36} {:<15} {:<20} {}", "ID", "Category", "Tags", "Preview");
    println!("{}", "-".repeat(100));

    for prompt in prompts {
        let category = prompt.category.as_deref().unwrap_or("-");
        let tags = if prompt.tags.is_empty() {
            "-".to_string()
        } else {
            prompt.tags.join(", ")
        };
        let preview = truncate_string(&prompt.content, 30);

        println!(
            "{:<36} {:<15} {:<20} {}",
            prompt.id, category, truncate_string(&tags, 18), preview
        );
    }

    Ok(())
}

fn cmd_get(db: &Database, id: &str) -> Result<(), String> {
    let prompt = db
        .get_prompt(id)
        .map_err(|e| format!("Failed to get prompt: {}", e))?
        .ok_or_else(|| format!("Prompt not found: {}", id))?;

    println!("ID: {}", prompt.id);
    println!("Created: {}", prompt.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Updated: {}", prompt.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Hash: {}", prompt.content_hash);

    if let Some(cat) = &prompt.category {
        println!("Category: {}", cat);
    }

    if !prompt.tags.is_empty() {
        println!("Tags: {}", prompt.tags.join(", "));
    }

    println!("\nMetadata:");
    println!("  Model: {}", prompt.metadata.model);
    if let Some(tokens) = prompt.metadata.input_tokens {
        println!("  Input Tokens: {}", tokens);
    }
    if let Some(tokens) = prompt.metadata.output_tokens {
        println!("  Output Tokens: {}", tokens);
    }
    if let Some(time) = prompt.metadata.execution_time_ms {
        println!("  Execution Time: {}ms", time);
    }
    if let Some(cost) = prompt.metadata.estimated_cost {
        println!("  Estimated Cost: ${:.4}", cost);
    }
    if let Some(context) = &prompt.metadata.context {
        println!("  Context: {}", context);
    }

    println!("\nContent:\n{}", prompt.content);

    // Show quality score if available
    if let Ok(Some(score)) = db.get_quality_score(id) {
        println!("\nQuality Analysis:");
        println!("  Total Score: {:.1}", score.total_score);
        println!("  Clarity: {:.1}", score.clarity);
        println!("  Completeness: {:.1}", score.completeness);
        println!("  Specificity: {:.1}", score.specificity);
        println!("  Guidance: {:.1}", score.guidance);
    }

    // Show efficiency metrics if available
    if let Ok(Some(metrics)) = db.get_efficiency_metrics(id) {
        println!("\nEfficiency Metrics:");
        println!("  Efficiency Score: {:.1}", metrics.efficiency_score);
        println!("  Token Efficiency: {:.1}", metrics.token_efficiency);
        println!("  Time Efficiency: {:.1}", metrics.time_efficiency);
        println!("  Cost Efficiency: {:.1}", metrics.cost_efficiency);
    }

    Ok(())
}

fn cmd_analyze(db: &Database, config: &Config, id: &str) -> Result<(), String> {
    let quality_analyzer = QualityAnalyzer::new(config.analysis.quality_weights.clone());
    let efficiency_analyzer = EfficiencyAnalyzer::default();

    if id == "all" {
        // Analyze all prompts
        let prompts = db
            .list_prompts(&PromptFilter::default())
            .map_err(|e| format!("Failed to list prompts: {}", e))?;

        if prompts.is_empty() {
            println!("No prompts to analyze.");
            return Ok(());
        }

        println!("Analyzing {} prompts...\n", prompts.len());

        let mut total_quality = 0.0;
        let mut total_efficiency = 0.0;

        for prompt in &prompts {
            if let Ok(quality_score) = quality_analyzer.analyze(prompt) {
                db.save_quality_score(&quality_score)
                    .map_err(|e| format!("Failed to save quality score: {}", e))?;
                total_quality += quality_score.total_score;
            }

            if let Ok(efficiency) = efficiency_analyzer.analyze(prompt) {
                db.save_efficiency_metrics(&efficiency)
                    .map_err(|e| format!("Failed to save efficiency metrics: {}", e))?;
                total_efficiency += efficiency.efficiency_score;
            }
        }

        let avg_quality = total_quality / prompts.len() as f64;
        let avg_efficiency = total_efficiency / prompts.len() as f64;

        println!("Analysis complete!");
        println!("Average Quality Score: {:.1}", avg_quality);
        println!("Average Efficiency Score: {:.1}", avg_efficiency);
    } else {
        // Analyze single prompt
        let prompt = db
            .get_prompt(id)
            .map_err(|e| format!("Failed to get prompt: {}", e))?
            .ok_or_else(|| format!("Prompt not found: {}", id))?;

        let quality_score = quality_analyzer
            .analyze(&prompt)
            .map_err(|e| format!("Failed to analyze quality: {}", e))?;

        let efficiency = efficiency_analyzer
            .analyze(&prompt)
            .map_err(|e| format!("Failed to analyze efficiency: {}", e))?;

        // Save results
        db.save_quality_score(&quality_score)
            .map_err(|e| format!("Failed to save quality score: {}", e))?;
        db.save_efficiency_metrics(&efficiency)
            .map_err(|e| format!("Failed to save efficiency metrics: {}", e))?;

        println!("Quality Analysis:");
        println!("  Total Score: {:.1}", quality_score.total_score);
        println!("  Clarity: {:.1}", quality_score.clarity);
        println!("  Completeness: {:.1}", quality_score.completeness);
        println!("  Specificity: {:.1}", quality_score.specificity);
        println!("  Guidance: {:.1}", quality_score.guidance);

        println!("\nEfficiency Metrics:");
        println!("  Efficiency Score: {:.1}", efficiency.efficiency_score);
        println!("  Token Efficiency: {:.1}", efficiency.token_efficiency);
        println!("  Time Efficiency: {:.1}", efficiency.time_efficiency);
        println!("  Cost Efficiency: {:.1}", efficiency.cost_efficiency);
    }

    Ok(())
}

fn cmd_report(
    db: &Database,
    report_type: &str,
    format: &str,
    output: Option<PathBuf>,
) -> Result<(), String> {
    // Parse report type
    let rtype = match report_type.to_lowercase().as_str() {
        "weekly" => ReportType::Weekly,
        "monthly" => ReportType::Monthly,
        _ => return Err(format!("Invalid report type: {}. Use 'weekly' or 'monthly'.", report_type)),
    };

    // Parse format
    let rformat = ReportFormat::from_str(format)
        .ok_or_else(|| format!("Invalid format: {}. Use 'markdown', 'html', 'json', or 'csv'.", format))?;

    // Get all data
    let prompts = db
        .list_prompts(&PromptFilter::default())
        .map_err(|e| format!("Failed to list prompts: {}", e))?;

    let quality_scores = db
        .get_all_quality_scores()
        .map_err(|e| format!("Failed to get quality scores: {}", e))?;

    // Get efficiency metrics (we need to collect them)
    let mut efficiency_metrics = Vec::new();
    for prompt in &prompts {
        if let Ok(Some(metrics)) = db.get_efficiency_metrics(&prompt.id) {
            efficiency_metrics.push(metrics);
        }
    }

    // Build report data
    let report_data = build_report_data(rtype, &prompts, &quality_scores, &efficiency_metrics);

    // Generate report
    let generator = ReportGenerator::new(rformat);
    let report = generator
        .generate(&report_data)
        .map_err(|e| format!("Failed to generate report: {}", e))?;

    // Output
    if let Some(path) = output {
        generator
            .save_to_file(&report_data, &path)
            .map_err(|e| format!("Failed to save report: {}", e))?;
        println!("Report saved to: {}", path.display());
    } else {
        println!("{}", report);
    }

    Ok(())
}

fn cmd_search(db: &Database, query: &str, limit: usize) -> Result<(), String> {
    let filter = PromptFilter {
        search_query: Some(query.to_string()),
        limit: Some(limit),
        ..Default::default()
    };

    let prompts = db
        .list_prompts(&filter)
        .map_err(|e| format!("Failed to search prompts: {}", e))?;

    if prompts.is_empty() {
        println!("No prompts found matching '{}'", query);
        return Ok(());
    }

    println!("Found {} prompt(s) matching '{}':\n", prompts.len(), query);

    for prompt in prompts {
        let category = prompt.category.as_deref().unwrap_or("-");
        let preview = truncate_string(&prompt.content, 50);

        println!("ID: {}", prompt.id);
        println!("Category: {}", category);
        println!("Preview: {}", preview);
        println!("Created: {}", prompt.created_at.format("%Y-%m-%d %H:%M"));
        println!();
    }

    Ok(())
}

fn cmd_status(db: &Database) -> Result<(), String> {
    let total_prompts = db
        .count_prompts()
        .map_err(|e| format!("Failed to count prompts: {}", e))?;

    let quality_scores = db
        .get_all_quality_scores()
        .map_err(|e| format!("Failed to get quality scores: {}", e))?;

    println!("Prompt Tracking System v{}", prompt_tracking::VERSION);
    println!();
    println!("Database Status:");
    println!("  Total Prompts: {}", total_prompts);
    println!("  Analyzed Prompts: {}", quality_scores.len());

    if !quality_scores.is_empty() {
        let avg_quality: f64 =
            quality_scores.iter().map(|s| s.total_score).sum::<f64>() / quality_scores.len() as f64;
        println!("  Average Quality Score: {:.1}", avg_quality);
    }

    // Category breakdown
    let prompts = db
        .list_prompts(&PromptFilter::default())
        .map_err(|e| format!("Failed to list prompts: {}", e))?;

    if !prompts.is_empty() {
        use std::collections::HashMap;
        let mut categories: HashMap<String, usize> = HashMap::new();

        for prompt in &prompts {
            let cat = prompt.category.clone().unwrap_or_else(|| "uncategorized".to_string());
            *categories.entry(cat).or_insert(0) += 1;
        }

        println!("\nCategories:");
        let mut sorted: Vec<_> = categories.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (cat, count) in sorted {
            println!("  {}: {}", cat, count);
        }
    }

    Ok(())
}

fn cmd_delete(db: &Database, id: &str) -> Result<(), String> {
    // Check if prompt exists
    let _ = db
        .get_prompt(id)
        .map_err(|e| format!("Failed to get prompt: {}", e))?
        .ok_or_else(|| format!("Prompt not found: {}", id))?;

    db.delete_prompt(id)
        .map_err(|e| format!("Failed to delete prompt: {}", e))?;

    println!("Prompt {} deleted successfully.", id);

    Ok(())
}
