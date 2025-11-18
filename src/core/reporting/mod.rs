//! Report generation (weekly, monthly)
//!
//! Generates reports in various formats (Markdown, HTML, JSON, CSV).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::models::{EfficiencyMetrics, Prompt, QualityScore};
use crate::{PromptTrackingError, Result};

/// Report format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Markdown,
    Html,
    Json,
    Csv,
}

impl ReportFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &str {
        match self {
            ReportFormat::Markdown => "md",
            ReportFormat::Html => "html",
            ReportFormat::Json => "json",
            ReportFormat::Csv => "csv",
        }
    }

    /// Parse format from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Some(ReportFormat::Markdown),
            "html" => Some(ReportFormat::Html),
            "json" => Some(ReportFormat::Json),
            "csv" => Some(ReportFormat::Csv),
            _ => None,
        }
    }
}

/// Report type (period)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    Weekly,
    Monthly,
    Custom { days: u32 },
}

/// Report data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub title: String,
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub summary: ReportSummary,
    pub quality_breakdown: QualityBreakdown,
    pub efficiency_breakdown: EfficiencyBreakdown,
    pub top_prompts: Vec<PromptSummary>,
    pub category_stats: Vec<CategoryStat>,
    pub tag_stats: Vec<TagStat>,
}

/// Summary statistics for report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_prompts: usize,
    pub new_prompts: usize,
    pub avg_quality_score: f64,
    pub avg_efficiency_score: f64,
    pub total_tokens_used: u64,
    pub total_cost: f64,
}

/// Quality score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityBreakdown {
    pub avg_clarity: f64,
    pub avg_completeness: f64,
    pub avg_specificity: f64,
    pub avg_guidance: f64,
}

/// Efficiency metrics breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyBreakdown {
    pub avg_token_efficiency: f64,
    pub avg_time_efficiency: f64,
    pub avg_cost_efficiency: f64,
}

/// Prompt summary for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSummary {
    pub id: String,
    pub content_preview: String,
    pub quality_score: f64,
    pub efficiency_score: f64,
    pub created_at: DateTime<Utc>,
}

/// Category statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStat {
    pub name: String,
    pub count: usize,
    pub avg_quality: f64,
}

/// Tag statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStat {
    pub name: String,
    pub count: usize,
}

/// Report generator
pub struct ReportGenerator {
    format: ReportFormat,
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self {
            format: ReportFormat::Markdown,
        }
    }
}

impl ReportGenerator {
    /// Create generator with specific format
    pub fn new(format: ReportFormat) -> Self {
        Self { format }
    }

    /// Generate report from data
    pub fn generate(&self, data: &ReportData) -> Result<String> {
        match self.format {
            ReportFormat::Markdown => self.generate_markdown(data),
            ReportFormat::Html => self.generate_html(data),
            ReportFormat::Json => self.generate_json(data),
            ReportFormat::Csv => self.generate_csv(data),
        }
    }

