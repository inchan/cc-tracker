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
    filter::parse_filter_query,
    reporting::{build_report_data, ReportGenerator, ReportType},
    utils::truncate_string,
    watcher::{FileWatcher, WatcherConfig},
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

        /// Watch directory for new prompt files
        #[arg(short, long)]
        watch: bool,

        /// Directory to watch (requires --watch)
        #[arg(long)]
        watch_dir: Option<PathBuf>,
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

    /// Update a prompt
    Update {
        /// Prompt ID to update
        id: String,

        /// New category
        #[arg(short, long)]
        category: Option<String>,

        /// New content
        #[arg(long)]
        content: Option<String>,

        /// Add tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// New context
        #[arg(long)]
        context: Option<String>,
    },

    /// Show version history of a prompt
    History {
        /// Prompt ID
        id: String,
    },

    /// Revert prompt to a previous version
    Revert {
        /// Prompt ID
        id: String,

        /// Version number to revert to
        #[arg(short, long)]
        to: i32,
    },

    /// Show trends and statistics
    Trends {
        /// Number of days to analyze
        #[arg(short, long, default_value = "30")]
        days: i32,

        /// Show category distribution
        #[arg(long)]
        categories: bool,
    },

    /// Export data to file
    Export {
        /// Output file path
        output: PathBuf,

        /// Export format (json)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Import data from file
    Import {
        /// Input file path
        input: PathBuf,

        /// Skip duplicates
        #[arg(long)]
        skip_duplicates: bool,
    },

    /// Initialize database and configuration
    Init {
        /// Force re-initialization
        #[arg(short, long)]
        force: bool,
    },

    /// Archive a prompt
    Archive {
        /// Prompt ID to archive
        id: String,
    },

    /// Unarchive a prompt
    Unarchive {
        /// Prompt ID to unarchive
        id: String,
    },

    /// Query prompts with advanced filter syntax
    ///
    /// Supports filters like: category:code tag:rust quality:>80 date:>2024-01-01
    Query {
        /// Filter query string (e.g., "category:code tag:rust quality:>80")
        query: String,
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
            watch,
            watch_dir,
        } => {
            if watch {
                cmd_watch(&db, &config, watch_dir, category, tags)
            } else {
                cmd_capture(&db, &config, content, file, category, tags)
            }
        }

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

        Commands::Update {
            id,
            category,
            content,
            tags,
            context,
        } => cmd_update(&db, &id, category, content, tags, context),

        Commands::History { id } => cmd_history(&db, &id),

        Commands::Revert { id, to } => cmd_revert(&db, &id, to),

        Commands::Trends { days, categories } => cmd_trends(&db, days, categories),

        Commands::Export { output, format } => cmd_export(&db, &output, &format),

        Commands::Import {
            input,
            skip_duplicates,
        } => cmd_import(&db, &input, skip_duplicates),

        Commands::Init { force } => cmd_init(&config, force),

        Commands::Archive { id } => cmd_archive(&db, &id),

        Commands::Unarchive { id } => cmd_unarchive(&db, &id),

        Commands::Query { query } => cmd_query(&db, &query),
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
    let rformat = format
        .parse()
        .map_err(|_| format!("Invalid format: {}. Use 'markdown', 'html', 'json', or 'csv'.", format))?;

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

fn cmd_update(
    db: &Database,
    id: &str,
    category: Option<String>,
    content: Option<String>,
    tags: Option<String>,
    context: Option<String>,
) -> Result<(), String> {
    // Get existing prompt
    let mut prompt = db
        .get_prompt(id)
        .map_err(|e| format!("Failed to get prompt: {}", e))?
        .ok_or_else(|| format!("Prompt not found: {}", id))?;

    // Save current version before updating
    db.save_version(&prompt)
        .map_err(|e| format!("Failed to save version: {}", e))?;

    let mut updated = false;

    // Update fields if provided
    if let Some(cat) = category {
        prompt.category = Some(cat);
        updated = true;
    }

    if let Some(new_content) = content {
        prompt.content = new_content;
        prompt.content_hash = prompt_tracking::utils::calculate_hash(&prompt.content);
        updated = true;
    }

    if let Some(tag_str) = tags {
        let new_tags: Vec<String> = tag_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        prompt.tags.extend(new_tags);
        updated = true;
    }

    if let Some(ctx) = context {
        prompt.metadata.context = Some(ctx);
        updated = true;
    }

    if !updated {
        return Err("No updates specified. Use --category, --content, --tags, or --context.".to_string());
    }

    // Update in database
    db.update_prompt(&prompt)
        .map_err(|e| format!("Failed to update prompt: {}", e))?;

    println!("Prompt {} updated successfully.", id);

    Ok(())
}

fn cmd_history(db: &Database, id: &str) -> Result<(), String> {
    // Check if prompt exists
    let _ = db
        .get_prompt(id)
        .map_err(|e| format!("Failed to get prompt: {}", e))?
        .ok_or_else(|| format!("Prompt not found: {}", id))?;

    let history = db
        .get_version_history(id)
        .map_err(|e| format!("Failed to get history: {}", e))?;

    if history.is_empty() {
        println!("No version history for prompt {}.", id);
        return Ok(());
    }

    println!("Version history for {}:\n", id);
    println!("{:<8} {:<25} {}", "Version", "Created At", "Preview");
    println!("{}", "-".repeat(80));

    for vh in history {
        let preview = truncate_string(&vh.content, 40);
        println!("{:<8} {:<25} {}", vh.version, vh.created_at.format("%Y-%m-%d %H:%M:%S"), preview);
    }

    Ok(())
}

fn cmd_revert(db: &Database, id: &str, version: i32) -> Result<(), String> {
    // Check if prompt exists
    let _ = db
        .get_prompt(id)
        .map_err(|e| format!("Failed to get prompt: {}", e))?
        .ok_or_else(|| format!("Prompt not found: {}", id))?;

    db.restore_version(id, version)
        .map_err(|e| format!("Failed to revert: {}", e))?;

    println!("Prompt {} reverted to version {}.", id, version);

    Ok(())
}

fn cmd_trends(db: &Database, days: i32, show_categories: bool) -> Result<(), String> {
    if show_categories {
        // Show category distribution
        let distribution = db
            .get_category_distribution()
            .map_err(|e| format!("Failed to get distribution: {}", e))?;

        if distribution.is_empty() {
            println!("No data available for category distribution.");
            return Ok(());
        }

        println!("Category Distribution:\n");
        println!("{:<20} {:<10} {}", "Category", "Count", "Bar");
        println!("{}", "-".repeat(60));

        let max_count = distribution.iter().map(|(_, c)| *c).max().unwrap_or(1) as f64;

        for (category, count) in distribution {
            let bar_len = ((count as f64 / max_count) * 30.0) as usize;
            let bar = "█".repeat(bar_len);
            println!("{:<20} {:<10} {}", category, count, bar);
        }
    } else {
        // Show daily trends
        let trends = db
            .get_daily_trends(days)
            .map_err(|e| format!("Failed to get trends: {}", e))?;

        if trends.is_empty() {
            println!("No data available for the last {} days.", days);
            return Ok(());
        }

        println!("Daily Trends (last {} days):\n", days);
        println!("{:<12} {:<8} {:<15} {:<15}", "Date", "Count", "Avg Quality", "Avg Efficiency");
        println!("{}", "-".repeat(55));

        for trend in trends {
            println!(
                "{:<12} {:<8} {:<15.1} {:<15.1}",
                trend.date, trend.count, trend.avg_quality, trend.avg_efficiency
            );
        }
    }

    Ok(())
}

fn cmd_export(db: &Database, output: &PathBuf, format: &str) -> Result<(), String> {
    match format.to_lowercase().as_str() {
        "json" => {
            let json_str = db
                .export_to_json()
                .map_err(|e| format!("Failed to export: {}", e))?;

            std::fs::write(output, json_str)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            println!("Data exported to: {}", output.display());
        }
        _ => {
            return Err(format!("Unsupported export format: {}. Use 'json'.", format));
        }
    }

    Ok(())
}

fn cmd_import(db: &Database, input: &PathBuf, _skip_duplicates: bool) -> Result<(), String> {
    let json_str = std::fs::read_to_string(input)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let imported = db
        .import_from_json(&json_str)
        .map_err(|e| format!("Failed to import: {}", e))?;

    println!("Successfully imported {} prompts from: {}", imported, input.display());

    Ok(())
}

fn cmd_init(config: &Config, force: bool) -> Result<(), String> {
    let db_path = std::path::Path::new(&config.database.path);

    if db_path.exists() && !force {
        return Err(format!(
            "Database already exists at {}. Use --force to reinitialize.",
            config.database.path
        ));
    }

    // Create parent directories if needed
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Remove existing database if force
    if db_path.exists() && force {
        std::fs::remove_file(db_path)
            .map_err(|e| format!("Failed to remove existing database: {}", e))?;
    }

    // Initialize new database
    let _db = Database::new(&config.database.path)
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    println!("Database initialized at: {}", config.database.path);
    println!("Configuration loaded from defaults.");
    println!("\nYou can now use:");
    println!("  prompt-tracking capture \"Your prompt\"");
    println!("  prompt-tracking list");
    println!("  prompt-tracking status");

    Ok(())
}

fn cmd_archive(db: &Database, id: &str) -> Result<(), String> {
    db.archive_prompt(id)
        .map_err(|e| format!("Failed to archive prompt: {}", e))?;

    println!("Prompt {} archived successfully.", id);

    Ok(())
}

fn cmd_unarchive(db: &Database, id: &str) -> Result<(), String> {
    db.unarchive_prompt(id)
        .map_err(|e| format!("Failed to unarchive prompt: {}", e))?;

    println!("Prompt {} unarchived successfully.", id);

    Ok(())
}

fn cmd_watch(
    db: &Database,
    config: &Config,
    watch_dir: Option<PathBuf>,
    _category: Option<String>,
    _tags: Option<String>,
) -> Result<(), String> {
    let watch_path = watch_dir
        .unwrap_or_else(|| PathBuf::from(&config.capture.watch_directory));

    let watcher_config = WatcherConfig {
        watch_path: watch_path.clone(),
        recursive: true,
        file_extensions: vec!["txt".to_string(), "md".to_string(), "prompt".to_string()],
        similarity_threshold: config.capture.similarity_threshold,
    };

    println!("Starting file watcher...");
    println!("Watching directory: {}", watch_path.display());
    println!("File extensions: .txt, .md, .prompt");
    println!("Press Ctrl+C to stop.\n");

    let mut watcher = FileWatcher::new(watcher_config)
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

    watcher.start().map_err(|e| format!("Failed to start watcher: {}", e))?;

    let quality_analyzer = QualityAnalyzer::default();
    let efficiency_analyzer = EfficiencyAnalyzer::default();

    loop {
        let captured_ids = watcher
            .process_events(db)
            .map_err(|e| format!("Failed to process events: {}", e))?;

        for id in captured_ids {
            println!("Captured prompt: {}", id);

            // Auto-analyze if enabled
            if config.analysis.auto_analyze {
                if let Ok(Some(prompt)) = db.get_prompt(&id) {
                    if let Ok(quality) = quality_analyzer.analyze(&prompt) {
                        if let Err(e) = db.save_quality_score(&quality) {
                            eprintln!("Failed to save quality score: {}", e);
                        }
                    }
                    if let Ok(efficiency) = efficiency_analyzer.analyze(&prompt) {
                        if let Err(e) = db.save_efficiency_metrics(&efficiency) {
                            eprintln!("Failed to save efficiency metrics: {}", e);
                        }
                    }
                }
            }
        }

        // Sleep to avoid busy-waiting
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn cmd_query(db: &Database, query: &str) -> Result<(), String> {
    // Parse the advanced filter query
    let filter = parse_filter_query(query)
        .map_err(|e| format!("Failed to parse query: {}", e))?;

    // Query prompts
    let prompts = db
        .list_prompts(&filter)
        .map_err(|e| format!("Failed to query prompts: {}", e))?;

    if prompts.is_empty() {
        println!("No prompts found matching query.");
        return Ok(());
    }

    println!("Found {} prompt(s):\n", prompts.len());

    for prompt in prompts {
        // Get quality score if available
        let quality = db.get_quality_score(&prompt.id).ok().flatten();

        println!("ID: {}", prompt.id);
        println!("Created: {}", prompt.created_at.format("%Y-%m-%d %H:%M"));
        println!(
            "Content: {}",
            truncate_string(&prompt.content.replace('\n', " "), 100)
        );

        if let Some(category) = &prompt.category {
            println!("Category: {}", category);
        }

        if !prompt.tags.is_empty() {
            println!("Tags: {}", prompt.tags.join(", "));
        }

        println!("Status: {}", prompt.status);

        if let Some(q) = quality {
            println!("Quality: {:.1}", q.total_score);
        }

        println!("{}", "-".repeat(50));
    }

    Ok(())
}