    /// Save report to file
    pub fn save_to_file(&self, data: &ReportData, path: &Path) -> Result<()> {
        let content = self.generate(data)?;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                PromptTrackingError::IoError(std::io::Error::other(
                    format!("Failed to create directory: {}", e),
                ))
            })?;
        }

        std::fs::write(path, content).map_err(PromptTrackingError::IoError)?;
        Ok(())
    }

    /// Generate Markdown report
    fn generate_markdown(&self, data: &ReportData) -> Result<String> {
        let mut output = String::new();

        // Title
        output.push_str(&format!("# {}\n\n", data.title));
        output.push_str(&format!(
            "**Generated:** {}\n\n",
            data.generated_at.format("%Y-%m-%d %H:%M UTC")
        ));
        output.push_str(&format!(
            "**Period:** {} to {}\n\n",
            data.period_start.format("%Y-%m-%d"),
            data.period_end.format("%Y-%m-%d")
        ));

        // Summary
        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "- **Total Prompts:** {}\n",
            data.summary.total_prompts
        ));
        output.push_str(&format!(
            "- **New Prompts:** {}\n",
            data.summary.new_prompts
        ));
        output.push_str(&format!(
            "- **Average Quality Score:** {:.1}\n",
            data.summary.avg_quality_score
        ));
        output.push_str(&format!(
            "- **Average Efficiency Score:** {:.1}\n",
            data.summary.avg_efficiency_score
        ));
        output.push_str(&format!(
            "- **Total Tokens Used:** {}\n",
            data.summary.total_tokens_used
        ));
        output.push_str(&format!(
            "- **Total Cost:** ${:.4}\n\n",
            data.summary.total_cost
        ));

        // Quality Breakdown
        output.push_str("## Quality Breakdown\n\n");
        output.push_str("| Metric | Score |\n");
        output.push_str("|--------|-------|\n");
        output.push_str(&format!(
            "| Clarity | {:.1} |\n",
            data.quality_breakdown.avg_clarity
        ));
        output.push_str(&format!(
            "| Completeness | {:.1} |\n",
            data.quality_breakdown.avg_completeness
        ));
        output.push_str(&format!(
            "| Specificity | {:.1} |\n",
            data.quality_breakdown.avg_specificity
        ));
        output.push_str(&format!(
            "| Guidance | {:.1} |\n\n",
            data.quality_breakdown.avg_guidance
        ));

        // Efficiency Breakdown
        output.push_str("## Efficiency Breakdown\n\n");
        output.push_str("| Metric | Score |\n");
        output.push_str("|--------|-------|\n");
        output.push_str(&format!(
            "| Token Efficiency | {:.1} |\n",
            data.efficiency_breakdown.avg_token_efficiency
        ));
        output.push_str(&format!(
            "| Time Efficiency | {:.1} |\n",
            data.efficiency_breakdown.avg_time_efficiency
        ));
        output.push_str(&format!(
            "| Cost Efficiency | {:.1} |\n\n",
            data.efficiency_breakdown.avg_cost_efficiency
        ));

        // Top Prompts
        if !data.top_prompts.is_empty() {
            output.push_str("## Top Prompts\n\n");
            output.push_str("| Preview | Quality | Efficiency | Created |\n");
            output.push_str("|---------|---------|------------|--------|\n");
            for prompt in &data.top_prompts {
                output.push_str(&format!(
                    "| {} | {:.1} | {:.1} | {} |\n",
                    prompt.content_preview,
                    prompt.quality_score,
                    prompt.efficiency_score,
                    prompt.created_at.format("%Y-%m-%d")
                ));
            }
            output.push('\n');
        }

        // Category Statistics
        if !data.category_stats.is_empty() {
            output.push_str("## Categories\n\n");
            output.push_str("| Category | Count | Avg Quality |\n");
            output.push_str("|----------|-------|-------------|\n");
            for stat in &data.category_stats {
                output.push_str(&format!(
                    "| {} | {} | {:.1} |\n",
                    stat.name, stat.count, stat.avg_quality
                ));
            }
            output.push('\n');
        }

        // Tag Statistics
        if !data.tag_stats.is_empty() {
            output.push_str("## Tags\n\n");
            output.push_str("| Tag | Count |\n");
            output.push_str("|-----|-------|\n");
            for stat in &data.tag_stats {
                output.push_str(&format!("| {} | {} |\n", stat.name, stat.count));
            }
        }

        Ok(output)
    }

    /// Generate HTML report
    fn generate_html(&self, data: &ReportData) -> Result<String> {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        output.push_str(&format!("<title>{}</title>\n", data.title));
        output.push_str("<style>\n");
        output.push_str("body { font-family: sans-serif; margin: 40px; }\n");
        output.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        output.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        output.push_str("th { background-color: #4CAF50; color: white; }\n");
        output.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
        output.push_str(".summary { background: #f5f5f5; padding: 20px; border-radius: 5px; }\n");
        output.push_str("</style>\n</head>\n<body>\n");

        // Title
        output.push_str(&format!("<h1>{}</h1>\n", data.title));
        output.push_str(&format!(
            "<p><strong>Generated:</strong> {}</p>\n",
            data.generated_at.format("%Y-%m-%d %H:%M UTC")
        ));
        output.push_str(&format!(
            "<p><strong>Period:</strong> {} to {}</p>\n",
            data.period_start.format("%Y-%m-%d"),
            data.period_end.format("%Y-%m-%d")
        ));

        // Summary
        output.push_str("<div class=\"summary\">\n<h2>Summary</h2>\n<ul>\n");
        output.push_str(&format!(
            "<li><strong>Total Prompts:</strong> {}</li>\n",
            data.summary.total_prompts
        ));
        output.push_str(&format!(
            "<li><strong>New Prompts:</strong> {}</li>\n",
            data.summary.new_prompts
        ));
        output.push_str(&format!(
            "<li><strong>Average Quality Score:</strong> {:.1}</li>\n",
            data.summary.avg_quality_score
        ));
        output.push_str(&format!(
            "<li><strong>Average Efficiency Score:</strong> {:.1}</li>\n",
            data.summary.avg_efficiency_score
        ));
        output.push_str(&format!(
            "<li><strong>Total Tokens Used:</strong> {}</li>\n",
            data.summary.total_tokens_used
        ));
        output.push_str(&format!(
            "<li><strong>Total Cost:</strong> ${:.4}</li>\n",
            data.summary.total_cost
        ));
        output.push_str("</ul>\n</div>\n");

        // Quality Breakdown
        output.push_str("<h2>Quality Breakdown</h2>\n<table>\n");
        output.push_str("<tr><th>Metric</th><th>Score</th></tr>\n");
        output.push_str(&format!(
            "<tr><td>Clarity</td><td>{:.1}</td></tr>\n",
            data.quality_breakdown.avg_clarity
        ));
        output.push_str(&format!(
            "<tr><td>Completeness</td><td>{:.1}</td></tr>\n",
            data.quality_breakdown.avg_completeness
        ));
        output.push_str(&format!(
            "<tr><td>Specificity</td><td>{:.1}</td></tr>\n",
            data.quality_breakdown.avg_specificity
        ));
        output.push_str(&format!(
            "<tr><td>Guidance</td><td>{:.1}</td></tr>\n",
            data.quality_breakdown.avg_guidance
        ));
        output.push_str("</table>\n");

        // Efficiency Breakdown
        output.push_str("<h2>Efficiency Breakdown</h2>\n<table>\n");
        output.push_str("<tr><th>Metric</th><th>Score</th></tr>\n");
        output.push_str(&format!(
            "<tr><td>Token Efficiency</td><td>{:.1}</td></tr>\n",
            data.efficiency_breakdown.avg_token_efficiency
        ));
        output.push_str(&format!(
            "<tr><td>Time Efficiency</td><td>{:.1}</td></tr>\n",
            data.efficiency_breakdown.avg_time_efficiency
        ));
        output.push_str(&format!(
            "<tr><td>Cost Efficiency</td><td>{:.1}</td></tr>\n",
            data.efficiency_breakdown.avg_cost_efficiency
        ));
        output.push_str("</table>\n");

        output.push_str("</body>\n</html>");

        Ok(output)
    }

    /// Generate JSON report
    fn generate_json(&self, data: &ReportData) -> Result<String> {
        serde_json::to_string_pretty(data).map_err(|e| {
            PromptTrackingError::IoError(std::io::Error::other(
                format!("JSON serialization error: {}", e),
            ))
        })
    }

    /// Generate CSV report
    fn generate_csv(&self, data: &ReportData) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str("Metric,Value\n");

        // Summary
        output.push_str(&format!("Total Prompts,{}\n", data.summary.total_prompts));
        output.push_str(&format!("New Prompts,{}\n", data.summary.new_prompts));
        output.push_str(&format!(
            "Avg Quality Score,{:.2}\n",
            data.summary.avg_quality_score
        ));
        output.push_str(&format!(
            "Avg Efficiency Score,{:.2}\n",
            data.summary.avg_efficiency_score
        ));
        output.push_str(&format!(
            "Total Tokens,{}\n",
            data.summary.total_tokens_used
        ));
        output.push_str(&format!("Total Cost,{:.4}\n", data.summary.total_cost));

        // Quality breakdown
        output.push_str(&format!(
            "Clarity,{:.2}\n",
            data.quality_breakdown.avg_clarity
        ));
        output.push_str(&format!(
            "Completeness,{:.2}\n",
            data.quality_breakdown.avg_completeness
        ));
        output.push_str(&format!(
            "Specificity,{:.2}\n",
            data.quality_breakdown.avg_specificity
        ));
        output.push_str(&format!(
            "Guidance,{:.2}\n",
            data.quality_breakdown.avg_guidance
        ));

        // Efficiency breakdown
        output.push_str(&format!(
            "Token Efficiency,{:.2}\n",
            data.efficiency_breakdown.avg_token_efficiency
        ));
        output.push_str(&format!(
            "Time Efficiency,{:.2}\n",
            data.efficiency_breakdown.avg_time_efficiency
        ));
        output.push_str(&format!(
            "Cost Efficiency,{:.2}\n",
            data.efficiency_breakdown.avg_cost_efficiency
        ));

        Ok(output)
    }
}

/// Build report data from prompts and analysis results
pub fn build_report_data(
    report_type: ReportType,
    prompts: &[Prompt],
    quality_scores: &[QualityScore],
    efficiency_metrics: &[EfficiencyMetrics],
) -> ReportData {
    use crate::utils::truncate_string;
    use std::collections::HashMap;

    let now = Utc::now();

    // Calculate period
    let (period_start, period_end, title) = match report_type {
        ReportType::Weekly => {
            let start = now - chrono::Duration::days(7);
            (start, now, "Weekly Prompt Report".to_string())
        }
        ReportType::Monthly => {
            let start = now - chrono::Duration::days(30);
            (start, now, "Monthly Prompt Report".to_string())
        }
        ReportType::Custom { days } => {
            let start = now - chrono::Duration::days(days as i64);
            (start, now, format!("{}-Day Prompt Report", days))
        }
    };

    // Filter prompts in period
    let period_prompts: Vec<&Prompt> = prompts
        .iter()
        .filter(|p| p.created_at >= period_start && p.created_at <= period_end)
        .collect();

    // Build quality score map
    let quality_map: HashMap<String, &QualityScore> = quality_scores
        .iter()
        .map(|s| (s.prompt_id.clone(), s))
        .collect();

    // Build efficiency metrics map
    let efficiency_map: HashMap<String, &EfficiencyMetrics> = efficiency_metrics
        .iter()
        .map(|m| (m.prompt_id.clone(), m))
        .collect();

    // Calculate summary statistics
    let total_tokens: u64 = prompts
        .iter()
        .map(|p| {
            (p.metadata.input_tokens.unwrap_or(0) + p.metadata.output_tokens.unwrap_or(0)) as u64
        })
        .sum();

    let total_cost: f64 = prompts
        .iter()
        .map(|p| p.metadata.estimated_cost.unwrap_or(0.0))
        .sum();

    let avg_quality = if quality_scores.is_empty() {
        0.0
    } else {
        quality_scores.iter().map(|s| s.total_score).sum::<f64>() / quality_scores.len() as f64
    };

    let avg_efficiency = if efficiency_metrics.is_empty() {
        0.0
    } else {
        efficiency_metrics
            .iter()
            .map(|m| m.efficiency_score)
            .sum::<f64>()
            / efficiency_metrics.len() as f64
    };

    // Quality breakdown
    let quality_breakdown = if quality_scores.is_empty() {
        QualityBreakdown {
            avg_clarity: 0.0,
            avg_completeness: 0.0,
            avg_specificity: 0.0,
            avg_guidance: 0.0,
        }
    } else {
        let len = quality_scores.len() as f64;
        QualityBreakdown {
            avg_clarity: quality_scores.iter().map(|s| s.clarity).sum::<f64>() / len,
            avg_completeness: quality_scores.iter().map(|s| s.completeness).sum::<f64>() / len,
            avg_specificity: quality_scores.iter().map(|s| s.specificity).sum::<f64>() / len,
            avg_guidance: quality_scores.iter().map(|s| s.guidance).sum::<f64>() / len,
        }
    };

    // Efficiency breakdown
    let efficiency_breakdown = if efficiency_metrics.is_empty() {
        EfficiencyBreakdown {
            avg_token_efficiency: 0.0,
            avg_time_efficiency: 0.0,
            avg_cost_efficiency: 0.0,
        }
    } else {
        let len = efficiency_metrics.len() as f64;
        EfficiencyBreakdown {
            avg_token_efficiency: efficiency_metrics
                .iter()
                .map(|m| m.token_efficiency)
                .sum::<f64>()
                / len,
            avg_time_efficiency: efficiency_metrics
                .iter()
                .map(|m| m.time_efficiency)
                .sum::<f64>()
                / len,
            avg_cost_efficiency: efficiency_metrics
                .iter()
                .map(|m| m.cost_efficiency)
                .sum::<f64>()
                / len,
        }
    };

    // Top prompts (by quality score)
    let mut top_prompts: Vec<PromptSummary> = prompts
        .iter()
        .filter_map(|p| {
            let quality = quality_map.get(&p.id).map(|s| s.total_score).unwrap_or(0.0);
            let efficiency = efficiency_map
                .get(&p.id)
                .map(|m| m.efficiency_score)
                .unwrap_or(0.0);
            Some(PromptSummary {
                id: p.id.clone(),
                content_preview: truncate_string(&p.content, 50),
                quality_score: quality,
                efficiency_score: efficiency,
                created_at: p.created_at,
            })
        })
        .collect();
    top_prompts.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap());
    top_prompts.truncate(10);

    // Category statistics
    let mut category_counts: HashMap<String, (usize, Vec<f64>)> = HashMap::new();
    for prompt in prompts {
        if let Some(ref cat) = prompt.category {
            let entry = category_counts.entry(cat.clone()).or_insert((0, Vec::new()));
            entry.0 += 1;
            if let Some(score) = quality_map.get(&prompt.id) {
                entry.1.push(score.total_score);
            }
        }
    }
    let mut category_stats: Vec<CategoryStat> = category_counts
        .into_iter()
        .map(|(name, (count, scores))| {
            let avg_quality = if scores.is_empty() {
                0.0
            } else {
                scores.iter().sum::<f64>() / scores.len() as f64
            };
            CategoryStat {
                name,
                count,
                avg_quality,
            }
        })
        .collect();
    category_stats.sort_by(|a, b| b.count.cmp(&a.count));

    // Tag statistics
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for prompt in prompts {
        for tag in &prompt.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut tag_stats: Vec<TagStat> = tag_counts
        .into_iter()
        .map(|(name, count)| TagStat { name, count })
        .collect();
    tag_stats.sort_by(|a, b| b.count.cmp(&a.count));
    tag_stats.truncate(20);

    ReportData {
        title,
        generated_at: now,
        period_start,
        period_end,
        summary: ReportSummary {
            total_prompts: prompts.len(),
            new_prompts: period_prompts.len(),
            avg_quality_score: avg_quality,
            avg_efficiency_score: avg_efficiency,
            total_tokens_used: total_tokens,
            total_cost,
        },
        quality_breakdown,
        efficiency_breakdown,
        top_prompts,
        category_stats,
        tag_stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Prompt;

    fn create_test_data() -> (Vec<Prompt>, Vec<QualityScore>, Vec<EfficiencyMetrics>) {
        let mut prompt = Prompt::new("Test prompt content".to_string());
        prompt.content_hash = "test_hash".to_string();
        prompt.category = Some("code-generation".to_string());
        prompt.tags = vec!["rust".to_string()];

        let quality = QualityScore {
            prompt_id: prompt.id.clone(),
            total_score: 85.0,
            clarity: 80.0,
            completeness: 90.0,
            specificity: 85.0,
            guidance: 85.0,
            analyzed_at: Utc::now(),
        };

        let efficiency = EfficiencyMetrics {
            prompt_id: prompt.id.clone(),
            efficiency_score: 75.0,
            token_efficiency: 70.0,
            time_efficiency: 80.0,
            cost_efficiency: 75.0,
            calculated_at: Utc::now(),
        };

        (vec![prompt], vec![quality], vec![efficiency])
    }

    #[test]
    fn test_report_format_extension() {
        assert_eq!(ReportFormat::Markdown.extension(), "md");
        assert_eq!(ReportFormat::Html.extension(), "html");
        assert_eq!(ReportFormat::Json.extension(), "json");
        assert_eq!(ReportFormat::Csv.extension(), "csv");
    }

    #[test]
    fn test_report_format_from_str() {
        assert_eq!(
            ReportFormat::from_str("markdown"),
            Some(ReportFormat::Markdown)
        );
        assert_eq!(ReportFormat::from_str("html"), Some(ReportFormat::Html));
        assert_eq!(ReportFormat::from_str("json"), Some(ReportFormat::Json));
        assert_eq!(ReportFormat::from_str("csv"), Some(ReportFormat::Csv));
        assert_eq!(ReportFormat::from_str("invalid"), None);
    }

    #[test]
    fn test_build_report_data_weekly() {
        let (prompts, quality, efficiency) = create_test_data();
        let data = build_report_data(ReportType::Weekly, &prompts, &quality, &efficiency);

        assert_eq!(data.title, "Weekly Prompt Report");
        assert_eq!(data.summary.total_prompts, 1);
        assert_eq!(data.summary.avg_quality_score, 85.0);
    }

    #[test]
    fn test_build_report_data_monthly() {
        let (prompts, quality, efficiency) = create_test_data();
        let data = build_report_data(ReportType::Monthly, &prompts, &quality, &efficiency);

        assert_eq!(data.title, "Monthly Prompt Report");
    }

    #[test]
    fn test_generate_markdown() {
        let (prompts, quality, efficiency) = create_test_data();
        let data = build_report_data(ReportType::Weekly, &prompts, &quality, &efficiency);

        let generator = ReportGenerator::new(ReportFormat::Markdown);
        let report = generator.generate(&data).unwrap();

        assert!(report.contains("# Weekly Prompt Report"));
        assert!(report.contains("## Summary"));
        assert!(report.contains("Total Prompts"));
    }

    #[test]
    fn test_generate_html() {
        let (prompts, quality, efficiency) = create_test_data();
        let data = build_report_data(ReportType::Weekly, &prompts, &quality, &efficiency);

        let generator = ReportGenerator::new(ReportFormat::Html);
        let report = generator.generate(&data).unwrap();

        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("<h1>Weekly Prompt Report</h1>"));
    }

    #[test]
    fn test_generate_json() {
        let (prompts, quality, efficiency) = create_test_data();
        let data = build_report_data(ReportType::Weekly, &prompts, &quality, &efficiency);

        let generator = ReportGenerator::new(ReportFormat::Json);
        let report = generator.generate(&data).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(parsed["title"], "Weekly Prompt Report");
    }

    #[test]
    fn test_generate_csv() {
        let (prompts, quality, efficiency) = create_test_data();
        let data = build_report_data(ReportType::Weekly, &prompts, &quality, &efficiency);

        let generator = ReportGenerator::new(ReportFormat::Csv);
        let report = generator.generate(&data).unwrap();

        assert!(report.contains("Metric,Value"));
        assert!(report.contains("Total Prompts,1"));
    }

    #[test]
    fn test_empty_data() {
        let data = build_report_data(
            ReportType::Weekly,
            &[],
            &[],
            &[],
        );

        assert_eq!(data.summary.total_prompts, 0);
        assert_eq!(data.summary.avg_quality_score, 0.0);
        assert_eq!(data.summary.avg_efficiency_score, 0.0);
    }
}
